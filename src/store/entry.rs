//! Entrée de journal + statut, découplés de `correction`.

use crate::dictionary::Dictionary;
use serde::Serialize;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogStatus {
    Corrected,
    Unchanged,
    FailSafe,
}

impl LogStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            LogStatus::Corrected => "corrected",
            LogStatus::Unchanged => "unchanged",
            LogStatus::FailSafe => "failsafe",
        }
    }
}

impl From<crate::correction::Status> for LogStatus {
    fn from(s: crate::correction::Status) -> Self {
        match s {
            crate::correction::Status::Corrected => LogStatus::Corrected,
            crate::correction::Status::Unchanged => LogStatus::Unchanged,
            crate::correction::Status::FailSafe => LogStatus::FailSafe,
        }
    }
}

/// Une correction à journaliser. Construite dans le handler (hot-path minimal :
/// juste des clones/Arc) ; les calculs coûteux (edit_count, hits dictionnaire)
/// sont faits dans l'acteur écrivain, hors du chemin de la dictée.
pub struct LogEntry {
    pub ts_ms: i64,
    pub status: LogStatus,
    pub input: String,
    pub output: String,
    pub latency_ms: u64,
    pub backend_kind: String,
    pub model: String,
    pub stream: bool,
    pub user_agent: Option<String>,
    /// Snapshot du dictionnaire au moment de la correction (pour les hits).
    pub dict: Arc<Dictionary>,
}

/// Ligne de journal renvoyée aux lecteurs (fenêtre Journal, stats).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct JournalRow {
    pub id: i64,
    pub ts_ms: i64,
    pub status: String,
    pub input: Option<String>,
    pub output: Option<String>,
    pub input_chars: i64,
    pub output_chars: i64,
    pub edit_count: i64,
    pub latency_ms: i64,
    pub backend_kind: String,
    pub model: String,
}
