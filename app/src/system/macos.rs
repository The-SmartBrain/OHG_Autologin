use super::SystemError;

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const LABEL: &str = "de.ohg.wlanautologin";

fn plist_path() -> Result<PathBuf, SystemError> {
    let home = env::var("HOME")
        .map_err(|_| SystemError::CommandFailed("HOME ist nicht gesetzt.".to_string()))?;

    Ok(PathBuf::from(home)
        .join("Library")
        .join("LaunchAgents")
        .join("de.ohg.wlanautologin.plist"))
}

fn executable_path() -> Result<PathBuf, SystemError> {
    env::current_exe().map_err(SystemError::Io)
}

pub fn enable() -> Result<(), SystemError> {
    let path = plist_path()?;

    let exe = executable_path()?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(SystemError::Io)?;
    }

    let plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd"> <plist version="1.0"> <dict> <key>Label</key> <string>{}</string>

<key>ProgramArguments</key>
<array>
    <string>{}</string>
</array>

<key>RunAtLoad</key>
<true/>

<key>KeepAlive</key>
<true/>

<key>ProcessType</key>
<string>Background</string>

</dict> </plist> "#,
        LABEL,
        exe.display()
    );

    fs::write(&path, plist).map_err(SystemError::Io)?;

    // Bestehenden LaunchAgent entfernen,
    // falls vorhanden.
    let _ = Command::new("launchctl")
        .args(["bootout", "gui/", path.to_string_lossy().as_ref()])
        .output();

    let output = Command::new("launchctl")
        .args(["bootstrap", "gui/", path.to_string_lossy().as_ref()])
        .output()
        .map_err(SystemError::Io)?;

    if !output.status.success() {
        return Err(SystemError::CommandFailed(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }

    Ok(())
}

pub fn disable() -> Result<(), SystemError> {
    let path = plist_path()?;

    if path.exists() {
        let _ = Command::new("launchctl")
            .args(["bootout", "gui/", path.to_string_lossy().as_ref()])
            .output();

        fs::remove_file(&path).map_err(SystemError::Io)?;
    }

    Ok(())
}

pub fn is_enabled() -> Result<bool, SystemError> {
    Ok(plist_path()?.exists())
}
