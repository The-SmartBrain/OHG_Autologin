use eframe::egui;

use crate::platform;
use crate::privilege;

const APP_BINARY: &[u8] = include_bytes!(env!("OHG_APP_BINARY"));

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Welcome,
    Installing,
    Finished,
    Failed,
}

pub struct InstallerApp {
    state: State,
    message: String,
    error: Option<String>,
}

impl InstallerApp {
    pub fn new() -> Self {
        Self {
            state: State::Welcome,
            message: String::new(),
            error: None,
        }
    }

    fn install(&mut self) {
        self.state = State::Installing;

        self.message = "Administratorrechte werden angefordert...".to_string();

        self.error = None;

        match privilege::install_with_privileges() {
            Ok(()) => {
                self.state = State::Finished;

                self.message = "Die Installation wurde erfolgreich abgeschlossen.".to_string();
            }

            Err(error) => {
                self.state = State::Failed;

                self.error = Some(error);
            }
        }
    }

    fn uninstall(&mut self) {
        self.state = State::Installing;

        self.message = "Administratorrechte werden angefordert...".to_string();

        self.error = None;

        match privilege::uninstall_with_privileges() {
            Ok(()) => {
                self.state = State::Finished;

                self.message = "Die Deinstallation wurde erfolgreich abgeschlossen.".to_string();
            }

            Err(error) => {
                self.state = State::Failed;

                self.error = Some(error);
            }
        }
    }
}

impl eframe::App for InstallerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(35.0);

                match self.state {
                    State::Welcome => {
                        self.show_welcome(ui);
                    }

                    State::Installing => {
                        self.show_installing(ui);
                    }

                    State::Finished => {
                        self.show_finished(ui);
                    }

                    State::Failed => {
                        self.show_failed(ui);
                    }
                }
            });
        });
    }
}

impl InstallerApp {
    fn show_welcome(&mut self, ui: &mut egui::Ui) {
        ui.heading("OHG WLAN Autologin");

        ui.add_space(12.0);

        ui.label("Willkommen beim Installationsprogramm.");

        ui.add_space(10.0);

        ui.label(
            "Die Anwendung wird auf diesem \
             Computer installiert und für den \
             automatischen Start eingerichtet.",
        );

        ui.add_space(25.0);

        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.label(format!("Betriebssystem: {}", operating_system()));

            ui.label(format!("App-Version: {}", env!("CARGO_PKG_VERSION")));
        });

        ui.add_space(30.0);

        ui.horizontal(|ui| {
            if ui.button("Abbrechen").clicked() {
                std::process::exit(0);
            }

            if ui.button("Installieren").clicked() {
                self.install();
            }
        });
    }

    fn show_installing(&self, ui: &mut egui::Ui) {
        ui.heading("Installation");

        ui.add_space(25.0);

        ui.spinner();

        ui.add_space(10.0);

        ui.label(&self.message);
    }

    fn show_finished(&mut self, ui: &mut egui::Ui) {
        ui.heading("Fertig");

        ui.add_space(20.0);

        ui.label(&self.message);

        ui.add_space(30.0);

        if ui.button("Anwendung starten").clicked() {
            if let Err(error) = launch_app() {
                self.state = State::Failed;

                self.error = Some(error);

                return;
            }

            std::process::exit(0);
        }

        if ui.button("Schließen").clicked() {
            std::process::exit(0);
        }
    }

    fn show_failed(&mut self, ui: &mut egui::Ui) {
        ui.heading("Vorgang fehlgeschlagen");

        ui.add_space(20.0);

        if let Some(error) = &self.error {
            ui.colored_label(egui::Color32::RED, error);
        }

        ui.add_space(25.0);

        ui.horizontal(|ui| {
            if ui.button("Erneut versuchen").clicked() {
                self.state = State::Welcome;

                self.error = None;
            }

            if ui.button("Schließen").clicked() {
                std::process::exit(1);
            }
        });
    }
}

pub fn run_privileged_install() -> eframe::Result<()> {
    platform::install(APP_BINARY).map_err(|error| {
        eframe::Error::AppCreation(Box::new(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            error,
        )))
    })
}

pub fn run_privileged_uninstall() -> eframe::Result<()> {
    platform::uninstall().map_err(|error| {
        eframe::Error::AppCreation(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            error,
        )))
    })
}

pub fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // eigene TTF hier einbinden

    fonts
        .families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap();

    ctx.set_fonts(fonts);
}

fn operating_system() -> &'static str {
    #[cfg(target_os = "linux")]
    {
        "Linux"
    }

    #[cfg(target_os = "windows")]
    {
        "Windows"
    }

    #[cfg(target_os = "macos")]
    {
        "macOS"
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        "Unbekannt"
    }
}

fn launch_app() -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("/opt/ohg-wlan-autologin/ohg-wlan-autologin --config")
            .spawn()
            .map_err(|e| e.to_string())?;

        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new(
            r"C:\Program Files\OHG WLAN Autologin\ohg-wlan-autologin.exe --config",
        )
        .spawn()
        .map_err(|e| e.to_string())?;

        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("/Applications/OHG WLAN Autologin.app")
            .spawn()
            .map_err(|e| e.to_string())?;

        return Ok(());
    }

    #[allow(unreachable_code)]
    Err("Betriebssystem nicht unterstützt.".to_string())
}
