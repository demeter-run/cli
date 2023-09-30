use miette::IntoDiagnostic;
use std::path::{Path, PathBuf};

fn default_root_dir() -> miette::Result<PathBuf> {
    let defined = dirs::home_dir()
        .ok_or(miette::Error::msg("no home directory"))?
        .join(".dmtr");

    Ok(defined)
}

pub fn ensure_root_dir(explicit: Option<&Path>) -> miette::Result<PathBuf> {
    let default = default_root_dir()?;

    let defined = explicit.map(|p| p.to_path_buf()).unwrap_or(default);

    std::fs::create_dir_all(&defined).into_diagnostic()?;

    Ok(defined)
}

pub struct Dirs {
    root_dir: PathBuf,
}

impl Dirs {
    pub fn try_new(root_dir: Option<&Path>) -> miette::Result<Self> {
        let root_dir = ensure_root_dir(root_dir)?;

        Ok(Self { root_dir })
    }

    pub fn ensure_extension_dir(
        &self,
        extension_key: &str,
        version_key: &str,
    ) -> miette::Result<PathBuf> {
        let defined = self
            .root_dir
            .join("ext")
            .join(extension_key)
            .join(version_key);

        std::fs::create_dir_all(&defined).into_diagnostic()?;

        Ok(defined)
    }

    pub fn ensure_project_dir(
        &self,
        cloud_key: &str,
        project_key: &str,
    ) -> miette::Result<PathBuf> {
        let defined = self.root_dir.join("prj").join(cloud_key).join(project_key);

        std::fs::create_dir_all(&defined).into_diagnostic()?;

        Ok(defined)
    }
}
