use anyhow::Result;
use regex::Regex;
use aho_corasick::AhoCorasick;
use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};

use crate::document::ProcessedDocument;

/// Entity types for healthcare insurance documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityType {
    Plan,
    Policy,
    Benefit,
    Exclusion,
    Procedure,
    Claim,
    Provider,
    Coverage,
    Condition,
    Limitation,
}

/// Entity extracted from a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// Entity type
    pub entity_type: String,

    /// Entity name or identifier
    pub name: String,

    /// Entity description or details
    pub description: Option<String>,

    /// Related entities (by name)
    pub related: Vec<String>,

    /// Additional attributes
    pub attributes: HashMap<String, String>,
}

/// Extract entities from a processed document
pub fn extract(doc: &ProcessedDocument, doc_type: &str) -> Result<Vec<Entity>> {
    let mut entities = Vec::new();

    // Extract entities based on document type
    match doc_type {
        "plan" => extract_plan_entities(doc, &mut entities)?,
        "policy" => extract_policy_entities(doc, &mut entities)?,
        "claim" => extract_claim_entities(doc, &mut entities)?,
        _ => extract_generic_entities(doc, &mut entities)?,
    }

    // Extract common entities across all document types
    extract_benefits(doc, &mut entities)?;
    extract_exclusions(doc, &mut entities)?;
    extract_procedures(doc, &mut entities)?;

    // Deduplicate entities
    deduplicate_entities(&mut entities);

    Ok(entities)
}

/// Extract plan-specific entities
fn extract_plan_entities(doc: &ProcessedDocument, entities: &mut Vec<Entity>) -> Result<()> {
    // Extract plan entity
    let mut plan_entity = Entity {
        entity_type: "Plan".to_string(),
        name: doc.title.clone().unwrap_or_else(|| "Unnamed Plan".to_string()),
        description: None,
        related: Vec::new(),
        attributes: HashMap::new(),
    };

    // Add metadata as attributes
    for (key, value) in &doc.metadata {
        plan_entity.attributes.insert(key.clone(), value.clone());
    }

    // Look for plan description in sections
    for section in &doc.sections {
        if let Some(title) = &section.title {
            if title.to_lowercase().contains("overview") || 
               title.to_lowercase().contains("description") || 
               title.to_lowercase().contains("summary") {
                plan_entity.description = Some(section.content.clone());
                break;
            }
        }
    }

    entities.push(plan_entity);

    // Extract coverage entities
    extract_coverage(doc, entities)?;

    Ok(())
}

/// Extract policy-specific entities
fn extract_policy_entities(doc: &ProcessedDocument, entities: &mut Vec<Entity>) -> Result<()> {
    // Extract policy entity
    let mut policy_entity = Entity {
        entity_type: "Policy".to_string(),
        name: doc.title.clone().unwrap_or_else(|| "Unnamed Policy".to_string()),
        description: None,
        related: Vec::new(),
        attributes: HashMap::new(),
    };

    // Add metadata as attributes
    for (key, value) in &doc.metadata {
        policy_entity.attributes.insert(key.clone(), value.clone());
    }

    // Look for policy description in sections
    for section in &doc.sections {
        if let Some(title) = &section.title {
            if title.to_lowercase().contains("overview") || 
               title.to_lowercase().contains("description") || 
               title.to_lowercase().contains("summary") {
                policy_entity.description = Some(section.content.clone());
                break;
            }
        }
    }

    entities.push(policy_entity);

    // Extract conditions and limitations
    extract_conditions_and_limitations(doc, entities)?;

    Ok(())
}

/// Extract claim-specific entities
fn extract_claim_entities(doc: &ProcessedDocument, entities: &mut Vec<Entity>) -> Result<()> {
    // Extract claim entity
    let mut claim_entity = Entity {
        entity_type: "Claim".to_string(),
        name: doc.metadata.get("claim_number")
            .cloned()
            .unwrap_or_else(|| "Unnamed Claim".to_string()),
        description: None,
        related: Vec::new(),
        attributes: HashMap::new(),
    };

    // Add metadata as attributes
    for (key, value) in &doc.metadata {
        claim_entity.attributes.insert(key.clone(), value.clone());
    }

    // Store claim name before moving claim_entity
    let claim_name = claim_entity.name.clone();
    entities.push(claim_entity);

    // Extract provider entity if available
    if let Some(provider) = doc.metadata.get("provider") {
        let provider_entity = Entity {
            entity_type: "Provider".to_string(),
            name: provider.clone(),
            description: None,
            related: vec![claim_name],
            attributes: HashMap::new(),
        };

        entities.push(provider_entity);

        // Add provider to claim's related entities
        if let Some(claim) = entities.iter_mut().find(|e| e.entity_type == "Claim") {
            claim.related.push(provider.clone());
        }
    }

    Ok(())
}

/// Extract generic entities for unknown document types
fn extract_generic_entities(doc: &ProcessedDocument, entities: &mut Vec<Entity>) -> Result<()> {
    // Create a generic document entity
    let mut doc_entity = Entity {
        entity_type: "Document".to_string(),
        name: doc.title.clone().unwrap_or_else(|| "Unnamed Document".to_string()),
        description: None,
        related: Vec::new(),
        attributes: HashMap::new(),
    };

    // Add metadata as attributes
    for (key, value) in &doc.metadata {
        doc_entity.attributes.insert(key.clone(), value.clone());
    }

    entities.push(doc_entity);

    Ok(())
}

/// Extract benefits from the document
fn extract_benefits(doc: &ProcessedDocument, entities: &mut Vec<Entity>) -> Result<()> {
    // Keywords that indicate benefit sections
    let benefit_keywords = [
        "benefit", "benefits", "covered", "coverage", "covers", "included", "includes"
    ];

    // Find sections related to benefits
    for section in &doc.sections {
        let section_text = if let Some(title) = &section.title {
            format!("{} {}", title, section.content)
        } else {
            section.content.clone()
        };

        let section_text_lower = section_text.to_lowercase();

        // Check if this section is about benefits
        if benefit_keywords.iter().any(|&kw| section_text_lower.contains(kw)) {
            // Extract benefit items using patterns like bullet points or numbered lists
            let benefit_patterns = [
                Regex::new(r"(?m)^[\s•\-*]+([^:]+)(?::|\s-\s)(.+)$").unwrap(),
                Regex::new(r"(?m)^[\s]*\d+\.\s+([^:]+)(?::|\s-\s)(.+)$").unwrap(),
            ];

            for pattern in &benefit_patterns {
                for captures in pattern.captures_iter(&section_text) {
                    if captures.len() > 2 {
                        let name = captures[1].trim().to_string();
                        let description = captures[2].trim().to_string();

                        // Create benefit entity
                        let benefit_entity = Entity {
                            entity_type: "Benefit".to_string(),
                            name,
                            description: Some(description),
                            related: Vec::new(),
                            attributes: HashMap::new(),
                        };

                        entities.push(benefit_entity);
                    }
                }
            }

            // If no structured benefits found, try to extract from plain text
            if !pattern_found_entities(&benefit_patterns, &section_text) {
                // Split by sentences or line breaks
                let sentences: Vec<&str> = section_text
                    .split(|c| c == '.' || c == '\n')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .collect();

                for sentence in sentences {
                    if benefit_keywords.iter().any(|&kw| sentence.to_lowercase().contains(kw)) {
                        // Create benefit entity from sentence
                        let benefit_entity = Entity {
                            entity_type: "Benefit".to_string(),
                            name: sentence.to_string(),
                            description: None,
                            related: Vec::new(),
                            attributes: HashMap::new(),
                        };

                        entities.push(benefit_entity);
                    }
                }
            }
        }
    }

    Ok(())
}

/// Extract exclusions from the document
fn extract_exclusions(doc: &ProcessedDocument, entities: &mut Vec<Entity>) -> Result<()> {
    // Keywords that indicate exclusion sections
    let exclusion_keywords = [
        "exclusion", "exclusions", "excluded", "not covered", "not include", "limitation", "limitations"
    ];

    // Find sections related to exclusions
    for section in &doc.sections {
        let section_text = if let Some(title) = &section.title {
            format!("{} {}", title, section.content)
        } else {
            section.content.clone()
        };

        let section_text_lower = section_text.to_lowercase();

        // Check if this section is about exclusions
        if exclusion_keywords.iter().any(|&kw| section_text_lower.contains(kw)) {
            // Extract exclusion items using patterns like bullet points or numbered lists
            let exclusion_patterns = [
                Regex::new(r"(?m)^[\s•\-*]+([^:]+)(?::|\s-\s)(.+)$").unwrap(),
                Regex::new(r"(?m)^[\s]*\d+\.\s+([^:]+)(?::|\s-\s)(.+)$").unwrap(),
            ];

            for pattern in &exclusion_patterns {
                for captures in pattern.captures_iter(&section_text) {
                    if captures.len() > 2 {
                        let name = captures[1].trim().to_string();
                        let description = captures[2].trim().to_string();

                        // Create exclusion entity
                        let exclusion_entity = Entity {
                            entity_type: "Exclusion".to_string(),
                            name,
                            description: Some(description),
                            related: Vec::new(),
                            attributes: HashMap::new(),
                        };

                        entities.push(exclusion_entity);
                    }
                }
            }

            // If no structured exclusions found, try to extract from plain text
            if !pattern_found_entities(&exclusion_patterns, &section_text) {
                // Split by sentences or line breaks
                let sentences: Vec<&str> = section_text
                    .split(|c| c == '.' || c == '\n')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .collect();

                for sentence in sentences {
                    if exclusion_keywords.iter().any(|&kw| sentence.to_lowercase().contains(kw)) {
                        // Create exclusion entity from sentence
                        let exclusion_entity = Entity {
                            entity_type: "Exclusion".to_string(),
                            name: sentence.to_string(),
                            description: None,
                            related: Vec::new(),
                            attributes: HashMap::new(),
                        };

                        entities.push(exclusion_entity);
                    }
                }
            }
        }
    }

    Ok(())
}

/// Extract procedures from the document
fn extract_procedures(doc: &ProcessedDocument, entities: &mut Vec<Entity>) -> Result<()> {
    // Common medical procedures
    let procedures = [
        "surgery", "consultation", "examination", "x-ray", "mri", "ct scan", "ultrasound",
        "blood test", "vaccination", "immunization", "therapy", "treatment", "screening",
        "checkup", "physical", "dental cleaning", "filling", "root canal", "crown",
        "prescription", "medication", "injection", "infusion", "dialysis", "transplant",
        "rehabilitation", "physical therapy", "occupational therapy", "speech therapy",
        "chemotherapy", "radiation", "anesthesia", "biopsy", "colonoscopy", "endoscopy",
        "mammogram", "pap smear", "prenatal care", "delivery", "maternity", "emergency",
        "ambulance", "hospitalization", "inpatient", "outpatient", "preventive care"
    ];

    // Build Aho-Corasick automaton for efficient string matching
    let ac = AhoCorasick::new(procedures).unwrap();

    // Find procedures in all sections
    for section in &doc.sections {
        let section_text = if let Some(title) = &section.title {
            format!("{} {}", title, section.content)
        } else {
            section.content.clone()
        };

        // Find all matches
        let matches: Vec<_> = ac.find_iter(&section_text.to_lowercase()).collect();

        for mat in matches {
            let procedure_name = &procedures[mat.pattern()];

            // Create procedure entity
            let procedure_entity = Entity {
                entity_type: "Procedure".to_string(),
                name: procedure_name.to_string(),
                description: None,
                related: Vec::new(),
                attributes: HashMap::new(),
            };

            entities.push(procedure_entity);
        }
    }

    Ok(())
}

/// Extract coverage information
fn extract_coverage(doc: &ProcessedDocument, entities: &mut Vec<Entity>) -> Result<()> {
    // Keywords that indicate coverage sections
    let coverage_keywords = [
        "coverage", "covers", "covered", "benefit", "benefits"
    ];

    // Find sections related to coverage
    for section in &doc.sections {
        let section_text = if let Some(title) = &section.title {
            format!("{} {}", title, section.content)
        } else {
            section.content.clone()
        };

        let section_text_lower = section_text.to_lowercase();

        // Check if this section is about coverage
        if coverage_keywords.iter().any(|&kw| section_text_lower.contains(kw)) {
            // Create coverage entity
            let coverage_name = if let Some(title) = &section.title {
                title.clone()
            } else {
                "Coverage".to_string()
            };

            let coverage_entity = Entity {
                entity_type: "Coverage".to_string(),
                name: coverage_name,
                description: Some(section.content.clone()),
                related: Vec::new(),
                attributes: HashMap::new(),
            };

            entities.push(coverage_entity);
        }
    }

    Ok(())
}

/// Extract conditions and limitations
fn extract_conditions_and_limitations(doc: &ProcessedDocument, entities: &mut Vec<Entity>) -> Result<()> {
    // Keywords that indicate conditions and limitations sections
    let condition_keywords = [
        "condition", "conditions", "requirement", "requirements", "prerequisite", "prerequisites"
    ];

    let limitation_keywords = [
        "limitation", "limitations", "limit", "limits", "restricted", "restriction", "restrictions"
    ];

    // Find sections related to conditions
    for section in &doc.sections {
        let section_text = if let Some(title) = &section.title {
            format!("{} {}", title, section.content)
        } else {
            section.content.clone()
        };

        let section_text_lower = section_text.to_lowercase();

        // Check if this section is about conditions
        if condition_keywords.iter().any(|&kw| section_text_lower.contains(kw)) {
            // Create condition entity
            let condition_name = if let Some(title) = &section.title {
                title.clone()
            } else {
                "Condition".to_string()
            };

            let condition_entity = Entity {
                entity_type: "Condition".to_string(),
                name: condition_name,
                description: Some(section.content.clone()),
                related: Vec::new(),
                attributes: HashMap::new(),
            };

            entities.push(condition_entity);
        }

        // Check if this section is about limitations
        if limitation_keywords.iter().any(|&kw| section_text_lower.contains(kw)) {
            // Create limitation entity
            let limitation_name = if let Some(title) = &section.title {
                title.clone()
            } else {
                "Limitation".to_string()
            };

            let limitation_entity = Entity {
                entity_type: "Limitation".to_string(),
                name: limitation_name,
                description: Some(section.content.clone()),
                related: Vec::new(),
                attributes: HashMap::new(),
            };

            entities.push(limitation_entity);
        }
    }

    Ok(())
}

/// Check if any pattern found entities in the text
fn pattern_found_entities(patterns: &[Regex], text: &str) -> bool {
    patterns.iter().any(|pattern| pattern.is_match(text))
}

/// Deduplicate entities by name and type
fn deduplicate_entities(entities: &mut Vec<Entity>) {
    let mut seen = HashSet::new();
    let mut i = 0;

    while i < entities.len() {
        let key = format!("{}:{}", entities[i].entity_type, entities[i].name);

        if seen.contains(&key) {
            entities.remove(i);
        } else {
            seen.insert(key);
            i += 1;
        }
    }
}
