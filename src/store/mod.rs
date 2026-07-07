//! Journal SQLite : acteur mono-écrivain (thread dédié + canal borné drop-on-full,
//! pour ne JAMAIS bloquer ni ralentir la dictée) + lectures via connexions courtes.

mod entry;
mod schema;

pub use entry::{JournalRow, LogEntry, LogStatus};

use crate::config::JournalConfig;
use rusqlite::{params, Connection};
use serde::Serialize;

/// Agrégats pour la fenêtre Statistiques.
#[derive(Debug, Clone, Default, Serialize)]
pub struct StatsSummary {
    pub total: i64,
    pub corrected: i64,
    pub unchanged: i64,
    pub failsafe: i64,
    pub avg_latency_ms: f64,
    pub by_day: Vec<DayCount>,
    pub top_terms: Vec<TermCount>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DayCount {
    pub day: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TermCount {
    pub canonical: String,
    pub count: i64,
}
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

const CHANNEL_CAP: usize = 1024;

/// Handle clonable vers le journal. `disabled()` = no-op total (tests, journal off).
#[derive(Clone)]
pub struct Store {
    inner: Arc<Inner>,
}

struct Inner {
    sender: Option<SyncSender<Msg>>,
    enabled: AtomicBool,
    store_text: Arc<AtomicBool>,
    path: Option<PathBuf>,
}

enum Msg {
    Log(LogEntry),
    Flush(tokio::sync::oneshot::Sender<()>),
    Clear(tokio::sync::oneshot::Sender<()>),
}

pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

impl Store {
    /// Journal inactif : `log` est un no-op, aucune connexion ouverte.
    pub fn disabled() -> Store {
        Store {
            inner: Arc::new(Inner {
                sender: None,
                enabled: AtomicBool::new(false),
                store_text: Arc::new(AtomicBool::new(false)),
                path: None,
            }),
        }
    }

    /// Ouvre le journal, applique le schéma, purge la rétention, lance l'acteur écrivain.
    pub fn open(path: &Path, cfg: &JournalConfig) -> rusqlite::Result<Store> {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let conn = Connection::open(path)?;
        schema::migrate(&conn)?;
        if cfg.retention_days > 0 {
            let cutoff = now_ms() - (cfg.retention_days as i64 * 86_400_000);
            let _ = conn.execute("DELETE FROM corrections WHERE ts_ms < ?1", params![cutoff]);
        }
        let store_text = Arc::new(AtomicBool::new(cfg.store_text));
        let (tx, rx) = sync_channel::<Msg>(CHANNEL_CAP);
        let st = store_text.clone();
        std::thread::Builder::new()
            .name("lucid-journal".into())
            .spawn(move || run_writer(conn, rx, st))
            .expect("spawn writer journal");
        Ok(Store {
            inner: Arc::new(Inner {
                sender: Some(tx),
                enabled: AtomicBool::new(cfg.enabled),
                store_text,
                path: Some(path.to_path_buf()),
            }),
        })
    }

    pub fn is_enabled(&self) -> bool {
        self.inner.enabled.load(Ordering::Relaxed) && self.inner.sender.is_some()
    }

    pub fn set_enabled(&self, on: bool) {
        self.inner.enabled.store(on, Ordering::Relaxed);
    }

    pub fn set_store_text(&self, on: bool) {
        self.inner.store_text.store(on, Ordering::Relaxed);
    }

    /// Journalise (non bloquant, drop-on-full : jamais de backpressure sur la dictée).
    pub fn log(&self, entry: LogEntry) {
        if !self.is_enabled() {
            return;
        }
        if let Some(tx) = &self.inner.sender {
            let _ = tx.try_send(Msg::Log(entry));
        }
    }

    /// Attend que l'acteur ait traité toutes les entrées en attente (déterminisme des tests).
    pub async fn flush(&self) {
        if let Some(tx) = &self.inner.sender {
            let (o, r) = tokio::sync::oneshot::channel();
            if tx.try_send(Msg::Flush(o)).is_ok() {
                let _ = r.await;
            }
        }
    }

    /// Vide le journal (« vider le journal »).
    pub async fn clear(&self) {
        if let Some(tx) = &self.inner.sender {
            let (o, r) = tokio::sync::oneshot::channel();
            if tx.try_send(Msg::Clear(o)).is_ok() {
                let _ = r.await;
            }
        }
    }

    pub async fn count(&self) -> i64 {
        let Some(path) = self.inner.path.clone() else {
            return 0;
        };
        tokio::task::spawn_blocking(move || {
            Connection::open(&path)
                .and_then(|c| c.query_row("SELECT COUNT(*) FROM corrections", [], |r| r.get(0)))
                .unwrap_or(0)
        })
        .await
        .unwrap_or(0)
    }

    pub async fn recent(&self, limit: i64) -> Vec<JournalRow> {
        let Some(path) = self.inner.path.clone() else {
            return vec![];
        };
        tokio::task::spawn_blocking(move || read_recent(&path, limit).unwrap_or_default())
            .await
            .unwrap_or_default()
    }

    pub async fn stats_summary(&self) -> StatsSummary {
        let Some(path) = self.inner.path.clone() else {
            return StatsSummary::default();
        };
        tokio::task::spawn_blocking(move || read_stats(&path).unwrap_or_default())
            .await
            .unwrap_or_default()
    }
}

fn run_writer(conn: Connection, rx: Receiver<Msg>, store_text: Arc<AtomicBool>) {
    for msg in rx {
        match msg {
            Msg::Log(e) => {
                let _ = insert(&conn, &e, store_text.load(Ordering::Relaxed));
            }
            Msg::Flush(tx) => {
                let _ = tx.send(());
            }
            Msg::Clear(tx) => {
                let _ = conn.execute_batch("DELETE FROM correction_hits; DELETE FROM corrections;");
                let _ = tx.send(());
            }
        }
    }
}

fn insert(conn: &Connection, e: &LogEntry, store_text: bool) -> rusqlite::Result<()> {
    let input_chars = e.input.chars().count() as i64;
    let output_chars = e.output.chars().count() as i64;
    let edit_count = word_edit_count(&e.input, &e.output) as i64;
    let (input_col, output_col): (Option<&str>, Option<&str>) = if store_text {
        (Some(e.input.as_str()), Some(e.output.as_str()))
    } else {
        (None, None)
    };
    conn.execute(
        "INSERT INTO corrections \
         (ts_ms,status,input,output,input_chars,output_chars,edit_count,latency_ms,\
          prompt_tokens,completion_tokens,backend_kind,model,stream,user_agent) \
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",
        params![
            e.ts_ms,
            e.status.as_str(),
            input_col,
            output_col,
            input_chars,
            output_chars,
            edit_count,
            e.latency_ms as i64,
            input_chars / 3,  // prompt_tokens estimés (chars/3)
            output_chars / 3, // completion_tokens estimés
            e.backend_kind,
            e.model,
            e.stream as i64,
            e.user_agent,
        ],
    )?;
    let id = conn.last_insert_rowid();
    // Hits dictionnaire : termes canoniques apparaissant dans la sortie (top termes).
    let out_lower = e.output.to_lowercase();
    for term in &e.dict.terms {
        if !term.canonical.is_empty() && out_lower.contains(&term.canonical.to_lowercase()) {
            conn.execute(
                "INSERT INTO correction_hits (correction_id, canonical, alias) VALUES (?1,?2,NULL)",
                params![id, term.canonical],
            )?;
        }
    }
    Ok(())
}

fn read_recent(path: &Path, limit: i64) -> rusqlite::Result<Vec<JournalRow>> {
    let conn = Connection::open(path)?;
    let mut stmt = conn.prepare(
        "SELECT id,ts_ms,status,input,output,input_chars,output_chars,edit_count,latency_ms,backend_kind,model \
         FROM corrections ORDER BY ts_ms DESC, id DESC LIMIT ?1",
    )?;
    let rows = stmt.query_map([limit], |r| {
        Ok(JournalRow {
            id: r.get(0)?,
            ts_ms: r.get(1)?,
            status: r.get(2)?,
            input: r.get(3)?,
            output: r.get(4)?,
            input_chars: r.get(5)?,
            output_chars: r.get(6)?,
            edit_count: r.get(7)?,
            latency_ms: r.get(8)?,
            backend_kind: r.get(9)?,
            model: r.get(10)?,
        })
    })?;
    rows.collect()
}

fn read_stats(path: &Path) -> rusqlite::Result<StatsSummary> {
    let conn = Connection::open(path)?;
    let total: i64 = conn.query_row("SELECT COUNT(*) FROM corrections", [], |r| r.get(0))?;
    let count_status = |s: &str| -> i64 {
        conn.query_row(
            "SELECT COUNT(*) FROM corrections WHERE status = ?1",
            [s],
            |r| r.get(0),
        )
        .unwrap_or(0)
    };
    let avg_latency_ms: f64 = conn.query_row(
        "SELECT COALESCE(AVG(latency_ms), 0) FROM corrections",
        [],
        |r| r.get(0),
    )?;

    let mut by_day = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT date(ts_ms/1000, 'unixepoch', 'localtime') AS d, COUNT(*) \
             FROM corrections GROUP BY d ORDER BY d DESC LIMIT 14",
        )?;
        let rows = stmt.query_map([], |r| Ok(DayCount { day: r.get(0)?, count: r.get(1)? }))?;
        for row in rows {
            by_day.push(row?);
        }
    }

    let mut top_terms = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT canonical, COUNT(*) AS c FROM correction_hits \
             GROUP BY canonical ORDER BY c DESC LIMIT 10",
        )?;
        let rows = stmt.query_map([], |r| Ok(TermCount { canonical: r.get(0)?, count: r.get(1)? }))?;
        for row in rows {
            top_terms.push(row?);
        }
    }

    Ok(StatsSummary {
        total,
        corrected: count_status("corrected"),
        unchanged: count_status("unchanged"),
        failsafe: count_status("failsafe"),
        avg_latency_ms,
        by_day,
        top_terms,
    })
}

/// Mesure d'édition simple (positions de mots divergentes + écart de longueur).
fn word_edit_count(a: &str, b: &str) -> u32 {
    let aw: Vec<&str> = a.split_whitespace().collect();
    let bw: Vec<&str> = b.split_whitespace().collect();
    let common = aw.iter().zip(bw.iter()).filter(|(x, y)| x == y).count();
    (aw.len().max(bw.len()) - common) as u32
}
