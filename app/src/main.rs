mod auth;
mod config;
mod gui;
mod network;
mod system;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.iter().any(|arg| arg == "--config") {
        if let Err(error) = gui::run_config_window() {
            eprintln!("GUI konnte nicht gestartet werden: {error}");

            std::process::exit(1);
        }

        return;
    }

    if let Err(error) = auth::run_pinger() {
        eprintln!("Pinger-Fehler: {error}");

        std::process::exit(1);
    }
}
