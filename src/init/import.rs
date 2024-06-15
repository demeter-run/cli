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

enum MaxKeysOptions {
    TryAgain,
    EnterManually,
}

impl Display for MaxKeysOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MaxKeysOptions::TryAgain => write!(f, "I've deleted one, try again"),
            MaxKeysOptions::EnterManually => write!(f, "Enter my API KEY manually"),
        }
    }
}

pub async fn define_api_key(access_token: &str, project_id: &str) -> miette::Result<String> {
    let mut api_key: String = api::account::create_api_key(access_token, &project_id, "dmtrctl")
        .await
        .into_diagnostic()?;

    if !api_key.is_empty() {
        return Ok(api_key);
    }

    println!("We need to configure an API KEY for your project but you've already generated the max amount (2).");
    println!("You can manage your existing key from the web console:");
    println!("https://console.us1.demeter.run/{}/settings", project_id);
    println!();

    while api_key.is_empty() {
        let next = inquire::Select::new(
            "how do you want to continue?",
            vec![MaxKeysOptions::TryAgain, MaxKeysOptions::EnterManually],
        )
        .prompt()
        .into_diagnostic()?;

        match next {
            MaxKeysOptions::TryAgain => {
                api_key = api::account::create_api_key(access_token, &project_id, "dmtrctl")
                    .await
                    .into_diagnostic()?;
            }
            MaxKeysOptions::EnterManually => {
                api_key = inquire::Password::new("API Key")
                    .with_display_mode(inquire::PasswordDisplayMode::Masked)
                    .without_confirmation()
                    .with_help_message("eg: dmtr_apikey_xxxxxxxxxxxxx")
                    .prompt()
                    .into_diagnostic()?;
            }
        }
    }

    Ok(api_key)
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
