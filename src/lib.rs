//! Lucid — noyau headless : API compatible OpenAI + correction FR via Luciole-1B.

pub mod api_info;
pub mod backends;
pub mod config;
pub mod correction;
pub mod dictionary;
pub mod openai;
pub mod runtime;
pub mod server;
pub mod store;
pub mod supervisor;

#[cfg(feature = "gui")]
pub mod app;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Démarre le serveur HTTP local (bloquant jusqu'à Ctrl-C).
pub async fn run_server(cfg: config::Config) -> anyhow::Result<()> {
    let mut mgr = runtime::ServerManager::new(cfg);

    // Affiche l'endpoint + le bearer pour copier-coller dans VoiceInk.
    let info = api_info::api_info(mgr.config());
    println!("Lucid en écoute sur {}", info.base_url);
    match &mgr.config().server.bearer_token {
        Some(t) if !t.is_empty() => println!("Token bearer : {t}"),
        _ => println!("Auth bearer : désactivée"),
    }

    mgr.start().await?;
    tokio::signal::ctrl_c().await?;
    mgr.stop().await;
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
