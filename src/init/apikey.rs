use std::fmt::Display;

use miette::IntoDiagnostic as _;

use crate::api;

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
    let mut api_key: String = api::account::create_api_key(access_token, &project_id, "dmtrctl")
        .await
        .into_diagnostic()?;

    if !api_key.is_empty() {
        return Ok(api_key);
    }

    println!("We need to configure an API KEY for your project but you've already generated the max amount (2).");
    println!("You can manage your existing key from the web console:");
    println!("https://console.us1.demeter.run/{}/settings", project_id);
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
                api_key = api::account::create_api_key(access_token, &project_id, "dmtrctl")
                    .await
                    .into_diagnostic()?;
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
