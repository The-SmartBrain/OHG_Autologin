use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use std::{env, fs, thread};
use ureq::Agent;

// Struktur für verschlüsselte JSON-Datei
#[derive(Serialize, Deserialize)]
struct EncryptedConfig {
    nonce: Vec<u8>,
    ciphertext: Vec<u8>,
    username: String,
}

// Haupt-Strukt
pub struct OhgWifiAuthorizer {
    pub username: Option<String>,
    pub password: Option<String>,
}

impl OhgWifiAuthorizer {
    // Erstellt leere Instanz im RAM
    pub fn new() -> Self {
        Self {
            username: None,
            password: None,
        }
    }

    //Lädt die Daten aus der Datei und entschlüsselt das Passwort per Hardware-ID in den RAM
    pub fn read_config(&mut self) -> std::io::Result<()> {
        let config_path = get_config_path();
        if !config_path.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Keine Konfigurationsdatei gefunden",
            ));
        }

        let json = fs::read_to_string(config_path)?;
        let config_data: EncryptedConfig = serde_json::from_str(&json)?;

        //Hardwareschlüssel generieren
        let key = get_encryption_key();
        let cipher = Aes256Gcm::new(&key);
        let nonce = Nonce::from_slice(&config_data.nonce);

        // AES-GCM Entschlüsselung
        let decrypted_bytes = cipher
            .decrypt(nonce, config_data.ciphertext.as_slice())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

        let password = String::from_utf8(decrypted_bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

        // Daten im RAM ablegen/speichern
        self.username = Some(config_data.username);
        self.password = Some(password);
        Ok(())
    }

    pub fn authorize(&self) -> bool {
        if self.username.is_none() || self.password.is_none() {
            eprintln!("Error: Keine Zugangsdaten im RAM .");
            return false;
        }

        let url = "https://10.80.0.1/api/captiveportal/access/logon/0";

        // ureq-Agent
        // Akzeptiert selbstsignierte Zertifikate
        let config = ureq::config::Config::builder()
            .tls_config(
                ureq::tls::TlsConfig::builder()
                    .disable_verification(true)
                    .build(),
            )
            .timeout_global(Some(Duration::from_secs(5)))
            .build();

        // Erstellt den Agenten aus der Konfiguration
        let agent = ureq::Agent::from(config);

        // Sendet Request über Agenten
        let response = agent.post(url).send_form(vec![
            ("user", self.username.as_ref().unwrap().as_str()),
            ("password", self.password.as_ref().unwrap().as_str()),
        ]);

        match response {
            Ok(res) => {
                if let Ok(body) = res.into_body().read_to_string() {
                    if body.contains("\"AUTHORIZED\"") {
                        println!("Authentication was successful!");
                        return true;
                    } else if body.contains("\"NOT_AUTHORIZED\"") {
                        eprintln!("Authentication failed by ohg_login.rs");
                    }
                }
                false
            }
            Err(e) => {
                eprintln!("Netzwerkfehler bei der Autorisierung: {}", e);
                false
            }
        }
    }
}

//Wloan-Check
fn get_current_wifi_ssid() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        get_windows_wifi()
    }
    #[cfg(target_os = "macos")]
    {
        get_macos_wifi()
    }
    #[cfg(target_os = "linux")]
    {
        get_linux_wifi()
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        None
    }
}

// WINDOWS IMPLEMENTIERUNG (Über Network List Manager)
#[cfg(target_os = "windows")]
fn get_windows_wifi() -> Option<String> {
    use windows::Win32::Networking::NetworkListManager::{
        CoCreateInstance, INetworkListManager, NLM_CONNECTIVITY, NLM_NETWORK_CATEGORY,
        NetworkListManager,
    };
    use windows::Win32::System::Com::{COINIT_MULTITHREADED, CoInitializeEx, CoUninitialize};

    unsafe {
        // COM-Bibliothek für den aktuellen Thread initialisieren
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

        // Network List Manager instanziieren
        if let Ok(manager) = CoCreateInstance::<_, INetworkListManager>(
            &NetworkListManager,
            None,
            windows::Win32::System::Com::CLSCTX_ALL,
        ) {
            // Alle aktuell verbundenen Netzwerke abfragen
            if let Ok(networks) = manager.GetNetworks(
                windows::Win32::Networking::NetworkListManager::NLM_ENUM_NETWORK_CONNECTED,
            ) {
                loop {
                    let mut fetched = 0;
                    let mut network = None;
                    if networks.Next(&mut network, &mut fetched).is_ok() && fetched == 1 {
                        if let Some(net) = network {
                            if let Ok(name) = net.GetName() {
                                let ssid = name.to_string();
                                if !ssid.is_empty() {
                                    CoUninitialize();
                                    return Some(ssid);
                                }
                            }
                        }
                    } else {
                        break;
                    }
                }
            }
        }
        CoUninitialize();
    }
    None
}

// MACOS IMPLEMENTIERUNG (Über CoreWLAN Framework)
#[cfg(target_os = "macos")]
fn get_macos_wifi() -> Option<String> {
    use objc2_core_wlan::CWWiFiClient;

    // Erstellt den WiFi-Client von macOS
    let client = CWWiFiClient::sharedWiFiClient();

    if let Some(interface) = client.interface() {
        if let Some(ns_ssid) = unsafe { interface.ssid() } {
            return Some(ns_ssid.to_string());
        }
    }
    None
}

// LINUX IMPLEMENTIERUNG (Über D-Bus & NetworkManager)
#[cfg(target_os = "linux")]
fn get_linux_wifi() -> Option<String> {
    use zbus::blocking::Connection;
    use zbus::zvariant::OwnedObjectPath;

    let conn = Connection::system().ok()?;

    // 1. Fragt alle aktiven Verbindungen ab
    let active_conns: Vec<OwnedObjectPath> = conn
        .call_method(
            Some("org.freedesktop.NetworkManager"),
            "/org/freedesktop/NetworkManager",
            Some("org.freedesktop.DBus.Properties"),
            "Get",
            &("org.freedesktop.NetworkManager", "ActiveConnections"),
        )
        .ok()?
        .body()
        .deserialize()
        .ok()?;

    // 2. Verbindungen filtern und SSID auslesen
    for path in active_conns {
        let conn_type: String = conn
            .call_method(
                Some("org.freedesktop.NetworkManager"),
                &path,
                Some("org.freedesktop.DBus.Properties"),
                "Get",
                &("org.freedesktop.NetworkManager.Connection.Active", "Type"),
            )
            .ok()?
            .body()
            .deserialize()
            .ok()?;

        // Wenn Typ WLAN,  "Id" = SSID
        if conn_type == "802-11-wireless" {
            let ssid: String = conn
                .call_method(
                    Some("org.freedesktop.NetworkManager"),
                    &path,
                    Some("org.freedesktop.DBus.Properties"),
                    "Get",
                    &("org.freedesktop.NetworkManager.Connection.Active", "Id"),
                )
                .ok()?
                .body()
                .deserialize()
                .ok()?;

            if !ssid.is_empty() {
                return Some(ssid);
            }
        }
    }
    None
}

//Hintergrund-Schleife
fn run_pinger() {
    println!("Initialisiere Pinger-Skript für Internetüberprüfung");

    let config_path = get_config_path();

    let mut authorizer = OhgWifiAuthorizer::new();

    //Initialisieren und Ladevorgang in den RAM
    let mut last_modified = if authorizer.read_config().is_ok() {
        fs::metadata(&config_path)
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::now())
    } else {
        SystemTime::now()
    };

    loop {
        //Überwachung Datei-Zeitstempel
        if let Ok(metadata) = fs::metadata(&config_path) {
            if let Ok(current_modified) = metadata.modified() {
                if current_modified > last_modified {
                    println!("Konfiguration wurde geändert!");
                    last_modified = if authorizer.read_config().is_ok() {
                        fs::metadata(&config_path)
                            .and_then(|m| m.modified())
                            .unwrap_or(SystemTime::now())
                    } else {
                        SystemTime::now()
                    };
                }
            }
        }

        //WLAN-Prüfung
        if get_current_wifi_ssid().as_deref() == Some("ohg") && !check_internet() {
            authorizer.authorize();
        } else {
            println!("Nicht mehr mit WLAN verbunden. Beende Programm");
            std::process::exit(0);
        }
        thread::sleep(Duration::from_secs(30));
    }
}
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.contains(&"--config".to_string()) {
        println!("GUI-Konfiguration wird gestartet");
    } else {
        run_pinger();
    }
}

//Hilfsfunktionen
fn check_internet() -> bool {
    let config = ureq::config::Config::builder()
        .tls_config(
            ureq::tls::TlsConfig::builder()
                .disable_verification(false)
                .build(),
        )
        .max_redirects(0)
        .timeout_global(Some(Duration::from_secs(5)))
        .build();

    let agent = Agent::from(config);

    // Anfrage an Google
    match agent.get("https://google.com").call() {
        Ok(response) => {
            // NurStatus 200
            response.status() == 200
        }
        Err(_) => {
            // Alle Timeouts, DNS-Fehler, HTTP-Umleitungen und SSL-Zertifikatsfehler
            false
        }
    }
}

//Generiert encryption Schlüssel
fn get_encryption_key() -> Key<Aes256Gcm> {
    let mut key_bytes = [0u8; 32];

    #[cfg(target_os = "linux")]
    {
        if let Ok(id) = fs::read_to_string("/etc/machine-id") {
            let len = id.trim().len().min(32);
            key_bytes[..len].copy_from_slice(&id.trim().as_bytes()[..len]);
        }
    }

    #[cfg(target_os = "windows")]
    {
        let name = env::var("COMPUTERNAME").unwrap_or_else(|_| "WindowsFallback".to_string());
        let len = name.len().min(32);
        key_bytes[..len].copy_from_slice(&name.as_bytes()[..len]);
    }
    #[cfg(target_os = "macos")]
    {
        let name = env::var("HOME").unwrap_or_default();
        let len = name.len().min(32);
        key_bytes[..len].copy_from_slice(&name.as_bytes()[..len]);
    }

    *Key::<Aes256Gcm>::from_slice(&key_bytes)
}

//config-Pfad nach Betriebssystem
fn get_config_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        PathBuf::from(env::var("APPDATA").unwrap())
            .join("WlanAutologin")
            .join("config.json")
    }
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        PathBuf::from(env::var("HOME").unwrap())
            .join(".config")
            .join("wlan_autologin")
            .join("config.json")
    }
}
