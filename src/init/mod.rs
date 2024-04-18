use clap::Parser;
use miette::IntoDiagnostic;

#[derive(Parser)]
pub struct Args {
    /// Name of the namespace we're working on
    #[arg(short, long, global = true, env = "DMTR_NAMESPACE")]
    namespace: Option<String>,

    /// The api key to use as authentication
    #[arg(short, long, global = true, env = "DMTR_API_KEY")]
    api_key: Option<String>,
}

mod create_project;
mod list_projects;
mod login;
mod manual;

pub async fn run(args: Args, dirs: &crate::dirs::Dirs) -> miette::Result<()> {
    if args.namespace.is_some() && args.api_key.is_some() {
        let namespace = args.namespace.unwrap();
        let api_key = args.api_key.unwrap();
        manual::run(&namespace, &api_key, dirs).await?;
        return Ok(());
    }

    let options = vec!["list projects", "create project"];
    println!("");
    println!("                  Welcome to");
    println!(
        r"     ____                      _            
    |  _ \  ___ _ __ ___   ___| |_ ___ _ __ 
    | | | |/ _ \ '_ ` _ \ / _ \ __/ _ \ '__|
    | |_| |  __/ | | | | |  __/ ||  __/ |   
    |____/ \___|_| |_| |_|\___|\__\___|_|"
    );
    println!("");
    println!("This process will help you set up your project to use the Demeter platform.");
    println!("If you are an existing Demeter user, you can:");
    println!("  • Choose from a list of your existing projects.");
    println!("  • Create a brand new project.\n");
    println!("If you are new to Demeter:");
    println!("  • Select 'create project' and we will guide you through the setup process.\n");
    println!("Let's get started!\n");

    let access_token = login::run().await?;

    let setup = inquire::Select::new("How would you like to start?", options)
        .with_help_message("")
        .prompt()
        .into_diagnostic()?;

    match setup {
        "list projects" => {
            list_projects::run(&access_token, dirs).await?;
        }
        "create project" => {
            create_project::run(&access_token, dirs).await?;
        }
        _ => {
            println!("Invalid option selected. Exiting...");
            std::process::exit(1);
        }
    }

    Ok(())
}

pub fn parse_project_id(project: &str) -> (String, String) {
    let parts: Vec<&str> = project.split('/').collect();
    if parts.len() == 1 {
        return (parts[0].to_string(), "default".to_string());
    }
    (parts[0].to_string(), parts[1].to_string())
}
