use app::config::Config;

#[test]
fn encrypted_config_roundtrip() {
    let directory = tempfile::tempdir().expect("Temp-Verzeichnis konnte nicht erstellt werden");

    let path = directory.path().join("config.json");

    Config::create(&path, "testuser", "SuperGeheimesPasswort123!", true)
        .expect("Config konnte nicht erstellt werden");

    let credentials = Config::load(&path).expect("Config konnte nicht geladen werden");

    assert_eq!(credentials.username, "testuser");

    assert_eq!(credentials.password, "SuperGeheimesPasswort123!");
}

#[test]
fn unencrypted_config_roundtrip() {
    let directory = tempfile::tempdir().expect("Temp-Verzeichnis konnte nicht erstellt werden");

    let path = directory.path().join("config.json");

    Config::create(&path, "testuser", "testpassword", false)
        .expect("Config konnte nicht erstellt werden");

    let credentials = Config::load(&path).expect("Config konnte nicht geladen werden");

    assert_eq!(credentials.username, "testuser");

    assert_eq!(credentials.password, "testpassword");
}

#[test]
fn encrypted_password_is_not_stored_plaintext() {
    let directory = tempfile::tempdir().expect("Temp-Verzeichnis konnte nicht erstellt werden");

    let path = directory.path().join("config.json");

    let password = "SuperGeheimesPasswort123!";

    Config::create(&path, "testuser", password, true).expect("Config konnte nicht erstellt werden");

    let raw = std::fs::read_to_string(&path).expect("Config konnte nicht gelesen werden");

    assert!(
        !raw.contains(password),
        "Passwort steht im Klartext in der Config"
    );
}

#[test]
fn unencrypted_password_is_stored_plaintext() {
    let directory = tempfile::tempdir().expect("Temp-Verzeichnis konnte nicht erstellt werden");

    let path = directory.path().join("config.json");

    let password = "testpassword";

    Config::create(&path, "testuser", password, false)
        .expect("Config konnte nicht erstellt werden");

    let raw = std::fs::read_to_string(&path).expect("Config konnte nicht gelesen werden");

    assert!(
        raw.contains(password),
        "Passwort sollte bei encrypted=false im Klartext stehen"
    );
}

#[test]
fn config_delete_removes_file() {
    let directory = tempfile::tempdir().expect("Temp-Verzeichnis konnte nicht erstellt werden");

    let path = directory.path().join("config.json");

    Config::create(&path, "testuser", "testpassword", false)
        .expect("Config konnte nicht erstellt werden");

    assert!(path.exists());

    Config::delete(&path).expect("Config konnte nicht gelöscht werden");

    assert!(!path.exists(), "Config-Datei existiert nach delete() noch");
}

#[test]
fn empty_username_is_rejected() {
    let directory = tempfile::tempdir().expect("Temp-Verzeichnis konnte nicht erstellt werden");

    let path = directory.path().join("config.json");

    let result = Config::create(&path, "", "password", true);

    assert!(result.is_err());
    assert!(!path.exists());
}

#[test]
fn empty_password_is_rejected() {
    let directory = tempfile::tempdir().expect("Temp-Verzeichnis konnte nicht erstellt werden");

    let path = directory.path().join("config.json");

    let result = Config::create(&path, "username", "", true);

    assert!(result.is_err());
    assert!(!path.exists());
}
