use crate::api::{Instance, PortInfo};
use colored::*;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{ContentArrangement, Table};
use dmtri::demeter::ops::v1alpha::Resource;

pub fn pretty_print_port(port: PortInfo) {
    let mut lines = vec![port.name.clone()];
    lines.push("".to_string()); // Empty line for spacing
    lines.push(format!("ID: {}/{}", port.kind, port.id));

    // TODO: for rpc calls we need to deserialize the port.data to get the version
    // and tier?
    match &port.instance {
        Instance::PostgresPort(instance) => {
            lines.push(format!("Network: {}", port.network));
            lines.push(format!("Hostname: {}", instance.hostname));
            lines.push(format!("Database: {}", instance.database));
            lines.push(format!("Port: {}", instance.port));
            lines.push(format!("Username: {}", instance.username));
            lines.push(format!("Password: {}", instance.password));
            lines.push(format!("Connection String: {}", instance.connection_string));
            lines.push(format!("Tier: {}", port.tier));
        }
        Instance::HttpPort(instance) => {
            lines.push(format!("Network: {}", port.network));
            lines.push(format!("API Key: {}", instance.api_key));
            lines.push(format!("Endpoint: {}", instance.endpoint));
            lines.push(format!(
                "Authenticated Endpoint: {}",
                instance.authenticated_endpoint
            ));
            lines.push(format!("Tier: {}", port.tier));
        }
        Instance::NodePort(instance) => {
            lines.push(format!("Network: {}", port.network));
            lines.push(format!(
                "Authenticated Endpoint: {}",
                instance.authenticated_endpoint
            ));
            lines.push(format!("Tier: {}", port.tier));
        }
    }

    for line in lines.iter().enumerate() {
        let (index, content) = line;

        // let mut padded_line = format!("{: <1$}", content, box_width - margin);
        if index == 0 {
            // print empty line
            println!("\n");
            println!("{}", content.color(Color::Green).bold());
        } else {
            println!("{}", content);
        }
    }

    println!(); // Optional: empty line for spacing between entries
}

pub fn pretty_print_ports_table(ports: Vec<Resource>) {
    let mut table = Table::new();

    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        // .set_header(vec!["Instance", "Version", "Tier"]);
        .set_header(vec!["Instance", "Raw Spec"]);

    for port in ports {
        let instance = format_instance(&port.id, &port.kind);
        // // TODO: must deserialize the port.data to get the version and tier
        // table.add_row(vec![instance, port.version, port.tier]);
        table.add_row(vec![instance, port.spec]);
    }

    println!("{table}");
}

fn format_instance(id: &str, kind: &str) -> String {
    format!("{}/{}", kind, id)
}
