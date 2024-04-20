use clap::{command, Parser};

pub mod create;
mod delete;
mod details;
mod format;
mod list;
mod tunnel;

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
    Details(details::Args),
    /// Create a new port
    Create(create::Args),
    /// Delete a port
    #[command(alias = "rm")]
    Delete(delete::Args),
    /// Create a tunnel to a port, for example, to access the node in a unix.socket file
    Tunnel(tunnel::Args),
    // Disable(list::Args),
}

pub async fn run(args: Args, cli: &crate::Cli) -> miette::Result<()> {
    match args.command {
        Commands::List(_x) => list::run(cli).await,
        Commands::Details(x) => details::run(x, cli).await,
        Commands::Create(x) => create::run(x, cli).await,
        Commands::Delete(x) => delete::run(x, cli).await,
        Commands::Tunnel(_x) => tunnel::run(cli).await,
    }
}
