//! This module contains the implementation of the `references` language feature.
use tower_lsp::lsp_types::*;

use crate::backend::{language_features::helpers::*, VexBackend};

impl VexBackend {
    pub async fn references(
        &self,
        params: ReferenceParams,
    ) -> tower_lsp::jsonrpc::Result<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        // Get document text
        let text = match self.documents.get(&uri.to_string()) {
            Some(t) => t.clone(),
            None => return Ok(None),
        };

        // Get the token at cursor position (supports operator overload "op+" style names)
        let word = get_token_at_position(&text, position);
        // If dotted call, include receiver-based free function name (e.g., counter_new)
        let receiver = get_receiver_at_position(&text, position);
        if word.is_empty() {
            return Ok(None);
        }

        let mut locations = Vec::new();
        let lines: Vec<&str> = text.lines().collect();

        // Search for all occurrences of the word in the document
        for (line_idx, line) in lines.iter().enumerate() {
            let mut start_pos = 0;
            while let Some(col_idx) = line[start_pos..].find(&word) {
                let actual_col = start_pos + col_idx;

                // Check if this is a whole word match (not part of another word)
                let token_is_op = is_operator_token(&word);
                let before_ok = actual_col == 0 || {
                    let prev = line.chars().nth(actual_col - 1).unwrap_or(' ');
                    if token_is_op {
                        // For operator tokens, ensure previous char is not alnum or an op char
                        !(prev.is_alphanumeric() || is_op_char(prev))
                    } else {
                        !prev.is_alphanumeric()
                    }
                };
                let after_pos = actual_col + word.len();
                let after_ok = after_pos >= line.len() || {
                    let next = line.chars().nth(after_pos).unwrap_or(' ');
                    if token_is_op {
                        // For operator tokens, ensure next char is not an identifier or op char
                        !(next.is_alphanumeric() || is_op_char(next))
                    } else {
                        !next.is_alphanumeric()
                    }
                };

                if before_ok && after_ok {
                    locations.push(Location {
                        uri: uri.clone(),
                        range: Range {
                            start: Position {
                                line: line_idx as u32,
                                character: actual_col as u32,
                            },
                            end: Position {
                                line: line_idx as u32,
                                character: (actual_col + word.len()) as u32,
                            },
                        },
                    });
                }

                start_pos = actual_col + 1;
            }
        }

        // If receiver exists, look for free function names like 'counter_new' too
        if let Some(recv) = receiver {
            let free_fn = format!("{}_{}", recv.to_lowercase(), word);
            for (line_idx, line) in lines.iter().enumerate() {
                let mut start_pos = 0;
                while let Some(col_idx) = line[start_pos..].find(&free_fn) {
                    let actual_col = start_pos + col_idx;
                    locations.push(Location {
                        uri: uri.clone(),
                        range: Range {
                            start: Position {
                                line: line_idx as u32,
                                character: actual_col as u32,
                            },
                            end: Position {
                                line: line_idx as u32,
                                character: (actual_col + free_fn.len()) as u32,
                            },
                        },
                    });
                    start_pos = actual_col + 1;
                }
            }
        }

        if locations.is_empty() {
            Ok(None)
        } else {
            Ok(Some(locations))
        }
    }
}
