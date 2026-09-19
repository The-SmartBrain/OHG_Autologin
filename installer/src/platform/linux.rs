use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

const SERVICE_NAME: &str = "ohg-wlan-autologin.service";

const SERVICE_PATH: &str = "/etc/systemd/system/ohg-wlan-autologin.service";

const INSTALL_DIR: &str = "/opt/ohg-wlan-autologin";

const APP_PATH: &str = "/opt/ohg-wlan-autologin/ohg-wlan-autologin";

const DESKTOP_PATH: &str = "/usr/share/applications/ohg-wlan-autologin.desktop";

pub fn install(app_binary: &[u8]) -> Result<(), String> {
    let install_dir = Path::new(INSTALL_DIR);

    fs::create_dir_all(install_dir).map_err(|e| {
        format!(
            "Installationsverzeichnis konnte \
             nicht erstellt werden: {}",
            e
        )
    })?;

    fs::write(APP_PATH, app_binary)
        .map_err(|e| format!("App konnte nicht installiert werden: {}", e))?;

    fs::set_permissions(APP_PATH, fs::Permissions::from_mode(0o755)).map_err(|e| e.to_string())?;

    write_service()?;
    write_desktop_entry()?;

    run("systemctl", &["daemon-reload"])?;

    run("systemctl", &["enable", "--now", SERVICE_NAME])?;

    Ok(())
}

pub fn uninstall() -> Result<(), String> {
    let _ = Command::new("systemctl")
        .args(["disable", "--now", SERVICE_NAME])
        .status();

    if Path::new(SERVICE_PATH).exists() {
        fs::remove_file(SERVICE_PATH)
            .map_err(|e| format!("Service konnte nicht entfernt werden: {}", e))?;
    }

    if Path::new(DESKTOP_PATH).exists() {
        fs::remove_file(DESKTOP_PATH)
            .map_err(|e| format!("Desktop-Eintrag konnte nicht entfernt werden: {}", e))?;
    }

    let _ = Command::new("systemctl").args(["daemon-reload"]).status();

    if Path::new(INSTALL_DIR).exists() {
        fs::remove_dir_all(INSTALL_DIR).map_err(|e| {
            format!(
                "Installationsverzeichnis konnte \
                 nicht entfernt werden: {}",
                e
            )
        })?;
    }

    Ok(())
}

fn write_service() -> Result<(), String> {
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
        APP_PATH
    );

    fs::write(SERVICE_PATH, service).map_err(|e| {
        format!(
            "systemd-Service konnte nicht \
             geschrieben werden: {}",
            e
        )
    })
}

fn write_desktop_entry() -> Result<(), String> {
    let desktop = format!(
        r#"[Desktop Entry]
Type=Application
Name=OHG WLAN Autologin
Comment=Automatische Anmeldung am OHG WLAN
Exec={} --config
Terminal=false
Categories=Network;
"#,
        APP_PATH
    );

    fs::write(DESKTOP_PATH, desktop).map_err(|e| {
        format!(
            "Desktop-Eintrag konnte nicht \
             geschrieben werden: {}",
            e
        )
    })
}

fn run(program: &str, args: &[&str]) -> Result<(), String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| format!("{} konnte nicht gestartet werden: {}", program, e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    Ok(())
}

pub fn installation_path() -> PathBuf {
    PathBuf::from(INSTALL_DIR)
}
