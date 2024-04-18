use clap::Parser;

use crate::api::{get, PortInfo};
use miette::IntoDiagnostic;

use super::format::pretty_print_ports_table;

#[derive(Parser)]
pub struct Args {}

pub async fn run(cli: &crate::Cli) -> miette::Result<()> {
    let _ctx = cli
        .context
        .as_ref()
        .ok_or(miette::miette!("can't list ports without a context"))?;

    let response: Vec<PortInfo> = get(cli, "ports").await.into_diagnostic()?; // Use the imported `get` function

    if response.is_empty() {
        println!("No ports found");
        return Ok(());
    }

    pretty_print_ports_table(response);

    Ok(())
}
