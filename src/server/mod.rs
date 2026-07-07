//! Serveur HTTP axum : routes OpenAI, auth, santé.

use crate::backends::Backend;
use crate::config::Config;
use crate::dictionary::Dictionary;
use std::sync::Arc;

pub mod auth;
pub mod error;
pub mod routes;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub backend: Arc<dyn Backend>,
    pub dictionary: Arc<Dictionary>,
}
