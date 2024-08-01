use std::fmt::Display;

use dmtri::demeter::ops::v1alpha as proto;
use miette::{Context as _, IntoDiagnostic};
use serde::{Deserialize, Serialize};

use crate::{api, rpc};

fn parse_project_ref(project: String) -> ProjectRef {
    let parts: Vec<_> = project.split('/').collect();

    if parts.len() == 1 {
        return ProjectRef {
            namespace: parts[0].to_owned(),
            name: String::new(),
        };
    }

    ProjectRef {
        namespace: parts[0].to_owned(),
        name: parts[1].to_owned(),
    }
}

pub struct ProjectRef {
    pub namespace: String,
    pub name: String,
}

enum ProjectOption {
    Existing(ProjectRef),
    New,
}

impl Display for ProjectOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectOption::Existing(x) => {
                write!(f, "{} ({})", x.namespace, x.name)
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
    let projects: Vec<proto::Project> = rpc::projects::find_projects(access_token).await?;

    if projects.is_empty() {
        return new_project(access_token).await;
    }

    let options = projects
        .iter()
        .map(|x| ProjectOption::Existing(parse_project_ref(x.namespace.clone())))
        .chain(std::iter::once(ProjectOption::New))
        .collect::<Vec<_>>();

    let selection = inquire::Select::new("Choose your project", options)
        .prompt()
        .into_diagnostic()?;

    match selection {
        ProjectOption::Existing(x) => Ok(x),
        ProjectOption::New => new_project(access_token).await,
    }
}
