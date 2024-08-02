use clap::Parser;

use crate::{
    context::extract_context_data,
    rpc::{self},
};

use super::format::pretty_print_ports_table;

#[derive(Parser)]
pub struct Args {}

pub async fn run(cli: &crate::Cli) -> miette::Result<()> {
    let _ctx = cli
        .context
        .as_ref()
        .ok_or(miette::miette!("can't list ports without a context"))?;

    let (access_token, _, id, _) = extract_context_data(cli);
    let response = rpc::resources::find(&access_token, &id).await?;

    if response.is_empty() {
        println!("No ports found");
        return Ok(());
    }

    pretty_print_ports_table(response);

    Ok(())
}
