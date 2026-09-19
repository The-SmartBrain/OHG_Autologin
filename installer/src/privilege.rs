use std::process::Command;

pub fn install_with_privileges() -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        return linux_install();
    }

    #[cfg(target_os = "windows")]
    {
        return windows_install();
    }

    #[cfg(target_os = "macos")]
    {
        return macos_install();
    }

    #[allow(unreachable_code)]
    Err("Dieses Betriebssystem wird nicht unterstützt.".to_string())
}

pub fn uninstall_with_privileges() -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        return linux_uninstall();
    }

    #[cfg(target_os = "windows")]
    {
        return windows_uninstall();
    }

    #[cfg(target_os = "macos")]
    {
        return macos_uninstall();
    }

    #[allow(unreachable_code)]
    Err("Dieses Betriebssystem wird nicht unterstützt.".to_string())
}

#[cfg(target_os = "linux")]
fn linux_install() -> Result<(), String> {
    if is_root() {
        return Ok(());
    }

    let executable = std::env::current_exe().map_err(|e| e.to_string())?;

    let status = Command::new("pkexec")
        .arg(executable)
        .arg("--privileged-install")
        .status()
        .map_err(|e| format!("pkexec konnte nicht gestartet werden: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err("Die Administratorauthentifizierung \
             wurde abgebrochen oder ist fehlgeschlagen."
            .to_string())
    }
}

#[cfg(target_os = "linux")]
fn linux_uninstall() -> Result<(), String> {
    if is_root() {
        return Ok(());
    }

    let executable = std::env::current_exe().map_err(|e| e.to_string())?;

    let status = Command::new("pkexec")
        .arg(executable)
        .arg("--privileged-uninstall")
        .status()
        .map_err(|e| format!("pkexec konnte nicht gestartet werden: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err("Die Administratorauthentifizierung \
             wurde abgebrochen oder ist fehlgeschlagen."
            .to_string())
    }
}

#[cfg(target_os = "linux")]
fn is_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}

#[cfg(target_os = "windows")]
fn windows_install() -> Result<(), String> {
    super::platform::windows::elevate("--privileged-install")
}

#[cfg(target_os = "windows")]
fn windows_uninstall() -> Result<(), String> {
    super::platform::windows::elevate("--privileged-uninstall")
}

#[cfg(target_os = "macos")]
fn macos_install() -> Result<(), String> {
    super::platform::macos::elevate("--privileged-install")
}

#[cfg(target_os = "macos")]
fn macos_uninstall() -> Result<(), String> {
    super::platform::macos::elevate("--privileged-uninstall")
}
