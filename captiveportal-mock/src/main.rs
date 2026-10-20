///! DISCLAMER: Test-Server mit KI-geschrieben
use axum::{
    Json, Router,
    extract::{ConnectInfo, Form, State},
    routing::post,
};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use serde::Serialize;
use serde_json::{Value, json};
use std::{
    collections::{HashMap, VecDeque},
    error::Error,
    io::stdout,
    net::SocketAddr,
    sync::{Arc, RwLock},
    time::Duration,
};
use tokio::sync::Mutex;

type AppResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
enum ClientState {
    #[serde(rename = "AUTHORIZED")]
    Authorized,
    #[serde(rename = "NOT_AUTHORIZED")]
    NotAuthorized,
    #[serde(rename = "UNKNOWN")]
    Unknown,
}

impl ClientState {
    const ALL: [ClientState; 3] = [Self::Authorized, Self::NotAuthorized, Self::Unknown];

    fn as_str(self) -> &'static str {
        match self {
            Self::Authorized => "AUTHORIZED",
            Self::NotAuthorized => "NOT_AUTHORIZED",
            Self::Unknown => "UNKNOWN",
        }
    }

    fn from_index(index: usize) -> Self {
        Self::ALL[index % Self::ALL.len()]
    }

    fn index(self) -> usize {
        Self::ALL.iter().position(|s| *s == self).unwrap()
    }
}

type SharedState = Arc<RwLock<ClientState>>;

#[derive(Clone)]
struct AppState {
    status: SharedState,
    login: SharedState,
    requests: Arc<Mutex<VecDeque<RequestEntry>>>,
}

struct RequestEntry {
    time: String,
    client: String,
    endpoint: String,
    response: String,
}

impl AppState {
    async fn record(&self, client: SocketAddr, endpoint: &str, response: ClientState) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Lokale Uhrzeit über die System-Zeitzone anzeigen.
        let time = format_epoch_time(now);

        let entry = RequestEntry {
            time,
            client: client.to_string(),
            endpoint: endpoint.to_string(),
            response: response.as_str().to_string(),
        };

        let mut requests = self.requests.lock().await;
        requests.push_front(entry);

        // Nur die letzten 10 Anfragen behalten.
        requests.truncate(10);
    }
}

// Einfache lokale Zeitformatierung ohne zusätzliche Crate.
// Die Uhrzeit wird in UTC angezeigt.
fn format_epoch_time(epoch: u64) -> String {
    let seconds = epoch % 86_400;
    let hours = seconds / 3_600;
    let minutes = (seconds % 3_600) / 60;
    let secs = seconds % 60;

    format!("{hours:02}:{minutes:02}:{secs:02} UTC")
}

// POST /api/captiveportal/access/status/
async fn status(
    State(app): State<AppState>,
    ConnectInfo(client): ConnectInfo<SocketAddr>,
) -> Json<Value> {
    let response = *app.status.read().unwrap();

    app.record(client, "status", response).await;

    Json(json!({
        "clientState": response.as_str()
    }))
}

// POST /api/captiveportal/access/logon/
async fn login(
    State(app): State<AppState>,
    ConnectInfo(client): ConnectInfo<SocketAddr>,
    Form(_form): Form<HashMap<String, String>>,
) -> Json<Value> {
    let response = *app.login.read().unwrap();

    app.record(client, "logon", response).await;

    Json(json!({
        "clientState": response.as_str()
    }))
}

fn change_state(shared: &SharedState, index: usize) {
    *shared.write().unwrap() = ClientState::from_index(index);
}

fn run_tui(app: AppState) -> AppResult<()> {
    enable_raw_mode()?;

    let mut out = stdout();

    if let Err(error) = execute!(out, EnterAlternateScreen) {
        let _ = disable_raw_mode();
        return Err(error.into());
    }

    let backend = CrosstermBackend::new(out);
    let mut terminal = Terminal::new(backend)?;

    let result = (|| -> AppResult<()> {
        let mut active_endpoint = 0usize;

        loop {
            let status_state = *app.status.read().unwrap();
            let login_state = *app.login.read().unwrap();

            // Die Anfrageliste kurz sperren und als Text übernehmen.
            let request_lines = {
                let requests = app.requests.blocking_lock();

                requests
                    .iter()
                    .map(|request| {
                        Line::from(vec![
                            Span::styled(
                                format!("{:<12}", request.time),
                                Style::default().fg(Color::Gray),
                            ),
                            Span::styled(
                                format!("{:<22}", request.client),
                                Style::default().fg(Color::Cyan),
                            ),
                            Span::styled(
                                format!("{:<8}", request.endpoint),
                                Style::default().fg(Color::Yellow),
                            ),
                            Span::styled(
                                request.response.clone(),
                                Style::default()
                                    .fg(Color::Green)
                                    .add_modifier(Modifier::BOLD),
                            ),
                        ])
                    })
                    .collect::<Vec<_>>()
            };

            terminal.draw(|frame| {
                let area = frame.area();

                let selected_style = Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD);

                let status_style = if active_endpoint == 0 {
                    selected_style
                } else {
                    Style::default().fg(Color::White)
                };

                let login_style = if active_endpoint == 1 {
                    selected_style
                } else {
                    Style::default().fg(Color::White)
                };

                let mut lines = vec![
                    Line::from(Span::styled(
                        " Captive Portal Mockserver ",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        format!(
                            "{} POST /api/captiveportal/access/status/",
                            if active_endpoint == 0 { ">" } else { " " }
                        ),
                        status_style,
                    )),
                    Line::from(format!("    Antwort: {}", status_state.as_str())),
                    Line::from(""),
                    Line::from(Span::styled(
                        format!(
                            "{} POST /api/captiveportal/access/logon/",
                            if active_endpoint == 1 { ">" } else { " " }
                        ),
                        login_style,
                    )),
                    Line::from(format!("    Antwort: {}", login_state.as_str())),
                    Line::from(""),
                    Line::from(" 1 AUTHORIZED | 2 NOT_AUTHORIZED | 3 UNKNOWN"),
                    Line::from(" Tab: Endpunkt wechseln | ↑/↓: Antwort | q: Beenden"),
                    Line::from(""),
                    Line::from(Span::styled(
                        " Letzte Anfragen:",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    )),
                    Line::from(vec![
                        Span::styled(
                            format!("{:<12}", "Uhrzeit"),
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            format!("{:<22}", "Client-IP:Port"),
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            format!("{:<8}", "API"),
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                        Span::styled("Antwort", Style::default().add_modifier(Modifier::BOLD)),
                    ]),
                ];

                lines.extend(request_lines);

                let widget = Paragraph::new(lines).block(
                    Block::default()
                        .title(" API-Steuerung & Live-Requests ")
                        .borders(Borders::ALL),
                );

                frame.render_widget(widget, area);
            })?;

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    let shared = if active_endpoint == 0 {
                        &app.status
                    } else {
                        &app.login
                    };

                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Tab => active_endpoint = 1 - active_endpoint,
                        KeyCode::Up => {
                            let current = shared.read().unwrap().index();
                            change_state(shared, (current + 2) % 3);
                        }
                        KeyCode::Down => {
                            let current = shared.read().unwrap().index();
                            change_state(shared, (current + 1) % 3);
                        }
                        KeyCode::Char('1') => change_state(shared, 0),
                        KeyCode::Char('2') => change_state(shared, 1),
                        KeyCode::Char('3') => change_state(shared, 2),
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    })();

    let raw_result = disable_raw_mode();
    let screen_result = execute!(terminal.backend_mut(), LeaveAlternateScreen);
    let cursor_result = terminal.show_cursor();

    result?;
    raw_result?;
    screen_result?;
    cursor_result?;

    Ok(())
}

#[tokio::main]
async fn main() -> AppResult<()> {
    let app = AppState {
        status: Arc::new(RwLock::new(ClientState::Unknown)),
        login: Arc::new(RwLock::new(ClientState::Authorized)),
        requests: Arc::new(Mutex::new(VecDeque::new())),
    };

    let router = Router::new()
        .route("/api/captiveportal/access/status/", post(status))
        .route("/api/captiveportal/access/logon/", post(login))
        .with_state(app.clone());

    let addr: SocketAddr = "10.80.0.1:8000".parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;

    println!("Mockserver läuft auf http://{addr}");

    tokio::spawn(async move {
        if let Err(error) = axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        {
            eprintln!("HTTP-Serverfehler: {error}");
        }
    });

    tokio::task::spawn_blocking(move || run_tui(app)).await??;

    Ok(())
}
