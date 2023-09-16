use std::{collections::HashMap, str::FromStr, time::Duration};

use reqwest::{header::HeaderValue, Method, StatusCode, Url};

async fn find_login_url() -> (String, String) {
    let mut params = HashMap::new();
    params.insert("client_id", "7QK7kqIVjzoUDlgs7Jr4a4R85TMb6ynZ");
    params.insert("scope", "profile");
    //params.insert("audience", "demeter-api");

    let client = reqwest::Client::new();

    let req = client
        .post("https://dev-dflg0ssi.us.auth0.com/oauth/device/code")
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

async fn poll_token(device_code: &str) -> StatusCode {
    let mut params = HashMap::new();
    params.insert("client_id", "7QK7kqIVjzoUDlgs7Jr4a4R85TMb6ynZ");
    params.insert("device_code", device_code);
    params.insert("grant_type", "urn:ietf:params:oauth:grant-type:device_code");

    let client = reqwest::Client::new();

    let req = client
        .post("https://dev-dflg0ssi.us.auth0.com/oauth/token")
        .header("content-type", "application/x-www-form-urlencoded")
        .form(&params)
        .build()
        .unwrap();

    let res = client.execute(req).await.unwrap();

    res.status()
}

pub async fn run() -> miette::Result<()> {
    let (url, device_code) = find_login_url().await;
    println!("click here to login: {}", url);

    for i in 0..20 {
        tokio::time::sleep(Duration::from_secs(5)).await;
        let status = poll_token(&device_code).await;

        if status.is_success() {
            println!("login successful!");
            break;
        }
    }

    Ok(())
}
