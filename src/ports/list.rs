use clap::Parser;

use crate::{
    context::extract_context_data,
    rpc,
};

use super::format::OutputFormat;

#[derive(Parser)]
pub struct Args {
    #[clap(short, long, default_value_t, value_enum)]
    pub output: OutputFormat,
}

pub async fn run(args: Args, cli: &crate::Cli) -> miette::Result<()> {
    let _ctx = cli
        .context
        .as_ref()
        .ok_or(miette::miette!("can't list ports without a context"))?;

    let (api_key, project_id, _) = extract_context_data(cli).await?;
    let response = rpc::resources::find(&api_key, &project_id).await?;

    if response.is_empty() {
        println!("No ports found");
        return Ok(());
    }

    args.output.pretty_print(response);
    Ok(())
}
