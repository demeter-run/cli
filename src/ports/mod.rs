use clap::{command, Parser};

pub mod create;
mod delete;
mod format;
mod list;
mod node_socket;
mod show;

#[derive(Parser)]
pub struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser)]
pub enum Commands {
    /// List all your ports
    #[command(alias = "ls")]
    List(list::Args),
    /// Get your port details
    Show(show::Args),
    /// Create a new port
    Create(create::Args),
    /// Delete a port
    #[command(alias = "rm")]
    Delete(delete::Args),
    /// Create a Unix socket that connect to a Cardano node port
    #[command(alias = "tunnel")]
    NodeSocket(node_socket::Args),
    // Disable(list::Args),
}

pub async fn run(args: Args, cli: &crate::Cli) -> miette::Result<()> {
    match args.command {
        Commands::List(_x) => list::run(cli).await,
        Commands::Show(x) => show::run(x, cli).await,
        Commands::Create(x) => create::run(x, cli).await,
        Commands::Delete(x) => delete::run(x, cli).await,
        Commands::NodeSocket(x) => node_socket::run(x, cli).await,
    }
}
