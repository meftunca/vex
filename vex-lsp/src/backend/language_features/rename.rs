//! This module contains the implementation of the `rename` language feature.
use tower_lsp::lsp_types::*;

use crate::backend::{language_features::helpers::*, VexBackend};

impl VexBackend {
    pub async fn rename(
        &self,
        params: RenameParams,
    ) -> tower_lsp::jsonrpc::Result<Option<WorkspaceEdit>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let new_name = params.new_name;

        // Get document text
        let text = match self.documents.get(&uri.to_string()) {
            Some(t) => t.clone(),
            None => return Ok(None),
        };

        // Get the word at cursor position (old name)
        let old_name = get_word_at_position(&text, position);
        if old_name.is_empty() {
            return Ok(None);
        }

        let mut edits = Vec::new();
        let lines: Vec<&str> = text.lines().collect();

        // Find all occurrences and create text edits
        for (line_idx, line) in lines.iter().enumerate() {
            let mut start_pos = 0;
            while let Some(col_idx) = line[start_pos..].find(&old_name) {
                let actual_col = start_pos + col_idx;

                // Check if this is a whole word match
                let before_ok = actual_col == 0
                    || !line
                        .chars()
                        .nth(actual_col - 1)
                        .unwrap_or(' ')
                        .is_alphanumeric();
                let after_pos = actual_col + old_name.len();
                let after_ok = after_pos >= line.len()
                    || !line.chars().nth(after_pos).unwrap_or(' ').is_alphanumeric();

                if before_ok && after_ok {
                    edits.push(TextEdit {
                        range: Range {
                            start: Position {
                                line: line_idx as u32,
                                character: actual_col as u32,
                            },
                            end: Position {
                                line: line_idx as u32,
                                character: (actual_col + old_name.len()) as u32,
                            },
                        },
                        new_text: new_name.clone(),
                    });
                }

                start_pos = actual_col + 1;
            }
        }

        if edits.is_empty() {
            return Ok(None);
        }

        // Create workspace edit
        let mut changes = std::collections::HashMap::new();
        changes.insert(uri, edits);

        Ok(Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        }))
    }
}
