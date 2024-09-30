use clap::Parser;

use crate::{context::extract_context_data, rpc};

use super::format::pretty_print_ports_table;

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

    let (_, resource_id) = get_instance_parts(&args.instance);
    let (api_key, project_id, _) = extract_context_data(cli);
    let response = rpc::resources::find_by_id(&api_key, &project_id, &resource_id).await?;

    if response.is_empty() {
        println!("No ports found");
        return Ok(());
    }

    // TODO: replace this method with the one bellow to show the port details
    // pretty_print_port(resource);
    pretty_print_ports_table(response);
    Ok(())
}
