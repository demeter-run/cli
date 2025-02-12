use clap::Parser;

use crate::{context::extract_context_data, rpc};

use super::format::OutputFormat;

#[derive(Parser)]
pub struct Args {
    /// the resource uuid
    id: String,

    #[clap(short, long, default_value_t, value_enum)]
    pub output: OutputFormat,
}

pub async fn run(args: Args, cli: &crate::Cli) -> miette::Result<()> {
    let _ctx = cli
        .context
        .as_ref()
        .ok_or(miette::miette!("can't list ports without a context"))?;

    let (api_key, project_id, _) = extract_context_data(cli).await?;
    let resouces = rpc::resources::find_by_id(&api_key, &project_id, &args.id).await?;

    if resouces.is_empty() {
        println!("No ports found");
        return Ok(());
    }

    let response = resouces.first().unwrap();
    args.output.pretty_print_single(response);
    Ok(())
}
