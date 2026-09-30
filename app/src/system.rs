#[cfg(target_os = "linux")]
mod platform {
    use std::{env, fs, path::PathBuf, process::Command};

    const SERVICE_NAME: &str = "ohg-autologin.service";

    fn service_directory() -> Result<PathBuf, String> {
        let home = env::var("HOME").map_err(|_| "HOME ist nicht gesetzt.".to_string())?;

        Ok(PathBuf::from(home)
            .join(".config")
            .join("systemd")
            .join("user"))
    }

    fn service_path() -> Result<PathBuf, String> {
        Ok(service_directory()?.join(SERVICE_NAME))
    }

    fn run_systemctl(args: &[&str]) -> Result<(), String> {
        let output = Command::new("systemctl")
            .args(["--user"])
            .args(args)
            .output()
            .map_err(|e| format!("systemctl konnte nicht gestartet werden: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);

            return Err(format!("systemctl fehlgeschlagen: {}", stderr.trim()));
        }

        Ok(())
    }

    fn run_systemctl_output(args: &[&str]) -> Result<std::process::Output, String> {
        Command::new("systemctl")
            .args(["--user"])
            .args(args)
            .output()
            .map_err(|e| format!("systemctl konnte nicht gestartet werden: {}", e))
    }

    fn executable_path() -> Result<PathBuf, String> {
        env::current_exe()
            .map_err(|e| format!("Pfad des Programms konnte nicht ermittelt werden: {}", e))
    }

    pub fn install() -> Result<(), String> {
        let directory = service_directory()?;

        fs::create_dir_all(&directory)
            .map_err(|e| format!("Systemd-Verzeichnis konnte nicht erstellt werden: {}", e))?;

        let executable = executable_path()?;

        let service = format!(
            r#"[Unit]
Description=OHG Autologin
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart="{}"
Restart=no

[Install]
WantedBy=default.target
"#,
            executable.display()
        );

        fs::write(service_path()?, service)
            .map_err(|e| format!("Systemd-Service konnte nicht geschrieben werden: {}", e))?;

        run_systemctl(&["daemon-reload"])?;

        Ok(())
    }

    pub fn uninstall() -> Result<(), String> {
        // Erst stoppen, falls er läuft.
        let _ = stop();

        // Dann deaktivieren.
        let _ = disable();

        let path = service_path()?;

        if path.exists() {
            fs::remove_file(path)
                .map_err(|e| format!("Systemd-Service konnte nicht gelöscht werden: {}", e))?;
        }

        run_systemctl(&["daemon-reload"])?;

        Ok(())
    }

    pub fn enable() -> Result<(), String> {
        run_systemctl(&["enable", SERVICE_NAME])
    }

    pub fn disable() -> Result<(), String> {
        run_systemctl(&["disable", SERVICE_NAME])
    }

    pub fn start() -> Result<(), String> {
        run_systemctl(&["start", SERVICE_NAME])
    }

    pub fn stop() -> Result<(), String> {
        run_systemctl(&["stop", SERVICE_NAME])
    }

    pub fn is_installed() -> bool {
        service_path().map(|path| path.exists()).unwrap_or(false)
    }

    pub fn is_enabled() -> Result<bool, String> {
        let output = run_systemctl_output(&["is-enabled", SERVICE_NAME])?;

        Ok(output.status.success())
    }

    pub fn is_running() -> Result<bool, String> {
        let output = run_systemctl_output(&["is-active", SERVICE_NAME])?;

        Ok(output.status.success())
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use std::{env, process::Command};

    const TASK_NAME: &str = "OHG Autologin";

    fn executable_path() -> Result<String, String> {
        env::current_exe()
            .map(|path| path.to_string_lossy().to_string())
            .map_err(|e| format!("Pfad des Programms konnte nicht ermittelt werden: {}", e))
    }

    fn run_schtasks(args: &[&str]) -> Result<(), String> {
        let output = Command::new("schtasks")
            .args(args)
            .output()
            .map_err(|e| format!("schtasks konnte nicht gestartet werden: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);

            return Err(format!(
                "Windows Aufgabenplanung fehlgeschlagen: {}",
                stderr.trim()
            ));
        }

        Ok(())
    }

    fn query_task() -> Result<String, String> {
        let output = Command::new("schtasks")
            .args(["/Query", "/TN", TASK_NAME, "/FO", "CSV", "/NH"])
            .output()
            .map_err(|e| format!("schtasks konnte nicht gestartet werden: {}", e))?;

        if !output.status.success() {
            return Err("Task ist nicht installiert.".to_string());
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub fn install() -> Result<(), String> {
        let executable = executable_path()?;

        run_schtasks(&[
            "/Create",
            "/TN",
            TASK_NAME,
            "/TR",
            &format!("\"{}\"", executable),
            "/SC",
            "ONLOGON",
            "/F",
        ])
    }

    pub fn uninstall() -> Result<(), String> {
        // Aktuell laufende Instanz beenden.
        let _ = stop();

        run_schtasks(&["/Delete", "/TN", TASK_NAME, "/F"])
    }

    pub fn enable() -> Result<(), String> {
        run_schtasks(&["/Change", "/TN", TASK_NAME, "/ENABLE"])
    }

    pub fn disable() -> Result<(), String> {
        run_schtasks(&["/Change", "/TN", TASK_NAME, "/DISABLE"])
    }

    pub fn start() -> Result<(), String> {
        run_schtasks(&["/Run", "/TN", TASK_NAME])
    }

    pub fn stop() -> Result<(), String> {
        run_schtasks(&["/End", "/TN", TASK_NAME])
    }

    pub fn is_installed() -> bool {
        query_task().is_ok()
    }

    pub fn is_enabled() -> Result<bool, String> {
        let output = query_task()?;

        // In der CSV-Ausgabe steht der Status der Aufgabe.
        // "Disabled" bedeutet deaktiviert.
        Ok(!output.contains("Disabled"))
    }

    pub fn is_running() -> Result<bool, String> {
        let output = query_task()?;

        Ok(output.contains("Running")
            || output.contains("RUNNING")
            || output.contains("Wird ausgeführt"))
    }
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
mod platform {
    pub fn install() -> Result<(), String> {
        Err("Betriebssystem wird nicht unterstützt.".into())
    }

    pub fn uninstall() -> Result<(), String> {
        Err("Betriebssystem wird nicht unterstützt.".into())
    }

    pub fn enable() -> Result<(), String> {
        Err("Betriebssystem wird nicht unterstützt.".into())
    }

    pub fn disable() -> Result<(), String> {
        Err("Betriebssystem wird nicht unterstützt.".into())
    }

    pub fn start() -> Result<(), String> {
        Err("Betriebssystem wird nicht unterstützt.".into())
    }

    pub fn stop() -> Result<(), String> {
        Err("Betriebssystem wird nicht unterstützt.".into())
    }

    pub fn is_installed() -> bool {
        false
    }

    pub fn is_enabled() -> Result<bool, String> {
        Err("Betriebssystem wird nicht unterstützt.".into())
    }

    pub fn is_running() -> Result<bool, String> {
        Err("Betriebssystem wird nicht unterstützt.".into())
    }
}

pub fn install() -> Result<(), String> {
    platform::install()
}

pub fn uninstall() -> Result<(), String> {
    platform::uninstall()
}

pub fn enable() -> Result<(), String> {
    platform::enable()
}

pub fn disable() -> Result<(), String> {
    platform::disable()
}

pub fn start() -> Result<(), String> {
    platform::start()
}

pub fn stop() -> Result<(), String> {
    platform::stop()
}

pub fn is_installed() -> bool {
    platform::is_installed()
}

pub fn is_enabled() -> Result<bool, String> {
    platform::is_enabled()
}

pub fn is_running() -> Result<bool, String> {
    platform::is_running()
}
