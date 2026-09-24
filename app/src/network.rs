use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")] 
pub enum ClientState {
    Authorized,
    Not_Authorized,
    Unknown,
}

#[derive(Deserialize)]
struct ApiResponse {
    #[serde(rename = "clientState")]
    client_state: ClientState,
}

/// Aktuell mit WLAN "ohg" verbunden?
pub fn ohg_wifi() -> bool {
    println!("Prüfe aktuell verbundenes WLAN...");

    match ssid::get_ssid() {
        Some(aktuelle_ssid) => {
            println!("Aktiv verbunden mit: '{}'", aktuelle_ssid);
            if aktuelle_ssid == "ohg" {
                println!("Ziel-Netzwerk erkannt!");
                true
            } else {
                println!("Mit einem anderen Netzwerk verbunden. Kein Autologin nötig.");
                false
            }
        }
        None => {
            println!("Fehler: Das System ist aktuell mit keinem WLAN-Netzwerk verbunden.");
            false
        }
    }
}

/// Sendet eine POST-Anfrage mit leeren Benutzerdaten an die Status-URL
pub async fn status(url: &str) -> ClientState {
    if cfg!(debug_assertions) {
        println!("DEBUG: Sende Status-POST-Abfrage an: {}", url);
    }

    let client = reqwest::Client::new();

    let form_data = [("user", ""), ("password", "")];

    match client.post(url).form(&form_data).send().await {
        Ok(response) => {
            match response.text().await {
                Ok(raw_text) => {
                    if cfg!(debug_assertions) {
                        println!("DEBUG - Rohe Server-Antwort: {}", raw_text);
                    }

                    match serde_json::from_str::<ApiResponse>(&raw_text) {
                        Ok(api_data) => api_data.client_state,
                        Err(e) => {
                            eprintln!("Fehler beim Parsen des JSON-Status: {}", e);
                            ClientState::Unknown
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Konnte Antwort-Text nicht lesen: {}", e);
                    ClientState::Unknown
                }
            }
        }
        Err(e) => {
            eprintln!("HTTP-Verbindungsfehler bei POST-Status-Abfrage: {}", e);
            ClientState::Unknown
        }
    }
}

/// Sendet eine POST-Anfrage mit den Zugangsdaten
/// Gibt den neuen `ClientState` nach dem Anmeldeversuch zurück.
pub async fn login(url: &str, benutzername: &str, passwort: &str) -> ClientState {
    if cfg!(debug_assertions) {
        println!("DEBUG: Sende Login-POST-Anfrage an: {}", url);
    }

    let custom_user_agent = "TSB: Autologin";

    let client = match reqwest::Client::builder()
        .user_agent(custom_user_agent)
        .danger_accept_invalid_certs(true) // Ohne gültiges Zertifikat
        .build() 
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Fehler beim Erstellen des HTTP-Clients: {}", e);
            return ClientState::Unknown;
        }
    };

    // Die Formulardaten für das OPNsense-Portal
    let form_data = [
        ("user", benutzername),
        ("password", passwort)
    ];

    match client.post(url).form(&form_data).send().await {
        Ok(response) => {
            match response.text().await {
                Ok(raw_text) => {
                    if cfg!(debug_assertions) {
                        println!("DEBUG - Rohe Login-Antwort: {}", raw_text);
                    }

                    match serde_json::from_str::<ApiResponse>(&raw_text) {
                        Ok(api_data) => api_data.client_state,
                        Err(e) => {
                            eprintln!("Fehler beim Parsen der Login-Antwort: {}", e);
                            ClientState::Unknown
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Konnte Login-Antworttext nicht lesen: {}", e);
                    ClientState::Unknown
                }
            }
        }
        Err(e) => {
            eprintln!("HTTP-Verbindungsfehler beim Login-POST: {}", e);
            ClientState::Unknown
        }
    }
}


