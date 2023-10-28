use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::Level;
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::prelude::*;

mod core;
mod dirs;
mod init;

// namespaces
mod cardano;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Args {
    #[command(subcommand)]
    command: Commands,

    /// Name of the context we're working on
    #[arg(short, long, global = true, env = "DMTR_CONTEXT")]
    context: Option<String>,

    /// The root location for dmtrctl files
    #[arg(short, long, global = true, env = "DMTR_ROOT_DIR")]
    root_dir: Option<PathBuf>,

    /// Add extra debugging outputs
    #[arg(short, long, global = true, action)]
    verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    Init(init::Args),
    Cardano(cardano::Args),
}

pub struct Cli {
    pub dirs: dirs::Dirs,
    pub context: Option<core::Context>,
}

#[tokio::main]
async fn main() -> miette::Result<()> {
    let args = Args::parse();
    let dirs = dirs::Dirs::try_new(args.root_dir.as_deref())?;
    let context = core::load_context(args.context.as_deref(), &dirs)?;

    let cli = Cli { context, dirs };

    let indicatif_layer = IndicatifLayer::new();

    let level = match args.verbose {
        true => Level::DEBUG,
        false => Level::INFO,
    };

    tracing_subscriber::registry()
        //.with(tracing_subscriber::filter::LevelFilter::INFO)
        .with(tracing_subscriber::filter::Targets::default().with_target("dmtr", level))
        .with(tracing_subscriber::fmt::layer().with_writer(indicatif_layer.get_stderr_writer()))
        .with(indicatif_layer)
        .init();

    match args.command {
        Commands::Init(args) => init::run(args, &cli.dirs).await,
        Commands::Cardano(args) => cardano::run(args, &cli).await,
    }
}
