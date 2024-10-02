use clap::{Parser, Subcommand};
use miette::Context as _;
use std::path::PathBuf;
use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _};

mod context;
mod dirs;
mod init;
mod pages;
mod ports;
mod rpc;

extern crate core;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Args {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true, env = "DMTR_PROJECT_ID")]
    project_id: Option<String>,

    /// Name of the namespace we're working on
    #[arg(short, long, global = true, env = "DMTR_NAMESPACE")]
    namespace: Option<String>,

    /// The api key to use as authentication
    #[arg(short, long, global = true, env = "DMTR_API_KEY")]
    api_key: Option<String>,

    /// Name of the context we're working on
    #[arg(short, long, global = true, env = "DMTR_CONTEXT")]
    context: Option<String>,

    /// The root location for dmtrctl files
    #[arg(short, long, global = true, env = "DMTR_ROOT_DIR")]
    root_dir: Option<PathBuf>,

    /// Add extra debugging outputs
    #[arg(short, long, global = true, action)]
    verbose: bool,

    /// Clear any previous config (use with caution)
    #[arg(long, action)]
    reset_config: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// initialize your Demeter project
    Init(init::Args),

    /// interact with Demeter Pages
    Pages(pages::Args),

    /// Ports-specific commands
    Ports(ports::Args),
}

#[derive(Debug)]
pub struct Cli {
    pub dirs: dirs::Dirs,
    pub context: Option<context::Context>,
}

#[tokio::main]
async fn main() -> miette::Result<()> {
    let args = Args::parse();
    let dirs = dirs::Dirs::try_new(args.root_dir.as_deref())?;

    if args.reset_config {
        println!("clearing previous config files...");
        crate::context::clear_config(&dirs).context("clearing previous config files")?;
    }

    let context = context::infer_context(
        args.context.as_deref(),
        args.project_id.as_deref(),
        args.namespace.as_deref(),
        args.api_key.as_deref(),
        &dirs,
    )?;

    let cli = Cli { context, dirs };

    if args.verbose {
        tracing_subscriber::registry()
            //.with(tracing_subscriber::filter::LevelFilter::INFO)
            .with(tracing_subscriber::filter::Targets::default().with_target("dmtr", Level::DEBUG))
            .init();
    }

    match args.command {
        Commands::Init(args) => init::run(args, &cli.dirs).await,
        Commands::Pages(args) => pages::run(args, &cli).await,
        Commands::Ports(args) => ports::run(args, &cli).await,
    }
}
