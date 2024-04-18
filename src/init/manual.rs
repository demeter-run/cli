use miette::IntoDiagnostic;

pub async fn run(namespace: &str, api_key: &str, dirs: &crate::dirs::Dirs) -> miette::Result<()> {
    println!("Setting up context for:\n");
    println!("  Namespace: {}", namespace);
    println!("  API key: {}\n", api_key);

    let is_default = inquire::Confirm::new("use as default context?")
        .with_help_message(
            "select this option to use this context when no explicit value is specified",
        )
        .prompt()
        .into_diagnostic()?;

    let dto = crate::core::Context::ephemeral(&namespace, &api_key);

    let name = dto.namespace.name.clone();

    crate::core::overwrite_context(&name, dto, is_default, &dirs)?;

    Ok(())
}
