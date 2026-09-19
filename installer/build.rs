use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-env-changed=OHG_APP_BINARY");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let app_binary = find_app_binary();

    let destination = out_dir.join(app_binary_name());

    fs::copy(&app_binary, &destination).unwrap_or_else(|error| {
        panic!(
            "Konnte App-Binary nicht in den Installer \
                 übernehmen.\nQuelle: {}\nZiel: {}\nFehler: {}",
            app_binary.display(),
            destination.display(),
            error
        )
    });

    println!("cargo:rustc-env=OHG_APP_BINARY={}", destination.display());
}

fn find_app_binary() -> PathBuf {
    if let Ok(path) = env::var("OHG_APP_BINARY") {
        let path = PathBuf::from(path);

        if path.exists() {
            return path;
        }

        panic!(
            "OHG_APP_BINARY wurde gesetzt, \
             aber die Datei existiert nicht: {}",
            path.display()
        );
    }

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    let workspace_root = manifest_dir.parent().unwrap();

    let target_dir = workspace_root.join("target");

    let path = target_dir.join("release").join(app_binary_name());

    if path.exists() {
        return path;
    }

    panic!(
        "\n\nDie App-Binary wurde nicht gefunden.\n\
         Bitte zuerst aus dem Workspace ausführen:\n\n\
             cargo build -p ohg-wlan-autologin --release\n\n\
         Gesucht wurde:\n\
             {}\n",
        path.display()
    );
}

fn app_binary_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "ohg-wlan-autologin.exe"
    } else {
        "ohg-wlan-autologin"
    }
}
