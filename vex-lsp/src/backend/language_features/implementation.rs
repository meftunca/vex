//! This module contains the implementation of the `implementation` language feature.
use tower_lsp::lsp_types::*;

use crate::backend::{language_features::helpers::*, VexBackend};

impl VexBackend {
    pub async fn implementation(
        &self,
        params: GotoDefinitionParams,
    ) -> tower_lsp::jsonrpc::Result<Option<GotoDefinitionResponse>> {
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .to_string();
        let position = params.text_document_position_params.position;

        // Get the AST for this document
        let _ast = match self.ast_cache.get(&uri) {
            Some(ast) => ast.clone(),
            None => return Ok(None),
        };

        // Get document text for word extraction
        let text = match self.documents.get(&uri) {
            Some(t) => t.clone(),
            None => return Ok(None),
        };

        // Get word at cursor position
        let word = get_word_at_position(&text, position);
        if word.is_empty() {
            return Ok(None);
        }

        // Search for implementations (impl blocks) in the current file
        let mut locations = Vec::new();

        for (line_idx, line) in text.lines().enumerate() {
            let trimmed = line.trim();

            // Check for impl blocks
            if trimmed.starts_with("impl ") && trimmed.contains(&word) {
                locations.push(Location {
                    uri: params
                        .text_document_position_params
                        .text_document
                        .uri
                        .clone(),
                    range: Range {
                        start: Position {
                            line: line_idx as u32,
                            character: 0,
                        },
                        end: Position {
                            line: line_idx as u32,
                            character: line.len() as u32,
                        },
                    },
                });
            }
            // Also check for trait implementations like "impl Trait for Type"
            else if trimmed.starts_with("impl ")
                && trimmed.contains(" for ")
                && trimmed.contains(&word)
            {
                locations.push(Location {
                    uri: params
                        .text_document_position_params
                        .text_document
                        .uri
                        .clone(),
                    range: Range {
                        start: Position {
                            line: line_idx as u32,
                            character: 0,
                        },
                        end: Position {
                            line: line_idx as u32,
                            character: line.len() as u32,
                        },
                    },
                });
            }
        }

        if locations.is_empty() {
            Ok(None)
        } else {
            Ok(Some(GotoDefinitionResponse::Array(locations)))
        }
    }
}
