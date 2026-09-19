use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use rand::{TryRngCore, rngs::OsRng};
use std::fs;
use std::io;
use std::path::Path;

const KEY_SIZE: usize = 32;
const NONCE_SIZE: usize = 12;

// Generiert einen kryptographisch sicheren zufälligen Schlüssel
fn generate_key() -> io::Result<[u8; KEY_SIZE]> {
    let mut key = [0u8; KEY_SIZE];

    OsRng.try_fill_bytes(&mut key).map_err(|error| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Zufallszahlengenerator fehlgeschlagen: {error}"),
        )
    })?;

    Ok(key)
}

// Lädt den Schlüssel aus der Key-Datei.
pub fn load_or_create_key(key_path: &Path) -> io::Result<Key<Aes256Gcm>> {
    if key_path.exists() {
        let key = fs::read(key_path)?;

        if key.len() != KEY_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Ungültige Key-Datei: Schlüssel muss 32 Bytes lang sein",
            ));
        }

        let mut key_bytes = [0u8; KEY_SIZE];
        key_bytes.copy_from_slice(&key);

        return Ok(*Key::<Aes256Gcm>::from_slice(&key_bytes));
    }

    // Übergeordnetes Verzeichnis anlegen
    if let Some(parent) = key_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let key_bytes = generate_key()?;

    // Schlüssel ausschließlich als Binärdaten speichern
    fs::write(key_path, key_bytes)?;

    restrict_key_file_permissions(key_path)?;

    Ok(*Key::<Aes256Gcm>::from_slice(&key_bytes))
}

/// Verschlüsselt Daten mit AES-256-GCM
///
/// Für jede Verschlüsselung gibt es neue zufällige Nonce
///
/// Rückgabe:
///
/// (Nonce, Ciphertext)
pub fn encrypt(key: &Key<Aes256Gcm>, plaintext: &[u8]) -> io::Result<(Vec<u8>, Vec<u8>)> {
    let mut nonce_bytes = [0u8; NONCE_SIZE];

    OsRng.try_fill_bytes(&mut nonce_bytes).map_err(|error| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Zufallszahlengenerator fehlgeschlagen: {error}"),
        )
    })?;

    let nonce = Nonce::from_slice(&nonce_bytes);

    let cipher = Aes256Gcm::new(key);

    let ciphertext = cipher.encrypt(nonce, plaintext).map_err(|error| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Verschlüsselung fehlgeschlagen: {error}"),
        )
    })?;

    Ok((nonce_bytes.to_vec(), ciphertext))
}

/// Entschlüsselt AES-256-GCM-Daten.
pub fn decrypt(key: &Key<Aes256Gcm>, nonce_bytes: &[u8], ciphertext: &[u8]) -> io::Result<Vec<u8>> {
    if nonce_bytes.len() != NONCE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Ungültige Nonce-Länge: {} Bytes", nonce_bytes.len()),
        ));
    }

    let nonce = Nonce::from_slice(nonce_bytes);

    let cipher = Aes256Gcm::new(key);

    cipher.decrypt(nonce, ciphertext).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Entschlüsselung fehlgeschlagen: {error}"),
        )
    })
}

// Setzt unter Unix Dateirechte.
fn restrict_key_file_permissions(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let permissions = fs::Permissions::from_mode(0o600);

        fs::set_permissions(path, permissions)?;
    }

    Ok(())
}
