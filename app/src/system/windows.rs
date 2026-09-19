use super::SystemError;

use std::env;
use std::path::PathBuf;
use std::process::Command;

const TASK_NAME: &str = "OHG WLAN Autologin";

fn executable_path() -> Result<PathBuf, SystemError> {
    env::current_exe().map_err(SystemError::Io)
}

fn run_schtasks(args: &[&str]) -> Result<String, SystemError> {
    let output = Command::new("schtasks.exe")
        .args(args)
        .output()
        .map_err(SystemError::Io)?;

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);

        let stderr = String::from_utf8_lossy(&output.stderr);

        return Err(SystemError::CommandFailed(format!(
            "schtasks.exe fehlgeschlagen.\nstdout: {}\nstderr: {}",
            stdout.trim(),
            stderr.trim()
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Erstellt die Windows-Aufgabe.
///
/// Die Aufgabe wird:
/// - beim Benutzer-Login gestartet
/// - bei einem relevanten WLAN-Ereignis gestartet
pub fn enable() -> Result<(), SystemError> {
    let exe = executable_path()?;

    // Vorhandene Aufgabe entfernen.

    let _ = run_schtasks(&["/Delete", "/TN", TASK_NAME, "/F"]);

    // Benutzer-Login

    let exe_string = exe.to_string_lossy();

    run_schtasks(&[
        "/Create",
        "/TN",
        TASK_NAME,
        "/TR",
        &format!("\"{}\"", exe_string),
        "/SC",
        "ONLOGON",
        "/F",
    ])?;

    // Netzwerk-/WLAN-Ereignis

    let event_task_name = "OHG WLAN Autologin - Network";

    let _ = run_schtasks(&["/Delete", "/TN", event_task_name, "/F"]);

    let event_query =
        r#"*[System[(Provider[@Name='Microsoft-Windows-WLAN-AutoConfig']) and (EventID=8001)]]"#;

    run_schtasks(&[
        "/Create",
        "/TN",
        event_task_name,
        "/TR",
        &format!("\"{}\"", exe_string),
        "/SC",
        "ONEVENT",
        "/EC",
        "Microsoft-Windows-WLAN-AutoConfig/Operational",
        "/MO",
        event_query,
        "/F",
    ])?;

    Ok(())
}

/// Entfernt die Windows-Aufgaben.
pub fn disable() -> Result<(), SystemError> {
    let mut first_error = None;

    for task in [TASK_NAME, "OHG WLAN Autologin - Network"] {
        match run_schtasks(&["/Delete", "/TN", task, "/F"]) {
            Ok(_) => {}

            Err(error) => {
                let message = error.to_string();

                if !message.contains("ERROR: The system cannot find") {
                    first_error = Some(error);
                }
            }
        }
    }

    if let Some(error) = first_error {
        return Err(error);
    }

    Ok(())
}

pub fn is_enabled() -> Result<bool, SystemError> {
    let output = Command::new("schtasks.exe")
        .args(["/Query", "/TN", TASK_NAME])
        .output()
        .map_err(SystemError::Io)?;

    Ok(output.status.success())
}
