//! Backend Ollama — relaie vers l'API OpenAI-compatible d'Ollama (:11434/v1).
//! Parité fonctionnelle avec llama.cpp (CA-5).

use super::openai_http::OpenAiHttp;
use super::{Backend, BackendError, BackendHealth, BackendRequest};
use crate::config::BackendConfig;
use async_trait::async_trait;

pub struct OllamaBackend {
    http: OpenAiHttp,
}

impl OllamaBackend {
    pub fn new(cfg: &BackendConfig) -> Self {
        OllamaBackend {
            http: OpenAiHttp::new(&cfg.base_url, &cfg.model, cfg.timeout_ms, cfg.health_timeout_ms),
        }
    }
}

#[async_trait]
impl Backend for OllamaBackend {
    async fn complete(&self, req: &BackendRequest) -> Result<String, BackendError> {
        self.http.complete(req).await
    }
    async fn health(&self) -> BackendHealth {
        self.http.health().await
    }
}
