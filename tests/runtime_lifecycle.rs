use lucid::backends::mock::MockBackend;
use lucid::config::Config;
use lucid::dictionary::{Dictionary, DictionaryStore};
use lucid::runtime::ServerManager;
use lucid::store::Store;
use std::sync::Arc;

fn mgr() -> ServerManager {
    let mut cfg = Config::default();
    cfg.server.port = 0; // port éphémère
    cfg.server.bearer_token = Some("".into());
    cfg.journal.enabled = false;
    ServerManager::with_parts(
        cfg,
        Arc::new(MockBackend::with_response("ok")),
        Arc::new(DictionaryStore::in_memory(Dictionary::default())),
        Store::disabled(),
    )
}

#[tokio::test]
async fn start_health_stop() {
    let mut m = mgr();
    assert!(!m.is_running());
    let addr = m.start().await.unwrap();
    assert!(m.is_running());

    let url = format!("http://{}/health", addr);
    let resp = reqwest::get(&url).await.unwrap();
    assert_eq!(resp.status().as_u16(), 200);

    m.stop().await;
    assert!(!m.is_running());
    // Connexion refusée après arrêt.
    assert!(reqwest::get(&url).await.is_err());
}

#[tokio::test]
async fn reload_dictionary_sans_restart() {
    let mut m = mgr();
    m.start().await.unwrap();
    assert!(m.reload_dictionary().is_ok());
    assert!(m.is_running()); // toujours en marche : pas de redémarrage
    m.stop().await;
}

#[tokio::test]
async fn apply_config_redemarre_sur_nouveau_bind() {
    let mut m = mgr();
    m.start().await.unwrap();
    let mut cfg2 = m.config().clone();
    cfg2.server.port = 0; // nouveau port éphémère
    let addr2 = m.apply_config(cfg2).await.unwrap();
    assert!(m.is_running());
    assert_eq!(m.local_addr(), Some(addr2));
    m.stop().await;
}
