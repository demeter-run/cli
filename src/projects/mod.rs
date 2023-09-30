mod add;
mod core;

use clap::Parser;

#[derive(Parser)]
pub struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser)]
pub enum Commands {
    Add(add::Args),
}

pub async fn run(args: Args, ctx: &crate::Context) -> miette::Result<()> {
    match args.command {
        Commands::Add(args) => add::run(args, ctx).await,
        _ => Ok(()),
    }
}
