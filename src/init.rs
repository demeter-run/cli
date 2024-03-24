use clap::Parser;
use miette::IntoDiagnostic;

#[derive(Parser)]
pub struct Args {
    // /// specify advance values during init
    // #[arg(action)]
    // advanced: bool,
}

pub async fn run(_args: Args, dirs: &crate::dirs::Dirs) -> miette::Result<()> {
    let namespace = inquire::Text::new("Namespace (aka: project id)")
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
        .into_diagnostic()?;

    let is_default = inquire::Confirm::new("use as default context?")
        .with_help_message(
            "select this option to use this context when no explicit value is specified",
        )
        .prompt()
        .into_diagnostic()?;

    let dto = crate::core::Context::ephemeral(&namespace, &api_key);

    let name = dto.namespace.name.clone();

    crate::core::overwrite_context(&name, dto, is_default, &dirs)?;

    Ok(())
}
