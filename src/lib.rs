//! Lucid — noyau headless : API compatible OpenAI + correction FR via Luciole-1B.

pub mod backends;
pub mod config;
pub mod correction;
pub mod dictionary;
pub mod openai;
pub mod server;

use std::sync::Arc;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Démarre le serveur HTTP local (bloquant).
pub async fn run_server(cfg: config::Config) -> anyhow::Result<()> {
    let dict_path = config::dictionary_path()?;
    let dictionary = Arc::new(dictionary::DictionaryStore::load(&dict_path));
    let backend = backends::from_config(&cfg.backend);
    let addr = format!("{}:{}", cfg.server.host, cfg.server.port);

    // Affiche l'endpoint + le bearer pour copier-coller dans VoiceInk.
    println!("Lucid en écoute sur http://{addr}/v1");
    match &cfg.server.bearer_token {
        Some(t) if !t.is_empty() => println!("Token bearer : {t}"),
        _ => println!("Auth bearer : désactivée"),
    }

    let state = server::AppState {
        config: Arc::new(cfg),
        backend,
        dictionary,
    };
    let app = server::build_app(state);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

/// Affiche la config résolue et teste la joignabilité du backend.
pub async fn run_doctor(cfg: config::Config) -> anyhow::Result<()> {
    println!("Config résolue :\n{}", toml::to_string_pretty(&cfg)?);
    let backend = backends::from_config(&cfg.backend);
    let h = backend.health().await;
    println!(
        "Backend {} @ {} : {}",
        cfg.backend.kind,
        cfg.backend.base_url,
        if h.reachable { "joignable" } else { "INJOIGNABLE" }
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_non_vide() {
        assert!(!version().is_empty());
    }
}
