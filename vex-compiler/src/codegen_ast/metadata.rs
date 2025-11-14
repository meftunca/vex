/// Metadata parsing and policy application utilities
/// This module handles:
/// - Parsing backtick metadata strings into HashMap
/// - Applying policies to struct fields
/// - Merging metadata from multiple sources
/// - Policy composition with parent resolution
use std::collections::{HashMap, HashSet};
use vex_ast::Policy;

/// Parse metadata string into HashMap
/// Format: `json:"id" db:"user_id" validate:"required"`
/// Result: {"json": "id", "db": "user_id", "validate": "required"}
pub fn parse_metadata(raw: &str) -> Result<HashMap<String, String>, String> {
    let mut metadata = HashMap::new();
    let mut chars = raw.chars().peekable();

    while chars.peek().is_some() {
        // Skip whitespace
        while chars.peek().map_or(false, |c| c.is_whitespace()) {
            chars.next();
        }

        if chars.peek().is_none() {
            break;
        }

        // Parse key (identifier before colon)
        let mut key = String::new();
        while chars
            .peek()
            .map_or(false, |c| c.is_alphanumeric() || *c == '_')
        {
            key.push(chars.next().unwrap());
        }

        if key.is_empty() {
            return Err("Expected metadata key (identifier before ':')".to_string());
        }

        // Expect colon
        while chars.peek().map_or(false, |c| c.is_whitespace()) {
            chars.next();
        }

        if chars.next() != Some(':') {
            return Err(format!("Expected ':' after metadata key '{}'", key));
        }

        // Skip whitespace
        while chars.peek().map_or(false, |c| c.is_whitespace()) {
            chars.next();
        }

        // Parse value (quoted string)
        if chars.next() != Some('"') {
            return Err(format!("Expected '\"' to start value for key '{}'", key));
        }

        let mut value = String::new();
        let mut escaped = false;

        loop {
            match chars.next() {
                Some('\\') if !escaped => {
                    escaped = true;
                }
                Some('"') if !escaped => {
                    break; // End of value
                }
                Some(c) => {
                    if escaped {
                        // Handle escape sequences
                        match c {
                            'n' => value.push('\n'),
                            't' => value.push('\t'),
                            'r' => value.push('\r'),
                            '\\' => value.push('\\'),
                            '"' => value.push('"'),
                            _ => {
                                value.push('\\');
                                value.push(c);
                            }
                        }
                        escaped = false;
                    } else {
                        value.push(c);
                    }
                }
                None => {
                    return Err(format!("Unterminated string value for key '{}'", key));
                }
            }
        }

        metadata.insert(key, value);
    }

    Ok(metadata)
}

/// Merge two metadata HashMaps
/// Strategy: new_metadata overrides old_metadata for same keys
/// Returns: (merged, conflicts)
/// - merged: Combined HashMap
/// - conflicts: Vec of (key, old_value, new_value) for overridden keys
pub fn merge_metadata(
    old: &HashMap<String, String>,
    new: &HashMap<String, String>,
) -> (HashMap<String, String>, Vec<(String, String, String)>) {
    let mut merged = old.clone();
    let mut conflicts = Vec::new();

    for (key, new_value) in new {
        if let Some(old_value) = merged.get(key) {
            if old_value != new_value {
                // Conflict: same key, different values
                conflicts.push((key.clone(), old_value.clone(), new_value.clone()));
            }
        }
        // New value wins (override)
        merged.insert(key.clone(), new_value.clone());
    }

    (merged, conflicts)
}

/// Apply policy fields to struct fields (by name matching)
/// Returns: Vec of (field_name, merged_metadata, warnings)
pub fn apply_policy_to_fields(
    policy: &Policy,
    field_names: &[String],
) -> Vec<(String, HashMap<String, String>, Vec<String>)> {
    let mut results = Vec::new();

    // Build policy field map: field_name -> metadata
    let mut policy_map: HashMap<String, HashMap<String, String>> = HashMap::new();
    for policy_field in &policy.fields {
        match parse_metadata(&policy_field.metadata) {
            Ok(metadata) => {
                policy_map.insert(policy_field.name.clone(), metadata);
            }
            Err(e) => {
                eprintln!(
                    "⚠️  Failed to parse metadata for field '{}': {}",
                    policy_field.name, e
                );
            }
        }
    }

    // Apply to each struct field
    for field_name in field_names {
        let warnings = Vec::new();

        let metadata = if let Some(policy_metadata) = policy_map.get(field_name) {
            policy_metadata.clone()
        } else {
            // Field not in policy - no metadata
            HashMap::new()
        };

        results.push((field_name.clone(), metadata, warnings));
    }

    // Check for policy fields not in struct (warning)
    for policy_field_name in policy_map.keys() {
        if !field_names.contains(policy_field_name) {
            eprintln!(
                "⚠️  Policy '{}' has field '{}' but struct doesn't have this field",
                policy.name, policy_field_name
            );
        }
    }

    results
}

/// Resolve policy with all its parents (recursive)
/// Returns policies in inheritance order: [Parent, Child]
/// Detects circular dependencies
pub fn resolve_policy_hierarchy<'a>(
    policy: &'a Policy,
    all_policies: &'a HashMap<String, Policy>,
    visited: &mut HashSet<String>,
) -> Result<Vec<&'a Policy>, String> {
    // Circular dependency check
    if visited.contains(&policy.name) {
        return Err(format!(
            "Circular policy dependency detected: '{}'",
            policy.name
        ));
    }

    visited.insert(policy.name.clone());

    let mut hierarchy = Vec::new();

    // Recursively resolve parents first
    for parent_name in &policy.parent_policies {
        let parent = all_policies
            .get(parent_name)
            .ok_or_else(|| format!("Parent policy '{}' not found", parent_name))?;

        // Get parent's full hierarchy
        let parent_hierarchy = resolve_policy_hierarchy(parent, all_policies, visited)?;

        // Add parent hierarchy (avoiding duplicates)
        for p in parent_hierarchy {
            if !hierarchy
                .iter()
                .any(|existing: &&Policy| existing.name == p.name)
            {
                hierarchy.push(p);
            }
        }
    }

    // Add current policy at the end (child overrides parent)
    hierarchy.push(policy);

    Ok(hierarchy)
}

/// Apply policy hierarchy (with parents) to struct fields
/// Returns merged metadata with child policies overriding parents
pub fn apply_policy_hierarchy_to_fields(
    policy_name: &str,
    all_policies: &HashMap<String, Policy>,
    field_names: &[String],
) -> Result<Vec<(String, HashMap<String, String>, Vec<String>)>, String> {
    let policy = all_policies
        .get(policy_name)
        .ok_or_else(|| format!("Policy '{}' not found", policy_name))?;

    // Resolve full hierarchy
    let mut visited = HashSet::new();
    let hierarchy = resolve_policy_hierarchy(policy, all_policies, &mut visited)?;

  
    // Apply policies in order (parent first, child overrides)
    let mut field_metadata: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut all_warnings: HashMap<String, Vec<String>> = HashMap::new();

    for current_policy in hierarchy {
        let field_results = apply_policy_to_fields(current_policy, field_names);

        for (field_name, field_meta, warnings) in field_results {
            // Merge with existing metadata
            let existing = field_metadata.get(&field_name);
            let (merged, conflicts) = if let Some(existing_meta) = existing {
                merge_metadata(existing_meta, &field_meta)
            } else {
                (field_meta.clone(), vec![])
            };

            // Track conflicts as warnings
            if !conflicts.is_empty() {
                let conflict_warnings: Vec<String> = conflicts
                    .iter()
                    .map(|(key, old_val, new_val)| {
                        format!(
                            "Policy '{}' overrides '{}': '{}' → '{}'",
                            current_policy.name, key, old_val, new_val
                        )
                    })
                    .collect();

                all_warnings
                    .entry(field_name.clone())
                    .or_insert_with(Vec::new)
                    .extend(conflict_warnings);
            }

            // Store warnings from field parsing
            all_warnings
                .entry(field_name.clone())
                .or_insert_with(Vec::new)
                .extend(warnings);

            field_metadata.insert(field_name, merged);
        }
    }

    // Convert back to result format
    let results: Vec<_> = field_names
        .iter()
        .map(|field_name| {
            let metadata = field_metadata.get(field_name).cloned().unwrap_or_default();
            let warnings = all_warnings.get(field_name).cloned().unwrap_or_default();
            (field_name.clone(), metadata, warnings)
        })
        .collect();

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_metadata() {
        let input = r#"json:"id""#;
        let result = parse_metadata(input).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result.get("json"), Some(&"id".to_string()));
    }

    #[test]
    fn test_parse_multiple_metadata() {
        let input = r#"json:"id" db:"user_id" validate:"required""#;
        let result = parse_metadata(input).unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result.get("json"), Some(&"id".to_string()));
        assert_eq!(result.get("db"), Some(&"user_id".to_string()));
        assert_eq!(result.get("validate"), Some(&"required".to_string()));
    }

    #[test]
    fn test_parse_with_spaces() {
        let input = r#"  json : "id"   db  :  "user_id"  "#;
        let result = parse_metadata(input).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result.get("json"), Some(&"id".to_string()));
        assert_eq!(result.get("db"), Some(&"user_id".to_string()));
    }

    #[test]
    fn test_parse_escaped_quotes() {
        let input = r#"pattern:"\"hello\"""#;
        let result = parse_metadata(input).unwrap();

        assert_eq!(result.get("pattern"), Some(&r#""hello""#.to_string()));
    }

    #[test]
    fn test_merge_no_conflict() {
        let mut old = HashMap::new();
        old.insert("json".to_string(), "id".to_string());

        let mut new = HashMap::new();
        new.insert("db".to_string(), "user_id".to_string());

        let (merged, conflicts) = merge_metadata(&old, &new);

        assert_eq!(merged.len(), 2);
        assert_eq!(conflicts.len(), 0);
        assert_eq!(merged.get("json"), Some(&"id".to_string()));
        assert_eq!(merged.get("db"), Some(&"user_id".to_string()));
    }

    #[test]
    fn test_merge_with_conflict() {
        let mut old = HashMap::new();
        old.insert("json".to_string(), "id".to_string());

        let mut new = HashMap::new();
        new.insert("json".to_string(), "userId".to_string());

        let (merged, conflicts) = merge_metadata(&old, &new);

        assert_eq!(merged.len(), 1);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(merged.get("json"), Some(&"userId".to_string())); // New wins

        let conflict = &conflicts[0];
        assert_eq!(conflict.0, "json");
        assert_eq!(conflict.1, "id");
        assert_eq!(conflict.2, "userId");
    }
}
