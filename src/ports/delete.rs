use clap::Parser;
use miette::IntoDiagnostic;

use crate::{context::extract_context_data, rpc};

#[derive(Parser)]
pub struct Args {
    /// the resource uuid
    id: String,
}

pub async fn run(args: Args, cli: &crate::Cli) -> miette::Result<()> {
    let _ctx = cli
        .context
        .as_ref()
        .ok_or(miette::miette!("can't list ports without a context"))?;

    let msg = format!(
        "You are about to delete {}. This action cannot be undone. Do you want to proceed?",
        args.id
    );

    let confirm = inquire::Confirm::new(&msg).prompt().into_diagnostic()?;

    if !confirm {
        println!("Aborted");
        return Ok(());
    }

    let (api_key, project_id, _) = extract_context_data(cli).await?;

    rpc::resources::delete(&api_key, &project_id, &args.id)
        .await
        .unwrap();

    println!("Successfully deleted port: {}", args.id);
    Ok(())
}
