//! Abstraction des runtimes (llama.cpp, plus tard Ollama) derrière un trait.

use crate::openai::ChatMessage;
use async_trait::async_trait;

pub mod llamacpp;
pub mod mock;

pub struct BackendRequest {
    pub messages: Vec<ChatMessage>,
    pub temperature: f32,
    pub top_p: f32,
    pub max_tokens: u32,
    pub model: String,
}

#[derive(Debug, Clone, Copy)]
pub struct BackendHealth {
    pub reachable: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    #[error("timeout backend")]
    Timeout,
    #[error("réseau: {0}")]
    Network(String),
    #[error("statut HTTP {0}")]
    Http(u16),
    #[error("corps de réponse invalide: {0}")]
    Body(String),
}

#[async_trait]
pub trait Backend: Send + Sync {
    /// Renvoie le texte corrigé complet (non streamé).
    async fn complete(&self, req: &BackendRequest) -> Result<String, BackendError>;
    async fn health(&self) -> BackendHealth;
}

/// Fabrique un backend selon la config.
pub fn from_config(cfg: &crate::config::BackendConfig) -> std::sync::Arc<dyn Backend> {
    match cfg.kind.as_str() {
        // "ollama" viendra dans un incrément ultérieur.
        _ => std::sync::Arc::new(llamacpp::LlamaCppBackend::new(cfg)),
    }
}
