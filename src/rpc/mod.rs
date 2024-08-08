use std::env;

pub mod auth;
pub mod metadata;
pub mod projects;
pub mod resources;

pub fn get_base_url() -> String {
    let api_base_url = "http://0.0.0.0:5000".into();
    env::var("RPC_BASE_URL").unwrap_or(api_base_url)
}
