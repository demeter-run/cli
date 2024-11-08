use base64::prelude::*;
use miette::IntoDiagnostic;
use ocipkg::{image::Builder, ImageName};
use std::path::{Path, PathBuf};

use clap::Parser;

#[derive(Parser)]
pub struct Args {
    #[arg(long, short)]
    source: Option<PathBuf>,

    #[arg(long)]
    commit_hash: Option<String>,

    #[arg(long)]
    channel: Option<String>,

    #[arg(long, env = "DMTR_REGISTRY_AUTH")]
    registry_auth: String,
}

fn define_image_name(
    namespace: &str,
    channel: Option<&str>,
    commit: Option<&str>,
) -> miette::Result<ocipkg::ImageName> {
    let channel = channel.unwrap_or("main");
    let commit = commit.unwrap_or("latest");
    let raw = format!(
        "ghcr.io/demeter-run/pages-{}-{}:{}",
        namespace, channel, commit
    );
    ImageName::parse(&raw).into_diagnostic()
}

pub async fn run(args: Args, cli: &crate::Cli) -> miette::Result<()> {
    let img = std::fs::File::create("img.tar").into_diagnostic()?;
    let mut builder = Builder::new(img);

    let ctx = cli
        .context
        .as_ref()
        .ok_or(miette::miette!("can't deploy without a context"))?;

    let source = args.source.unwrap_or_else(|| Path::new("./dist").into());
    let name = define_image_name(
        &ctx.namespace.name,
        args.channel.as_deref(),
        args.commit_hash.as_deref(),
    )?;

    builder.append_dir_all(&source).into_diagnostic()?;
    builder.set_name(&name);

    let _ = builder.into_inner();

    let registry_url = name.registry_url().into_diagnostic()?;
    let registry_auth = BASE64_STANDARD.encode(&args.registry_auth);

    let mut new_auth = ocipkg::distribution::StoredAuth::default();
    new_auth.insert(registry_url.domain().unwrap(), registry_auth);
    new_auth.save().into_diagnostic()?;

    ocipkg::distribution::push_image(Path::new("img.tar")).into_diagnostic()?;

    Ok(())
}
