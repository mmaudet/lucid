use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "lucid", about = "Correcteur de dictée FR, 100% local (API OpenAI)")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Démarre le serveur HTTP local
    Serve,
    /// Affiche la config résolue et teste le backend
    Doctor,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "lucid=info".into()),
        )
        .init();

    let cli = Cli::parse();
    let cfg = lucid::config::Config::load()?;
    match cli.command {
        Command::Serve => lucid::run_server(cfg).await,
        Command::Doctor => lucid::run_doctor(cfg).await,
    }
}
