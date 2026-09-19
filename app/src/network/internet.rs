use std::time::Duration;
use ureq::Agent;

pub fn check_internet() -> bool {
    let config = ureq::config::Config::builder()
        .max_redirects(5)
        .timeout_global(Some(Duration::from_secs(5)))
        .build();

    let agent = Agent::from(config);

    match agent.get("https://www.google.com/generate_204").call() {
        Ok(response) => response.status() == 204,
        Err(_) => false,
    }
}
