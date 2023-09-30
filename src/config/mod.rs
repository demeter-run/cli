mod edit;

use clap::Parser;

#[derive(Parser)]
pub struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser)]
pub enum Commands {
    Edit(edit::Args),
}

pub async fn run(args: Args, ctx: &crate::Context) -> miette::Result<()> {
    match args.command {
        Commands::Edit(args) => edit::run(args, ctx).await,
        _ => Ok(()),
    }
}
