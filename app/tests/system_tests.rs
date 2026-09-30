#![cfg(feature = "test-system")]

use std::thread;
use std::time::{Duration, Instant};

use ohg_autologin::system::testing;

struct Cleanup;

impl Drop for Cleanup {
    fn drop(&mut self) {
        testing::cleanup();
    }
}

fn cleanup_before_test() {
    testing::cleanup();

    assert!(
        !testing::is_installed(),
        "Test-Dienst konnte vor dem Test nicht entfernt werden"
    );
}

fn wait_until_running() {
    let timeout = Duration::from_secs(10);
    let start = Instant::now();

    while start.elapsed() < timeout {
        if testing::is_running().unwrap_or(false) {
            return;
        }

        thread::sleep(Duration::from_millis(100));
    }

    panic!("Test-Dienst wurde nicht innerhalb von 10 Sekunden gestartet");
}

fn wait_until_stopped() {
    let timeout = Duration::from_secs(10);
    let start = Instant::now();

    while start.elapsed() < timeout {
        if !testing::is_running().unwrap_or(false) {
            return;
        }

        thread::sleep(Duration::from_millis(100));
    }

    panic!("Test-Dienst wurde nicht innerhalb von 10 Sekunden beendet");
}

#[test]
fn complete_service_lifecycle() {
    let _cleanup = Cleanup;

    cleanup_before_test();

    // ========================================================
    // Ausgangszustand
    // ========================================================

    assert!(
        !testing::is_installed(),
        "Test-Dienst ist bereits installiert"
    );

    // ========================================================
    // INSTALL
    // ========================================================

    testing::install().expect("install() ist fehlgeschlagen");

    assert!(
        testing::is_installed(),
        "Dienst ist nach install() nicht installiert"
    );

    // install() darf NICHT automatisch aktivieren.

    assert!(
        !testing::is_enabled().expect("is_enabled() fehlgeschlagen"),
        "Dienst wurde durch install() unerwartet aktiviert"
    );

    // Und er darf natürlich auch noch nicht laufen.

    assert!(
        !testing::is_running().expect("is_running() fehlgeschlagen"),
        "Dienst läuft bereits nach install()"
    );

    // ========================================================
    // ENABLE
    // ========================================================

    testing::enable().expect("enable() ist fehlgeschlagen");

    assert!(
        testing::is_enabled().expect("is_enabled() fehlgeschlagen"),
        "Dienst ist nach enable() nicht aktiviert"
    );

    // enable() darf nicht automatisch starten.

    assert!(
        !testing::is_running().expect("is_running() fehlgeschlagen"),
        "Dienst wurde durch enable() unerwartet gestartet"
    );

    // ========================================================
    // START
    // ========================================================

    testing::start().expect("start() ist fehlgeschlagen");

    wait_until_running();

    assert!(
        testing::is_running().expect("is_running() fehlgeschlagen"),
        "Dienst läuft nach start() nicht"
    );

    // ========================================================
    // STOP
    // ========================================================

    testing::stop().expect("stop() ist fehlgeschlagen");

    wait_until_stopped();

    assert!(
        !testing::is_running().expect("is_running() fehlgeschlagen"),
        "Dienst läuft nach stop() noch"
    );

    // Stop darf die Aktivierung nicht entfernen.

    assert!(
        testing::is_enabled().expect("is_enabled() fehlgeschlagen"),
        "stop() hat den Dienst unerwartet deaktiviert"
    );

    // ========================================================
    // DISABLE
    // ========================================================

    testing::disable().expect("disable() ist fehlgeschlagen");

    assert!(
        !testing::is_enabled().expect("is_enabled() fehlgeschlagen"),
        "Dienst ist nach disable() noch aktiviert"
    );

    // ========================================================
    // START TROTZ DISABLE
    // ========================================================

    // Ein deaktivierter Dienst kann manuell weiterhin
    // gestartet werden.

    testing::start().expect("Manueller Start eines deaktivierten Dienstes fehlgeschlagen");

    wait_until_running();

    assert!(
        testing::is_running().expect("is_running() fehlgeschlagen"),
        "Deaktivierter Dienst konnte nicht manuell gestartet werden"
    );

    testing::stop().expect("stop() nach deaktiviertem Start fehlgeschlagen");

    wait_until_stopped();

    // ========================================================
    // UNINSTALL
    // ========================================================

    testing::uninstall().expect("uninstall() ist fehlgeschlagen");

    assert!(
        !testing::is_installed(),
        "Dienst existiert nach uninstall() noch"
    );
}

#[test]
fn install_does_not_enable() {
    let _cleanup = Cleanup;

    cleanup_before_test();

    testing::install().expect("Installation fehlgeschlagen");

    assert!(testing::is_installed());

    assert!(!testing::is_enabled().expect("Status konnte nicht abgefragt werden"));
}

#[test]
fn enable_and_disable_work() {
    let _cleanup = Cleanup;

    cleanup_before_test();

    testing::install().expect("Installation fehlgeschlagen");

    testing::enable().expect("Enable fehlgeschlagen");

    assert!(testing::is_enabled().expect("Enabled-Status konnte nicht abgefragt werden"));

    testing::disable().expect("Disable fehlgeschlagen");

    assert!(!testing::is_enabled().expect("Enabled-Status konnte nicht abgefragt werden"));
}

#[test]
fn start_and_stop_work() {
    let _cleanup = Cleanup;

    cleanup_before_test();

    testing::install().expect("Installation fehlgeschlagen");

    testing::start().expect("Start fehlgeschlagen");

    wait_until_running();

    assert!(testing::is_running().expect("Running-Status konnte nicht abgefragt werden"));

    testing::stop().expect("Stop fehlgeschlagen");

    wait_until_stopped();

    assert!(!testing::is_running().expect("Running-Status konnte nicht abgefragt werden"));
}

#[test]
fn uninstall_also_cleans_up_running_service() {
    let _cleanup = Cleanup;

    cleanup_before_test();

    testing::install().expect("Installation fehlgeschlagen");

    testing::enable().expect("Enable fehlgeschlagen");

    testing::start().expect("Start fehlgeschlagen");

    wait_until_running();

    testing::uninstall().expect("Uninstall fehlgeschlagen");

    assert!(
        !testing::is_installed(),
        "Dienst ist nach uninstall() noch installiert"
    );
}
