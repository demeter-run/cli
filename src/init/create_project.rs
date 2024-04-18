use miette::IntoDiagnostic;
use serde::{Deserialize, Serialize};

use crate::api;

use super::parse_project_id;

#[derive(Deserialize, Serialize, Debug)]
struct Organization {
    id: u64,
    name: String,
}

pub async fn run(access_token: &str, dirs: &crate::dirs::Dirs) -> miette::Result<()> {
    let organizations: Vec<Organization> = api::account::get(access_token, "organizations")
        .await
        .into_diagnostic()?;

    let mut organization = 0;
    // check if there are any organizations
    if !organizations.is_empty() {
        // select the 1st organization by default
        organization = organizations[0].id.clone();
        // don't prompt if there is only one organization
        if organizations.len() > 1 {
            let organization_options: Vec<String> =
                organizations.iter().map(|o| o.name.clone()).collect();

            let selected_organization =
                inquire::Select::new("Choose your organization", organization_options)
                    .prompt()
                    .into_diagnostic()?;
            organization = organizations
                .iter()
                .find(|o| o.name == selected_organization)
                .unwrap()
                .id
                .clone();
        }
    }

    let project_name = inquire::Text::new("Enter the project name")
        .with_help_message("The name of the project")
        .prompt()
        .into_diagnostic()?;
    let project_description = inquire::Text::new("Enter the project description") // Use the imported `Text` struct
        .with_help_message("The description of the project")
        .prompt()
        .into_diagnostic()?;

    let project = api::account::create_project(
        access_token,
        &organization,
        &project_name,
        &project_description,
    )
    .await
    .into_diagnostic()?;

    let (project_id, _project_name) = parse_project_id(&project);

    let api_key = api::account::create_api_key(access_token, &project_id, "dmtr-cli")
        .await
        .into_diagnostic()?;

    println!("Project created successfully");
    println!("API Key: {} - This will not be displayed again. Make sure to store it somewhere safe. The CLI will automatically use this API key for this project.", api_key);

    let is_default = inquire::Confirm::new("use as default context?")
        .with_help_message(
            "select this option to use this context when no explicit value is specified",
        )
        .prompt()
        .into_diagnostic()?;

    let dto = crate::core::Context::ephemeral(&project_id, &api_key);

    let name = dto.namespace.name.clone();

    crate::core::overwrite_context(&name, dto, is_default, &dirs)?;

    Ok(())
}
