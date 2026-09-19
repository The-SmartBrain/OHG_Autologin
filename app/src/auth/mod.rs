mod authorizer;

use std::fs;
use std::thread;
use std::time::{Duration, SystemTime};

use authorizer::OhgWifiAuthorizer;

pub fn run_pinger() -> std::io::Result<()> {
    println!("Initialisiere Pinger für Internetüberprüfung");

    let config_path = crate::config::get_config_path();

    let mut authorizer = OhgWifiAuthorizer::new();

    let mut last_modified = match authorizer.load_config() {
        Ok(()) => fs::metadata(&config_path)
            .and_then(|metadata| metadata.modified())
            .unwrap_or_else(|_| SystemTime::now()),

        Err(error) => {
            eprintln!("Konfiguration konnte nicht geladen werden: {error}");

            SystemTime::now()
        }
    };

    loop {
        if let Ok(metadata) = fs::metadata(&config_path) {
            if let Ok(modified) = metadata.modified() {
                if modified > last_modified {
                    println!("Konfiguration wurde geändert.");

                    if let Err(error) = authorizer.load_config() {
                        eprintln!("Neue Konfiguration konnte nicht geladen werden: {error}");
                    }

                    last_modified = modified;
                }
            }
        }

        let ssid = crate::network::get_current_wifi_ssid();

        if ssid.as_deref() == Some("ohg") {
            if !crate::network::check_internet() {
                authorizer.authorize();
            }
        } else {
            println!("Nicht mehr mit WLAN verbunden. Beende Programm.");

            return Ok(());
        }

        thread::sleep(Duration::from_secs(30));
    }
}
