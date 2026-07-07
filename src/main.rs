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
    let cli = Cli::parse();
    match cli.command {
        Command::Serve => println!("serve (stub)"),
        Command::Doctor => println!("doctor (stub)"),
    }
    Ok(())
}
