use clap::Parser;
use miette::IntoDiagnostic;

use crate::{
    context::extract_context_data,
    rpc,
    utils::{get_spec_from_crd, KnownField},
};

#[derive(Parser)]
pub struct Args {}

pub async fn run(_args: Args, cli: &crate::Cli) -> miette::Result<()> {
    let (api_key, id, _) = extract_context_data(cli);

    let crds = rpc::metadata::find().await?;

    let kinds = crds
        .iter()
        .map(|x| x.spec.names.kind.clone())
        .collect::<Vec<String>>();

    let kind = inquire::Select::new("Choose the port kind", kinds.clone())
        .with_page_size(kinds.len())
        .prompt()
        .into_diagnostic()?;

    let crd_selected = crds.iter().find(|crd| crd.spec.names.kind == kind).unwrap();
    let spec = get_spec_from_crd(crd_selected).unwrap();

    let mut payload = serde_json::Map::default();

    for (field, value) in spec {
        let is_nullable = value.nullable.unwrap_or_default();

        if !is_nullable {
            if let Ok(known_field) = field.parse::<KnownField>() {
                match known_field {
                    KnownField::Network => {
                        let network_options = vec!["mainnet", "preprod", "preview"];

                        let selected_network =
                            inquire::Select::new("Choose the network", network_options)
                                .prompt()
                                .into_diagnostic()?;
                        payload.insert(
                            field.to_string(),
                            serde_json::Value::String(selected_network.into()),
                        );
                    }
                    KnownField::OperatorVersion => {
                        payload.insert(field.to_string(), serde_json::Value::String("1".into()));
                    }
                }
                continue;
            }

            let value = inquire::Text::new(&format!("Fill out the {field}"))
                .prompt()
                .into_diagnostic()?;
            payload.insert(field.to_string(), serde_json::Value::String(value));
        }
    }

    let confirm = inquire::Confirm::new("Do you want to proceed?")
        .prompt()
        .into_diagnostic()?;

    if !confirm {
        println!("Aborted");
        return Ok(());
    }

    let spec = serde_json::Value::Object(payload);
    let result = rpc::resources::create(&api_key, &id, &kind, &spec.to_string()).await?;

    println!("Port {}({}) created", result.kind, result.id);

    Ok(())
}
