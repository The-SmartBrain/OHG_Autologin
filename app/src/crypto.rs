use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::RngCore;
use sha2::{Digest, Sha256};

#[cfg(target_os = "linux")]
fn machine_id() -> Result<String, String> {
    std::fs::read_to_string("/etc/machine-id")
        .map(|id| id.trim().to_string())
        .map_err(|e| format!("Machine-ID konnte nicht gelesen werden: {}", e))
}

#[cfg(target_os = "windows")]
fn machine_id() -> Result<String, String> {
    use std::process::Command;

    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "(Get-ItemProperty 'HKLM:\\SOFTWARE\\Microsoft\\Cryptography').MachineGuid",
        ])
        .output()
        .map_err(|e| format!("Windows Machine GUID konnte nicht gelesen werden: {}", e))?;

    if !output.status.success() {
        return Err("Windows Machine GUID konnte nicht gelesen werden.".to_string());
    }

    let id = String::from_utf8(output.stdout)
        .map_err(|_| "Windows Machine GUID ist kein gültiges UTF-8.".to_string())?;

    let id = id.trim();

    if id.is_empty() {
        return Err("Windows Machine GUID ist leer.".to_string());
    }

    Ok(id.to_string())
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn machine_id() -> Result<String, String> {
    Err("Dieses Betriebssystem wird nicht unterstützt.".to_string())
}

/// Erzeugt aus der Machine-ID einen 256-Bit-Key.
fn encryption_key() -> Result<Key<Aes256Gcm>, String> {
    let machine_id = machine_id()?;

    let mut hasher = Sha256::new();

    hasher.update(b"OHG-Autologin-v1");
    hasher.update(machine_id.as_bytes());

    let hash = hasher.finalize();

    Ok(*Key::<Aes256Gcm>::from_slice(&hash))
}

/// Verschlüsselt ein Passwort.
/// Rückgabe: Base64 aus Nonce + Ciphertext + Authentication Tag.
pub fn encrypt_password(password: &str) -> Result<String, String> {
    let key = encryption_key()?;

    let cipher = Aes256Gcm::new(&key);

    // AES-GCM benötigt einen einzigartigen Nonce.
    let mut nonce_bytes = [0u8; 12];

    rand::thread_rng().fill_bytes(&mut nonce_bytes);

    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, password.as_bytes())
        .map_err(|_| "Passwort konnte nicht verschlüsselt werden.".to_string())?;

    // Nonce vorne an den Ciphertext hängen.
    let mut result = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());

    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);

    Ok(BASE64.encode(result))
}

/// Entschlüsselt ein zuvor mit encrypt_password()
/// verschlüsseltes Passwort.
pub fn decrypt_password(encoded: &str) -> Result<String, String> {
    let key = encryption_key()?;

    let encrypted = BASE64
        .decode(encoded)
        .map_err(|e| format!("Ungültiges Base64 im verschlüsselten Passwort: {}", e))?;

    if encrypted.len() < 12 {
        return Err("Verschlüsseltes Passwort ist zu kurz.".to_string());
    }

    let (nonce_bytes, ciphertext) = encrypted.split_at(12);

    let nonce = Nonce::from_slice(nonce_bytes);

    let cipher = Aes256Gcm::new(&key);

    let plaintext = cipher.decrypt(nonce, ciphertext).map_err(|_| {
        "Passwort konnte nicht entschlüsselt werden. \
             Ist dies dieselbe Maschine?"
            .to_string()
    })?;

    String::from_utf8(plaintext)
        .map_err(|_| "Entschlüsseltes Passwort ist kein gültiges UTF-8.".to_string())
}
