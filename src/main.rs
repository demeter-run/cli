use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::Level;
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::prelude::*;

mod cardano_nodes;
mod config;
mod core;
mod dirs;

const DEFAULT_CLOUD: &str = "cloud0.txpipe.io";

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Name of the project we're working with
    #[arg(short, long, global = true, env = "DMTR_PROJECT")]
    project: Option<String>,

    /// Name of the cloud we're connecting to
    #[arg(short, long, global = true, env = "DMTR_CLOUD", default_value = DEFAULT_CLOUD)]
    cloud: String,

    /// API Key to use for authentication with cloud
    #[arg(short, long, global = true, env = "DMTR_API_KEY")]
    api_key: Option<String>,

    /// The root location for dmtrctl files
    #[arg(short, long, global = true, env = "DMTR_ROOT_DIR")]
    root_dir: Option<PathBuf>,

    /// Add extra debugging outputs
    #[arg(short, long, global = true, action)]
    verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    Config(config::Args),
    CardanoNodes(cardano_nodes::Args),
}

pub struct Context {
    pub config: core::Config,
    pub dirs: dirs::Dirs,
}

impl Context {
    fn for_cli(cli: &Cli) -> miette::Result<Self> {
        let dirs = dirs::Dirs::try_new(cli.root_dir.as_deref())?;

        let project = cli
            .project
            .as_deref()
            .ok_or(miette::miette!("missing project id"))?;

        let config =
            core::load_or_infer_config(&project, &cli.cloud, cli.api_key.as_deref(), &dirs)?;

        Ok(Context { config, dirs })
    }
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

    let ctx = Context::for_cli(&cli)?;

    match cli.command {
        Commands::Config(args) => config::run(args, &ctx).await,
        Commands::CardanoNodes(args) => cardano_nodes::run(args, &ctx).await,
    }
}
