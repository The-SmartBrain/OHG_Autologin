use clap::{Parser, Subcommand};
mod dienst;

#[derive(Parser)]
#[command(
    name = "ohg-autologin",
    version,
    about = "Automatischer Login ins Schulwlan"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    //Konfigurationsverwaltung
    Config,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Some(Command::Config) => {
            //Configwindow
            Ok(())
        }

        None => dienst::run().await,
    };
}
