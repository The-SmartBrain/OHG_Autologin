use crate::{config, network};

const CONFIG_FILE: &str = "config.json";

pub async fn run() -> Result<(), String> {
    let credentials = config::Config::load(CONFIG_FILE)?;

    if !network::ohg_wifi() {
        return Ok(());
    }

    match network::status().await {
        network::ClientState::NotAuthorized => {
            let status = network::login(&credentials.username, &credentials.password).await;

            if status != network::ClientState::Authorized {
                return Err(format!("Login fehlgeschlagen: {:?}", status));
            }
        }

        network::ClientState::Authorized => {}

        network::ClientState::Unknown => {
            return Err("Authentifizierungsstatus konnte nicht ermittelt werden.".into());
        }
    }

    Ok(())
}
