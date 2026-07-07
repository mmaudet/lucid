//! Réponse SSE : le texte (déjà bufferisé + validé) est ré-émis en chunks OpenAI.

use axum::response::sse::{Event, Sse};
use axum::response::{IntoResponse, Response};
use futures_util::stream;
use std::convert::Infallible;

use crate::openai::{new_id, unix_now, ChunkChoice, Delta, StreamChunk};

pub fn sse_response(text: String, model: String) -> Response {
    let id = new_id();
    let created = unix_now();

    let chunk_content = StreamChunk {
        id: id.clone(),
        object: "chat.completion.chunk",
        created,
        model: model.clone(),
        choices: vec![ChunkChoice {
            index: 0,
            delta: Delta {
                role: Some("assistant".into()),
                content: Some(text),
            },
            finish_reason: None,
        }],
    };
    let chunk_stop = StreamChunk {
        id,
        object: "chat.completion.chunk",
        created,
        model,
        choices: vec![ChunkChoice {
            index: 0,
            delta: Delta::default(),
            finish_reason: Some("stop"),
        }],
    };

    let events: Vec<Result<Event, Infallible>> = vec![
        Ok(Event::default().data(serde_json::to_string(&chunk_content).unwrap())),
        Ok(Event::default().data(serde_json::to_string(&chunk_stop).unwrap())),
        Ok(Event::default().data("[DONE]")),
    ];
    Sse::new(stream::iter(events)).into_response()
}
