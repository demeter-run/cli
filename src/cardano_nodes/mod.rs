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

pub async fn run(args: &Args, ctx: &crate::Context) -> miette::Result<()> {
    match &args.command {
        Commands::Proxy(args) => proxy::run(args, ctx).await,
        _ => Ok(()),
    }
}
