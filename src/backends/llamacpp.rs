//! Client HTTP vers llama-server (API OpenAI de llama.cpp).

use super::{Backend, BackendError, BackendHealth, BackendRequest};
use crate::config::BackendConfig;
use crate::openai::ChatMessage;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct LlamaCppBackend {
    base_url: String,
    model: String,
    client: reqwest::Client,
    health_timeout: Duration,
}

impl LlamaCppBackend {
    pub fn new(cfg: &BackendConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(cfg.timeout_ms))
            .build()
            .expect("client reqwest");
        LlamaCppBackend {
            base_url: cfg.base_url.trim_end_matches('/').to_string(),
            model: cfg.model.clone(),
            client,
            health_timeout: Duration::from_millis(cfg.health_timeout_ms),
        }
    }
}

#[derive(Serialize)]
struct UpstreamRequest<'a> {
    model: &'a str,
    messages: &'a [ChatMessage],
    temperature: f32,
    top_p: f32,
    max_tokens: u32,
    stream: bool,
    #[serde(skip_serializing_if = "<[String]>::is_empty")]
    stop: &'a [String],
}

#[derive(Deserialize)]
struct UpstreamResponse {
    choices: Vec<UpstreamChoice>,
}
#[derive(Deserialize)]
struct UpstreamChoice {
    message: UpstreamMessage,
}
#[derive(Deserialize)]
struct UpstreamMessage {
    content: String,
}

#[async_trait]
impl Backend for LlamaCppBackend {
    async fn complete(&self, req: &BackendRequest) -> Result<String, BackendError> {
        let body = UpstreamRequest {
            model: &self.model,
            messages: &req.messages,
            temperature: req.temperature,
            top_p: req.top_p,
            max_tokens: req.max_tokens,
            stream: false,
            stop: &req.stop,
        };
        let resp = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    BackendError::Timeout
                } else {
                    BackendError::Network(e.to_string())
                }
            })?;

        if !resp.status().is_success() {
            return Err(BackendError::Http(resp.status().as_u16()));
        }
        let parsed: UpstreamResponse = resp
            .json()
            .await
            .map_err(|e| BackendError::Body(e.to_string()))?;
        parsed
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or_else(|| BackendError::Body("aucun choix".into()))
    }

    async fn health(&self) -> BackendHealth {
        let ok = self
            .client
            .get(format!("{}/models", self.base_url))
            .timeout(self.health_timeout)
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false);
        BackendHealth { reachable: ok }
    }
}
