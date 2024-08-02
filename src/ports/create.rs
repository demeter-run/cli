use clap::Parser;
use dmtri::demeter::ops::v1alpha::Resource;
use miette::IntoDiagnostic;

use crate::{
    api::{self, PortOptions},
    context::extract_context_data,
    ports::format::{pretty_print_port, pretty_print_ports_table},
    rpc,
};

#[derive(Parser)]
pub struct Args {
    // /// specify advance values during init
    // #[arg(action)]
    // advanced: bool,
}

pub async fn run(_args: Args, cli: &crate::Cli) -> miette::Result<()> {
    let (access_token, _, id, _) = extract_context_data(cli);
    let kind_options: Vec<Resource> = rpc::resources::find(&access_token, &id).await?;

    // let kind_options: Vec<String> = api::get(cli, "ports?listOnly=true")
    //     .await
    //     .into_diagnostic()?;

    let kind = inquire::Select::new("Choose the port kind", Vec::from(["CardanoNodePort"]))
        .with_page_size(1)
        .prompt()
        .into_diagnostic()?;

    // let options: PortOptions = api::get_public(&format!("metadata/ports/{}",
    // kind))     .await
    //     .into_diagnostic()?;
    //
    // let network_options = options.get_networks();
    //
    // let selected_network = inquire::Select::new("Choose the network",
    // network_options)     .prompt()
    //     .into_diagnostic()?;
    //
    // let payload_network = options
    //     .find_network_key_by_value(&selected_network)
    //     .unwrap();

    // versions could be empty. If so, skip the version selection
    // let mut selected_version = String::new();
    // let mut payload_version = String::new();
    // let network_versions = options.get_network_versions(&payload_network);
    // if !network_versions.is_empty() {
    //     selected_version = inquire::Select::new("Choose the version",
    // network_versions)         .prompt()
    //         .into_diagnostic()?;
    //     payload_version = options
    //         .find_version_label_by_number(&payload_network, &selected_version)
    //         .unwrap();
    // }

    // let tier_options = options.get_tiers();

    // let selected_tier = inquire::Select::new("Choose the throughput tier",
    // tier_options)     .prompt()
    //     .into_diagnostic()?;
    //
    // let payload_tier: String =
    // options.find_tier_key_by_value(&selected_tier).unwrap();
    //
    println!("You are about to create a new port with the following configuration:");
    println!("Kind: {}", kind);
    // println!("Network: {}", selected_network);
    // if !selected_version.is_empty() {
    //     println!("Version: {}", selected_version);
    // }
    // println!("Tier: {}", selected_tier);

    let confirm = inquire::Confirm::new("Do you want to proceed?")
        .prompt()
        .into_diagnostic()?;

    if !confirm {
        println!("Aborted");
        return Ok(());
    }

    let result = rpc::resources::create(&access_token, &id, kind).await?;

    // pretty_print_port(result);
    pretty_print_ports_table(Vec::from([result]));

    Ok(())
}
