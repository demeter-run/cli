use miette::IntoDiagnostic;

pub async fn run(
    id: &str,
    namespace: &str,
    api_key: &str,
    dirs: &crate::dirs::Dirs,
) -> miette::Result<()> {
    println!("Setting up context for:\n");
    println!("  Namespace: {}", namespace);
    println!("  API key: {}\n", api_key);

    let is_default = inquire::Confirm::new("use as default context?")
        .with_help_message(
            "select this option to use this context when no explicit value is specified",
        )
        .prompt()
        .into_diagnostic()?;

    let dto = crate::context::Context::ephemeral(id, namespace, api_key);

    let namespace = dto.project.namespace.clone();

    crate::context::overwrite_context(&namespace, dto, is_default, dirs)?;

    Ok(())
}
