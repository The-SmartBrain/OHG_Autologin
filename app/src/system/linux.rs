use super::SystemError;

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const SERVICE_NAME: &str = "ohg-wlan-autologin.service";

const SERVICE_PATH: &str = "/etc/systemd/system/ohg-wlan-autologin.service";

fn executable_path() -> Result<PathBuf, SystemError> {
    env::current_exe().map_err(SystemError::Io)
}

fn run_systemctl(args: &[&str]) -> Result<(), SystemError> {
    let output = Command::new("systemctl")
        .args(args)
        .output()
        .map_err(SystemError::Io)?;

    if !output.status.success() {
        return Err(SystemError::CommandFailed(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }

    Ok(())
}

pub fn enable() -> Result<(), SystemError> {
    let exe = executable_path()?;

    let service = format!(
        r#"[Unit]
Description=OHG WLAN Autologin
After=NetworkManager.service network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart={}
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
"#,
        exe.display()
    );

    // Service-Datei schreiben
    // benötigt Root
    fs::write(SERVICE_PATH, service).map_err(SystemError::Io)?;

    run_systemctl(&["daemon-reload"])?;

    run_systemctl(&["enable", SERVICE_NAME])?;

    run_systemctl(&["restart", SERVICE_NAME])?;

    Ok(())
}

pub fn disable() -> Result<(), SystemError> {
    // Service deaktivieren
    let _ = run_systemctl(&["disable", "--now", SERVICE_NAME]);

    // Service-Datei entfernen
    if std::path::Path::new(SERVICE_PATH).exists() {
        fs::remove_file(SERVICE_PATH).map_err(SystemError::Io)?;
    }

    run_systemctl(&["daemon-reload"])?;

    Ok(())
}

pub fn is_enabled() -> Result<bool, SystemError> {
    let output = Command::new("systemctl")
        .args(["is-enabled", SERVICE_NAME])
        .output()
        .map_err(SystemError::Io)?;

    Ok(output.status.success())
}
