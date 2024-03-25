use clap::Parser;

mod deploy;

#[derive(Parser)]
pub struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser)]
pub enum Commands {
    Deploy(deploy::Args),
}

pub async fn run(args: Args, cli: &crate::Cli) -> miette::Result<()> {
    match args.command {
        Commands::Deploy(x) => deploy::run(x, cli).await,
    }
}
