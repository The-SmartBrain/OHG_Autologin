use eframe::egui;

use crate::config;

pub fn run_config_window() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 500.0])
            .with_min_inner_size([500.0, 400.0]),
        ..Default::default()
    };

    eframe::run_native(
        "OHG WLAN Autologin",
        options,
        Box::new(|cc| {
            setup_fonts(&cc.egui_ctx);

            Ok(Box::new(ConfigApp::new()))
        }),
    )
}

fn setup_fonts(ctx: &egui::Context) {
    let fonts = egui::FontDefinitions::default();

    ctx.set_fonts(fonts);
}

struct ConfigApp {
    username: String,

    // Zugangsdaten-Fenster
    credentials_window: bool,

    credentials_username: String,
    credentials_password: String,

    credentials_error: Option<String>,

    // Allgemeiner Status
    status_message: Option<String>,

    // Erweitert
    uninstall_requested: bool,
}

impl ConfigApp {
    fn new() -> Self {
        let username = match config::read_config() {
            Ok((username, _password)) => username,
            Err(_) => String::new(),
        };

        Self {
            username: username.clone(),

            credentials_window: false,

            credentials_username: username,
            credentials_password: String::new(),

            credentials_error: None,

            status_message: None,

            uninstall_requested: false,
        }
    }

    // HAUPTFENSTER

    fn show_main_window(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(10.0);

            // Überschrift

            ui.heading("OHG WLAN Autologin");

            ui.add_space(20.0);

            // Zugangsdaten

            ui.group(|ui| {
                ui.heading("Zugangsdaten");

                ui.add_space(8.0);

                if self.username.is_empty() {
                    ui.label("Keine Zugangsdaten konfiguriert.");
                } else {
                    ui.horizontal(|ui| {
                        ui.label("Benutzername:");

                        ui.strong(&self.username);
                    });
                }

                ui.add_space(10.0);

                if ui.button("Zugangsdaten ändern").clicked() {
                    self.credentials_username = self.username.clone();

                    self.credentials_password.clear();

                    self.credentials_error = None;

                    self.credentials_window = true;
                }
            });

            ui.add_space(15.0);

            // Status

            ui.group(|ui| {
                ui.heading("Status");

                ui.add_space(8.0);

                if self.username.is_empty() {
                    ui.colored_label(
                        egui::Color32::from_rgb(220, 160, 40),
                        "● Keine Konfiguration",
                    );
                } else {
                    ui.colored_label(
                        egui::Color32::from_rgb(70, 180, 100),
                        "● Konfiguration vorhanden",
                    );
                }

                if let Some(status) = &self.status_message {
                    ui.add_space(5.0);

                    ui.label(status);
                }
            });

            ui.add_space(15.0);

            // Erweitert

            egui::CollapsingHeader::new("Erweitert")
                .default_open(false)
                .show(ui, |ui| {
                    self.show_advanced(ui);
                });
        });
    }

    // ERWEITERTE EINSTELLUNGEN

    fn show_advanced(&mut self, ui: &mut egui::Ui) {
        ui.add_space(5.0);

        // Autostart

        ui.label("Autostart");

        ui.add_space(8.0);

        ui.horizontal(|ui| {
            if ui.button("Autostart stoppen").clicked() {
                match crate::system::disable_autostart() {
                    Ok(()) => {
                        self.status_message = Some("Autostart wurde deaktiviert.".to_string());
                    }

                    Err(error) => {
                        self.status_message =
                            Some(format!("Fehler beim Deaktivieren des Autostarts: {error}"));
                    }
                }
            }

            if ui.button("Autostart neu einrichten").clicked() {
                match crate::system::enable_autostart() {
                    Ok(()) => {
                        self.status_message = Some("Autostart wurde eingerichtet.".to_string());
                    }

                    Err(error) => {
                        self.status_message =
                            Some(format!("Fehler beim Einrichten des Autostarts: {error}"));
                    }
                }
            }
        });

        ui.add_space(20.0);

        ui.separator();

        ui.add_space(10.0);

        // Deinstallation

        ui.label("Deinstallation");

        ui.label("Entfernt Konfiguration und Autostart.");

        ui.add_space(8.0);

        if ui.button("Anwendung deinstallieren").clicked() {
            self.uninstall_requested = true;
        }

        // Deinstallationsbestätigung

        if self.uninstall_requested {
            self.show_uninstall_confirmation(ui.ctx());
        }
    }

    // DEINSTALLATIONSBESTÄTIGUNG

    fn show_uninstall_confirmation(&mut self, ctx: &egui::Context) {
        egui::Window::new("Deinstallation bestätigen")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("Möchtest du die Anwendung wirklich entfernen?");

                ui.add_space(8.0);

                ui.label("Dabei werden die Konfiguration und der Autostart entfernt.");

                ui.add_space(15.0);

                ui.horizontal(|ui| {
                    if ui.button("Abbrechen").clicked() {
                        self.uninstall_requested = false;
                    }

                    if ui.button("Deinstallieren").clicked() {
                        match crate::system::uninstall() {
                            Ok(()) => {
                                self.status_message =
                                    Some("Deinstallation abgeschlossen.".to_string());

                                self.uninstall_requested = false;
                            }

                            Err(error) => {
                                self.status_message =
                                    Some(format!("Deinstallation fehlgeschlagen: {error}"));

                                self.uninstall_requested = false;
                            }
                        }
                    }
                });
            });
    }

    // ZUGANGSDATEN-FENSTER

    fn show_credentials_window(&mut self, ctx: &egui::Context) {
        if !self.credentials_window {
            return;
        }

        egui::Window::new("Zugangsdaten ändern")
            .collapsible(false)
            .resizable(false)
            .default_width(450.0)
            .show(ctx, |ui| {
                ui.heading("Zugangsdaten");

                ui.add_space(15.0);

                // Benutzername

                ui.label("Benutzername");

                ui.add(
                    egui::TextEdit::singleline(&mut self.credentials_username)
                        .desired_width(f32::INFINITY),
                );

                ui.add_space(12.0);

                // Passwort

                ui.label("Passwort");

                ui.add(
                    egui::TextEdit::singleline(&mut self.credentials_password)
                        .password(true)
                        .desired_width(f32::INFINITY),
                );

                ui.add_space(10.0);

                // Fehlermeldung

                if let Some(error) = &self.credentials_error {
                    ui.colored_label(egui::Color32::RED, error);

                    ui.add_space(5.0);
                }

                ui.add_space(10.0);

                // Buttons

                ui.horizontal(|ui| {
                    if ui.button("Abbrechen").clicked() {
                        // Passwort aus dem GUI-State entfernen.
                        self.credentials_password.clear();

                        self.credentials_error = None;

                        // Fenster schließen.
                        self.credentials_window = false;
                    }

                    if ui.button("Speichern").clicked() {
                        self.save_credentials();
                    }
                });
            });
    }

    // ZUGANGSDATEN SPEICHERN

    fn save_credentials(&mut self) {
        let username = self.credentials_username.trim();

        // Benutzername prüfen

        if username.is_empty() {
            self.credentials_error = Some("Bitte einen Benutzernamen eingeben.".to_string());

            return;
        }

        // Passwort prüfen

        if self.credentials_password.is_empty() {
            self.credentials_error = Some("Bitte ein Passwort eingeben.".to_string());

            return;
        }

        // Speichern

        match config::save_config(username, &self.credentials_password) {
            Ok(()) => {
                // Benutzername in der Hauptansicht aktualisieren.
                self.username = username.to_string();

                // Passwort nicht länger im GUI-State behalten.
                self.credentials_password.clear();

                self.credentials_error = None;

                self.status_message = Some("Zugangsdaten wurden gespeichert.".to_string());

                // Zugangsdaten-Fenster schließen.
                self.credentials_window = false;
            }

            Err(error) => {
                // Bei einem Fehler bleibt das Fenster geöffnet.
                self.credentials_error = Some(format!("Speichern fehlgeschlagen: {error}"));
            }
        }
    }
}

// EFRAME APP

impl eframe::App for ConfigApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.show_main_window(ctx);

        self.show_credentials_window(ctx);
    }
}
