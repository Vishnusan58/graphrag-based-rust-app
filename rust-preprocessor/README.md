# Using the Rust Preprocessor

The Rust preprocessor is a CLI tool for preprocessing raw documents before ingestion into the system. This guide explains how to use it effectively.

## Overview

The preprocessor takes raw text documents (like the sample `doc` file in the project root) and extracts structured information from them, including:

- Document metadata (plan IDs, policy numbers, claim numbers, etc.)
- Document sections
- Entities (plans, policies, benefits, exclusions, procedures, etc.)

The extracted information is saved in a structured format (JSON or CSV) that can be loaded into Neo4j and Pinecone for use by the AI assistant.

## Basic Usage

The basic command structure is:

```bash
cd rust-preprocessor
cargo run -- process --input=<input_path> --output=<output_path> --format=<format> --doc-type=<doc_type>
```

### Parameters

- `--input` (required): Path to the input document file
- `--output` (required): Path to the output file where processed data will be written
- `--format` (optional, default: "json"): Output format, either "json" or "csv"
- `--doc-type` (optional): Type of document, either "plan", "policy", or "claim". If not provided, the preprocessor will try to infer the type from the filename.

## Output Path

The `--output` parameter specifies where the processed data will be saved. This should be a full path to a file, including the filename and extension. For example:

```bash
--output=C:\Users\vishn\PycharmProjects\aip\processed\plan_data.json
```

The preprocessor will create any necessary directories in the path if they don't exist.

## Supported Formats

The preprocessor supports two output formats:

1. **JSON** (default): Outputs a JSON file containing all extracted entities with their properties and relationships.
2. **CSV**: Outputs a CSV file with columns for entity type, name, description, related entities, and attributes.

## Example Commands

Here are some example commands using the sample document in the project:

### Process as a Plan Document with JSON Output

```bash
cd rust-preprocessor
cargo run -- process --input=..\doc --output=..\processed\plan_data.json --format=json --doc-type=plan
```

### Process as a Policy Document with CSV Output

```bash
cd rust-preprocessor
cargo run -- process --input=..\doc --output=..\processed\policy_data.csv --format=csv --doc-type=policy
```

### Process with Automatic Document Type Detection

```bash
cd rust-preprocessor
cargo run -- process --input=..\doc --output=..\processed\processed_data.json
```

## Next Steps

After preprocessing documents:

1. Load the JSON or CSV output into Neo4j and Pinecone
2. Start the backend server
3. Start the frontend development server
4. Interact with the chatbot through the web interface

## Troubleshooting

- If you encounter errors during processing, check that the input file exists and is readable.
- Make sure the output directory exists or that you have permission to create it.
- For Rust preprocessor issues, check that you have the latest stable Rust version installed.
