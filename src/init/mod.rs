use crate::core::Context;
use clap::Parser;
use miette::{Context as _, IntoDiagnostic as _};
use std::fmt::Display;

#[derive(Parser)]
pub struct Args {
    /// Name of the namespace we're working on
    #[arg(short, long, global = true, env = "DMTR_NAMESPACE")]
    namespace: Option<String>,

    /// The api key to use as authentication
    #[arg(short, long, global = true, env = "DMTR_API_KEY")]
    api_key: Option<String>,
}

mod apikey;
mod login;
mod manual;
mod project;

enum ContextOption<'a> {
    Existing(&'a Context),
    ImportProject,
}

impl<'a> Display for ContextOption<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContextOption::Existing(x) => match &x.project.name {
                Some(name) => write!(f, "{} ({})", x.project.namespace, name),
                _ => write!(f, "{}", x.project.namespace),
            },
            ContextOption::ImportProject => f.write_str("<import from cloud>"),
        }
    }
}

pub async fn import_context(dirs: &crate::dirs::Dirs) -> miette::Result<Context> {
    let access_token = login::run().await?;

    let project = project::define_project(&access_token).await?;
    println!("Project found: {}", project.namespace);

    let api_key = apikey::define_api_key(&access_token, &project.namespace).await?;

    let ctx = crate::core::Context {
        project: crate::core::Project::new(&project.namespace, Some(project.name)),
        auth: crate::core::Auth::api_key(&api_key),
        cloud: crate::core::Cloud::default(),
        operator: crate::core::Operator::default(),
    };

    crate::core::overwrite_context(&project.namespace, ctx.clone(), false, &dirs)?;

    Ok(ctx)
}

async fn define_context(dirs: &crate::dirs::Dirs) -> miette::Result<Context> {
    let config = crate::core::load_config(dirs).context("loading config")?;

    if config.contexts.is_empty() {
        return import_context(dirs).await;
    }

    let options = config
        .contexts
        .values()
        .map(ContextOption::Existing)
        .chain(std::iter::once(ContextOption::ImportProject))
        .collect();

    let selection = inquire::Select::new("Choose your context", options)
        .prompt()
        .into_diagnostic()?;

    match selection {
        ContextOption::Existing(x) => Ok(x.clone()),
        ContextOption::ImportProject => import_context(dirs).await,
    }
}

pub async fn run(args: Args, dirs: &crate::dirs::Dirs) -> miette::Result<()> {
    if args.namespace.is_some() && args.api_key.is_some() {
        let namespace = args.namespace.unwrap();
        let api_key = args.api_key.unwrap();
        manual::run(&namespace, &api_key, dirs).await?;
        return Ok(());
    };

    println!("Welcome to");
    println!(include_str!("asciiart.txt"));
    println!("");
    println!("This process will help you set up your CLI to use Demeter platform.");
    println!("Let's get started!");
    println!("");

    let ctx = define_context(&dirs).await?;

    crate::core::set_default_context(&ctx.project.namespace, &dirs)?;

    println!(
        "You CLI is now configured to use context {}",
        ctx.project.namespace
    );

    println!("Check out the ports sub-command to start operating");

    Ok(())
}
