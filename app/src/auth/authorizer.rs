use std::time::Duration;

use ureq::Agent;

pub struct OhgWifiAuthorizer {
    pub username: Option<String>,
    pub password: Option<String>,
}

impl OhgWifiAuthorizer {
    pub fn new() -> Self {
        Self {
            username: None,
            password: None,
        }
    }

    pub fn load_config(&mut self) -> std::io::Result<()> {
        let (username, password) = crate::config::read_config()?;

        self.username = Some(username);
        self.password = Some(password);

        Ok(())
    }

    pub fn authorize(&self) -> bool {
        let (Some(username), Some(password)) = (&self.username, &self.password) else {
            eprintln!("Keine Zugangsdaten im RAM.");
            return false;
        };

        let url = "https://10.80.0.1/api/captiveportal/access/logon/0";

        let config = ureq::config::Config::builder()
            .tls_config(
                ureq::tls::TlsConfig::builder()
                    .disable_verification(true)
                    .build(),
            )
            .timeout_global(Some(Duration::from_secs(5)))
            .build();

        let agent = Agent::from(config);

        let response = agent.post(url).send_form(vec![
            ("user", username.as_str()),
            ("password", password.as_str()),
        ]);

        match response {
            Ok(response) => {
                let body = match response.into_body().read_to_string() {
                    Ok(body) => body,
                    Err(error) => {
                        eprintln!("Antwort konnte nicht gelesen werden: {error}");
                        return false;
                    }
                };

                if body.contains("\"AUTHORIZED\"") {
                    println!("Authentication was successful!");
                    true
                } else {
                    eprintln!("Authentication failed.");
                    false
                }
            }

            Err(error) => {
                eprintln!("Netzwerkfehler bei der Autorisierung: {error}");
                false
            }
        }
    }
}
