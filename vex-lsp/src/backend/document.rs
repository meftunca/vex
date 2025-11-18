// LSP Document management (open, change, close)

use std::sync::Arc;
use tower_lsp::lsp_types::*;

use super::VexBackend;

impl VexBackend {
    /// Clone backend for async tasks (Arc cloning is cheap)
    pub fn clone_backend(&self) -> Self {
        Self {
            client: self.client.clone(),
            document_cache: Arc::clone(&self.document_cache),
            documents: Arc::clone(&self.documents),
            ast_cache: Arc::clone(&self.ast_cache),
            debounce_tasks: Arc::clone(&self.debounce_tasks),
            module_resolver: Arc::clone(&self.module_resolver),
            workspace_root: self.workspace_root.clone(),
        }
    }

    pub async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        let text = Arc::new(params.text_document.text.clone());
        let version = params.text_document.version;

        tracing::info!("Document opened: {}", uri);

        // Store document (Arc for cheap cloning)
        self.documents.insert(uri.clone(), Arc::clone(&text));

        // Parse and send diagnostics (with version tracking)
        let diagnostics = self.parse_and_diagnose(&uri, &text, version).await;
        self.publish_diagnostics(params.text_document.uri, diagnostics, Some(version))
            .await;
    }

    pub async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        let version = params.text_document.version;

        if let Some(change) = params.content_changes.first() {
            let text = Arc::new(change.text.clone());

            tracing::info!("Document changed: {} (version {})", uri, version);

            // Update document (Arc for cheap cloning)
            self.documents.insert(uri.clone(), Arc::clone(&text));

            // Cancel existing debounce task if any
            if let Some(task) = self.debounce_tasks.write().await.remove(&uri) {
                task.abort();
            }

            // Spawn debounced parsing task (300ms delay prevents UI freezing)
            let backend = self.clone_backend();
            let uri_clone = uri.clone();
            let text_clone = Arc::clone(&text);
            let doc_uri = params.text_document.uri.clone();

            let task = tokio::spawn(async move {
                // Wait 300ms for more edits
                tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

                // Parse in background thread with timeout protection (5s max)
                let parse_future = backend.parse_and_diagnose(&uri_clone, &text_clone, version);
                match tokio::time::timeout(tokio::time::Duration::from_secs(5), parse_future).await
                {
                    Ok(diagnostics) => {
                        backend
                            .publish_diagnostics(doc_uri, diagnostics, Some(version))
                            .await;
                    }
                    Err(_) => {
                        tracing::error!("Parse timeout for {} (version {})", uri_clone, version);
                    }
                }
            });

            self.debounce_tasks.write().await.insert(uri, task);
        }
    }

    pub async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        tracing::info!("Document closed: {}", uri);

        // Cancel debounce task if exists (with timeout)
        let task_removed = {
            let mut tasks = self.debounce_tasks.write().await;
            tasks.remove(&uri)
        };

        if let Some(task) = task_removed {
            task.abort();
            // Wait briefly for task to actually abort
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        // Remove from all caches
        self.documents.remove(&uri);
        self.ast_cache.remove(&uri);
        self.document_cache.remove(&uri);
    }
}
