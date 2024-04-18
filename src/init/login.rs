use miette::bail;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::Duration};

async fn find_login_url() -> (String, String) {
    let mut params = HashMap::new();
    params.insert("client_id", "gpJ63MG5g1V1PKufM9WHGjjeAe7yCT8L");
    params.insert("scope", "profile openid email");
    params.insert("audience", "demeter-api");

    let client = reqwest::Client::new();

    let req = client
        .post("https://txpipe.us.auth0.com/oauth/device/code")
        .header("content-type", "application/x-www-form-urlencoded")
        .form(&params)
        .build()
        .unwrap();

    let res = client.execute(req).await.unwrap();

    let body: serde_json::Value = res.json().await.unwrap();

    let uri = body
        .get("verification_uri_complete")
        .unwrap()
        .as_str()
        .unwrap();

    let device_code = body.get("device_code").unwrap().as_str().unwrap();

    (uri.into(), device_code.into())
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
enum Auth0ResponseBody {
    Error(Auth0Error),
    Success(Auth0Success),
}

#[derive(Deserialize, Serialize, Debug)]
struct Auth0Error {
    error: String,
    error_description: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct Auth0Success {
    access_token: String,
    expires_in: u64,
    token_type: String,
}

async fn poll_token(device_code: &str) -> (StatusCode, String) {
    let mut params = HashMap::new();
    params.insert("client_id", "gpJ63MG5g1V1PKufM9WHGjjeAe7yCT8L");
    params.insert("device_code", device_code);
    params.insert("grant_type", "urn:ietf:params:oauth:grant-type:device_code");

    let client = reqwest::Client::new();

    let req = client
        .post("https://txpipe.us.auth0.com/oauth/token")
        .header("content-type", "application/x-www-form-urlencoded")
        .form(&params)
        .build()
        .unwrap();

    let res = client.execute(req).await.unwrap();

    // print the text and return the statuscode
    let status_code = res.status();
    let body: Auth0ResponseBody = res.json().await.unwrap();

    if status_code.is_success() {
        match body {
            Auth0ResponseBody::Success(success) => return (status_code, success.access_token),
            Auth0ResponseBody::Error(error) => return (status_code, error.error_description),
        }
    }

    (status_code, "".into())
}

pub async fn run() -> miette::Result<String> {
    let (url, device_code) = find_login_url().await;
    println!("click here to login: {}", url);

    for _i in 0..20 {
        tokio::time::sleep(Duration::from_secs(5)).await;
        let (status, access_token) = poll_token(&device_code).await;

        if status.is_success() {
            println!("login successful!");
            return Ok(access_token);
        }
    }

    bail!("Error: login failed")
}
