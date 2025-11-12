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

        // Get the word at cursor position
        let word = get_word_at_position(&text, position);
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
                let before_ok = actual_col == 0
                    || !line
                        .chars()
                        .nth(actual_col - 1)
                        .unwrap_or(' ')
                        .is_alphanumeric();
                let after_pos = actual_col + word.len();
                let after_ok = after_pos >= line.len()
                    || !line.chars().nth(after_pos).unwrap_or(' ').is_alphanumeric();

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

        if locations.is_empty() {
            Ok(None)
        } else {
            Ok(Some(locations))
        }
    }
}
