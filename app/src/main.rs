mod config;
mod crypto;
mod dienst;
mod network;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ohg-autologin", version, about = "OHG Autologin")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Konfigurationsverwaltung starten
    Config,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Config) => {
            println!("Config-GUI ");
        }

        None => {
            if let Err(error) = dienst::run().await {
                eprintln!("Fehler: {}", error);
                std::process::exit(1);
            }
        }
    }
}
