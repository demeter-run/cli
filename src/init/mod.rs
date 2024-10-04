use crate::context::{load_config, Context};
use clap::Parser;
use miette::{Context as _, IntoDiagnostic as _};
use std::fmt::Display;

#[derive(Parser, Debug)]
pub struct Args {}

mod apikey;
mod login;
mod manual;
pub mod project;

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

    let api_key = apikey::define_api_key(&access_token, &project.id).await?;

    let ctx = crate::context::Context {
        project: crate::context::Project::new(&project.id, &project.namespace, Some(project.name)),
        auth: crate::context::Auth::api_key(&api_key),
    };

    crate::context::overwrite_context(&project.namespace, ctx.clone(), false, dirs)?;

    Ok(ctx)
}

async fn define_context(dirs: &crate::dirs::Dirs) -> miette::Result<Context> {
    let config = crate::context::load_config(dirs).context("loading config")?;

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

pub async fn run(_args: Args, cli: &crate::Cli) -> miette::Result<()> {
    println!("Welcome to");
    println!(include_str!("asciiart.txt"));
    println!("\n");
    println!("This process will help you set up your CLI to use Demeter platform.");
    println!("Let's get started!");
    println!("\n");

    let config = load_config(&cli.dirs)?;

    if let Some(context) = cli.context.as_ref() {
        if !config.contexts.contains_key(&context.project.namespace) {
            manual::run(context, &cli.dirs).await?;
            return Ok(());
        }
    }

    let ctx = define_context(&cli.dirs).await?;

    crate::context::set_default_context(&ctx.project.namespace, &cli.dirs)?;

    println!(
        "You CLI is now configured to use context {}",
        ctx.project.namespace
    );

    println!("Check out the ports sub-command to start operating");

    Ok(())
}
