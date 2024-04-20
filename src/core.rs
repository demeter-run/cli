use std::collections::HashMap;

use miette::{Context as MietteContext, IntoDiagnostic};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Config {
    pub contexts: HashMap<String, Context>,
    pub default_context: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Context {
    pub namespace: Namespace,
    pub cloud: Cloud,
    pub operator: Operator,
    pub auth: Auth,
}

impl Context {
    pub fn ephemeral(namespace: &str, api_key: &str) -> Self {
        let namespace = crate::core::Namespace::new(namespace);
        let auth = crate::core::Auth::api_key(api_key);
        let cloud = crate::core::Cloud::default();
        let operator = crate::core::Operator::default();

        Self {
            namespace,
            auth,
            cloud,
            operator,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Namespace {
    pub name: String,
}

impl Namespace {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Auth {
    pub name: String,
    pub method: String,
    pub token: String,
}

impl Auth {
    pub fn api_key(token: &str) -> Self {
        Self {
            name: "default".to_owned(),
            method: "ApiKey".to_owned(),
            token: token.to_owned(),
        }
    }
}

const DEFAULT_CLOUD: &str = "cloud0.txpipe.io";

#[derive(Serialize, Deserialize, Clone)]
pub struct Cloud {
    pub name: String,
}

impl Default for Cloud {
    fn default() -> Self {
        Self {
            name: DEFAULT_CLOUD.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Operator {
    pub name: String,
    pub entrypoint: String,
}

impl Default for Operator {
    fn default() -> Self {
        Self {
            name: "TxPipe".to_owned(),
            entrypoint: "us1.demeter.run".to_owned(),
        }
    }
}

fn load_config(dirs: &crate::dirs::Dirs) -> miette::Result<Config> {
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

fn save_config(value: Config, dirs: &crate::dirs::Dirs) -> miette::Result<()> {
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
    return Ok(out);
}

pub fn load_default_context(dirs: &crate::dirs::Dirs) -> miette::Result<Option<Context>> {
    let mut config = load_config(dirs)?;

    if let Some(name) = config.default_context {
        let out = config.contexts.remove(&name);

        return Ok(out);
    }

    Ok(None)
}
pub fn infer_context(
    name: Option<&str>,
    namespace: Option<&str>,
    api_key: Option<&str>,
    dirs: &crate::dirs::Dirs,
) -> miette::Result<Option<Context>> {
    match (name, namespace, api_key) {
        (None, Some(ns), Some(ak)) => Ok(Some(Context::ephemeral(ns, ak))),
        (None, None, None) => load_default_context(dirs),
        (Some(context), None, None) => load_context_by_name(context, dirs),
        (None, None, Some(_)) => Err(miette::miette!("missing namespace value")),
        (None, Some(_), None) => Err(miette::miette!("missing api key value")),
        (..) => Err(miette::miette!(
            "conflicting values, specify either a context or namespace"
        )),
    }
}
