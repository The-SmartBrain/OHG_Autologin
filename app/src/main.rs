use wifi_scan

fn main() {
    match wifi_scan::scan() {
        Ok(networks) => {
            // Sucht nach dem Netzwerk, mit dem du gerade aktiv verbunden bist
            if let Some(current) = networks.iter().find(|n| n.is_connected) {
                println!("Verbunden mit WLAN: {}", current.ssid);
            } else {
                println!("Mit keinem WLAN verbunden.");
            }
        }
        Err(e) => println!("Fehler beim Auslesen: {:?}", e),
    }
}

