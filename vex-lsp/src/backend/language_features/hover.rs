//! This module contains the implementation of the `hover` language feature.
use tower_lsp::lsp_types::*;

use crate::backend::{language_features::helpers::*, VexBackend};

impl VexBackend {
    pub async fn hover(&self, params: HoverParams) -> tower_lsp::jsonrpc::Result<Option<Hover>> {
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .to_string();
        let position = params.text_document_position_params.position;

        // Get the AST for this document
        let _ast = match self.ast_cache.get(&uri) {
            Some(ast) => ast.clone(),
            None => return Ok(None), // Document not parsed yet
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

        // Simple hover: show the word that was found
        Ok(Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("**Symbol**: `{}`\n\n*Vex Language*", word),
            }),
            range: None,
        }))
    }
}
