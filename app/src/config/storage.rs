use super::crypto;
use super::model::{CONFIG_VERSION, EncryptedConfig};

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Gibt den Ordner zurück, in dem unsere Anwendung ihre
/// Konfigurationsdateien ablegt.
pub fn get_config_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        PathBuf::from(env::var("APPDATA").expect("APPDATA ist nicht gesetzt")).join("WlanAutologin")
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        PathBuf::from(env::var("HOME").expect("HOME ist nicht gesetzt"))
            .join(".config")
            .join("wlan_autologin")
    }
}

/// Pfad zur verschlüsselten Konfiguration.
pub fn get_config_path() -> PathBuf {
    get_config_dir().join("config.json")
}

/// Pfad zum zufällig erzeugten Master-Key.
pub fn get_key_path() -> PathBuf {
    get_config_dir().join("master.key")
}

/// Lädt den AES-Key oder erzeugt ihn beim ersten Start.
fn get_encryption_key() -> io::Result<aes_gcm::Key<aes_gcm::Aes256Gcm>> {
    let key_path = get_key_path();

    crypto::load_or_create_key(&key_path)
}

/// Liest und entschlüsselt die Konfiguration.
pub fn read_config() -> io::Result<(String, String)> {
    let config_path = get_config_path();

    if !config_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Keine Konfigurationsdatei gefunden",
        ));
    }

    let json = fs::read_to_string(&config_path)?;

    let config: EncryptedConfig = serde_json::from_str(&json).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Ungültige Konfigurationsdatei: {error}"),
        )
    })?;

    // Version überprüfen.
    if config.version != CONFIG_VERSION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Nicht unterstützte Konfigurationsversion: {}",
                config.version
            ),
        ));
    }

    let key = get_encryption_key()?;

    let password = crypto::decrypt(&key, &config.nonce, &config.ciphertext)?;

    let password = String::from_utf8(password).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Passwort enthält ungültige UTF-8-Daten: {error}"),
        )
    })?;

    Ok((config.username, password))
}

/// Verschlüsselt und speichert Benutzername + Passwort.
pub fn save_config(username: &str, password: &str) -> io::Result<()> {
    let config_dir = get_config_dir();

    fs::create_dir_all(&config_dir)?;

    let key = get_encryption_key()?;

    let (nonce, ciphertext) = crypto::encrypt(&key, password.as_bytes())?;

    let config = EncryptedConfig {
        version: CONFIG_VERSION,
        nonce,
        ciphertext,
        username: username.to_string(),
    };

    let json = serde_json::to_string_pretty(&config).map_err(|error| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Konfiguration konnte nicht serialisiert werden: {error}"),
        )
    })?;

    let config_path = get_config_path();

    // Erst in eine temporäre Datei schreiben.
    let temp_path = config_path.with_extension("json.tmp");

    fs::write(&temp_path, json.as_bytes())?;

    restrict_config_permissions(&temp_path)?;

    // Alte Datei ersetzen.
    fs::rename(&temp_path, &config_path)?;

    Ok(())
}

fn restrict_config_permissions(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let permissions = fs::Permissions::from_mode(0o600);

        fs::set_permissions(path, permissions)?;
    }

    Ok(())
}
