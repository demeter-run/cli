use std::path::{Path, PathBuf};

use miette::{Context, IntoDiagnostic};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub project: Project,
    pub cloud: Cloud,
    pub operator: Operator,
    pub auth: Option<Auth>,
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

pub fn define_config_location(
    project_key: &str,
    cloud_key: &str,
    dirs: &crate::dirs::Dirs,
) -> miette::Result<PathBuf> {
    let defined = dirs
        .ensure_project_dir(cloud_key, project_key)?
        .join("dmtrctl.toml");

    Ok(defined)
}

fn load_config_file(location: &Path) -> miette::Result<Config> {
    let toml = std::fs::read_to_string(location)
        .into_diagnostic()
        .context("reading project config file")?;

    let dto = toml::from_str(&toml)
        .into_diagnostic()
        .context("serializing project config")?;

    Ok(dto)
}

fn infer_auth(apikey: &str) -> Auth {
    crate::core::Auth {
        name: "root".to_owned(),
        method: "ApiKey".to_owned(),
        token: apikey.to_owned(),
    }
}

fn infer_config(project: &str, cloud: &str, apikey: Option<&str>) -> Config {
    crate::core::Config {
        project: Project {
            name: project.to_owned(),
        },
        auth: apikey.map(infer_auth),
        cloud: Cloud {
            name: cloud.to_owned(),
        },
        operator: Operator {
            name: "TxPipe".to_owned(),
            entrypoint: "us1.demeter.run".to_owned(),
        },
    }
}

pub fn load_or_infer_config(
    project: &str,
    cloud: &str,
    apikey: Option<&str>,
    dirs: &crate::dirs::Dirs,
) -> miette::Result<Config> {
    let location = define_config_location(&project, &cloud, dirs)?;

    if location.is_file() {
        load_config_file(&location)
    } else {
        Ok(infer_config(project, cloud, apikey))
    }
}

pub fn overwrite_current_config(
    project: &str,
    cloud: &str,
    dto: Config,
    dirs: &crate::dirs::Dirs,
) -> miette::Result<()> {
    let location = define_config_location(project, cloud, dirs)?;

    let toml = toml::to_string(&dto)
        .into_diagnostic()
        .context("serializing project config")?;

    std::fs::write(location, toml)
        .into_diagnostic()
        .context("writing project config file")?;

    Ok(())
}
