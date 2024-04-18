use miette::IntoDiagnostic;

use crate::api;

use super::parse_project_id;
use crate::init::create_project; // Import the create_project module

pub async fn run(access_token: &str, dirs: &crate::dirs::Dirs) -> miette::Result<()> {
    let project_options: Vec<String> = api::account::get(access_token, "projects")
        .await
        .into_diagnostic()?;

    if project_options.is_empty() {
        println!("No projects found");
        let create = inquire::Confirm::new("Would you like to create a new project?")
            .prompt()
            .into_diagnostic()?;
        if create {
            create_project::run(access_token, dirs).await?; // Use the create_project module
        }
    }

    let project = inquire::Select::new("Choose your project", project_options)
        .prompt()
        .into_diagnostic()?;

    let (project_id, _project_name) = parse_project_id(&project);

    let mut api_key: String = api::account::create_api_key(access_token, &project_id, "dmtr-cli")
        .await
        .into_diagnostic()?;

    if api_key.is_empty() {
        println!("You Reached the limit of API keys for this project. Please delete an existing key to create a new one. Visit the web console to create a new API Key and add it below.");
        println!(
            "Visit: https://console.us1.demeter.run/{}/settings \n",
            project_id
        );

        api_key = inquire::Password::new("API Key")
            .with_display_mode(inquire::PasswordDisplayMode::Masked)
            .without_confirmation()
            .with_help_message(
                "you can find this value on the web console (eg: dmtr_apikey_xxxxxxxxxxxxx)",
            )
            .prompt()
            .into_diagnostic()?;
    } else {
        println!("API Key: {} - This will not be displayed again. Make sure to store it somewhere safe. The CLI will automatically use this API key for this project.", api_key);
    }

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
