use std::fmt::Display;

use dmtri::demeter::ops::v1alpha as proto;
use miette::IntoDiagnostic;

use crate::rpc;

pub fn parse_project_ref(id: String, namespace: String, name: String) -> ProjectRef {
    ProjectRef {
        id,
        namespace,
        name,
    }
}

pub struct ProjectRef {
    pub id: String,
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
    let projects: Vec<proto::Project> = rpc::projects::find(access_token).await?;

    if projects.is_empty() {
        return new_project(access_token).await;
    }

    let options = projects
        .iter()
        .map(|project| {
            ProjectOption::Existing(parse_project_ref(
                project.id.clone(),
                project.namespace.clone(),
                project.name.clone(),
            ))
        })
        .chain(std::iter::once(ProjectOption::New))
        .collect::<Vec<_>>();

    let selection = inquire::Select::new("Choose your project", options)
        .prompt()
        .into_diagnostic()?;

    match selection {
        ProjectOption::Existing(project) => Ok(project),
        ProjectOption::New => new_project(access_token).await,
    }
}
