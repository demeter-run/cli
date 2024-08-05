use clap::Parser;
use miette::IntoDiagnostic;

use crate::{context::extract_context_data, rpc};

#[derive(Parser)]
pub struct Args {
    /// the instance in kind/id format. e.g. kupo/mainnet-222222
    instance: String,
}

fn get_instance_parts(instance: &str) -> (String, String) {
    let parts: Vec<&str> = instance.split('/').collect();
    (parts[0].to_string(), parts[1].to_string())
}

pub async fn run(args: Args, cli: &crate::Cli) -> miette::Result<()> {
    let _ctx = cli
        .context
        .as_ref()
        .ok_or(miette::miette!("can't list ports without a context"))?;

    let msg = format!(
        "You are about to delete {}. This action cannot be undone. Do you want to proceed?",
        args.instance
    );

    let confirm = inquire::Confirm::new(&msg).prompt().into_diagnostic()?;

    if !confirm {
        println!("Aborted");
        return Ok(());
    }

    let (api_key, project_id, _) = extract_context_data(cli);

    // parse args
    let (_, id) = get_instance_parts(&args.instance);

    rpc::resources::delete(&api_key, &project_id, &id)
        .await
        .unwrap();

    println!("Successfully deleted port: {}", args.instance);
    Ok(())
}
