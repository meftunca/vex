// LSP Formatting features

use tower_lsp::lsp_types::*;

use super::VexBackend;

impl VexBackend {
    pub async fn formatting(
        &self,
        params: DocumentFormattingParams,
    ) -> tower_lsp::jsonrpc::Result<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri.to_string();

        let text = match self.documents.get(&uri) {
            Some(doc) => doc.clone(),
            None => {
                tracing::warn!("Formatting: document not found: {}", uri);
                return Ok(None);
            }
        };

        // Format the document
        use vex_formatter::{format_source, Config};

        let config = Config::default();
        match format_source(&text, &config) {
            Ok(formatted) => {
                if formatted == text.as_str() {
                    return Ok(None);
                }

                let end_line = text.lines().count() as u32;
                let end_char = text.lines().last().map(|l| l.len()).unwrap_or(0) as u32;

                Ok(Some(vec![TextEdit {
                    range: Range {
                        start: Position {
                            line: 0,
                            character: 0,
                        },
                        end: Position {
                            line: end_line,
                            character: end_char,
                        },
                    },
                    new_text: formatted,
                }]))
            }
            Err(e) => {
                tracing::error!("Formatting error for {}: {}", uri, e);
                Ok(None)
            }
        }
    }

    pub async fn range_formatting(
        &self,
        _params: DocumentRangeFormattingParams,
    ) -> tower_lsp::jsonrpc::Result<Option<Vec<TextEdit>>> {
        // Range formatting not supported
        Ok(None)
    }
}
