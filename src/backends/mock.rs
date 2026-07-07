//! Backend déterministe pour tests/dev.

use super::{Backend, BackendError, BackendHealth, BackendRequest};
use async_trait::async_trait;

pub struct MockBackend {
    response: Result<String, ()>,
    reachable: bool,
}

impl MockBackend {
    pub fn with_response(text: &str) -> Self {
        MockBackend { response: Ok(text.to_string()), reachable: true }
    }
    pub fn failing() -> Self {
        MockBackend { response: Err(()), reachable: false }
    }
}

#[async_trait]
impl Backend for MockBackend {
    async fn complete(&self, _req: &BackendRequest) -> Result<String, BackendError> {
        self.response.clone().map_err(|_| BackendError::Network("mock".into()))
    }
    async fn health(&self) -> BackendHealth {
        BackendHealth { reachable: self.reachable }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backends::{Backend, BackendRequest};

    fn req() -> BackendRequest {
        BackendRequest {
            messages: vec![],
            temperature: 0.15,
            top_p: 0.9,
            max_tokens: 128,
            model: "luciole".into(),
            stop: vec![],
        }
    }

    #[tokio::test]
    async fn mock_renvoie_la_reponse_configuree() {
        let m = MockBackend::with_response("corrigé");
        assert_eq!(m.complete(&req()).await.unwrap(), "corrigé");
        assert!(m.health().await.reachable);
    }
}
