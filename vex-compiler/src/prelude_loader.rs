/// Prelude loader - Parses and prepares embedded Layer 1 Vex code
///
/// This module handles loading the embedded prelude modules from the
/// compiler binary and parsing them into AST nodes for injection into
/// user programs.
use crate::prelude;
use vex_ast::Program;
use vex_diagnostics::SpanMap;
use vex_parser::Parser;

/// Error type for prelude loading failures
#[derive(Debug)]
pub enum PreludeLoadError {
    ParseError { module_name: String, error: String },
    EmptyPrelude,
}

impl std::fmt::Display for PreludeLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PreludeLoadError::ParseError { module_name, error } => {
                write!(
                    f,
                    "Failed to parse prelude module '{}': {}",
                    module_name, error
                )
            }
            PreludeLoadError::EmptyPrelude => {
                write!(f, "Prelude is empty - no modules found")
            }
        }
    }
}

impl std::error::Error for PreludeLoadError {}

/// Load and parse all embedded prelude modules
///
/// Returns a Program containing all prelude items in the correct initialization order:
/// 1. core::lib - Core traits (Display, Clone, Debug, Default)
/// 2. core::ops - Operator traits
/// 3. core::builtin_contracts - Type contracts
/// 4. core::option - Option<T> type
/// 5. core::result - Result<T, E> type
/// 6. core::vec - Vec<T> type
/// 7. core::box - Box<T> type
///
/// # Errors
/// Returns PreludeLoadError if any prelude module fails to parse
pub fn load_embedded_prelude() -> Result<Program, PreludeLoadError> {
    let modules = prelude::get_embedded_prelude();

    if modules.is_empty() {
        return Err(PreludeLoadError::EmptyPrelude);
    }

    let mut combined_items = Vec::new();
    let combined_span_map = SpanMap::new();

    for (module_name, source_code) in modules {
        // Parse each prelude module
        let mut parser = Parser::new_with_file(module_name, source_code).map_err(|e| {
            PreludeLoadError::ParseError {
                module_name: module_name.to_string(),
                error: format!("{:?}", e),
            }
        })?;

        let program = parser.parse().map_err(|errors| {
            let error_msg = format!("{:?}", errors);

            PreludeLoadError::ParseError {
                module_name: module_name.to_string(),
                error: error_msg,
            }
        })?;

        // Collect all items from this module
        combined_items.extend(program.items);

        // Merge span maps (for better error reporting)
        // Note: SpanMap doesn't have public API for merging, so we skip this
        // The prelude should parse without errors anyway
    }

    Ok(Program {
        items: combined_items,
        imports: Vec::new(), // Prelude has no imports
    })
}

/// Inject prelude into a user program
///
/// Prepends all prelude items to the beginning of the user's program,
/// making core types and traits available without explicit imports.
///
/// # Arguments
/// * `user_program` - The user's parsed program
///
/// # Returns
/// A new Program with prelude items prepended
pub fn inject_prelude_into_program(user_program: Program) -> Result<Program, PreludeLoadError> {
    let prelude = load_embedded_prelude()?;

    let mut items = prelude.items;
    items.extend(user_program.items);

    Ok(Program {
        items,
        imports: user_program.imports, // Keep user's imports
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_embedded_prelude() {
        let result = load_embedded_prelude();
        assert!(result.is_ok(), "Prelude should load successfully");

        let program = result.unwrap();
        assert!(!program.items.is_empty(), "Prelude should have items");
    }

    #[test]
    fn test_inject_prelude() {
        let user_program = Program {
            items: vec![],
            imports: vec![],
        };

        let result = inject_prelude_into_program(user_program);
        assert!(result.is_ok(), "Prelude injection should succeed");

        let combined = result.unwrap();
        assert!(
            !combined.items.is_empty(),
            "Combined program should have prelude items"
        );
    }
}
