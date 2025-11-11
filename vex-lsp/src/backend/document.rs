// LSP Document management (open, change, close)

use tower_lsp::lsp_types::*;

use super::VexBackend;

impl VexBackend {
    pub async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        let text = params.text_document.text.clone();
        let version = params.text_document.version;

        tracing::info!("Document opened: {}", uri);

        // Store document (legacy)
        self.documents.insert(uri.clone(), text.clone());

        // Parse and send diagnostics (with version tracking)
        let diagnostics = self.parse_and_diagnose(&uri, &text, version).await;
        self.publish_diagnostics(params.text_document.uri, diagnostics)
            .await;
    }

    pub async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        let version = params.text_document.version;

        if let Some(change) = params.content_changes.first() {
            let text = change.text.clone();

            tracing::info!("Document changed: {} (version {})", uri, version);

            // Update document (legacy)
            self.documents.insert(uri.clone(), text.clone());

            // Re-parse and send diagnostics (with version tracking)
            let diagnostics = self.parse_and_diagnose(&uri, &text, version).await;
            self.publish_diagnostics(params.text_document.uri, diagnostics)
                .await;
        }
    }

    pub async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        tracing::info!("Document closed: {}", uri);

        // Remove from all caches
        self.documents.remove(&uri);
        self.ast_cache.remove(&uri);
        self.document_cache.remove(&uri);
    }
}
