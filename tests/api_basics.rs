use axum::body::Body;
use axum::http::{Request, StatusCode};
use lucid::backends::mock::MockBackend;
use lucid::config::Config;
use lucid::dictionary::{Dictionary, DictionaryStore};
use lucid::server::{build_app, AppState};
use lucid::store::Store;
use std::sync::Arc;
use tower::ServiceExt;

fn state(bearer: Option<&str>) -> AppState {
    let mut cfg = Config::default();
    cfg.server.bearer_token = bearer.map(|s| s.to_string());
    AppState {
        config: Arc::new(cfg),
        backend: Arc::new(MockBackend::with_response("ok")),
        dictionary: Arc::new(DictionaryStore::in_memory(Dictionary::default())),
        store: Store::disabled(),
    }
}

#[tokio::test]
async fn health_est_ouvert() {
    let app = build_app(state(Some("secret")));
    let resp = app
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["status"], "ok");
}

#[tokio::test]
async fn models_exige_le_bearer() {
    let app = build_app(state(Some("secret")));
    let resp = app
        .oneshot(Request::builder().uri("/v1/models").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn models_ok_avec_bearer() {
    let app = build_app(state(Some("secret")));
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/v1/models")
                .header("Authorization", "Bearer secret")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["data"][0]["id"], "lucid");
}
