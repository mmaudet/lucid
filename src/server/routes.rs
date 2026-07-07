//! Handlers HTTP.

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde_json::json;

use super::AppState;

pub async fn health(State(state): State<AppState>) -> Json<serde_json::Value> {
    let h = state.backend.health().await;
    Json(json!({
        "status": "ok",
        "backend": {
            "kind": state.config.backend.kind,
            "base_url": state.config.backend.base_url,
            "model": state.config.backend.model,
            "reachable": h.reachable,
        }
    }))
}

pub async fn models() -> Json<serde_json::Value> {
    Json(json!({
        "object": "list",
        "data": [ { "id": "lucid", "object": "model", "owned_by": "lucid" } ]
    }))
}

/// Stub — remplacé par l'implémentation complète en Task 12.
pub async fn chat_completions() -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
}
