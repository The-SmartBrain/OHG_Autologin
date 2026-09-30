mod config;
mod crypto;

use std::io::{self, stdout, Write};

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

const CONFIG_FILE: &str = "config.json";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;

    let mut stdout = stdout();

    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal);

    disable_raw_mode()?;

    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    terminal.show_cursor()?;

    result
}

fn run(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        terminal.draw(|frame| {
            let text = "\
OHG Autologin - Config Test

[s]  Konfiguration speichern
[r]  Konfiguration auslesen
[d]  Alle Daten löschen
[q]  Beenden

";

            let paragraph = Paragraph::new(text)
                .block(Block::default().title("OHG Config").borders(Borders::ALL));

            frame.render_widget(paragraph, frame.area());
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('s') => {
                    save_config()?;
                }

                KeyCode::Char('r') => {
                    read_config()?;
                }

                KeyCode::Char('d') => {
                    delete_config()?;
                }

                KeyCode::Char('q') => {
                    break;
                }

                _ => {}
            }
        }
    }

    Ok(())
}

fn save_config() -> Result<(), Box<dyn std::error::Error>> {
    disable_raw_mode()?;

    println!();
    println!("=== Konfiguration speichern ===");
    println!();

    let username = ask("Benutzername: ")?;

    let password = rpassword::prompt_password("Passwort: ")?;

    println!();
    println!("Passwort verschlüsselt speichern?");
    println!("[j] Ja");
    println!("[n] Nein");

    let encrypted = loop {
        let answer = ask("Auswahl: ")?;

        match answer.trim().to_lowercase().as_str() {
            "j" | "ja" | "y" | "yes" => break true,

            "n" | "nein" | "no" => break false,

            _ => {
                println!("Bitte j oder n eingeben.");
            }
        }
    };

    println!();

    match config::Config::create(CONFIG_FILE, &username, &password, encrypted) {
        Ok(()) => {
            println!("Konfiguration erfolgreich gespeichert.");

            if encrypted {
                println!(
                    "Das Passwort wurde mit der Machine-ID \
                     verschlüsselt."
                );
            } else {
                println!(
                    "Das Passwort wurde unverschlüsselt \
                     gespeichert."
                );
            }
        }

        Err(error) => {
            println!("FEHLER: {}", error);
        }
    }

    wait_for_enter()?;

    enable_raw_mode()?;

    Ok(())
}

fn read_config() -> Result<(), Box<dyn std::error::Error>> {
    disable_raw_mode()?;

    println!();
    println!("=== Konfiguration auslesen ===");
    println!();

    match config::Config::load(CONFIG_FILE) {
        Ok(credentials) => {
            println!("Username:");
            println!("  {}", credentials.username);

            println!();

            println!("Password:");
            println!("  {}", credentials.password);

            println!();
            println!("Erfolgreich entschlüsselt/gelesen.");
        }

        Err(error) => {
            println!("FEHLER:");
            println!("{}", error);
        }
    }

    wait_for_enter()?;

    enable_raw_mode()?;

    Ok(())
}

fn delete_config() -> Result<(), Box<dyn std::error::Error>> {
    disable_raw_mode()?;

    println!();
    println!("=== ALLE DATEN LÖSCHEN ===");
    println!();
    println!("Dabei wird gelöscht:");
    println!("  config.json");
    println!();

    let confirmation = ask("Wirklich löschen? [ja/nein]: ")?;

    if confirmation.trim().to_lowercase() != "ja" {
        println!("Abgebrochen.");
        wait_for_enter()?;
        enable_raw_mode()?;
        return Ok(());
    }

    match config::Config::delete(CONFIG_FILE) {
        Ok(()) => {
            println!("Konfiguration erfolgreich gelöscht.");
        }

        Err(error) => {
            println!("FEHLER: {}", error);
        }
    }

    wait_for_enter()?;

    enable_raw_mode()?;

    Ok(())
}

fn ask(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    print!("{}", prompt);
    stdout().flush()?;

    let mut input = String::new();

    io::stdin().read_line(&mut input)?;

    Ok(input.trim_end().to_string())
}

fn wait_for_enter() -> Result<(), Box<dyn std::error::Error>> {
    println!();
    println!("ENTER drücken, um zurückzukehren...");

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(())
}
