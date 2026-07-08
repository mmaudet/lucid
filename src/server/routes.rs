//! Handlers HTTP.

use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use std::time::Instant;

use super::AppState;
use crate::correction;
use crate::openai::{ChatMessage, ChatRequest, ChatResponse, Choice, Usage, new_id, unix_now};
use crate::store::LogEntry;

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

pub async fn models(State(state): State<AppState>) -> Json<serde_json::Value> {
    let model = state.config.backend.model.clone();
    Json(json!({
        "object": "list",
        "data": [ { "id": model, "object": "model", "owned_by": "lucid" } ]
    }))
}

pub async fn chat_completions(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ChatRequest>,
) -> Response {
    let dict = state.dictionary.snapshot();
    let started = Instant::now();
    let outcome = correction::correct(
        &*state.backend,
        &dict,
        &state.config.correction,
        &req,
    )
    .await;
    let latency_ms = started.elapsed().as_millis() as u64;
    let model = req
        .model
        .clone()
        .unwrap_or_else(|| state.config.backend.model.clone());

    // Journalisation : hors de correct() (signature figée), couvre stream + non-stream.
    if state.store.is_enabled() {
        let user_agent = headers
            .get(axum::http::header::USER_AGENT)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        state.store.log(LogEntry {
            ts_ms: crate::store::now_ms(),
            status: outcome.status.into(),
            input: correction::extract_input(&req.messages),
            output: outcome.text.clone(),
            latency_ms,
            backend_kind: state.config.backend.kind.clone(),
            model: model.clone(),
            stream: req.stream,
            user_agent,
            dict: dict.clone(),
        });
    }

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
