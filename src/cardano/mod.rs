mod connect_node_socket;

use clap::Parser;

#[derive(Parser)]
pub struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser)]
pub enum Commands {
    ConnectNodeSocket(connect_node_socket::Args),
}

pub async fn run(args: Args, cli: &crate::Cli) -> miette::Result<()> {
    match args.command {
        Commands::ConnectNodeSocket(args) => connect_node_socket::run(args, cli).await,
    }
}
