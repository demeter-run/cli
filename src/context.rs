use miette::IntoDiagnostic;
use std::path::PathBuf;

use crate::Cli;

fn default_root_dir() -> miette::Result<PathBuf> {
    let defined = dirs::home_dir()
        .ok_or(miette::Error::msg("no home directory"))?
        .join(".dmtr");

    Ok(defined)
}

pub fn ensure_root_dir(explicit: &Option<PathBuf>) -> miette::Result<PathBuf> {
    let default = default_root_dir()?;

    let defined = explicit.to_owned().unwrap_or(default);

    std::fs::create_dir_all(&defined).into_diagnostic()?;

    Ok(defined)
}

const DEFAULT_CLUSTER: &str = "us1.demeter.run";

fn define_cluster(explicit: &Option<String>) -> String {
    explicit.to_owned().unwrap_or(DEFAULT_CLUSTER.to_owned())
}

impl Context {
    pub fn ensure_ext_dir(
        &self,
        extension_key: &str,
        version_key: &str,
    ) -> miette::Result<PathBuf> {
        let defined = self.root_dir.join(extension_key).join(version_key);
        std::fs::create_dir_all(&defined).into_diagnostic()?;

        Ok(defined)
    }
}

pub struct Context {
    pub root_dir: PathBuf,
    pub cluster: String,
    pub project: Option<String>,
}

pub fn from_cli(cli: &Cli) -> miette::Result<Context> {
    let root_dir = ensure_root_dir(&cli.root_dir)?;
    let cluster = define_cluster(&cli.cluster);

    Ok(Context {
        root_dir,
        cluster,
        project: cli.project.to_owned(),
    })
}
