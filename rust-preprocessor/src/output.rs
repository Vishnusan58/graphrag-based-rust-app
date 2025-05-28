use anyhow::{Result, Context};
use std::path::Path;
use std::fs::File;
use std::io::BufWriter;
use serde_json;
use csv;
use polars::prelude::*;

use crate::entities::Entity;

/// Write entities to a JSON file
pub fn write_json<P: AsRef<Path>>(entities: &[Entity], output_path: P) -> Result<()> {
    // Create the output file
    let file = File::create(output_path.as_ref())
        .with_context(|| format!("Failed to create output file: {:?}", output_path.as_ref()))?;

    // Create a buffered writer
    let writer = BufWriter::new(file);

    // Serialize entities to JSON
    serde_json::to_writer_pretty(writer, entities)
        .with_context(|| "Failed to serialize entities to JSON")?;

    Ok(())
}

/// Write entities to a CSV file
pub fn write_csv<P: AsRef<Path>>(entities: &[Entity], output_path: P) -> Result<()> {
    // If there are no entities, create an empty CSV file
    if entities.is_empty() {
        let file = File::create(output_path.as_ref())
            .with_context(|| format!("Failed to create output file: {:?}", output_path.as_ref()))?;

        let mut writer = csv::Writer::from_writer(file);
        writer.write_record(&["entity_type", "name", "description", "related", "attributes"])
            .with_context(|| "Failed to write CSV header")?;

        writer.flush()
            .with_context(|| "Failed to flush CSV writer")?;

        return Ok(());
    }

    // Convert entities to a format suitable for CSV
    let mut entity_types = Vec::with_capacity(entities.len());
    let mut names = Vec::with_capacity(entities.len());
    let mut descriptions = Vec::with_capacity(entities.len());
    let mut related_lists = Vec::with_capacity(entities.len());
    let mut attribute_lists = Vec::with_capacity(entities.len());

    for entity in entities {
        entity_types.push(entity.entity_type.clone());
        names.push(entity.name.clone());
        descriptions.push(entity.description.clone().unwrap_or_default());

        // Join related entities with a semicolon
        let related_str = entity.related.join(";");
        related_lists.push(related_str);

        // Convert attributes to a string
        let attributes_str = entity.attributes.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join(";");
        attribute_lists.push(attributes_str);
    }

    // Create a DataFrame
    let mut df = DataFrame::new(vec![
        Series::new("entity_type", entity_types),
        Series::new("name", names),
        Series::new("description", descriptions),
        Series::new("related", related_lists),
        Series::new("attributes", attribute_lists),
    ])
    .with_context(|| "Failed to create DataFrame")?;

    // Write to CSV
    let mut file = File::create(output_path.as_ref())
        .with_context(|| format!("Failed to create output file: {:?}", output_path.as_ref()))?;

    CsvWriter::new(&mut file)
        .finish(&mut df)
        .with_context(|| "Failed to write DataFrame to CSV")?;

    Ok(())
}

/// Write entities to a graph-friendly format (nodes and edges)
pub fn write_graph_format<P: AsRef<Path>>(entities: &[Entity], output_dir: P) -> Result<()> {
    // Create nodes file
    let nodes_path = output_dir.as_ref().join("nodes.csv");
    let nodes_file = File::create(&nodes_path)
        .with_context(|| format!("Failed to create nodes file: {:?}", nodes_path))?;

    let mut nodes_writer = csv::Writer::from_writer(nodes_file);

    // Write nodes header
    nodes_writer.write_record(&["id", "label", "name", "description"])
        .with_context(|| "Failed to write nodes CSV header")?;

    // Create edges file
    let edges_path = output_dir.as_ref().join("edges.csv");
    let edges_file = File::create(&edges_path)
        .with_context(|| format!("Failed to create edges file: {:?}", edges_path))?;

    let mut edges_writer = csv::Writer::from_writer(edges_file);

    // Write edges header
    edges_writer.write_record(&["source", "target", "type"])
        .with_context(|| "Failed to write edges CSV header")?;

    // Write nodes and collect relationships
    for (i, entity) in entities.iter().enumerate() {
        // Write node
        nodes_writer.write_record(&[
            i.to_string(),
            entity.entity_type.clone(),
            entity.name.clone(),
            entity.description.clone().unwrap_or_default(),
        ])
        .with_context(|| format!("Failed to write node record for entity: {}", entity.name))?;

        // Write edges for related entities
        for related_name in &entity.related {
            // Find the related entity
            if let Some((j, _)) = entities.iter().enumerate()
                .find(|(_, e)| e.name == *related_name) {

                // Write edge
                edges_writer.write_record(&[
                    i.to_string(),
                    j.to_string(),
                    "RELATED_TO".to_string(),
                ])
                .with_context(|| format!("Failed to write edge record for relationship: {} -> {}", entity.name, related_name))?;
            }
        }

        // Write edges for attributes that reference other entities
        for (key, value) in &entity.attributes {
            // Check if this attribute references another entity
            if key.ends_with("_id") || key.contains("reference") {
                // Find the referenced entity
                if let Some((j, _)) = entities.iter().enumerate()
                    .find(|(_, e)| e.name == *value || e.attributes.values().any(|v| v == value)) {

                    // Write edge
                    edges_writer.write_record(&[
                        i.to_string(),
                        j.to_string(),
                        key.to_uppercase(),
                    ])
                    .with_context(|| format!("Failed to write edge record for attribute relationship: {} -> {}", entity.name, value))?;
                }
            }
        }
    }

    // Flush writers
    nodes_writer.flush()
        .with_context(|| "Failed to flush nodes CSV writer")?;

    edges_writer.flush()
        .with_context(|| "Failed to flush edges CSV writer")?;

    Ok(())
}
