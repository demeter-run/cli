use std::collections::HashMap;

use miette::{Context as MietteContext, IntoDiagnostic};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Config {
    pub contexts: HashMap<String, Context>,
    pub default_context: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Context {
    pub project: Project,
    pub cloud: Cloud,
    pub operator: Operator,
    pub auth: Auth,
}

impl Context {
    pub fn ephemeral(id: &str, namespace: &str, api_key: &str, access_token: &str) -> Self {
        let project = crate::context::Project::new(id, namespace, None);
        let auth = crate::context::Auth::api_key(access_token, api_key);
        let cloud = crate::context::Cloud::default();
        let operator = crate::context::Operator::default();

        Self {
            project,
            auth,
            cloud,
            operator,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Project {
    pub id: String,
    pub name: Option<String>,
    pub namespace: String,
}

impl Project {
    pub fn new(id: &str, namespace: &str, name: Option<String>) -> Self {
        Self {
            id: id.to_owned(),
            namespace: namespace.to_owned(),
            name,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Auth {
    pub name: String,
    pub method: String,
    pub token: String,
    pub access_token: String,
}

impl Auth {
    pub fn api_key(access_token: &str, api_key: &str) -> Self {
        Self {
            name: "default".to_owned(),
            method: "ApiKey".to_owned(),
            token: api_key.to_owned(),
            access_token: access_token.to_owned(),
        }
    }
}

const DEFAULT_CLOUD: &str = "cloud0.txpipe.io";

#[derive(Serialize, Deserialize, Clone, Debug)]
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

#[derive(Serialize, Deserialize, Clone, Debug)]
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
    id: Option<&str>,
    name: Option<&str>,
    namespace: Option<&str>,
    api_key: Option<&str>,
    access_token: Option<&str>,
    dirs: &crate::dirs::Dirs,
) -> miette::Result<Option<Context>> {
    match (id, name, namespace, api_key, access_token) {
        (Some(id), None, Some(ns), Some(ak), Some(t)) => {
            Ok(Some(Context::ephemeral(id, ns, ak, t)))
        }
        (None, None, None, None, None) => load_default_context(dirs),
        (None, Some(context), None, None, None) => load_context_by_name(context, dirs),
        (None, None, None, Some(_), None) => Err(miette::miette!("missing namespace or id value")),
        (Some(_), None, Some(_), None, None) => Err(miette::miette!("missing api key value")),
        (..) => Err(miette::miette!(
            "conflicting values, specify either a context or namespace"
        )),
    }
}

pub fn extract_context_data(cli: &crate::Cli) -> (String, String, String, String) {
    let api_key = cli.context.as_ref().unwrap().auth.token.clone();
    let access_token = cli.context.as_ref().unwrap().auth.access_token.clone();
    let namespace = cli.context.as_ref().unwrap().project.namespace.clone();
    let id = cli.context.as_ref().unwrap().project.id.clone();

    (access_token, api_key, id, namespace)
}
