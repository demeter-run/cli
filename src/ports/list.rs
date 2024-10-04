use clap::Parser;

use crate::{
    context::extract_context_data,
    rpc::{self},
};

use super::format::pretty_print_resource_table;

#[derive(Parser)]
pub struct Args {}

pub async fn run(cli: &crate::Cli) -> miette::Result<()> {
    let _ctx = cli
        .context
        .as_ref()
        .ok_or(miette::miette!("can't list ports without a context"))?;

    let (api_key, id, _) = extract_context_data(cli);
    let response = rpc::resources::find(&api_key, &id).await?;

    if response.is_empty() {
        println!("No ports found");
        return Ok(());
    }

    pretty_print_resource_table(response);

    Ok(())
}
