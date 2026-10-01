use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

use crate::crypto;

const CURRENT_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub version: u32,
    pub encrypted: bool,
    pub username: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

impl Config {
    pub fn create<P: AsRef<Path>>(
        path: P,
        username: &str,
        password: &str,
        encrypted: bool,
    ) -> Result<(), String> {
        if username.trim().is_empty() {
            return Err("Username darf nicht leer sein.".into());
        }

        if password.is_empty() {
            return Err("Password darf nicht leer sein.".into());
        }

        let stored_password = if encrypted {
            crypto::encrypt_password(password)?
        } else {
            password.to_string()
        };

        let config = Self {
            version: CURRENT_VERSION,
            encrypted,
            username: username.to_string(),
            password: Some(stored_password),
        };

        let json = serde_json::to_string_pretty(&config)
            .map_err(|e| format!("Konfiguration konnte nicht serialisiert werden: {}", e))?;

        fs::write(path, json)
            .map_err(|e| format!("Konfiguration konnte nicht gespeichert werden: {}", e))?;

        Ok(())
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Credentials, String> {
        let raw = fs::read_to_string(path)
            .map_err(|e| format!("Konfiguration konnte nicht gelesen werden: {}", e))?;

        let config: Self =
            serde_json::from_str(&raw).map_err(|e| format!("Ungültiges JSON: {}", e))?;

        if config.version != CURRENT_VERSION {
            return Err(format!(
                "Nicht unterstützte Config-Version: {}",
                config.version
            ));
        }

        if config.username.trim().is_empty() {
            return Err("Username ist leer.".into());
        }

        let password = config
            .password
            .ok_or("Kein Passwort in der Konfiguration vorhanden.")?;

        let password = if config.encrypted {
            crypto::decrypt_password(&password)?
        } else {
            password
        };

        Ok(Credentials {
            username: config.username,
            password,
        })
    }

    pub fn delete<P: AsRef<Path>>(path: P) -> Result<(), String> {
        if path.as_ref().exists() {
            fs::remove_file(path)
                .map_err(|e| format!("Konfiguration konnte nicht gelöscht werden: {}", e))?;
        }

        Ok(())
    }
}
