use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "lucid", about = "Correcteur de dictée FR, 100% local (API OpenAI)")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Démarre le serveur HTTP local (mode headless)
    Serve,
    /// Affiche la config résolue et teste le backend
    Doctor,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "lucid=info".into()),
        )
        .init();

    let cli = Cli::parse();
    let cfg = lucid::config::Config::load()?;

    match cli.command {
        Some(Command::Serve) => block_on(lucid::run_server(cfg)),
        Some(Command::Doctor) => block_on(lucid::run_doctor(cfg)),
        // Pas de sous-commande : lancer l'app barre de menus (Tauri possède la boucle).
        None => run_app(cfg),
    }
}

#[cfg(feature = "gui")]
fn run_app(cfg: lucid::config::Config) -> anyhow::Result<()> {
    lucid::app::run(cfg)
}

#[cfg(not(feature = "gui"))]
fn run_app(_cfg: lucid::config::Config) -> anyhow::Result<()> {
    println!("Lucid (build headless) — utilisez `lucid serve` ou `lucid doctor`.");
    println!("Pour l'interface barre de menus : construire avec `--features gui`.");
    Ok(())
}

/// Exécute un futur async sans #[tokio::main] (Tauri doit posséder la boucle macOS).
fn block_on<F: std::future::Future<Output = anyhow::Result<()>>>(fut: F) -> anyhow::Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(fut)
}
