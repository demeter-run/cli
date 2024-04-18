use std::{collections::HashMap, env};

use reqwest::{Client, Error};
use serde::{Deserialize, Serialize};
use serde_json::json;
use spinners::{Spinner, Spinners};

use self::format::format_new_cli_version_available;

pub mod account;
mod format;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PortInfo {
    pub id: String,
    pub kind: String,
    pub key: String,
    pub name: String,
    pub network: String,
    pub tier: String,
    pub version: String,
    pub instance: Instance,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)] // Allows for different shapes of the "instance" object
pub enum Instance {
    PostgresPort(PostgresPortInstance),
    HttpPort(HttpPortInstance),
    NodePort(NodePortInstance),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PostgresPortInstance {
    pub hostname: String,
    pub database: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    #[serde(rename = "connectionString")]
    pub connection_string: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct HttpPortInstance {
    #[serde(rename = "apiKey")]
    pub api_key: String,
    pub endpoint: String,
    #[serde(rename = "authenticatedEndpoint")]
    pub authenticated_endpoint: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NodePortInstance {
    #[serde(rename = "apiKey")]
    pub api_key: String,
    #[serde(rename = "authenticatedEndpoint")]
    pub authenticated_endpoint: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PortOptions {
    pub networks: Vec<String>,
    pub versions: Option<HashMap<String, Vec<String>>>,
    pub tiers: Vec<String>,
}

impl PortOptions {
    pub fn get_network_versions(&self, network: &str) -> Vec<String> {
        self.versions
            .as_ref()
            .unwrap()
            .get(network)
            .unwrap_or(&vec![])
            .clone()
    }
}

pub async fn get_public<T>(path: &str) -> Result<T, Error>
where
    T: for<'de> Deserialize<'de>,
{
    let url = format!("{}/mgmt/{}", get_base_url(), path);

    let client = Client::new();

    let mut sp = Spinner::new(Spinners::Dots, "".into());

    let resp = client
        .get(url)
        .header("agent", build_agent_header())
        .send()
        .await?;

    sp.stop_with_symbol("".into());

    check_response_update_header(&resp)?;
    let response = resp.json::<T>().await?;
    Ok(response)
}

pub async fn get<T>(cli: &crate::Cli, path: &str) -> Result<T, Error>
where
    T: for<'de> Deserialize<'de>,
{
    let (api_key, namespace, base_url) = extract_context_data(cli);

    let url = format!("{}/{}/{}", base_url, namespace, path);

    let client = Client::new();

    let mut sp = Spinner::new(Spinners::Dots, "".into());

    let resp = client
        .get(url)
        .header("dmtr-api-key", api_key)
        .header("agent", build_agent_header())
        .send()
        .await?;

    sp.stop_with_symbol("".into());

    check_response_update_header(&resp)?;
    let response = resp.json::<T>().await?;
    Ok(response)
}

pub async fn create_port(
    cli: &crate::Cli,
    kind: &str,
    network: &str,
    version: &str,
    tier: &str,
) -> Result<PortInfo, Error> {
    let (api_key, namespace, base_url) = extract_context_data(cli);

    let url = format!("{}/{}/ports", base_url, namespace);

    let client = Client::new();

    let mut sp = Spinner::new(Spinners::Dots, "".into());

    let resp = client
        .post(url)
        .header("dmtr-api-key", api_key)
        .header("agent", build_agent_header())
        .json(&json!({
            "kind": kind,
            "network": network,
            "version": version,
            "tier": tier
        }))
        .send()
        .await?;

    sp.stop_with_symbol("".into());

    check_response_update_header(&resp)?;
    let response = resp.json::<PortInfo>().await?;
    Ok(response)
}

pub async fn delete_port(cli: &crate::Cli, kind: &str, id: &str) -> Result<(), Error> {
    let (api_key, namespace, base_url) = extract_context_data(cli);

    let url = format!("{}/{}/ports/{}/{}", base_url, namespace, kind, id);

    let mut sp = Spinner::new(Spinners::Dots, "".into());
    let client = Client::new();
    let _resp = client
        .delete(url)
        .header("dmtr-api-key", api_key)
        .header("agent", build_agent_header())
        .send()
        .await?;

    sp.stop_with_symbol("".into());

    check_response_update_header(&_resp)?;
    Ok(())
}

fn extract_context_data(cli: &crate::Cli) -> (String, String, String) {
    let api_key = cli.context.as_ref().unwrap().auth.token.clone();
    let namespace = cli.context.as_ref().unwrap().namespace.name.clone();
    let base_url = format!("{}/mgmt/project", get_base_url());

    (api_key, namespace, base_url)
}

pub fn check_response_update_header(resp: &reqwest::Response) -> Result<&reqwest::Response, Error> {
    let headers = resp.headers();
    let version = headers.get("dmtr-cli-update");
    if let Some(version) = version {
        let version = version.to_str().unwrap();
        if version != VERSION {
            format_new_cli_version_available(version);
        }
    }
    Ok(resp)
}

pub fn build_agent_header() -> String {
    format!("dmtr-cli/{}", VERSION)
}

fn get_base_url() -> String {
    let api_base_url = "https://console.us1.demeter.run".into();
    env::var("API_BASE_URL").unwrap_or(api_base_url)
}
