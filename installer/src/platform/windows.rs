use std::ffi::OsStr;
use std::iter;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::process::Command;

use windows::core::PCWSTR;

use windows::Win32::Foundation::HWND;

use windows::Win32::UI::Shell::ShellExecuteW;

use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

const TASK_NAME: &str = "OHG WLAN Autologin";

const NETWORK_TASK_NAME: &str = "OHG WLAN Autologin - WLAN";

const INSTALL_DIR: &str = r"C:\Program Files\OHG WLAN Autologin";

const APP_PATH: &str = r"C:\Program Files\OHG WLAN Autologin\ohg-wlan-autologin.exe";

pub fn install(app_binary: &[u8]) -> Result<(), String> {
    std::fs::create_dir_all(INSTALL_DIR).map_err(|e| {
        format!(
            "Installationsverzeichnis konnte \
             nicht erstellt werden: {}",
            e
        )
    })?;

    std::fs::write(APP_PATH, app_binary)
        .map_err(|e| format!("App konnte nicht installiert werden: {}", e))?;

    create_login_task()?;
    create_network_task()?;

    Ok(())
}

pub fn uninstall() -> Result<(), String> {
    delete_task(TASK_NAME);
    delete_task(NETWORK_TASK_NAME);

    if Path::new(INSTALL_DIR).exists() {
        std::fs::remove_dir_all(INSTALL_DIR)
            .map_err(|e| format!("Installation konnte nicht entfernt werden: {}", e))?;
    }

    Ok(())
}

fn create_login_task() -> Result<(), String> {
    delete_task(TASK_NAME);

    let output = Command::new("schtasks.exe")
        .args([
            "/Create",
            "/TN",
            TASK_NAME,
            "/TR",
            &format!("\"{}\"", APP_PATH),
            "/SC",
            "ONLOGON",
            "/F",
        ])
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    Ok(())
}

fn create_network_task() -> Result<(), String> {
    delete_task(NETWORK_TASK_NAME);

    let query =
        r#"*[System[(Provider[@Name='Microsoft-Windows-WLAN-AutoConfig']) and (EventID=8001)]]"#;

    let output = Command::new("schtasks.exe")
        .args([
            "/Create",
            "/TN",
            NETWORK_TASK_NAME,
            "/TR",
            &format!("\"{}\"", APP_PATH),
            "/SC",
            "ONEVENT",
            "/EC",
            "Microsoft-Windows-WLAN-AutoConfig/Operational",
            "/MO",
            query,
            "/F",
        ])
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Ok(());
    }

    Ok(())
}

fn delete_task(task_name: &str) {
    let _ = Command::new("schtasks.exe")
        .args(["/Delete", "/TN", task_name, "/F"])
        .status();
}

pub fn elevate(argument: &str) -> Result<(), String> {
    let executable = std::env::current_exe().map_err(|e| e.to_string())?;

    let executable = to_wide(executable.to_string_lossy().as_ref());

    let argument = to_wide(argument);

    let operation = to_wide("runas");

    let result = unsafe {
        ShellExecuteW(
            HWND(0),
            PCWSTR(operation.as_ptr()),
            PCWSTR(executable.as_ptr()),
            PCWSTR(argument.as_ptr()),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        )
    };

    if result.0 <= 32 {
        return Err("Die Administratorrechte konnten \
             nicht angefordert werden."
            .to_string());
    }

    Ok(())
}

fn to_wide(value: &str) -> Vec<u16> {
    OsStr::new(value)
        .encode_wide()
        .chain(iter::once(0))
        .collect()
}

pub fn installation_path() -> &'static Path {
    Path::new(INSTALL_DIR)
}
