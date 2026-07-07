use axum::body::Body;
use axum::http::{Request, StatusCode};
use lucid::backends::mock::MockBackend;
use lucid::config::Config;
use lucid::dictionary::{Dictionary, DictionaryStore};
use lucid::server::{build_app, AppState};
use lucid::store::Store;
use std::sync::Arc;
use tower::ServiceExt;

fn app_with(backend: MockBackend) -> axum::Router {
    let mut cfg = Config::default();
    cfg.server.bearer_token = Some("".into()); // auth désactivée pour ces tests
    build_app(AppState {
        config: Arc::new(cfg),
        backend: Arc::new(backend),
        dictionary: Arc::new(DictionaryStore::in_memory(Dictionary::default())),
        store: Store::disabled(),
    })
}

fn post(body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

#[tokio::test]
async fn completion_non_stream_renvoie_le_texte_corrige() {
    let app = app_with(MockBackend::with_response("Michel-Marie Maudet"));
    let resp = app
        .oneshot(post(serde_json::json!({
            "model": "lucid",
            "messages": [ { "role": "user", "content": "michel marie mode" } ]
        })))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["object"], "chat.completion");
    assert_eq!(v["choices"][0]["message"]["content"], "Michel-Marie Maudet");
    assert_eq!(v["choices"][0]["finish_reason"], "stop");
}

#[tokio::test]
async fn failsafe_renvoie_l_entree_si_backend_echoue() {
    let app = app_with(MockBackend::failing());
    let resp = app
        .oneshot(post(serde_json::json!({
            "messages": [ { "role": "user", "content": "phrase originale" } ]
        })))
        .await
        .unwrap();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["choices"][0]["message"]["content"], "phrase originale");
}
