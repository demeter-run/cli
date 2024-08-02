use std::fmt::Display;

use miette::IntoDiagnostic as _;

use crate::rpc;

enum MaxKeysOptions {
    TryAgain,
    EnterManually,
}

impl Display for MaxKeysOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MaxKeysOptions::TryAgain => write!(f, "I've deleted one, try again"),
            MaxKeysOptions::EnterManually => write!(f, "Enter my API KEY manually"),
        }
    }
}

pub async fn define_api_key(access_token: &str, project_id: &str) -> miette::Result<String> {
    println!("Setting up API key for project {}", project_id);
    let api_key_result = rpc::projects::create_secret(access_token, project_id, "dmtrctl").await;
    let mut api_key = api_key_result.unwrap_or_default();

    if !api_key.is_empty() {
        return Ok(api_key);
    }

    println!("We need to configure an API KEY for your project but you've already generated the max amount (2).");
    println!("You can manage your existing key from the web console.");
    println!();

    while api_key.is_empty() {
        let next = inquire::Select::new(
            "how do you want to continue?",
            vec![MaxKeysOptions::TryAgain, MaxKeysOptions::EnterManually],
        )
        .prompt()
        .into_diagnostic()?;

        match next {
            MaxKeysOptions::TryAgain => {
                api_key = rpc::projects::create_secret(access_token, project_id, "dmtrctl").await?;
            }
            MaxKeysOptions::EnterManually => {
                api_key = inquire::Password::new("API Key")
                    .with_display_mode(inquire::PasswordDisplayMode::Masked)
                    .without_confirmation()
                    .with_help_message("eg: dmtr_apikey_xxxxxxxxxxxxx")
                    .prompt()
                    .into_diagnostic()?;
            }
        }
    }

    Ok(api_key)
}
