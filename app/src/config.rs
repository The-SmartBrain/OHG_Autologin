use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::crypto;

const CURRENT_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub version: u32,

    /// true  = Passwort wird im Secret-Store gespeichert
    /// false = Passwort steht direkt in Datei
    pub encrypted: bool,

    pub username: String,

    /// Wird nur verwendet, wenn encrypted == false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

#[derive(Debug)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

impl Config {
    /// Lädt config.json und ermittelt automatisch das Passwort.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Credentials, String> {
        let raw = fs::read_to_string(path.as_ref()).map_err(|e| {
            format!(
                "Konfiguration '{}' konnte nicht gelesen werden: {}",
                path.as_ref().display(),
                e
            )
        })?;

        let config: Config = serde_json::from_str(&raw)
            .map_err(|e| format!("Ungültiges JSON in der Konfiguration: {}", e))?;

        if config.version != CURRENT_VERSION {
            return Err(format!(
                "Nicht unterstützte Konfigurationsversion: {}",
                config.version
            ));
        }

        if config.username.trim().is_empty() {
            return Err("Username darf nicht leer sein.".to_string());
        }

        let password = if config.encrypted {
            // Passwort aus dem Secret-Store
            crypto::load_password()?
        } else {
            match config.password {
                Some(password) if !password.is_empty() => password,

                Some(_) => {
                    return Err("Password in config.json ist leer.".to_string());
                }

                None => {
                    return Err("encrypted=false, aber kein Password vorhanden.".to_string());
                }
            }
        };

        Ok(Credentials {
            username: config.username,
            password,
        })
    }

    /// Erstellt eine neue Konfiguration.
    ///
    /// Bei encrypted=true wird das Passwort im OS-Secret-Store
    /// gespeichert und NICHT in config.json geschrieben.
    pub fn create<P: AsRef<Path>>(
        path: P,
        username: &str,
        password: &str,
        encrypted: bool,
    ) -> Result<(), String> {
        if username.trim().is_empty() {
            return Err("Username darf nicht leer sein.".to_string());
        }

        if password.is_empty() {
            return Err("Password darf nicht leer sein.".to_string());
        }

        let config = if encrypted {
            // Passwort im Betriebssystem speichern.
            crypto::store_password(password)?;

            Config {
                version: CURRENT_VERSION,
                encrypted: true,
                username: username.to_string(),
                password: None,
            }
        } else {
            Config {
                version: CURRENT_VERSION,
                encrypted: false,
                username: username.to_string(),
                password: Some(password.to_string()),
            }
        };

        let json = serde_json::to_string_pretty(&config)
            .map_err(|e| format!("Konfiguration konnte nicht serialisiert werden: {}", e))?;

        fs::write(path.as_ref(), json).map_err(|e| {
            format!(
                "Konfiguration '{}' konnte nicht geschrieben werden: {}",
                path.as_ref().display(),
                e
            )
        })?;

        Ok(())
    }
}
