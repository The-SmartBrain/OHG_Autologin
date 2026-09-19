use serde::{Deserialize, Serialize};

pub const CONFIG_VERSION: u8 = 1;

#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptedConfig {
    pub version: u8,
    /// AES-GCM Nonce
    pub nonce: Vec<u8>,

    /// Verschlüsseltes Passwort mit GCM Authentication Tag
    pub ciphertext: Vec<u8>,

    /// Benutzername bleibt unverschlüsselt
    pub username: String,
}
