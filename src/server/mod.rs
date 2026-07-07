//! Serveur HTTP axum : routes OpenAI, auth, santé.

use crate::backends::Backend;
use crate::config::Config;
use crate::dictionary::DictionaryStore;
use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;

pub mod auth;
pub mod error;
pub mod routes;
pub mod stream;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub backend: Arc<dyn Backend>,
    pub dictionary: Arc<DictionaryStore>,
}

pub fn build_app(state: AppState) -> Router {
    // Routes protégées par le bearer.
    let protected = Router::new()
        .route("/v1/models", get(routes::models))
        .route("/v1/chat/completions", post(routes::chat_completions))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth::require_bearer,
        ));

    // /health reste ouvert.
    Router::new()
        .route("/health", get(routes::health))
        .merge(protected)
        .with_state(state)
}
