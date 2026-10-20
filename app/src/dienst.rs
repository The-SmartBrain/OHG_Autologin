use std::time::Duration;

use tokio::time::sleep;

#[path = "config.rs"]
mod config;
#[path = "network.rs"]
mod network;

const CONFIG_FILE: &str = "config.json";
const INTERVAL: Duration = Duration::from_secs(30);

pub async fn run() -> Result<(), String> {
    loop {
        /*  if !network::ohg_wifi() {
                    return Ok(());
                }
        */
        check_and_login().await?;
        sleep(INTERVAL).await;
    }
}

async fn check_and_login() -> Result<(), String> {
    match network::status().await {
        network::ClientState::Authorized => {
            println!("Bereits angemeldet.");
        }

        network::ClientState::NotAuthorized => {
            println!("Nicht angemeldet. Versuche Login...");

            let credentials = config::Config::load(CONFIG_FILE)?;

            match network::login(&credentials.username, &credentials.password).await {
                network::ClientState::Authorized => {
                    println!("Login erfolgreich.");
                }

                network::ClientState::NotAuthorized => {
                    return Err("Login fehlgeschlagen: Nicht autorisiert.".to_string());
                }

                network::ClientState::Unknown => {
                    return Err("Login fehlgeschlagen: Status unbekannt.".to_string());
                }
            }
        }

        network::ClientState::Unknown => {
            return Err("Netzwerkstatus konnte nicht ermittelt werden.".to_string());
        }
    }

    Ok(())
}
