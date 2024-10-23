use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{ContentArrangement, Table};
use dmtri::demeter::ops::v1alpha::Resource;
use miette::IntoDiagnostic;

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

pub fn pretty_print_resouce_detail_table(resources: Vec<Resource>) -> miette::Result<()> {
    let mut table = Table::new();

    let Some(first_resource) = resources.first() else {
        return Ok(());
    };

    let annotations = serde_json::from_str::<serde_json::Value>(
        &first_resource.annotations.clone().unwrap_or_default(),
    )
    .into_diagnostic()?;
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

    for resource in resources {
        let mut values: Vec<String> = vec![resource.name];

        if let Some(annotations) = resource.annotations {
            let annotations: serde_json::Value =
                serde_json::from_str(&annotations).into_diagnostic()?;

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
    }

    println!("{table}");

    Ok(())
}
