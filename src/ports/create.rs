use clap::Parser;
use miette::IntoDiagnostic;

use crate::{context::extract_context_data, rpc};

#[derive(Parser)]
pub struct Args {}

pub async fn run(_args: Args, cli: &crate::Cli) -> miette::Result<()> {
    let (api_key, id, _) = extract_context_data(cli);

    let metadata = rpc::metadata::find().await?;

    let resouce_kinds = metadata
        .iter()
        .map(|m| m.crd.spec.names.kind.clone())
        .collect::<Vec<String>>();

    let kind_selected =
        inquire::Select::new("What resource do want to create?", resouce_kinds.clone())
            .with_page_size(resouce_kinds.len())
            .prompt()
            .into_diagnostic()?;

    let resource_metadata = metadata
        .iter()
        .find(|m| m.crd.spec.names.kind == kind_selected)
        .unwrap();

    let resource_options = resource_metadata
        .options
        .iter()
        .map(|o| o.description.clone())
        .collect::<Vec<String>>();

    let option_selected = inquire::Select::new("Select an option", resource_options.clone())
        .with_page_size(resource_options.len())
        .prompt()
        .into_diagnostic()?;

    let resource_option_selected = resource_metadata
        .options
        .iter()
        .find(|r| r.description == option_selected)
        .unwrap();

    let confirm = inquire::Confirm::new("Do you want to proceed?")
        .prompt()
        .into_diagnostic()?;

    if !confirm {
        println!("Aborted");
        return Ok(());
    }

    let spec = resource_option_selected.spec.to_string();
    let result = rpc::resources::create(&api_key, &id, &kind_selected, &spec).await?;

    println!("Port {}({}) created", result.kind, result.id);

    Ok(())
}
