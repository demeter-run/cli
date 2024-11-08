use std::collections::HashMap;

use miette::{Context as MietteContext, IntoDiagnostic};
use serde::{Deserialize, Serialize};

use crate::rpc;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Config {
    pub contexts: HashMap<String, Context>,
    pub default_context: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Context {
    pub namespace: Namespace,
    pub auth: Auth,
}

impl Context {
    pub async fn ephemeral(namespace: &str, api_key: &str) -> miette::Result<Self> {
        let namespace = crate::context::Namespace::new(namespace, None);
        let auth = crate::context::Auth::api_key(api_key);

        Ok(Self { namespace, auth })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Namespace {
    pub name: String,
    pub caption: Option<String>,
}
impl Namespace {
    pub fn new(name: &str, caption: Option<String>) -> Self {
        Self {
            name: name.to_owned(),
            caption,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Auth {
    pub name: String,
    pub method: String,
    pub token: String,
}

impl Auth {
    pub fn api_key(api_key: &str) -> Self {
        Self {
            name: "default".to_owned(),
            method: "ApiKey".to_owned(),
            token: api_key.to_owned(),
        }
    }
}

pub fn load_config(dirs: &crate::dirs::Dirs) -> miette::Result<Config> {
    let location = dirs.root_dir().join("config.toml");

    if !location.exists() {
        return Ok(Config::default());
    }

    let toml = std::fs::read_to_string(location)
        .into_diagnostic()
        .context("reading project config file")?;

    let dto = toml::from_str(&toml)
        .into_diagnostic()
        .context("deserializing config")?;

    Ok(dto)
}

pub fn save_config(value: Config, dirs: &crate::dirs::Dirs) -> miette::Result<()> {
    let location = dirs.root_dir().join("config.toml");

    let toml = toml::to_string(&value)
        .into_diagnostic()
        .context("serializing config")?;

    std::fs::write(location, toml)
        .into_diagnostic()
        .context("writing config file")?;

    Ok(())
}

pub fn clear_config(dirs: &crate::dirs::Dirs) -> miette::Result<()> {
    let location = dirs.root_dir().join("config.toml");

    std::fs::remove_file(location)
        .into_diagnostic()
        .context("deleting toml file")?;

    Ok(())
}

pub fn set_default_context(name: &str, dirs: &crate::dirs::Dirs) -> miette::Result<()> {
    let mut config = load_config(dirs)?;

    config.default_context = Some(name.to_string());

    save_config(config, dirs)?;

    Ok(())
}

pub fn overwrite_context(
    name: &str,
    dto: Context,
    set_default: bool,
    dirs: &crate::dirs::Dirs,
) -> miette::Result<()> {
    let mut config = load_config(dirs)?;

    config.contexts.insert(name.to_string(), dto);

    if set_default {
        config.default_context = Some(name.to_string());
    }

    save_config(config, dirs)?;

    Ok(())
}

pub fn load_context_by_name(
    name: &str,
    dirs: &crate::dirs::Dirs,
) -> miette::Result<Option<Context>> {
    let mut config = load_config(dirs)?;
    let out = config.contexts.remove(name);
    Ok(out)
}

pub fn load_default_context(dirs: &crate::dirs::Dirs) -> miette::Result<Option<Context>> {
    let mut config = load_config(dirs)?;

    if let Some(name) = config.default_context {
        let out = config.contexts.remove(&name);

        return Ok(out);
    }

    Ok(None)
}

pub async fn infer_context(
    name: Option<&str>,
    namespace: Option<&str>,
    api_key: Option<&str>,
    dirs: &crate::dirs::Dirs,
) -> miette::Result<Option<Context>> {
    match (name, namespace, api_key) {
        (None, Some(namespace), Some(ak)) => Ok(Some(Context::ephemeral(namespace, ak).await?)),
        (None, None, Some(_)) => Err(miette::miette!("missing project id value")),
        (None, Some(_), None) => Err(miette::miette!("missing api key value")),
        (Some(context), _, _) => load_context_by_name(context, dirs),
        _ => load_default_context(dirs),
    }
}

pub async fn extract_context_data(cli: &crate::Cli) -> miette::Result<(String, String, String)> {
    let api_key = cli.context.as_ref().unwrap().auth.token.clone();
    let namespace = cli.context.as_ref().unwrap().namespace.name.clone();

    let project = rpc::projects::find_by_namespace(
        rpc::auth::Credential::Secret((namespace.clone(), api_key.clone())),
        &namespace,
    )
    .await?;

    Ok((api_key, project.id, namespace))
}
