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
    pub project: Project,
    pub cloud: Cloud,
    pub operator: Operator,
    pub auth: Auth,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Project {
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Auth {
    pub name: String,
    pub method: String,
    pub token: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Cloud {
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Operator {
    pub name: String,
    pub entrypoint: String,
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
        .context("serializing config")?;

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

pub fn load_context(
    name: Option<&str>,
    dirs: &crate::dirs::Dirs,
) -> miette::Result<Option<Context>> {
    let mut config = load_config(dirs)?;

    if let Some(name) = name {
        let out = config.contexts.remove(name);

        return Ok(out);
    }

    if let Some(name) = config.default_context {
        let out = config.contexts.remove(&name);

        return Ok(out);
    }

    Ok(None)
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
