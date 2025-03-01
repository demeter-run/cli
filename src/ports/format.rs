use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{ContentArrangement, Table};
use dmtri::demeter::ops::v1alpha::Resource;
use miette::IntoDiagnostic;
use serde::Serialize;

#[derive(clap::ValueEnum, Clone, Default, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    #[default]
    Table,
    Json,
}

impl OutputFormat {
    pub fn pretty_print(&self, resources: Vec<Resource>) {
        match self {
            OutputFormat::Table => pretty_print_resource_table(resources),
            OutputFormat::Json => pretty_print_resource_json(resources),
        }
    }

    pub fn pretty_print_single(&self, resource: &Resource) {
        match self {
            OutputFormat::Table => pretty_print_resource_detail_table(resource),
            OutputFormat::Json => pretty_print_resource_detail_json(resource),
        }
    }
}

pub fn pretty_print_resource_table(resources: Vec<Resource>) {
    let mut table = Table::new();

    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["", "ID", "Kind", "Name", "Created At"]);

    for (i, resource) in resources.iter().enumerate() {
        table.add_row(vec![
            &(i + 1).to_string(),
            &resource.id,
            &resource.name,
            &resource.kind,
            &resource.created_at,
        ]);
    }

    println!("{table}");
}

pub fn pretty_print_resource_json(resources: Vec<Resource>) {
    let mut collector = vec![];
    for resource in resources {
        collector.push(serde_json::Map::from_iter(vec![
            ("id".to_string(), serde_json::Value::from(resource.id)),
            ("name".to_string(), serde_json::Value::from(resource.name)),
            ("kind".to_string(), serde_json::Value::from(resource.kind)),
            (
                "created_at".to_string(),
                serde_json::Value::from(resource.created_at),
            ),
            (
                "spec".to_string(),
                serde_json::from_str::<serde_json::Value>(&resource.spec).unwrap(),
            ),
            (
                "annotations".to_string(),
                serde_json::from_str::<serde_json::Value>(
                    &resource.annotations.unwrap_or_default(),
                )
                .unwrap(),
            ),
        ]))
    }
    println!("{}", serde_json::to_string_pretty(&collector).unwrap());
}

pub fn pretty_print_resource_detail_json(resource: &Resource) {
    println!(
        "{}",
        serde_json::to_string_pretty(
            &(serde_json::Map::from_iter(vec![
                (
                    "id".to_string(),
                    serde_json::Value::from(resource.id.clone())
                ),
                (
                    "name".to_string(),
                    serde_json::Value::from(resource.name.clone())
                ),
                (
                    "kind".to_string(),
                    serde_json::Value::from(resource.kind.clone())
                ),
                (
                    "created_at".to_string(),
                    serde_json::Value::from(resource.created_at.clone()),
                ),
                (
                    "spec".to_string(),
                    serde_json::from_str::<serde_json::Value>(&resource.spec).unwrap(),
                ),
                (
                    "annotations".to_string(),
                    serde_json::from_str::<serde_json::Value>(
                        &resource.annotations.clone().unwrap_or_default(),
                    )
                    .unwrap(),
                ),
            ]))
        )
        .unwrap()
    );
}

pub fn pretty_print_resource_detail_table(resource: &Resource) {
    let mut table = Table::new();

    let annotations = serde_json::from_str::<serde_json::Value>(
        &resource.annotations.clone().unwrap_or_default(),
    )
    .unwrap();
    let mut annotations_headers: Vec<String> = annotations
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.get("label").unwrap().as_str().unwrap_or_default().into())
        .collect();

    let mut headers = vec![String::from("Name")];
    headers.append(&mut annotations_headers);

    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(headers);

    let mut values: Vec<String> = vec![resource.name.clone()];

    if let Some(annotations) = &resource.annotations {
        let annotations: serde_json::Value =
            serde_json::from_str(annotations).into_diagnostic().unwrap();

        for value in annotations.as_array().unwrap().iter() {
            values.push(
                value
                    .get("value")
                    .unwrap()
                    .as_str()
                    .unwrap_or_default()
                    .into(),
            );
        }
    }

    table.add_row(values);

    println!("{table}");
}
