use clap::{Parser, Subcommand};
use std::path::PathBuf;
use anyhow::{Result, Context};
use log::{info, error};

mod document;
mod entities;
mod output;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Process a document and extract structured data
    Process {
        /// Input file path
        #[arg(short, long)]
        input: PathBuf,
        
        /// Output file path
        #[arg(short, long)]
        output: PathBuf,
        
        /// Output format (json or csv)
        #[arg(short, long, default_value = "json")]
        format: String,
        
        /// Document type (plan, policy, claim)
        #[arg(short, long)]
        doc_type: Option<String>,
    },
}

fn main() -> Result<()> {
    // Initialize logger
    env_logger::init();
    
    // Parse command line arguments
    let cli = Cli::parse();
    
    // Match command
    match &cli.command {
        Commands::Process { input, output, format, doc_type } => {
            process_document(input, output, format, doc_type.as_deref())
        }
    }
}

fn process_document(
    input: &PathBuf,
    output: &PathBuf,
    format: &str,
    doc_type: Option<&str>
) -> Result<()> {
    // Log start of processing
    info!("Processing document: {:?}", input);
    
    // Check if input file exists
    if !input.exists() {
        error!("Input file does not exist: {:?}", input);
        anyhow::bail!("Input file does not exist");
    }
    
    // Determine document type if not provided
    let doc_type = match doc_type {
        Some(dt) => dt.to_string(),
        None => {
            // Try to infer from filename
            let filename = input.file_name()
                .and_then(|f| f.to_str())
                .unwrap_or("");
            
            if filename.contains("plan") {
                "plan".to_string()
            } else if filename.contains("policy") {
                "policy".to_string()
            } else if filename.contains("claim") {
                "claim".to_string()
            } else {
                // Default to generic
                "generic".to_string()
            }
        }
    };
    
    // Read and parse the document
    let content = std::fs::read_to_string(input)
        .with_context(|| format!("Failed to read input file: {:?}", input))?;
    
    // Process the document based on its type
    let processed_data = document::process(&content, &doc_type)
        .with_context(|| "Failed to process document")?;
    
    // Extract entities
    let entities = entities::extract(&processed_data, &doc_type)
        .with_context(|| "Failed to extract entities")?;
    
    // Write output
    match format.to_lowercase().as_str() {
        "json" => {
            output::write_json(&entities, output)
                .with_context(|| format!("Failed to write JSON output to {:?}", output))?;
        },
        "csv" => {
            output::write_csv(&entities, output)
                .with_context(|| format!("Failed to write CSV output to {:?}", output))?;
        },
        _ => {
            error!("Unsupported output format: {}", format);
            anyhow::bail!("Unsupported output format: {}", format);
        }
    }
    
    info!("Successfully processed document and wrote output to {:?}", output);
    Ok(())
}