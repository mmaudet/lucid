//! Client HTTP partagé pour les backends OpenAI-compatibles (llama.cpp, Ollama).

use super::{BackendError, BackendHealth, BackendRequest};
use crate::openai::ChatMessage;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct OpenAiHttp {
    base_url: String,
    model: String,
    client: reqwest::Client,
    health_timeout: Duration,
}

impl OpenAiHttp {
    pub fn new(base_url: &str, model: &str, timeout_ms: u64, health_timeout_ms: u64) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(timeout_ms))
            .build()
            .expect("client reqwest");
        OpenAiHttp {
            base_url: base_url.trim_end_matches('/').to_string(),
            model: model.to_string(),
            client,
            health_timeout: Duration::from_millis(health_timeout_ms),
        }
    }

    pub async fn complete(&self, req: &BackendRequest) -> Result<String, BackendError> {
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

    pub async fn health(&self) -> BackendHealth {
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
