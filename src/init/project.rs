use std::fmt::Display;

use dmtri::demeter::ops::v1alpha as proto;
use miette::{Context as _, IntoDiagnostic};

use crate::rpc;

pub fn parse_project_ref(namespace: String, name: String) -> ProjectRef {
    ProjectRef { namespace, name }
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

async fn new_project(access_token: &str) -> miette::Result<ProjectRef> {
    let project_name = inquire::Text::new("Project name?")
        .with_help_message("Human readable name to identify the project")
        .prompt()
        .into_diagnostic()?;

    let project = rpc::projects::create_project(access_token, &project_name).await?;

    Ok(project)
}

pub async fn define_project(access_token: &str) -> miette::Result<ProjectRef> {
    let projects: Vec<proto::Project> = rpc::projects::find_projects(access_token).await?;

    if projects.is_empty() {
        return new_project(access_token).await;
    }

    let options = projects
        .iter()
        .map(|x| ProjectOption::Existing(parse_project_ref(x.namespace.clone(), x.name.clone())))
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
