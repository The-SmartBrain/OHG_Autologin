#[cfg(target_os = "linux")]
mod platform {
    use std::env;
    use std::fs;
    use std::path::Path;

    const CREDENTIAL_NAME: &str = "ohg-password";

    pub fn store_password(password: &str) -> Result<(), String> {
        Err("Das Erstellen des systemd-Credentials erfolgt über \
             systemd-creds bzw. die Deployment-Installation."
            .to_string())
    }

    pub fn load_password() -> Result<String, String> {
        let directory = env::var("CREDENTIALS_DIRECTORY").map_err(|_| {
            "CREDENTIALS_DIRECTORY ist nicht gesetzt. \
                 Wird das Programm als systemd-Service ausgeführt?"
                .to_string()
        })?;

        let path = Path::new(&directory).join(CREDENTIAL_NAME);

        let password = fs::read_to_string(&path).map_err(|e| {
            format!(
                "systemd-Credential '{}' konnte nicht gelesen werden: {}",
                CREDENTIAL_NAME, e
            )
        })?;

        let password = password.trim_end_matches(['\r', '\n']);

        if password.is_empty() {
            return Err("Das gespeicherte Passwort ist leer.".to_string());
        }

        Ok(password.to_string())
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use windows::Win32::Security::Credentials::{
        CRED_PERSIST_LOCAL_MACHINE, CRED_TYPE_GENERIC, CREDENTIALW, CredDeleteW, CredReadW,
        CredWriteW,
    };
    use windows::core::PCWSTR;

    const TARGET_NAME: &str = "OHG-Autologin";

    pub fn store_password(password: &str) -> Result<(), String> {
        let target: Vec<u16> = TARGET_NAME.encode_utf16().chain(Some(0)).collect();

        let secret: Vec<u8> = password.as_bytes().to_vec();

        let credential = CREDENTIALW {
            Type: CRED_TYPE_GENERIC,
            TargetName: PCWSTR(target.as_ptr()),
            CredentialBlobSize: secret.len() as u32,
            CredentialBlob: secret.as_ptr() as *mut u8,
            Persist: CRED_PERSIST_LOCAL_MACHINE,
            ..Default::default()
        };

        unsafe {
            CredWriteW(&credential, 0).map_err(|e| {
                format!(
                    "Windows Credential Manager konnte das \
                         Passwort nicht speichern: {}",
                    e
                )
            })?;
        }

        Ok(())
    }

    pub fn load_password() -> Result<String, String> {
        let target: Vec<u16> = TARGET_NAME.encode_utf16().chain(Some(0)).collect();

        let mut credential_ptr: *mut CREDENTIALW = std::ptr::null_mut();

        unsafe {
            CredReadW(
                PCWSTR(target.as_ptr()),
                CRED_TYPE_GENERIC,
                0,
                &mut credential_ptr,
            )
            .map_err(|e| {
                format!(
                    "Passwort konnte nicht aus dem Windows Credential \
                     Manager gelesen werden: {}",
                    e
                )
            })?;

            let credential = &*credential_ptr;

            let bytes = std::slice::from_raw_parts(
                credential.CredentialBlob,
                credential.CredentialBlobSize as usize,
            );

            let password = String::from_utf8(bytes.to_vec())
                .map_err(|_| "Das gespeicherte Passwort ist kein gültiges UTF-8.".to_string())?;

            windows::Win32::Security::Credentials::CredFree(credential_ptr as *const _);

            if password.is_empty() {
                return Err("Das gespeicherte Passwort ist leer.".to_string());
            }

            Ok(password)
        }
    }

    pub fn delete_password() -> Result<(), String> {
        let target: Vec<u16> = TARGET_NAME.encode_utf16().chain(Some(0)).collect();

        unsafe {
            CredDeleteW(PCWSTR(target.as_ptr()), CRED_TYPE_GENERIC, 0).map_err(|e| {
                format!(
                    "Passwort konnte nicht aus dem Credential Manager \
                     gelöscht werden: {}",
                    e
                )
            })?;
        }

        Ok(())
    }
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
mod platform {
    pub fn store_password(_: &str) -> Result<(), String> {
        Err("Dieses Betriebssystem wird nicht unterstützt.".to_string())
    }

    pub fn load_password() -> Result<String, String> {
        Err("Dieses Betriebssystem wird nicht unterstützt.".to_string())
    }
}

pub fn store_password(password: &str) -> Result<(), String> {
    platform::store_password(password)
}

pub fn load_password() -> Result<String, String> {
    platform::load_password()
}
