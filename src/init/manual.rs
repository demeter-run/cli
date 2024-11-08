use miette::IntoDiagnostic;

use crate::context::Context;

pub async fn run(context: &Context, dirs: &crate::dirs::Dirs) -> miette::Result<()> {
    println!("Setting up context for:\n");
    println!("  Project: {}", context.namespace.name);
    println!("  API key: {}\n", context.auth.token);

    let is_default = inquire::Confirm::new("use as default context?")
        .with_help_message(
            "select this option to use this context when no explicit value is specified",
        )
        .prompt()
        .into_diagnostic()?;

    crate::context::overwrite_context(&context.namespace.name, context.clone(), is_default, dirs)?;

    Ok(())
}
