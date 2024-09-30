use reqwest::{Client, Error};
use std::env;

use super::{build_agent_header, check_response_update_header};

pub async fn initialize_user(access_token: &str) -> Result<String, Error> {
    let url = format!("{}/users", build_api_url());

    let client = Client::new();
    let resp = client
        .post(url)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("agent", build_agent_header())
        .send()
        .await?;

    check_response_update_header(&resp)?;
    let response = resp.json::<serde_json::Value>().await?;

    let userid = response
        .as_object()
        .and_then(|o| o.get("userId"))
        .and_then(|x| x.as_str())
        .unwrap_or_default()
        .to_owned();

    Ok(userid)
}

fn build_api_url() -> String {
    format!("{}/mgmt/account", get_base_url())
}

fn get_base_url() -> String {
    let api_base_url = "https://console.us1.demeter.run".into();
    env::var("API_BASE_URL").unwrap_or(api_base_url)
}
