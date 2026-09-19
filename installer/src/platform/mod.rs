#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
pub fn install(app_binary: &[u8]) -> Result<(), String> {
    linux::install(app_binary)
}

#[cfg(target_os = "linux")]
pub fn uninstall() -> Result<(), String> {
    linux::uninstall()
}

#[cfg(target_os = "windows")]
pub fn install(app_binary: &[u8]) -> Result<(), String> {
    windows::install(app_binary)
}

#[cfg(target_os = "windows")]
pub fn uninstall() -> Result<(), String> {
    windows::uninstall()
}

#[cfg(target_os = "macos")]
pub fn install(app_binary: &[u8]) -> Result<(), String> {
    macos::install(app_binary)
}

#[cfg(target_os = "macos")]
pub fn uninstall() -> Result<(), String> {
    macos::uninstall()
}
