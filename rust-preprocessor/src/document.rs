use anyhow::{Result, Context};
use regex::Regex;
use std::collections::HashMap;

/// Processed document data
#[derive(Debug, Clone)]
pub struct ProcessedDocument {
    /// Document type (plan, policy, claim)
    pub doc_type: String,

    /// Document title or name
    pub title: Option<String>,

    /// Document sections
    pub sections: Vec<Section>,

    /// Extracted key-value pairs
    pub metadata: HashMap<String, String>,
}

/// Document section
#[derive(Debug, Clone)]
pub struct Section {
    /// Section title
    pub title: Option<String>,

    /// Section content
    pub content: String,

    /// Section level (1 = top level, 2 = subsection, etc.)
    pub level: usize,
}

/// Process a document and return structured data
pub fn process(content: &str, doc_type: &str) -> Result<ProcessedDocument> {
    // Create a new processed document
    let mut doc = ProcessedDocument {
        doc_type: doc_type.to_string(),
        title: None,
        sections: Vec::new(),
        metadata: HashMap::new(),
    };

    // Extract document title
    doc.title = extract_title(content);

    // Extract sections
    doc.sections = extract_sections(content)?;

    // Extract metadata
    doc.metadata = extract_metadata(content, doc_type)?;

    Ok(doc)
}

/// Extract the document title
fn extract_title(content: &str) -> Option<String> {
    // Try to find the title in the first few lines
    let first_lines: Vec<&str> = content.lines().take(10).collect();

    // Look for patterns like "Title: X" or just a standalone line that looks like a title
    for line in first_lines {
        let line = line.trim();

        // Skip empty lines
        if line.is_empty() {
            continue;
        }

        // Check for "Title: X" pattern
        if let Some(title) = line.strip_prefix("Title:") {
            return Some(title.trim().to_string());
        }

        // Check for "Name: X" pattern
        if let Some(title) = line.strip_prefix("Name:") {
            return Some(title.trim().to_string());
        }

        // If it's a short line (likely a title) and not all uppercase (likely a header)
        if line.len() < 100 && line != line.to_uppercase() && !line.contains(":") {
            return Some(line.to_string());
        }
    }

    None
}

/// Extract sections from the document
fn extract_sections(content: &str) -> Result<Vec<Section>> {
    let mut sections = Vec::new();

    // Define regex patterns for section headers
    let section_patterns = [
        // Level 1: Roman numerals or uppercase headers
        (Regex::new(r"^(?:\s*)(I{1,3}|IV|V|VI{1,3}|IX|X)(?:\.\s+)(.+)$").unwrap(), 1),
        (Regex::new(r"^(?:\s*)([A-Z][A-Z\s]+)(?:\s*):").unwrap(), 1),

        // Level 2: Alphabetic or numeric headers
        (Regex::new(r"^(?:\s*)([A-Z])(?:\.\s+)(.+)$").unwrap(), 2),
        (Regex::new(r"^(?:\s*)(\d+)(?:\.\s+)(.+)$").unwrap(), 2),

        // Level 3: Lowercase or numeric+alphabetic headers
        (Regex::new(r"^(?:\s*)([a-z])(?:\.\s+)(.+)$").unwrap(), 3),
        (Regex::new(r"^(?:\s*)(\d+\.\d+)(?:\s+)(.+)$").unwrap(), 3),
    ];

    let lines: Vec<&str> = content.lines().collect();
    let mut current_section = Section {
        title: None,
        content: String::new(),
        level: 1,
    };

    for (_i, line) in lines.iter().enumerate() {
        let line = line.trim();

        // Skip empty lines
        if line.is_empty() {
            continue;
        }

        // Check if this line is a section header
        let mut is_header = false;
        for (pattern, level) in &section_patterns {
            if let Some(captures) = pattern.captures(line) {
                // If we have content in the current section, add it to the list
                if !current_section.content.is_empty() || current_section.title.is_some() {
                    sections.push(current_section.clone());
                }

                // Start a new section
                current_section = Section {
                    title: Some(if captures.len() > 2 {
                        format!("{} {}", &captures[1], &captures[2])
                    } else {
                        captures[1].to_string()
                    }),
                    content: String::new(),
                    level: *level,
                };

                is_header = true;
                break;
            }
        }

        // If not a header, add to current section content
        if !is_header {
            if !current_section.content.is_empty() {
                current_section.content.push('\n');
            }
            current_section.content.push_str(line);
        }
    }

    // Add the last section if it has content
    if !current_section.content.is_empty() || current_section.title.is_some() {
        sections.push(current_section);
    }

    Ok(sections)
}

/// Extract metadata from the document
fn extract_metadata(content: &str, doc_type: &str) -> Result<HashMap<String, String>> {
    let mut metadata = HashMap::new();

    // Define patterns based on document type
    let patterns: Vec<(&str, &str)> = match doc_type {
        "plan" => vec![
            (r"(?i)Plan\s+ID\s*:\s*([A-Z0-9-]+)", "plan_id"),
            (r"(?i)Effective\s+Date\s*:\s*(\d{1,2}/\d{1,2}/\d{2,4})", "effective_date"),
            (r"(?i)Coverage\s+Type\s*:\s*([A-Za-z\s]+)", "coverage_type"),
            (r"(?i)Premium\s*:\s*\$?(\d+(?:\.\d{2})?)", "premium"),
        ],
        "policy" => vec![
            (r"(?i)Policy\s+Number\s*:\s*([A-Z0-9-]+)", "policy_number"),
            (r"(?i)Policyholder\s*:\s*([A-Za-z\s]+)", "policyholder"),
            (r"(?i)Issue\s+Date\s*:\s*(\d{1,2}/\d{1,2}/\d{2,4})", "issue_date"),
            (r"(?i)Expiration\s+Date\s*:\s*(\d{1,2}/\d{1,2}/\d{2,4})", "expiration_date"),
        ],
        "claim" => vec![
            (r"(?i)Claim\s+Number\s*:\s*([A-Z0-9-]+)", "claim_number"),
            (r"(?i)Date\s+of\s+Service\s*:\s*(\d{1,2}/\d{1,2}/\d{2,4})", "service_date"),
            (r"(?i)Provider\s*:\s*([A-Za-z\s]+)", "provider"),
            (r"(?i)Amount\s*:\s*\$?(\d+(?:\.\d{2})?)", "amount"),
            (r"(?i)Status\s*:\s*([A-Za-z\s]+)", "status"),
        ],
        _ => vec![
            // Generic patterns for any document type
            (r"(?i)ID\s*:\s*([A-Z0-9-]+)", "id"),
            (r"(?i)Date\s*:\s*(\d{1,2}/\d{1,2}/\d{2,4})", "date"),
            (r"(?i)Name\s*:\s*([A-Za-z\s]+)", "name"),
        ],
    };

    // Extract metadata using regex patterns
    for (pattern, key) in patterns {
        let re = Regex::new(pattern)
            .with_context(|| format!("Failed to compile regex pattern: {}", pattern))?;

        if let Some(captures) = re.captures(content) {
            if captures.len() > 1 {
                metadata.insert(key.to_string(), captures[1].trim().to_string());
            }
        }
    }

    Ok(metadata)
}
