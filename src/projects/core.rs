use miette::{Context, IntoDiagnostic};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub api_key: String,
    pub cloud: String,
}

pub fn add_project(dto: Project, ctx: &crate::Context) -> miette::Result<()> {
    // TODO: assert invariants

    let dest = ctx
        .ensure_project_dir(&dto.cloud, &dto.id)?
        .join("config.toml");

    let toml = toml::to_string(&dto)
        .into_diagnostic()
        .context("serializing project config")?;

    std::fs::write(dest, toml)
        .into_diagnostic()
        .context("writing project config file")?;

    Ok(())
}
