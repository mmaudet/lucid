//! Handlers HTTP.

use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

use super::AppState;
use crate::correction;
use crate::openai::{ChatMessage, ChatRequest, ChatResponse, Choice, Usage, new_id, unix_now};

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

pub async fn chat_completions(State(state): State<AppState>, Json(req): Json<ChatRequest>) -> Response {
    let dict = state.dictionary.snapshot();
    let outcome = correction::correct(
        &*state.backend,
        &dict,
        &state.config.correction,
        &req,
    )
    .await;
    let model = req.model.clone().unwrap_or_else(|| "lucid".into());

    if req.stream {
        return super::stream::sse_response(outcome.text, model);
    }

    let resp = ChatResponse {
        id: new_id(),
        object: "chat.completion",
        created: unix_now(),
        model,
        choices: vec![Choice {
            index: 0,
            message: ChatMessage {
                role: "assistant".into(),
                content: outcome.text,
            },
            finish_reason: "stop",
        }],
        usage: Usage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        },
    };
    Json(resp).into_response()
}
