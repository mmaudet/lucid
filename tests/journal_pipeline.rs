use axum::body::Body;
use axum::http::{Request, StatusCode};
use lucid::backends::mock::MockBackend;
use lucid::config::{Config, JournalConfig};
use lucid::dictionary::{Dictionary, DictionaryStore};
use lucid::server::{build_app, AppState};
use lucid::store::Store;
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn une_correction_cree_une_entree_de_journal() {
    let path = std::env::temp_dir().join(format!("lucid_pipeline_{}.sqlite", std::process::id()));
    let _ = std::fs::remove_file(&path);
    let store = Store::open(&path, &JournalConfig::default()).unwrap();

    let mut cfg = Config::default();
    cfg.server.bearer_token = Some("".into());
    let app = build_app(AppState {
        config: Arc::new(cfg),
        backend: Arc::new(MockBackend::with_response("Michel-Marie Maudet")),
        dictionary: Arc::new(DictionaryStore::in_memory(Dictionary::default())),
        store: store.clone(),
    });

    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("content-type", "application/json")
                .header("user-agent", "VoiceInk/1.0")
                .body(Body::from(
                    r#"{"messages":[{"role":"user","content":"michel marie mode"}]}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    store.flush().await;
    assert_eq!(store.count().await, 1);
    let rows = store.recent(1).await;
    assert_eq!(rows[0].output.as_deref(), Some("Michel-Marie Maudet"));
    assert_eq!(rows[0].status, "corrected");
    let _ = std::fs::remove_file(&path);
}
