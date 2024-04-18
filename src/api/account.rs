use std::env;

use reqwest::{Client, Error};
use serde::Deserialize;
use serde_json::json;
use spinners::{Spinner, Spinners};

use super::{build_agent_header, check_response_update_header};

pub async fn get<T>(access_token: &str, path: &str) -> Result<T, Error>
where
    T: for<'de> Deserialize<'de>,
{
    let url = format!("{}/{}", build_api_url(), path);

    let mut sp = Spinner::new(Spinners::Dots, "".into());

    let client = Client::new();
    let resp = client
        .get(url)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("agent", build_agent_header())
        .send()
        .await?;

    sp.stop_with_symbol("".into());

    check_response_update_header(&resp)?;
    let response = resp.json::<T>().await?;
    Ok(response)
}

pub async fn create_api_key(
    access_token: &str,
    project: &str,
    name: &str,
) -> Result<String, Error> {
    let url = format!("{}/{}/api-key", build_api_url(), project);

    let mut sp = Spinner::new(Spinners::Dots, "".into());

    let client = Client::new();
    let resp = client
        .post(url)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("agent", build_agent_header())
        .json(&json!({
            "name": name,
        }))
        .send()
        .await?;

    sp.stop_with_symbol("".into());

    check_response_update_header(&resp)?;
    let response = resp.json::<String>().await?;
    Ok(response)
}

pub async fn create_project(
    access_token: &str,
    organization: &u64,
    name: &str,
    description: &str,
) -> Result<String, Error> {
    let url = format!("{}/projects", build_api_url());

    let mut sp = Spinner::new(Spinners::Dots, "".into());

    let client = Client::new();
    let resp = client
        .post(url)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("agent", build_agent_header())
        .json(&json!({
            "name": name,
            "description": description,
            "organizationId": organization,
        }))
        .send()
        .await?;

    sp.stop_with_symbol("".into());

    check_response_update_header(&resp)?;
    let response = resp.json::<String>().await?;
    Ok(response)
}

fn build_api_url() -> String {
    format!("{}/mgmt/account", get_base_url())
}

fn get_base_url() -> String {
    let api_base_url = "http://localhost:3000".into();
    env::var("API_BASE_URL").unwrap_or(api_base_url)
}
