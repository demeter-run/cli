use std::fmt::Display;

use miette::{Context as _, IntoDiagnostic};
use serde::{Deserialize, Serialize};

use crate::api;

fn parse_project_ref(project: String) -> ProjectRef {
    let parts: Vec<_> = project.split('/').collect();

    if parts.len() == 1 {
        return ProjectRef {
            namespace: parts[0].to_owned(),
            caption: None,
        };
    }

    ProjectRef {
        namespace: parts[0].to_owned(),
        caption: Some(parts[1].to_owned()),
    }
}

pub struct ProjectRef {
    pub namespace: String,
    pub caption: Option<String>,
}

enum ProjectOption {
    Existing(ProjectRef),
    New,
}

impl Display for ProjectOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectOption::Existing(x) => {
                write!(
                    f,
                    "{} ({})",
                    x.namespace,
                    x.caption.as_deref().unwrap_or_default()
                )
            }
            ProjectOption::New => f.write_str("<new project>"),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Organization {
    id: u64,
    name: String,
}

impl Display for Organization {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}

async fn define_org(access_token: &str) -> miette::Result<Organization> {
    let orgs: Vec<Organization> = api::account::get(access_token, "organizations")
        .await
        .into_diagnostic()
        .context("looking for existing options")?;

    // if we have only one org, automatically use that one
    if orgs.len() == 1 {
        return Ok(orgs.first().cloned().unwrap());
    }

    inquire::Select::new("Choose your organization", orgs)
        .prompt()
        .into_diagnostic()
}

async fn new_project(access_token: &str) -> miette::Result<ProjectRef> {
    let project_name = inquire::Text::new("Project name?")
        .with_help_message("Human readable name to identify the project")
        .prompt()
        .into_diagnostic()?;

    let org = define_org(access_token)
        .await
        .context("defining organization")?;

    let project = api::account::create_project(access_token, &org.id, &project_name)
        .await
        .into_diagnostic()?;

    Ok(parse_project_ref(project))
}

pub async fn define_project(access_token: &str) -> miette::Result<ProjectRef> {
    let options: Vec<_> = api::account::get::<Vec<String>>(&access_token, "projects")
        .await
        .into_diagnostic()?
        .into_iter()
        .map(parse_project_ref)
        .map(ProjectOption::Existing)
        .chain(std::iter::once(ProjectOption::New))
        .collect();

    let selection = inquire::Select::new("Choose your project", options)
        .prompt()
        .into_diagnostic()?;

    match selection {
        ProjectOption::Existing(x) => Ok(x),
        ProjectOption::New => new_project(&access_token).await,
    }
}
