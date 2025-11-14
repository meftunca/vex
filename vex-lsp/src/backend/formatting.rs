// LSP Formatting features

use tower_lsp::lsp_types::*;

use super::VexBackend;

impl VexBackend {
    pub async fn formatting(
        &self,
        params: DocumentFormattingParams,
    ) -> tower_lsp::jsonrpc::Result<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri.to_string();

        // Get document text
        let text = match self.documents.get(&uri) {
            Some(doc) => doc.clone(),
            None => return Ok(None),
        };

        // Load config or use defaults
        let config = std::env::current_dir()
            .ok()
            .and_then(|dir| vex_formatter::Config::from_dir(&dir).ok())
            .unwrap_or_default();

        // Format using vex-formatter
        match vex_formatter::format_source(&text, &config) {
            Ok(formatted) => {
                // Return single edit replacing entire document
                let lines: Vec<&str> = text.lines().collect();
                let last_line = lines.len().saturating_sub(1);
                let last_char = lines.last().map(|l| l.len()).unwrap_or(0);

                Ok(Some(vec![TextEdit {
                    range: Range {
                        start: Position {
                            line: 0,
                            character: 0,
                        },
                        end: Position {
                            line: last_line as u32,
                            character: last_char as u32,
                        },
                    },
                    new_text: formatted,
                }]))
            }
            Err(e) => {
                tracing::error!("Formatting error: {}", e);
                Ok(None)
            }
        }
    }

    pub async fn range_formatting(
        &self,
        params: DocumentRangeFormattingParams,
    ) -> tower_lsp::jsonrpc::Result<Option<Vec<TextEdit>>> {
        // For now, just format the entire document
        // TODO: Implement proper range formatting
        self.formatting(DocumentFormattingParams {
            text_document: params.text_document,
            options: params.options,
            work_done_progress_params: params.work_done_progress_params,
        })
        .await
    }
}
