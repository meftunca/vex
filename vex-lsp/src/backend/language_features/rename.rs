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

        // Get the token at cursor position (old name), supports operator overload names like op+
        let old_name = get_token_at_position(&text, position);
        // Detect receiver for dotted calls (Type.method)
        let receiver = get_receiver_at_position(&text, position);
        let free_fn_old = receiver
            .as_ref()
            .map(|recv| format!("{}_{}", recv.to_lowercase(), old_name));
        let free_fn_new = receiver
            .as_ref()
            .map(|recv| format!("{}_{}", recv.to_lowercase(), new_name.clone()));
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
                let token_is_op = is_operator_token(&old_name);
                let before_ok = actual_col == 0 || {
                    let prev = line.chars().nth(actual_col - 1).unwrap_or(' ');
                    if token_is_op {
                        !(prev.is_alphanumeric() || is_op_char(prev))
                    } else {
                        !prev.is_alphanumeric()
                    }
                };
                let after_pos = actual_col + old_name.len();
                let after_ok = after_pos >= line.len() || {
                    let next = line.chars().nth(after_pos).unwrap_or(' ');
                    if token_is_op {
                        !(next.is_alphanumeric() || is_op_char(next))
                    } else {
                        !next.is_alphanumeric()
                    }
                };

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

        // Also replace free function names like 'counter_new' -> 'counter_create' if applicable
        if let (Some(free_old), Some(free_new)) = (free_fn_old, free_fn_new) {
            for (line_idx, line) in lines.iter().enumerate() {
                let mut start_pos = 0;
                while let Some(col_idx) = line[start_pos..].find(&free_old) {
                    let actual_col = start_pos + col_idx;
                    edits.push(TextEdit {
                        range: Range {
                            start: Position {
                                line: line_idx as u32,
                                character: actual_col as u32,
                            },
                            end: Position {
                                line: line_idx as u32,
                                character: (actual_col + free_old.len()) as u32,
                            },
                        },
                        new_text: free_new.clone(),
                    });
                    start_pos = actual_col + 1;
                }
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
