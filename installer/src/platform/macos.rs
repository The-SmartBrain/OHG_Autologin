use std::path::{Path, PathBuf};
use std::process::Command;

const APP_NAME: &str = "OHG WLAN Autologin.app";

const APP_DIR: &str = "/Applications/OHG WLAN Autologin.app";

const APP_BINARY: &str = "/Applications/OHG WLAN Autologin.app/Contents/MacOS/ohg-wlan-autologin";

const LAUNCH_LABEL: &str = "de.ohg.wlanautologin";

pub fn install(app_binary: &[u8]) -> Result<(), String> {
    let contents = Path::new(APP_DIR).join("Contents");

    let macos = contents.join("MacOS");

    let resources = contents.join("Resources");

    std::fs::create_dir_all(&macos).map_err(|e| e.to_string())?;

    std::fs::create_dir_all(&resources).map_err(|e| e.to_string())?;

    std::fs::write(APP_BINARY, app_binary)
        .map_err(|e| format!("App konnte nicht geschrieben werden: {}", e))?;

    use std::os::unix::fs::PermissionsExt;

    std::fs::set_permissions(APP_BINARY, std::fs::Permissions::from_mode(0o755))
        .map_err(|e| e.to_string())?;

    write_info_plist(&contents)?;

    install_launch_agent()?;

    Ok(())
}

pub fn uninstall() -> Result<(), String> {
    let home = std::env::var("HOME").map_err(|e| e.to_string())?;

    let plist = PathBuf::from(home)
        .join("Library")
        .join("LaunchAgents")
        .join("de.ohg.wlanautologin.plist");

    let _ = Command::new("launchctl")
        .args(["bootout", "gui/", plist.to_string_lossy().as_ref()])
        .status();

    if plist.exists() {
        std::fs::remove_file(plist).map_err(|e| e.to_string())?;
    }

    if Path::new(APP_DIR).exists() {
        std::fs::remove_dir_all(APP_DIR).map_err(|e| e.to_string())?;
    }

    Ok(())
}

pub fn elevate(argument: &str) -> Result<(), String> {
    let executable = std::env::current_exe().map_err(|e| e.to_string())?;

    let command = format!(
        "'{}' {}",
        shell_escape(&executable.to_string_lossy()),
        shell_escape(argument)
    );

    let script = format!(
        "do shell script \"{}\" \
             with administrator privileges",
        apple_script_escape(&command)
    );

    let status = Command::new("osascript")
        .args(["-e", &script])
        .status()
        .map_err(|e| e.to_string())?;

    if status.success() {
        Ok(())
    } else {
        Err("Die Administratorauthentifizierung \
             wurde abgebrochen oder ist fehlgeschlagen."
            .to_string())
    }
}

fn write_info_plist(contents: &Path) -> Result<(), String> {
    let plist = contents.join("Info.plist");

    let content = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
"http://www.apple.com/DTDs/PropertyList-1.0.dtd">

<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>ohg-wlan-autologin</string>

    <key>CFBundleIdentifier</key>
    <string>de.ohg.wlanautologin</string>

    <key>CFBundleName</key>
    <string>OHG WLAN Autologin</string>

    <key>CFBundleDisplayName</key>
    <string>OHG WLAN Autologin</string>

    <key>CFBundlePackageType</key>
    <string>APPL</string>

    <key>CFBundleVersion</key>
    <string>1.0.0</string>

    <key>CFBundleShortVersionString</key>
    <string>1.0.0</string>
</dict>
</plist>
"#;

    std::fs::write(plist, content).map_err(|e| e.to_string())
}

fn install_launch_agent() -> Result<(), String> {
    let home = std::env::var("HOME").map_err(|e| e.to_string())?;

    let launch_agents = PathBuf::from(home).join("Library").join("LaunchAgents");

    std::fs::create_dir_all(&launch_agents).map_err(|e| e.to_string())?;

    let plist = launch_agents.join("de.ohg.wlanautologin.plist");

    let content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
"http://www.apple.com/DTDs/PropertyList-1.0.dtd">

<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{}</string>

    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
    </array>

    <key>RunAtLoad</key>
    <true/>

    <key>KeepAlive</key>
    <true/>
</dict>
</plist>
"#,
        LAUNCH_LABEL, APP_BINARY
    );

    std::fs::write(&plist, content).map_err(|e| e.to_string())?;

    let _ = Command::new("launchctl")
        .args(["bootout", "gui/", plist.to_string_lossy().as_ref()])
        .status();

    let output = Command::new("launchctl")
        .args(["bootstrap", "gui/", plist.to_string_lossy().as_ref()])
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    Ok(())
}

fn apple_script_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn shell_escape(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

pub fn installation_path() -> &'static Path {
    Path::new(APP_DIR)
}

#[allow(dead_code)]
pub fn app_name() -> &'static str {
    APP_NAME
}
