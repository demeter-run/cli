use clap::Parser;
use miette::IntoDiagnostic;

use crate::{
    api::{self, PortOptions},
    ports::format::pretty_print_port,
};

#[derive(Parser)]
pub struct Args {
    // /// specify advance values during init
    // #[arg(action)]
    // advanced: bool,
}

pub async fn run(_args: Args, cli: &crate::Cli) -> miette::Result<()> {
    let kind_options: Vec<String> = api::get(cli, "ports?listOnly=true")
        .await
        .into_diagnostic()?;
    loop {
        let kind = inquire::Select::new("Choose the port kind", kind_options.clone())
            .prompt()
            .into_diagnostic()?;

        let options: PortOptions = api::get_public(&format!("metadata/ports/{}", kind))
            .await
            .into_diagnostic()?;

        let network_options = options.networks.clone();

        let network = inquire::Select::new("Choose the network", network_options)
            .prompt()
            .into_diagnostic()?;

        let mut version = String::new();
        let network_versions = options.get_network_versions(&network);
        if !network_versions.is_empty() {
            version = inquire::Select::new("Choose the version", network_versions)
                .prompt()
                .into_diagnostic()?;
        }

        let tier = inquire::Select::new("Choose the throughput tier", options.tiers)
            .prompt()
            .into_diagnostic()?;
        println!("You are about to create a new port with the following configuration:");
        println!("Kind: {}", kind);
        println!("Network: {}", network);
        if !version.is_empty() {
            println!("Version: {}", version);
        }
        println!("Tier: {}", tier);

        let confirm = inquire::Confirm::new("Do you want to proceed?")
            .prompt()
            .into_diagnostic()?;

        if !confirm {
            println!("Aborted");
            return Ok(());
        }

        let result = api::create_port(cli, &kind, &network, &version, &tier)
            .await
            .into_diagnostic()?;

        pretty_print_port(result);

        let create_another = inquire::Confirm::new("Do you want to create another port?")
            .prompt()
            .into_diagnostic()?;

        if !create_another {
            break;
        }
    }

    Ok(())
}
