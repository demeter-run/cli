mod proxy;

use clap::Parser;
use miette::{bail, miette};

#[derive(Parser)]
pub struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser)]
pub enum Commands {
    Proxy(proxy::Args),
}

pub async fn run(args: Args) -> miette::Result<()> {
    match args.command {
        Commands::Proxy(args) => proxy::run(args).await,
        _ => Ok(()),
    }
}
