use std::process;

fn main() {
    println!("========================================");
    println!(" OHG Autologin Test");
    println!("========================================");
    println!();

    println!("Betriebssystem: {}", std::env::consts::OS);
    println!("Architektur:    {}", std::env::consts::ARCH);
    println!();

    let mut failed = false;

    if !run_test("Crypto", test_crypto()) {
        failed = true;
    }

    if !run_test("Config", test_config()) {
        failed = true;
    }

    #[cfg(any(target_os = "linux", target_os = "windows"))]
    {
        if !run_test("System", test_system()) {
            failed = true;
        }
    }

    println!();
    println!("========================================");

    if failed {
        println!(" ERGEBNIS: FEHLGESCHLAGEN");
        println!("========================================");

        process::exit(1);
    } else {
        println!(" ERGEBNIS: ALLE TESTS OK");
        println!("========================================");
    }
}

fn run_test(name: &str, result: Result<(), String>) -> bool {
    match result {
        Ok(()) => {
            println!("[ OK ] {}", name);
            true
        }

        Err(error) => {
            println!("[FAIL] {}", name);
            println!();
            println!("Fehler:");
            println!("{}", error);
            println!();

            false
        }
    }
}

fn test_crypto() -> Result<(), String> {
    let password = "OHG-Test-Passwort-123!";

    let encrypted = app::crypto::encrypt_password(password)?;

    if encrypted == password {
        return Err("Verschlüsselter Wert entspricht dem Klartext.".into());
    }

    let decrypted = app::crypto::decrypt_password(&encrypted)?;

    if decrypted != password {
        return Err(format!(
            "Entschlüsseltes Passwort stimmt nicht überein.\n\
             Erwartet: {:?}\n\
             Erhalten: {:?}",
            password, decrypted
        ));
    }

    Ok(())
}

fn test_config() -> Result<(), String> {
    let directory = tempfile::tempdir().map_err(|e| e.to_string())?;

    let path = directory.path().join("config.json");

    let username = "testuser";
    let password = "OHG-Test-Passwort-123!";

    app::config::Config::create(&path, username, password, true)?;

    let credentials = app::config::Config::load(&path)?;

    if credentials.username != username {
        return Err(format!(
            "Username stimmt nicht.\n\
             Erwartet: {}\n\
             Erhalten: {}",
            username, credentials.username
        ));
    }

    if credentials.password != password {
        return Err("Passwort nach dem Laden stimmt nicht.".into());
    }

    let raw = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;

    if raw.contains(password) {
        return Err("Passwort befindet sich im Klartext in der \
             verschlüsselten Config."
            .into());
    }

    app::config::Config::delete(&path)?;

    if path.exists() {
        return Err("Config-Datei wurde nicht gelöscht.".into());
    }

    Ok(())
}

#[cfg(any(target_os = "linux", target_os = "windows"))]
fn test_system() -> Result<(), String> {
    use app::system::testing;

    // Sicherheitsmaßnahme:
    // Vor dem Test einen eventuell übrig gebliebenen
    // Testdienst entfernen.
    testing::cleanup();

    if testing::is_installed() {
        return Err("Testdienst konnte vor dem Test nicht entfernt werden.".into());
    }

    // --------------------------------------------------
    // INSTALL
    // --------------------------------------------------

    testing::install()?;

    if !testing::is_installed() {
        return Err("Dienst wurde nicht installiert.".into());
    }

    // --------------------------------------------------
    // ENABLE
    // --------------------------------------------------

    testing::enable()?;

    if !testing::is_enabled()? {
        return Err("Dienst wurde nicht aktiviert.".into());
    }

    // --------------------------------------------------
    // START
    // --------------------------------------------------

    testing::start()?;

    wait_until_running()?;

    if !testing::is_running()? {
        return Err("Dienst läuft nach start() nicht.".into());
    }

    // --------------------------------------------------
    // STOP
    // --------------------------------------------------

    testing::stop()?;

    wait_until_stopped()?;

    if testing::is_running()? {
        return Err("Dienst läuft nach stop() weiterhin.".into());
    }

    // --------------------------------------------------
    // DISABLE
    // --------------------------------------------------

    testing::disable()?;

    if testing::is_enabled()? {
        return Err("Dienst ist nach disable() weiterhin aktiviert.".into());
    }

    // --------------------------------------------------
    // UNINSTALL
    // --------------------------------------------------

    testing::uninstall()?;

    if testing::is_installed() {
        return Err("Dienst wurde nicht vollständig deinstalliert.".into());
    }

    Ok(())
}

#[cfg(any(target_os = "linux", target_os = "windows"))]
fn wait_until_running() -> Result<(), String> {
    use std::{
        thread,
        time::{Duration, Instant},
    };

    let start = Instant::now();
    let timeout = Duration::from_secs(10);

    while start.elapsed() < timeout {
        if app::system::testing::is_running()? {
            return Ok(());
        }

        thread::sleep(Duration::from_millis(100));
    }

    Err("Dienst wurde innerhalb von 10 Sekunden nicht aktiv.".into())
}

#[cfg(any(target_os = "linux", target_os = "windows"))]
fn wait_until_stopped() -> Result<(), String> {
    use std::{
        thread,
        time::{Duration, Instant},
    };

    let start = Instant::now();
    let timeout = Duration::from_secs(10);

    while start.elapsed() < timeout {
        if !app::system::testing::is_running()? {
            return Ok(());
        }

        thread::sleep(Duration::from_millis(100));
    }

    Err("Dienst wurde innerhalb von 10 Sekunden nicht beendet.".into())
}
