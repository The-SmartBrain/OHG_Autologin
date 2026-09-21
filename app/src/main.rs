mod network;

#[tokio::main]
async fn main() {
    // 1. Prüfen, ob im richtigen WLAN
    if network::ohg_wifi() {
        let status_url = "http://10.80.0.1:8000/api/captiveportal/access/status/0/";
        let login_url = "http://10.80.0.1:8000/api/captiveportal/access/logon/0/";

        // 2. Aktuellen Status abfragen
        let aktueller_status = network::status(status_url).await;
        println!("Aktueller Status: {:?}", aktueller_status);

        match aktueller_status {
            network::ClientState::Not_Authorized => {
                println!("Nicht eingeloggt! Starte Autologin-Sequenz...");

                let username = "";
                let password = "";

                // 3. Login ausführen
                let login_status = network::login(login_url, username, password).await;
                
                if login_status == network::ClientState::Authorized {
                    println!("Login erfolgreich!");
                } else {
                    println!("Login fehlgeschlagen. Status: {:?}", login_status);
                }
            }
            network::ClientState::Authorized => {
                println!("Bereits erfolgreich authentifiziert.");
            }
            network::ClientState::Unknown => {
                println!("Status unklar..");
            }
        }
    } else {
        println!("Nicht mit dem 'ohg' WLAN verbunden.");
    }
}
