//! Serveur HTTP axum : routes OpenAI, auth, santé.

use crate::backends::Backend;
use crate::config::Config;
use crate::dictionary::DictionaryStore;
use crate::store::Store;
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
    pub store: Store,
}

pub fn build_app(state: AppState) -> Router {
    // Routes protégées par le bearer.
    let protected = Router::new()
        .route("/v1/models", get(routes::models))
        .route("/v1/chat/completions", post(routes::chat_completions))
        // Certains clients (ex. VoiceInk) postent la complétion directement sur
        // l'« API Endpoint URL » sans ajouter /chat/completions. On accepte donc /v1
        // et /chat/completions (base_url sans /v1).
        .route("/v1", post(routes::chat_completions))
        .route("/chat/completions", post(routes::chat_completions))
        // Filet tolérant : TOUT POST (quel que soit le chemin) est traité comme une
        // complétion. Évite les 404 quand un client poste sur un chemin inattendu.
        .route("/{*rest}", post(routes::chat_completions))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth::require_bearer,
        ));

    // /health reste ouvert.
    Router::new()
        .route("/health", get(routes::health))
        .merge(protected)
        .fallback(routes::fallback)
        .layer(axum::middleware::from_fn(routes::log_request))
        .with_state(state)
}
