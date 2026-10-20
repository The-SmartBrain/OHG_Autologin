use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub enum ClientState {
    #[serde(rename = "AUTHORIZED")]
    Authorized,

    #[serde(rename = "NOT_AUTHORIZED")]
    NotAuthorized,

    #[serde(rename = "UNKNOWN")]
    Unknown,
}
#[derive(Deserialize)]
struct ApiResponse {
    #[serde(rename = "clientState")]
    client_state: ClientState,
}

/// Aktuell mit dem Schulwlan verbunden?
pub fn ohg_wifi() -> bool {
    match ssid::get_ssid() {
        Some(current_ssid) => {
            if current_ssid == "ohg" {
                true
            } else {
                false
            }
        }
        None => false,
    }
}

/// Sendet Anfrage an Server
async fn post_client_state(
    url: &str,
    form_data: &[(&str, &str)],
    custom_user_agent: Option<&str>,
) -> ClientState {
    let client = match custom_user_agent {
        Some(user_agent) => match reqwest::Client::builder()
            .user_agent(user_agent)
            .danger_accept_invalid_certs(true)
            .build()
        {
            Ok(client) => client,
            Err(_) => return ClientState::Unknown,
        },
        None => reqwest::Client::new(),
    };

    let response = match client.post(url).form(form_data).send().await {
        Ok(response) => response,
        Err(_) => return ClientState::Unknown,
    };

    let raw_text = match response.text().await {
        Ok(text) => text,
        Err(_) => return ClientState::Unknown,
    };

    match serde_json::from_str::<ApiResponse>(&raw_text) {
        Ok(api_data) => api_data.client_state,
        Err(_) => ClientState::Unknown,
    }
}

/// Get status  of Login
pub async fn status() -> ClientState {
    let url = "http://10.80.0.1:8000/api/captiveportal/access/status/";
    let form_data = [("user", ""), ("password", "")];

    post_client_state(url, &form_data, None).await
}

/// login
pub async fn login(username: &str, password: &str) -> ClientState {
    let url = "http://10.80.0.1:8000/api/captiveportal/access/logon/";
    let form_data = [("user", username), ("password", password)];

    post_client_state(url, &form_data, Some("TSB: Autologin")).await
}
