mod config;
mod crypto;
mod network;

#[tokio::main]
async fn main() {
    let credentials = match config::Config::load("config.json") {
        Ok(credentials) => credentials,

        Err(e) => {
            eprintln!("Konfigurationsfehler: {}", e);
            return;
        }
    };

    dbg!(
        "Konfiguration geladen für Benutzer '{}'",
        &credentials.username
    );

    if network::ohg_wifi() {
        let aktueller_status = network::status().await;

        dbg!("Aktueller Status: {:?}", &aktueller_status);

        match aktueller_status {
            network::ClientState::NotAuthorized => {
                dbg!("Nicht eingeloggt! Starte Autologin...");

                let login_status =
                    network::login(&credentials.username, &credentials.password).await;

                if login_status == network::ClientState::Authorized {
                    dbg!("Login erfolgreich!");
                } else {
                    dbg!("Login fehlgeschlagen: {:?}", login_status);
                }
            }

            network::ClientState::Authorized => {
                dbg!("Bereits erfolgreich authentifiziert.");
            }

            network::ClientState::Unknown => {
                dbg!("Status unklar.");
            }
        }
    } else {
        dbg!("Nicht mit dem 'ohg' WLAN verbunden.");
    }
}
