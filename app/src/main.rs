use clap::{Parser, Subcommand};

use app::{dienst, system};

#[derive(Parser)]
#[command(
    name = "ohg-autologin",
    version,
    about = "Automatischer Login in das OHG-WLAN"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Konfigurationsverwaltung
    Config,

    /// Dienst installieren
    Install,

    /// Dienst deinstallieren
    Uninstall,

    /// Automatischen Start aktivieren
    Enable,

    /// Automatischen Start deaktivieren
    Disable,

    /// Dienst jetzt starten
    Start,

    /// Dienst jetzt stoppen
    Stop,

    /// Dienststatus anzeigen
    Status,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Some(Command::Config) => {
            println!("Konfigurations-GUI ist noch nicht implementiert.");
            Ok(())
        }

        Some(Command::Install) => system::install(),

        Some(Command::Uninstall) => system::uninstall(),

        Some(Command::Enable) => system::enable(),

        Some(Command::Disable) => system::disable(),

        Some(Command::Start) => system::start(),

        Some(Command::Stop) => system::stop(),

        Some(Command::Status) => {
            print_status();
            Ok(())
        }

        None => dienst::run().await,
    };

    if let Err(error) = result {
        eprintln!("Fehler: {}", error);
        std::process::exit(1);
    }
}

fn print_status() {
    println!("Installiert: {}", system::is_installed());

    match system::is_enabled() {
        Ok(value) => println!("Aktiviert:   {}", value),
        Err(error) => println!("Aktiviert:   Fehler: {}", error),
    }

    match system::is_running() {
        Ok(value) => println!("Läuft:       {}", value),
        Err(error) => println!("Läuft:       Fehler: {}", error),
    }
}
