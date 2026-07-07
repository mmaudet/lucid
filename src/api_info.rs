//! Info d'intégration (base_url, clé, modèle) réutilisée par le tray, les Réglages
//! et les guides d'intégration. Fonction pure, non gatée (utilisable partout).

use crate::config::Config;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ApiInfo {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

pub fn api_info(cfg: &Config) -> ApiInfo {
    let api_key = cfg
        .server
        .bearer_token
        .clone()
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| "local".to_string());
    ApiInfo {
        base_url: format!("http://{}:{}/v1", cfg.server.host, cfg.server.port),
        api_key,
        model: "lucid".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_info_par_defaut() {
        let info = api_info(&Config::default());
        assert_eq!(info.base_url, "http://127.0.0.1:8790/v1");
        assert_eq!(info.model, "lucid");
        assert_eq!(info.api_key, "local"); // bearer None -> "local"
    }

    #[test]
    fn api_info_avec_bearer() {
        let mut cfg = Config::default();
        cfg.server.bearer_token = Some("secret123".into());
        assert_eq!(api_info(&cfg).api_key, "secret123");
    }
}
