use std::io;

pub fn enable_autostart() -> io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        return enable_windows();
    }

    #[cfg(target_os = "linux")]
    {
        return enable_linux();
    }

    #[cfg(target_os = "macos")]
    {
        return enable_macos();
    }

    #[allow(unreachable_code)]
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Autostart wird auf diesem Betriebssystem nicht unterstützt",
    ))
}

pub fn disable_autostart() -> io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        return disable_windows();
    }

    #[cfg(target_os = "linux")]
    {
        return disable_linux();
    }

    #[cfg(target_os = "macos")]
    {
        return disable_macos();
    }

    #[allow(unreachable_code)]
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Autostart wird auf diesem Betriebssystem nicht unterstützt",
    ))
}

pub fn uninstall() -> io::Result<()> {
    // Autostart entfernen
    disable_autostart()?;

    // Konfiguration entfernen
    let config_dir = crate::config::get_config_dir();

    if config_dir.exists() {
        std::fs::remove_dir_all(config_dir)?;
    }

    Ok(())
}

// WINDOWS

#[cfg(target_os = "windows")]
fn enable_windows() -> io::Result<()> {
    use std::env;

    use windows::Win32::System::Registry::{
        HKEY, HKEY_CURRENT_USER, KEY_WRITE, REG_SZ, RegCloseKey, RegCreateKeyExW, RegSetValueExW,
    };

    use windows::core::PCWSTR;

    let exe = env::current_exe()?;

    let exe = exe.to_string_lossy();

    let key_path = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";

    let key_path_w = windows::core::HSTRING::from(key_path);

    let value_name = windows::core::HSTRING::from("WlanAutologin");

    let value = windows::core::HSTRING::from(format!("\"{}\"", exe));

    unsafe {
        let mut key = HKEY::default();

        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(key_path_w.as_ptr()),
            0,
            None,
            Default::default(),
            KEY_WRITE,
            None,
            &mut key,
            None,
        )
        .ok()
        .map_err(|error| io::Error::new(io::ErrorKind::PermissionDenied, error.to_string()))?;

        let bytes = std::slice::from_raw_parts(value.as_ptr() as *const u8, value.len() * 2);

        RegSetValueExW(key, PCWSTR(value_name.as_ptr()), 0, REG_SZ, Some(bytes))
            .ok()
            .map_err(|error| io::Error::new(io::ErrorKind::Other, error.to_string()))?;

        let _ = RegCloseKey(key);
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn disable_windows() -> io::Result<()> {
    Ok(())
}

// LINUX

#[cfg(target_os = "linux")]
fn linux_autostart_path() -> io::Result<std::path::PathBuf> {
    let home = std::env::var("HOME")
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "HOME ist nicht gesetzt"))?;

    Ok(std::path::PathBuf::from(home)
        .join(".config")
        .join("autostart")
        .join("wlan-autologin.desktop"))
}

#[cfg(target_os = "linux")]
fn enable_linux() -> io::Result<()> {
    let path = linux_autostart_path()?;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let executable = std::env::current_exe()?;

    let desktop_entry = format!(
        "[Desktop Entry]
Type=Application
Name=OHG WLAN Autologin
Exec=\"{}\"
Terminal=false
Hidden=false
X-GNOME-Autostart-enabled=true
",
        executable.display()
    );

    std::fs::write(path, desktop_entry)?;

    Ok(())
}

#[cfg(target_os = "linux")]
fn disable_linux() -> io::Result<()> {
    let path = linux_autostart_path()?;

    if path.exists() {
        std::fs::remove_file(path)?;
    }

    Ok(())
}

// MACOS

#[cfg(target_os = "macos")]
fn macos_plist_path() -> io::Result<std::path::PathBuf> {
    let home = std::env::var("HOME")
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "HOME ist nicht gesetzt"))?;

    Ok(std::path::PathBuf::from(home)
        .join("Library")
        .join("LaunchAgents")
        .join("de.ohg.wlanautologin.plist"))
}

#[cfg(target_os = "macos")]
fn enable_macos() -> io::Result<()> {
    let path = macos_plist_path()?;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let executable = std::env::current_exe()?;

    let plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
"http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>de.ohg.wlanautologin</string>

    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
    </array>

    <key>RunAtLoad</key>
    <true/>
</dict>
</plist>
"#,
        executable.display()
    );

    std::fs::write(path, plist)?;

    Ok(())
}

#[cfg(target_os = "macos")]
fn disable_macos() -> io::Result<()> {
    let path = macos_plist_path()?;

    if path.exists() {
        std::fs::remove_file(path)?;
    }

    Ok(())
}
