use std::process::Command;

#[cfg(target_os = "linux")]
mod platform {
    use std::{
        env, fs,
        path::PathBuf,
        process::{Command, Output},
    };

    const SERVICE_NAME: &str = "ohg-autologin.service";

    fn service_directory() -> Result<PathBuf, String> {
        let home = env::var("HOME").map_err(|_| "HOME ist nicht gesetzt.".to_string())?;

        Ok(PathBuf::from(home)
            .join(".config")
            .join("systemd")
            .join("user"))
    }

    fn service_path(name: &str) -> Result<PathBuf, String> {
        Ok(service_directory()?.join(name))
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

    fn run_systemctl_output(args: &[&str]) -> Result<Output, String> {
        Command::new("systemctl")
            .args(["--user"])
            .args(args)
            .output()
            .map_err(|e| format!("systemctl konnte nicht gestartet werden: {}", e))
    }

    fn executable_path() -> Result<PathBuf, String> {
        env::current_exe()
            .map_err(|e| format!("Programm-Pfad konnte nicht ermittelt werden: {}", e))
    }

    fn install_with_definition(
        name: &str,
        executable: &str,
        arguments: &[&str],
    ) -> Result<(), String> {
        let directory = service_directory()?;

        fs::create_dir_all(&directory)
            .map_err(|e| format!("Systemd-Verzeichnis konnte nicht erstellt werden: {}", e))?;

        let exec_start = if arguments.is_empty() {
            format!("\"{}\"", executable)
        } else {
            format!("\"{}\" {}", executable, arguments.join(" "))
        };

        let service = format!(
            r#"[Unit]
Description=OHG Autologin
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart={}
Restart=no

[Install]
WantedBy=default.target
"#,
            exec_start
        );

        fs::write(service_path(name)?, service)
            .map_err(|e| format!("Systemd-Service konnte nicht geschrieben werden: {}", e))?;

        run_systemctl(&["daemon-reload"])
    }

    fn install_named(name: &str) -> Result<(), String> {
        let executable = executable_path()?;

        install_with_definition(name, executable.to_string_lossy().as_ref(), &[])
    }

    fn uninstall_named(name: &str) -> Result<(), String> {
        let _ = stop_named(name);
        let _ = disable_named(name);

        let path = service_path(name)?;

        if path.exists() {
            fs::remove_file(path)
                .map_err(|e| format!("Systemd-Service konnte nicht gelöscht werden: {}", e))?;
        }

        run_systemctl(&["daemon-reload"])
    }

    fn enable_named(name: &str) -> Result<(), String> {
        run_systemctl(&["enable", name])
    }

    fn disable_named(name: &str) -> Result<(), String> {
        run_systemctl(&["disable", name])
    }

    fn start_named(name: &str) -> Result<(), String> {
        run_systemctl(&["start", name])
    }

    fn stop_named(name: &str) -> Result<(), String> {
        run_systemctl(&["stop", name])
    }

    fn is_installed_named(name: &str) -> bool {
        service_path(name)
            .map(|path| path.exists())
            .unwrap_or(false)
    }

    fn is_enabled_named(name: &str) -> Result<bool, String> {
        Ok(run_systemctl_output(&["is-enabled", name])?
            .status
            .success())
    }

    fn is_running_named(name: &str) -> Result<bool, String> {
        Ok(run_systemctl_output(&["is-active", name])?.status.success())
    }

    pub fn install() -> Result<(), String> {
        install_named(SERVICE_NAME)
    }

    pub fn uninstall() -> Result<(), String> {
        uninstall_named(SERVICE_NAME)
    }

    pub fn enable() -> Result<(), String> {
        enable_named(SERVICE_NAME)
    }

    pub fn disable() -> Result<(), String> {
        disable_named(SERVICE_NAME)
    }

    pub fn start() -> Result<(), String> {
        start_named(SERVICE_NAME)
    }

    pub fn stop() -> Result<(), String> {
        stop_named(SERVICE_NAME)
    }

    pub fn is_installed() -> bool {
        is_installed_named(SERVICE_NAME)
    }

    pub fn is_enabled() -> Result<bool, String> {
        is_enabled_named(SERVICE_NAME)
    }

    pub fn is_running() -> Result<bool, String> {
        is_running_named(SERVICE_NAME)
    }

    pub mod testing {
        use super::*;

        const TEST_SERVICE_NAME: &str = "ohg-autologin-test.service";

        pub fn install() -> Result<(), String> {
            // `sleep` läuft lange genug, damit start/stop
            // zuverlässig getestet werden können.
            install_with_definition(TEST_SERVICE_NAME, "/usr/bin/sleep", &["300"])
        }

        pub fn uninstall() -> Result<(), String> {
            uninstall_named(TEST_SERVICE_NAME)
        }

        pub fn enable() -> Result<(), String> {
            enable_named(TEST_SERVICE_NAME)
        }

        pub fn disable() -> Result<(), String> {
            disable_named(TEST_SERVICE_NAME)
        }

        pub fn start() -> Result<(), String> {
            start_named(TEST_SERVICE_NAME)
        }

        pub fn stop() -> Result<(), String> {
            stop_named(TEST_SERVICE_NAME)
        }

        pub fn is_installed() -> bool {
            is_installed_named(TEST_SERVICE_NAME)
        }

        pub fn is_enabled() -> Result<bool, String> {
            is_enabled_named(TEST_SERVICE_NAME)
        }

        pub fn is_running() -> Result<bool, String> {
            is_running_named(TEST_SERVICE_NAME)
        }

        pub fn cleanup() {
            let _ = stop();
            let _ = disable();
            let _ = uninstall();
        }
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use std::{env, process::Command};

    const TASK_NAME: &str = "OHG Autologin";

    fn executable_path() -> Result<String, String> {
        env::current_exe()
            .map(|path| path.to_string_lossy().to_string())
            .map_err(|e| format!("Programm-Pfad konnte nicht ermittelt werden: {}", e))
    }

    fn run_schtasks(args: &[&str]) -> Result<(), String> {
        let output = Command::new("schtasks")
            .args(args)
            .output()
            .map_err(|e| format!("schtasks konnte nicht gestartet werden: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);

            return Err(format!("Aufgabenplanung fehlgeschlagen: {}", stderr.trim()));
        }

        Ok(())
    }

    fn query_task(name: &str) -> Result<String, String> {
        let output = Command::new("schtasks")
            .args(["/Query", "/TN", name, "/FO", "CSV", "/NH"])
            .output()
            .map_err(|e| format!("schtasks konnte nicht gestartet werden: {}", e))?;

        if !output.status.success() {
            return Err("Task ist nicht installiert.".into());
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn install_named(name: &str) -> Result<(), String> {
        let executable = executable_path()?;

        run_schtasks(&[
            "/Create",
            "/TN",
            name,
            "/TR",
            &format!("\"{}\"", executable),
            "/SC",
            "ONLOGON",
            "/F",
        ])
    }

    fn uninstall_named(name: &str) -> Result<(), String> {
        let _ = stop_named(name);

        if !is_installed_named(name) {
            return Ok(());
        }

        run_schtasks(&["/Delete", "/TN", name, "/F"])
    }

    fn enable_named(name: &str) -> Result<(), String> {
        run_schtasks(&["/Change", "/TN", name, "/ENABLE"])
    }

    fn disable_named(name: &str) -> Result<(), String> {
        run_schtasks(&["/Change", "/TN", name, "/DISABLE"])
    }

    fn start_named(name: &str) -> Result<(), String> {
        run_schtasks(&["/Run", "/TN", name])
    }

    fn stop_named(name: &str) -> Result<(), String> {
        run_schtasks(&["/End", "/TN", name])
    }

    fn is_installed_named(name: &str) -> bool {
        query_task(name).is_ok()
    }

    fn is_enabled_named(name: &str) -> Result<bool, String> {
        let output = query_task(name)?;

        Ok(!output.contains("Disabled"))
    }

    fn is_running_named(name: &str) -> Result<bool, String> {
        let output = query_task(name)?;

        Ok(output.contains("Running")
            || output.contains("RUNNING")
            || output.contains("Wird ausgeführt"))
    }

    pub fn install() -> Result<(), String> {
        install_named(TASK_NAME)
    }

    pub fn uninstall() -> Result<(), String> {
        uninstall_named(TASK_NAME)
    }

    pub fn enable() -> Result<(), String> {
        enable_named(TASK_NAME)
    }

    pub fn disable() -> Result<(), String> {
        disable_named(TASK_NAME)
    }

    pub fn start() -> Result<(), String> {
        start_named(TASK_NAME)
    }

    pub fn stop() -> Result<(), String> {
        stop_named(TASK_NAME)
    }

    pub fn is_installed() -> bool {
        is_installed_named(TASK_NAME)
    }

    pub fn is_enabled() -> Result<bool, String> {
        is_enabled_named(TASK_NAME)
    }

    pub fn is_running() -> Result<bool, String> {
        is_running_named(TASK_NAME)
    }

    pub mod testing {
        use super::*;

        const TEST_TASK_NAME: &str = "OHG Autologin Test";

        pub fn install() -> Result<(), String> {
            /*
             * Der Test-Task startet PowerShell versteckt.
             *
             * PowerShell schläft 300 Sekunden.
             * Dadurch haben wir einen echten laufenden Prozess,
             * den wir mit start/stop/status testen können.
             */
            let action = concat!(
                "powershell.exe ",
                "-NoProfile ",
                "-NonInteractive ",
                "-WindowStyle Hidden ",
                "-Command ",
                "\"Start-Sleep -Seconds 300\""
            );

            run_schtasks(&[
                "/Create",
                "/TN",
                TEST_TASK_NAME,
                "/TR",
                action,
                "/SC",
                "ONLOGON",
                "/F",
            ])
        }

        pub fn uninstall() -> Result<(), String> {
            uninstall_named(TEST_TASK_NAME)
        }

        pub fn enable() -> Result<(), String> {
            enable_named(TEST_TASK_NAME)
        }

        pub fn disable() -> Result<(), String> {
            disable_named(TEST_TASK_NAME)
        }

        pub fn start() -> Result<(), String> {
            start_named(TEST_TASK_NAME)
        }

        pub fn stop() -> Result<(), String> {
            stop_named(TEST_TASK_NAME)
        }

        pub fn is_installed() -> bool {
            is_installed_named(TEST_TASK_NAME)
        }

        pub fn is_enabled() -> Result<bool, String> {
            is_enabled_named(TEST_TASK_NAME)
        }

        pub fn is_running() -> Result<bool, String> {
            is_running_named(TEST_TASK_NAME)
        }

        pub fn cleanup() {
            let _ = stop();
            let _ = disable();
            let _ = uninstall();
        }
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

pub mod testing {
    #[cfg(target_os = "linux")]
    pub use super::platform::testing::*;

    #[cfg(target_os = "windows")]
    pub use super::platform::testing::*;
}
