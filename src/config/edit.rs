use clap::Parser;
use miette::{miette, IntoDiagnostic};

#[derive(Parser)]
pub struct Args {
    /// api key to use for this config
    #[arg(skip)]
    api_key: Option<String>,
}

fn inquire_changes(mut dto: crate::core::Config) -> miette::Result<crate::core::Config> {
    let token = inquire::Password::new("API Key")
        .with_display_mode(inquire::PasswordDisplayMode::Masked)
        .prompt()
        .into_diagnostic()?
        .into();

    dto.auth = Some(crate::core::Auth {
        name: "root".to_owned(),
        method: "ApiKey".to_owned(),
        token,
    });

    Ok(dto)
}

pub async fn run(args: Args, ctx: &crate::Context) -> miette::Result<()> {
    let dto = inquire_changes(ctx.config.clone())?;

    crate::core::overwrite_current_config(
        &ctx.config.project.name,
        &ctx.config.cloud.name,
        dto,
        &ctx.dirs,
    )?;

    Ok(())
}
