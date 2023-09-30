use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::Level;
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::prelude::*;

mod cardano_nodes;
mod context;
mod login;
pub mod projects;

pub use context::*;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true)]
    project: Option<String>,

    #[arg(short, long, global = true)]
    cluster: Option<String>,

    #[arg(short, long, global = true)]
    root_dir: Option<PathBuf>,

    #[arg(short, long, global = true, action)]
    verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    Login,
    Projects(projects::Args),
    CardanoNodes(cardano_nodes::Args),
}

#[tokio::main]
async fn main() -> miette::Result<()> {
    let cli = Cli::parse();

    let indicatif_layer = IndicatifLayer::new();

    let level = match cli.verbose {
        true => Level::DEBUG,
        false => Level::INFO,
    };

    tracing_subscriber::registry()
        //.with(tracing_subscriber::filter::LevelFilter::INFO)
        .with(tracing_subscriber::filter::Targets::default().with_target("dmtr", level))
        .with(tracing_subscriber::fmt::layer().with_writer(indicatif_layer.get_stderr_writer()))
        .with(indicatif_layer)
        .init();

    let ctx = context::from_cli(&cli)?;

    match cli.command {
        Commands::Login => login::run().await,
        Commands::Projects(args) => projects::run(args, &ctx).await,
        Commands::CardanoNodes(args) => cardano_nodes::run(args, &ctx).await,
    }
}
