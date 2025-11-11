// Core LSP Backend structures and initialization

use dashmap::DashMap;
use std::sync::Arc;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

// Simple document cache for incremental parsing
#[derive(Debug)]
pub struct DocumentCache {
    // Placeholder - implement incremental parsing later
}

impl DocumentCache {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&self, _uri: &str, _text: String, _version: i32) -> CachedDocument {
        // Placeholder implementation
        CachedDocument {
            parse_errors: Vec::new(),
            ast: None,
        }
    }

    pub fn remove(&self, _uri: &str) {
        // Placeholder implementation
    }
}

#[derive(Debug)]
pub struct CachedDocument {
    pub parse_errors: Vec<vex_diagnostics::Diagnostic>,
    pub ast: Option<vex_ast::Program>,
}

pub struct VexBackend {
    pub client: Client,
    /// Document cache with incremental parsing
    pub document_cache: Arc<DocumentCache>,
    /// Legacy: Document text cache (kept for compatibility)
    pub documents: Arc<DashMap<String, String>>,
    /// Legacy: Parsed AST cache (migrating to DocumentCache)
    pub ast_cache: Arc<DashMap<String, vex_ast::Program>>,
}

impl VexBackend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            document_cache: Arc::new(DocumentCache::new()),
            documents: Arc::new(DashMap::new()),
            ast_cache: Arc::new(DashMap::new()),
        }
    }
}
