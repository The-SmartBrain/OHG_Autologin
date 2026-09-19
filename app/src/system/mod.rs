#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
mod macos;

use std::io;

#[derive(Debug)]
pub enum SystemError {
    Io(io::Error),
    CommandFailed(String),
    Unsupported(String),
}

impl From<io::Error> for SystemError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl std::fmt::Display for SystemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => {
                write!(f, "{}", error)
            }

            Self::CommandFailed(error) => {
                write!(f, "{}", error)
            }

            Self::Unsupported(error) => {
                write!(f, "{}", error)
            }
        }
    }
}

impl std::error::Error for SystemError {}

pub fn enable_autostart() -> Result<(), SystemError> {
    #[cfg(target_os = "linux")]
    {
        return linux::enable();
    }

    #[cfg(target_os = "windows")]
    {
        return windows::enable();
    }

    #[cfg(target_os = "macos")]
    {
        return macos::enable();
    }

    #[allow(unreachable_code)]
    Err(SystemError::Unsupported(
        "Autostart wird auf diesem Betriebssystem nicht unterstützt.".to_string(),
    ))
}

pub fn disable_autostart() -> Result<(), SystemError> {
    #[cfg(target_os = "linux")]
    {
        return linux::disable();
    }

    #[cfg(target_os = "windows")]
    {
        return windows::disable();
    }

    #[cfg(target_os = "macos")]
    {
        return macos::disable();
    }

    #[allow(unreachable_code)]
    Err(SystemError::Unsupported(
        "Autostart wird auf diesem Betriebssystem nicht unterstützt.".to_string(),
    ))
}

pub fn is_autostart_enabled() -> Result<bool, SystemError> {
    #[cfg(target_os = "linux")]
    {
        return linux::is_enabled();
    }

    #[cfg(target_os = "windows")]
    {
        return windows::is_enabled();
    }

    #[cfg(target_os = "macos")]
    {
        return macos::is_enabled();
    }

    #[allow(unreachable_code)]
    Err(SystemError::Unsupported(
        "Autostart wird auf diesem Betriebssystem nicht unterstützt.".to_string(),
    ))
}

pub fn uninstall() -> Result<(), SystemError> {
    // Autostart entfernen
    disable_autostart()?;

    // Konfigurationsverzeichnis entfernen
    let config_dir = crate::config::get_config_dir();

    if config_dir.exists() {
        std::fs::remove_dir_all(config_dir)?;
    }

    Ok(())
}
