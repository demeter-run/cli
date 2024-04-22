use clap::Parser;

use crate::api::{get, PortInfo};

use super::format::pretty_print_port;

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

    // parse args
    let (kind, id) = get_instance_parts(&args.instance);

    let response: PortInfo = get(cli, format!("ports/{}/{}", kind, id).as_str())
        .await
        .unwrap(); // Use the imported `get` function

    // if !response {
    //     println!("No ports found for instance: {}", args.instance);
    //     return Ok(());
    // }

    pretty_print_port(response);
    Ok(())
}
