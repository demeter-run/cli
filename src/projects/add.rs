use clap::Parser;
use miette::{miette, IntoDiagnostic};

#[derive(Parser)]
pub struct Args {
    /// the id of the project
    id: String,

    /// api key to use for the project
    #[arg(skip)]
    api_key: Option<String>,

    /// override the default cloud
    #[arg(long)]
    cloud: Option<String>,
}

fn inquire_remaining(mut args: Args) -> miette::Result<Args> {
    args.api_key = inquire::Password::new("API KEY")
        .with_display_mode(inquire::PasswordDisplayMode::Masked)
        .prompt()
        .into_diagnostic()?
        .into();

    Ok(args)
}

const DEFAULT_CLOUD: &str = "cloud0.txpipe.io";

impl TryInto<super::core::Project> for Args {
    type Error = miette::Report;

    fn try_into(self) -> Result<super::core::Project, Self::Error> {
        let out = super::core::Project {
            id: self.id,
            api_key: self.api_key.ok_or(miette!("missing api key"))?,
            cloud: self.cloud.unwrap_or(DEFAULT_CLOUD.to_owned()),
        };

        Ok(out)
    }
}

pub async fn run(args: Args, ctx: &crate::Context) -> miette::Result<()> {
    let dto = inquire_remaining(args)?.try_into()?;

    super::core::add_project(dto, ctx)?;

    Ok(())
}
