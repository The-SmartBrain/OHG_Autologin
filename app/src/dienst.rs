use std::time::Duration;

use tokio::time::sleep;

use crate::{config, network};

const CONFIG_FILE: &str = "config.json";
const INTERVAL: Duration = Duration::from_secs(30);

pub async fn run() -> Result<(), String> {
    println!("OHG Autologin-Dienst gestartet.");

    loop {
        println!("Prüfe WLAN...");

        if !network::ohg_wifi() {
            println!(
                "Nicht mehr mit dem 'ohg'-WLAN verbunden. \
                 Dienst beendet sich."
            );

            return Ok(());
        }

        println!("Mit 'ohg' verbunden.");

        check_login().await?;

        println!("Nächste Überprüfung in {} Sekunden.", INTERVAL.as_secs());

        sleep(INTERVAL).await;
    }
}

async fn check_login() -> Result<(), String> {
    match network::status().await {
        network::ClientState::Authorized => {
            println!("Bereits authentifiziert.");
        }

        network::ClientState::NotAuthorized => {
            let credentials = config::Config::load(CONFIG_FILE)?;
            println!("Nicht authentifiziert. Starte Login...");

            let status = network::login(&credentials.username, &credentials.password).await;

            match status {
                network::ClientState::Authorized => {
                    println!("Login erfolgreich.");
                }

                network::ClientState::NotAuthorized => {
                    println!("Login fehlgeschlagen.");
                }

                network::ClientState::Unknown => {
                    println!("Login-Status unbekannt.");
                }
            }
        }

        network::ClientState::Unknown => {
            println!("Authentifizierungsstatus unbekannt.");
        }
    }

    Ok(())
}
