use axum::body::Body;
use axum::http::Request;
use lucid::backends::mock::MockBackend;
use lucid::config::Config;
use lucid::dictionary::{Dictionary, DictionaryStore};
use lucid::server::{build_app, AppState};
use lucid::store::Store;
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn stream_emet_des_chunks_et_done() {
    let mut cfg = Config::default();
    cfg.server.bearer_token = Some("".into());
    let app = build_app(AppState {
        config: Arc::new(cfg),
        backend: Arc::new(MockBackend::with_response("Bonjour Michel")),
        dictionary: Arc::new(DictionaryStore::in_memory(Dictionary::default())),
        store: Store::disabled(),
    });
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"messages":[{"role":"user","content":"bonjour michel"}],"stream":true}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let ct = resp
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert!(ct.starts_with("text/event-stream"));
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let text = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(text.contains("chat.completion.chunk"));
    assert!(text.contains("Bonjour Michel"));
    assert!(text.contains("[DONE]"));
}
