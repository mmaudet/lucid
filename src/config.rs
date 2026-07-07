//! Chargement de configuration : defaults -> fichier TOML -> surcharges LUCID_*.

use anyhow::Context;
use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub backend: BackendConfig,
    pub correction: CorrectionConfig,
    #[serde(default)]
    pub journal: JournalConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalConfig {
    /// Journalisation active.
    pub enabled: bool,
    /// Stocker le texte des dictées (sinon : métadonnées seules).
    pub store_text: bool,
    /// Rétention en jours (0 = illimité) ; purge au démarrage.
    pub retention_days: u32,
}

impl Default for JournalConfig {
    fn default() -> Self {
        // Décision produit : texte activé par défaut (100% local), rétention 30j.
        JournalConfig { enabled: true, store_text: true, retention_days: 30 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    /// None -> auto-généré au 1er lancement ; Some("") -> auth désactivée.
    pub bearer_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    pub kind: String,
    pub base_url: String,
    pub model: String,
    pub timeout_ms: u64,
    pub health_timeout_ms: u64,
    /// Lucid supervise-t-il le process backend (ollama serve / llama-server) ?
    #[serde(default)]
    pub auto_start: bool,
    /// Chemin du GGUF (llama-server) pour l'auto-start.
    #[serde(default)]
    pub model_path: Option<String>,
    /// Commande de lancement explicite (sinon déduite du kind).
    #[serde(default)]
    pub launch_command: Option<String>,
    #[serde(default)]
    pub launch_args: Vec<String>,
}

impl BackendConfig {
    pub fn default_base_url(kind: &str) -> &'static str {
        match kind {
            "ollama" => "http://127.0.0.1:11434/v1",
            _ => "http://127.0.0.1:8080/v1",
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.base_url.trim().is_empty() {
            return Err("base_url du backend vide".into());
        }
        if !matches!(self.kind.as_str(), "ollama" | "llamacpp") {
            return Err(format!("backend inconnu : {}", self.kind));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectionConfig {
    pub prompt_mode: PromptMode,
    pub temperature: f32,
    pub top_p: f32,
    pub max_output_ratio: f32,
    pub dict_token_budget: usize,
    /// Séquences d'arrêt passées au backend (jetons ChatML par défaut).
    #[serde(default)]
    pub stop: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PromptMode {
    Override,
    Prepend,
    Passthrough,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            server: ServerConfig {
                host: "127.0.0.1".into(),
                port: 8790,
                bearer_token: None,
            },
            backend: BackendConfig {
                kind: "llamacpp".into(),
                base_url: "http://127.0.0.1:8080/v1".into(),
                model: "luciole".into(),
                timeout_ms: 60_000,
                health_timeout_ms: 2_000,
                auto_start: false,
                model_path: None,
                launch_command: None,
                launch_args: vec![],
            },
            correction: CorrectionConfig {
                prompt_mode: PromptMode::Override,
                temperature: 0.15,
                top_p: 0.9,
                max_output_ratio: 2.0,
                dict_token_budget: 256,
                stop: vec!["<|im_start|>".into(), "<|im_end|>".into()],
            },
            journal: JournalConfig::default(),
        }
    }
}

/// ~/Library/Application Support/Lucid
pub fn support_dir() -> anyhow::Result<PathBuf> {
    let base = dirs::data_dir().context("dossier de support introuvable")?;
    Ok(base.join("Lucid"))
}

pub fn config_path() -> anyhow::Result<PathBuf> {
    Ok(support_dir()?.join("config.toml"))
}

pub fn dictionary_path() -> anyhow::Result<PathBuf> {
    Ok(support_dir()?.join("dictionary.json"))
}

pub fn journal_path() -> anyhow::Result<PathBuf> {
    Ok(support_dir()?.join("journal.sqlite"))
}

impl Config {
    /// Construit depuis defaults + fichier optionnel + env (testable).
    pub fn from_figment(file: Option<PathBuf>) -> anyhow::Result<Config> {
        let mut fig = Figment::from(Serialized::defaults(Config::default()));
        if let Some(path) = file {
            if path.exists() {
                fig = fig.merge(Toml::file(path));
            }
        }
        fig = fig.merge(Env::prefixed("LUCID_").split("__"));
        Ok(fig.extract()?)
    }

    /// Chargement réel : lit le fichier de support, génère+persiste le bearer si besoin.
    pub fn load() -> anyhow::Result<Config> {
        let path = config_path()?;
        let existait = path.exists();
        let mut cfg = Config::from_figment(Some(path.clone()))?;
        ensure_bearer(&mut cfg);
        if !existait {
            // 1er lancement : persiste la config (dont le bearer).
            std::fs::create_dir_all(support_dir()?)?;
            std::fs::write(&path, toml::to_string_pretty(&cfg)?)?;
        }
        Ok(cfg)
    }

    /// Persiste la config dans config.toml (écriture atomique tmp+rename).
    pub fn save(&self) -> anyhow::Result<()> {
        let path = config_path()?;
        std::fs::create_dir_all(support_dir()?)?;
        let content = toml::to_string_pretty(self)?;
        let tmp = path.with_extension("toml.tmp");
        std::fs::write(&tmp, content)?;
        std::fs::rename(&tmp, &path)?;
        Ok(())
    }
}

/// Génère un jeton bearer si None (aléatoire alphanumérique 40).
pub fn ensure_bearer(cfg: &mut Config) {
    if cfg.server.bearer_token.is_none() {
        use rand::{distributions::Alphanumeric, Rng};
        let tok: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(40)
            .map(char::from)
            .collect();
        cfg.server.bearer_token = Some(tok);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_conformes_prd() {
        let c = Config::default();
        assert_eq!(c.server.host, "127.0.0.1");
        assert_eq!(c.server.port, 8790);
        assert_eq!(c.backend.kind, "llamacpp");
        assert_eq!(c.backend.base_url, "http://127.0.0.1:8080/v1");
        assert_eq!(c.correction.prompt_mode, PromptMode::Override);
        assert!((c.correction.temperature - 0.15).abs() < 1e-6);
        assert!((c.correction.max_output_ratio - 2.0).abs() < 1e-6);
    }

    #[test]
    fn env_surcharge_le_port() {
        // figment: LUCID_SERVER__PORT surcharge server.port
        temp_env::with_var("LUCID_SERVER__PORT", Some("9999"), || {
            let c = Config::from_figment(None).unwrap();
            assert_eq!(c.server.port, 9999);
        });
    }

    #[test]
    fn config_roundtrip_toml() {
        // La config (dont [journal] et les champs backend) survit à un aller-retour TOML.
        let c = Config::default();
        let s = toml::to_string_pretty(&c).unwrap();
        let back: Config = toml::from_str(&s).unwrap();
        assert_eq!(back.server.port, c.server.port);
        assert_eq!(back.backend.kind, c.backend.kind);
        assert_eq!(back.journal.store_text, c.journal.store_text);
        assert_eq!(back.journal.retention_days, c.journal.retention_days);
    }

    #[test]
    fn ensure_bearer_genere_si_absent() {
        let mut c = Config::default();
        c.server.bearer_token = None;
        ensure_bearer(&mut c);
        assert!(c.server.bearer_token.as_ref().unwrap().len() >= 32);
    }
}
