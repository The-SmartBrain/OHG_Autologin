#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod installer;
mod platform;
mod privilege;

use installer::InstallerApp;

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|arg| arg == "--privileged-install") {
        return installer::run_privileged_install();
    }

    if args.iter().any(|arg| arg == "--privileged-uninstall") {
        return installer::run_privileged_uninstall();
    }

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([620.0, 460.0])
            .with_min_inner_size([520.0, 380.0])
            .with_resizable(false),

        ..Default::default()
    };

    eframe::run_native(
        "OHG WLAN Autologin – Installation",
        options,
        Box::new(|cc| {
            installer::setup_fonts(&cc.egui_ctx);

            Ok(Box::new(InstallerApp::new()))
        }),
    )
}
