use colored::*;
use semver::Version;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn format_new_cli_version_available(version: &str) {
    let message = generate_update_message(version);
    println!("{}", message);
}

fn generate_update_message(latest: &str) -> String {
    let current_version = Version::parse(VERSION).expect("Invalid current version format");
    let latest_version = Version::parse(latest).expect("Invalid latest version format");

    let update_type = if latest_version.major > current_version.major {
        "major"
    } else if latest_version.minor > current_version.minor {
        "minor"
    } else if latest_version.patch > current_version.patch {
        "patch"
    } else {
        "prerelease"
    };

    let type_colored = match update_type {
        "major" => update_type.red(),
        "minor" => update_type.yellow(),
        _ => update_type.green(),
    };

    let old_version_colored = VERSION.red();
    let latest_version_colored = latest.green();
    let update_command = "https://docs.demeter.run/guides/cli";
    let update_command_colored = update_command.cyan();

    format!(
        "\nNew {} version of dmtr available! {} -> {}\nVisit {} to update!\n",
        type_colored, old_version_colored, latest_version_colored, update_command_colored
    )
}
