//! This module contains the implementation of the `type_definition` language feature.
use tower_lsp::lsp_types::*;

use crate::backend::{language_features::helpers::*, VexBackend};

impl VexBackend {
    pub async fn type_definition(
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

            // Get the token at cursor position (supports operator overload "op+" style names)
            let word = get_token_at_position(&text, position);
        if word.is_empty() {
            return Ok(None);
        }

        // Search for type definitions (struct, enum, trait) in the current file
        let mut locations = Vec::new();

        for (line_idx, line) in text.lines().enumerate() {
            let trimmed = line.trim();

            // Check for struct definitions
            if trimmed.starts_with("struct ") && trimmed.contains(&word) {
                if let Some(name_start) = trimmed.find("struct ") {
                    let name_end = trimmed[name_start + 7..]
                        .find(' ')
                        .unwrap_or(trimmed.len() - name_start - 7);
                    let struct_name = &trimmed[name_start + 7..name_start + 7 + name_end];
                    if struct_name == word {
                        locations.push(Location {
                            uri: params
                                .text_document_position_params
                                .text_document
                                .uri
                                .clone(),
                            range: Range {
                                start: Position {
                                    line: line_idx as u32,
                                    character: (name_start + 7) as u32,
                                },
                                end: Position {
                                    line: line_idx as u32,
                                    character: (name_start + 7 + name_end) as u32,
                                },
                            },
                        });
                    }
                }
            }
            // Check for enum definitions
            else if trimmed.starts_with("enum ") && trimmed.contains(&word) {
                if let Some(name_start) = trimmed.find("enum ") {
                    let name_end = trimmed[name_start + 5..]
                        .find(' ')
                        .unwrap_or(trimmed.len() - name_start - 5);
                    let enum_name = &trimmed[name_start + 5..name_start + 5 + name_end];
                    if enum_name == word {
                        locations.push(Location {
                            uri: params
                                .text_document_position_params
                                .text_document
                                .uri
                                .clone(),
                            range: Range {
                                start: Position {
                                    line: line_idx as u32,
                                    character: (name_start + 5) as u32,
                                },
                                end: Position {
                                    line: line_idx as u32,
                                    character: (name_start + 5 + name_end) as u32,
                                },
                            },
                        });
                    }
                }
            }
            // Check for trait definitions
            else if trimmed.starts_with("trait ") && trimmed.contains(&word) {
                if let Some(name_start) = trimmed.find("trait ") {
                    let name_end = trimmed[name_start + 6..]
                        .find(' ')
                        .unwrap_or(trimmed.len() - name_start - 6);
                    let trait_name = &trimmed[name_start + 6..name_start + 6 + name_end];
                    if trait_name == word {
                        locations.push(Location {
                            uri: params
                                .text_document_position_params
                                .text_document
                                .uri
                                .clone(),
                            range: Range {
                                start: Position {
                                    line: line_idx as u32,
                                    character: (name_start + 6) as u32,
                                },
                                end: Position {
                                    line: line_idx as u32,
                                    character: (name_start + 6 + name_end) as u32,
                                },
                            },
                        });
                    }
                }
            }
        }

        if locations.is_empty() {
            Ok(None)
        } else {
            Ok(Some(GotoDefinitionResponse::Array(locations)))
        }
    }
}
