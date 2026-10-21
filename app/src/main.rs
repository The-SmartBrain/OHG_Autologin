// Prevents Windows Console window when executing
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::{Parser, Subcommand};
mod dienst;
mod gui;

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
            tokio::task::block_in_place(|| gui::run());
            Ok(())
        }

        None => dienst::run().await,
    };
}
