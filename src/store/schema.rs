//! Schéma SQLite v1 — FIGÉ M6-aware (colonnes stats incluses dès maintenant
//! pour éviter toute migration côté statistiques).

use rusqlite::Connection;

pub const USER_VERSION: i64 = 1;

const SCHEMA_V1: &str = r#"
CREATE TABLE IF NOT EXISTS corrections (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    ts_ms          INTEGER NOT NULL,
    status         TEXT    NOT NULL,   -- 'corrected' | 'unchanged' | 'failsafe'
    input          TEXT,               -- NULL si store_text=false
    output         TEXT,               -- NULL si store_text=false
    input_chars    INTEGER NOT NULL,
    output_chars   INTEGER NOT NULL,
    edit_count     INTEGER NOT NULL,
    latency_ms     INTEGER NOT NULL,
    prompt_tokens  INTEGER,            -- estimé (chars/3), nullable
    completion_tokens INTEGER,         -- estimé (chars/3), nullable
    backend_kind   TEXT    NOT NULL,
    model          TEXT    NOT NULL,
    stream         INTEGER NOT NULL,   -- 0/1
    user_agent     TEXT
);
CREATE INDEX IF NOT EXISTS idx_corrections_ts ON corrections(ts_ms);

CREATE TABLE IF NOT EXISTS correction_hits (
    correction_id  INTEGER NOT NULL REFERENCES corrections(id) ON DELETE CASCADE,
    canonical      TEXT    NOT NULL,
    alias          TEXT
);
CREATE INDEX IF NOT EXISTS idx_hits_canonical ON correction_hits(canonical, correction_id);
"#;

/// Applique les PRAGMA + le schéma. Idempotent (user_version).
pub fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    let version: i64 = conn.pragma_query_value(None, "user_version", |r| r.get(0))?;
    if version < USER_VERSION {
        conn.execute_batch(SCHEMA_V1)?;
        conn.pragma_update(None, "user_version", USER_VERSION)?;
    }
    Ok(())
}
