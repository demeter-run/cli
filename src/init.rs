use clap::Parser;
use miette::IntoDiagnostic;

const DEFAULT_CLOUD: &str = "cloud0.txpipe.io";

#[derive(Parser)]
pub struct Args {
    // /// specify advance values during init
    // #[arg(action)]
    // advanced: bool,
}

pub async fn run(_args: Args, dirs: &crate::dirs::Dirs) -> miette::Result<()> {
    let project_id = inquire::Text::new("Project ID")
        .with_placeholder("eg: romantic-calmness-b55bqg")
        .with_help_message("you can find this value on the web console")
        .prompt()
        .into_diagnostic()?;

    let api_key = inquire::Password::new("API Key")
        .with_display_mode(inquire::PasswordDisplayMode::Masked)
        .without_confirmation()
        .with_help_message(
            "you can find this value on the web console (eg: dmtr_apikey_xxxxxxxxxxxxx)",
        )
        .prompt()
        .into_diagnostic()?
        .into();

    let is_default = inquire::Confirm::new("use as default context?")
        .with_help_message(
            "select this option to use this context when no explicit value is specified",
        )
        .prompt()
        .into_diagnostic()?;

    let project = crate::core::Project { name: project_id };

    let auth = crate::core::Auth {
        name: "root".to_owned(),
        method: "ApiKey".to_owned(),
        token: api_key,
    };

    let cloud = crate::core::Cloud {
        name: DEFAULT_CLOUD.to_string(),
    };

    let operator = crate::core::Operator {
        name: "TxPipe".to_owned(),
        entrypoint: "us1.demeter.run".to_owned(),
    };

    let dto = crate::core::Context {
        project,
        auth,
        cloud,
        operator,
    };

    let name = dto.project.name.clone();

    crate::core::overwrite_context(&name, dto, is_default, &dirs)?;

    Ok(())
}
