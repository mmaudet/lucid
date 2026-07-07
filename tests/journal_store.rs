use lucid::config::JournalConfig;
use lucid::dictionary::Dictionary;
use lucid::store::{LogEntry, LogStatus, Store};
use std::sync::Arc;

fn tmp_db(tag: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("lucid_journal_{}_{}.sqlite", tag, std::process::id()))
}

fn entry(input: &str, output: &str, status: LogStatus, dict: Arc<Dictionary>) -> LogEntry {
    LogEntry {
        ts_ms: lucid::store::now_ms(),
        status,
        input: input.into(),
        output: output.into(),
        latency_ms: 12,
        backend_kind: "llamacpp".into(),
        model: "luciole".into(),
        stream: false,
        user_agent: Some("test".into()),
        dict,
    }
}

#[tokio::test]
async fn log_puis_count_et_recent() {
    let path = tmp_db("basic");
    let _ = std::fs::remove_file(&path);
    let store = Store::open(&path, &JournalConfig::default()).unwrap();
    let dict = Arc::new(Dictionary::from_json(
        r#"{"terms":[{"canonical":"VoiceInk","aliases":["voice inque"]}]}"#,
    ));
    store.log(entry("voice inque", "VoiceInk", LogStatus::Corrected, dict.clone()));
    store.log(entry("bonjour", "Bonjour", LogStatus::Corrected, dict));
    store.flush().await;
    assert_eq!(store.count().await, 2);
    let rows = store.recent(10).await;
    assert_eq!(rows.len(), 2);
    assert!(rows[0].output.is_some()); // texte stocké par défaut
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn mode_metadonnees_seules_ne_stocke_pas_le_texte() {
    let path = tmp_db("metaonly");
    let _ = std::fs::remove_file(&path);
    let cfg = JournalConfig { enabled: true, store_text: false, retention_days: 30 };
    let store = Store::open(&path, &cfg).unwrap();
    store.log(entry("secret dicté", "Secret dicté.", LogStatus::Corrected, Arc::new(Dictionary::default())));
    store.flush().await;
    let rows = store.recent(10).await;
    assert_eq!(rows.len(), 1);
    assert!(rows[0].input.is_none()); // pas de texte
    assert!(rows[0].output.is_none());
    assert_eq!(rows[0].input_chars, 12); // mais les métadonnées oui
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn disabled_ne_journalise_rien() {
    let store = Store::disabled();
    assert!(!store.is_enabled());
    store.log(entry("a", "A", LogStatus::Corrected, Arc::new(Dictionary::default())));
    store.flush().await;
    assert_eq!(store.count().await, 0);
}

#[tokio::test]
async fn clear_vide_le_journal() {
    let path = tmp_db("clear");
    let _ = std::fs::remove_file(&path);
    let store = Store::open(&path, &JournalConfig::default()).unwrap();
    store.log(entry("a", "A", LogStatus::Corrected, Arc::new(Dictionary::default())));
    store.flush().await;
    assert_eq!(store.count().await, 1);
    store.clear().await;
    assert_eq!(store.count().await, 0);
    let _ = std::fs::remove_file(&path);
}
