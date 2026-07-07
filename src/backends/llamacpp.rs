//! Stub temporaire — implémentation réelle (reqwest -> llama-server) en Task 8.

use super::{Backend, BackendError, BackendHealth, BackendRequest};
use crate::config::BackendConfig;
use async_trait::async_trait;

pub struct LlamaCppBackend;

impl LlamaCppBackend {
    pub fn new(_cfg: &BackendConfig) -> Self {
        LlamaCppBackend
    }
}

#[async_trait]
impl Backend for LlamaCppBackend {
    async fn complete(&self, _req: &BackendRequest) -> Result<String, BackendError> {
        Err(BackendError::Network("non implémenté".into()))
    }
    async fn health(&self) -> BackendHealth {
        BackendHealth { reachable: false }
    }
}
