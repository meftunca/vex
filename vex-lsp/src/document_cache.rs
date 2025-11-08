// Document cache for incremental parsing
// Stores parsed AST per document to avoid re-parsing on every keystroke

use dashmap::DashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tower_lsp::lsp_types::Url;
use vex_ast::Program;
use vex_diagnostics::Diagnostic;

/// Cached document information
#[derive(Clone)]
pub struct CachedDocument {
    /// Source code text
    pub text: String,
    /// Parsed AST (None if parse failed)
    pub ast: Option<Program>,
    /// Parse errors as diagnostics (if any)
    pub parse_errors: Vec<Diagnostic>,
    /// Timestamp of last parse
    pub timestamp: u64,
    /// Document version (increments on each change)
    pub version: i32,
}

impl CachedDocument {
    pub fn new(text: String, version: i32) -> Self {
        Self {
            text,
            ast: None,
            parse_errors: Vec::new(),
            timestamp: current_timestamp(),
            version,
        }
    }

    /// Parse the document and update cache
    pub fn parse(&mut self, uri: &str) {
        let filename = uri_to_filename(uri);

        match vex_parser::Parser::new_with_file(&filename, &self.text) {
            Ok(mut parser) => {
                // Use error recovery to collect all errors
                let (program_opt, diagnostics) = parser.parse_with_recovery();

                if !diagnostics.is_empty() {
                    // Store all diagnostics (errors + warnings)
                    self.parse_errors = diagnostics;
                    self.ast = program_opt; // May be partial AST or None
                    self.timestamp = current_timestamp();
                } else if let Some(program) = program_opt {
                    // Success - no errors
                    self.ast = Some(program);
                    self.parse_errors.clear();
                    self.timestamp = current_timestamp();
                } else {
                    // No program, no errors (edge case)
                    self.ast = None;
                    self.parse_errors = vec![Diagnostic::error(
                        "E0001",
                        "Parse failed without diagnostic".to_string(),
                        vex_diagnostics::Span::unknown(),
                    )];
                    self.timestamp = current_timestamp();
                }
            }
            Err(e) => {
                self.ast = None;
                // Store lexer error as diagnostic
                if let Some(diag) = e.as_diagnostic() {
                    self.parse_errors = vec![diag.clone()];
                } else {
                    self.parse_errors = vec![Diagnostic::error(
                        "E0001",
                        format!("Lexer error: {:?}", e),
                        vex_diagnostics::Span::unknown(),
                    )];
                }
                self.timestamp = current_timestamp();
            }
        }
    }

    /// Check if document has valid AST
    pub fn has_ast(&self) -> bool {
        self.ast.is_some()
    }
}

/// Document cache manager
pub struct DocumentCache {
    cache: Arc<DashMap<String, CachedDocument>>,
}

impl DocumentCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
        }
    }

    /// Get or create cached document
    pub fn get_or_insert(&self, uri: &str, text: String, version: i32) -> CachedDocument {
        let key = uri.to_string();

        // Check if we have a cached version
        if let Some(cached) = self.cache.get(&key) {
            // If version matches and we have AST, return cached
            if cached.version == version && cached.has_ast() {
                return cached.clone();
            }
        }

        // Create new cached document
        let mut doc = CachedDocument::new(text, version);
        doc.parse(uri);

        // Store in cache
        self.cache.insert(key, doc.clone());

        doc
    }

    /// Update document (incremental change)
    pub fn update(&self, uri: &str, text: String, version: i32) -> CachedDocument {
        let key = uri.to_string();

        // Create new document and parse
        let mut doc = CachedDocument::new(text, version);
        doc.parse(uri);

        // Update cache
        self.cache.insert(key, doc.clone());

        doc
    }

    /// Get cached document (without parsing)
    pub fn get(&self, uri: &str) -> Option<CachedDocument> {
        self.cache.get(uri).map(|entry| entry.clone())
    }

    /// Remove document from cache
    pub fn remove(&self, uri: &str) {
        self.cache.remove(uri);
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let total = self.cache.len();
        let with_ast = self
            .cache
            .iter()
            .filter(|entry| entry.value().has_ast())
            .count();

        CacheStats {
            total_documents: total,
            parsed_documents: with_ast,
            failed_documents: total - with_ast,
        }
    }
}

impl Default for DocumentCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_documents: usize,
    pub parsed_documents: usize,
    pub failed_documents: usize,
}

/// Get current Unix timestamp in milliseconds
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

/// Extract filename from URI
fn uri_to_filename(uri: &str) -> String {
    if let Ok(url) = Url::parse(uri) {
        if let Some(path) = url.path_segments() {
            if let Some(last) = path.last() {
                return last.to_string();
            }
        }
    }
    "unknown.vx".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_cache() {
        let cache = DocumentCache::new();

        // Test insertion
        let code = "fn main(): i32 { return 0; }".to_string();
        let doc = cache.get_or_insert("file:///test.vx", code.clone(), 1);

        assert_eq!(doc.version, 1);
        assert!(doc.has_ast());
        assert!(doc.parse_errors.is_empty());

        // Test cache hit (same version)
        let doc2 = cache.get_or_insert("file:///test.vx", code.clone(), 1);
        assert_eq!(doc2.version, 1);
        assert_eq!(doc2.timestamp, doc.timestamp); // Same timestamp = cache hit

        // Test cache miss (different version)
        let doc3 = cache.get_or_insert("file:///test.vx", code, 2);
        assert_eq!(doc3.version, 2);
        assert!(doc3.timestamp > doc.timestamp); // Different timestamp = re-parsed
    }

    #[test]
    fn test_parse_error_caching() {
        let cache = DocumentCache::new();

        // Invalid code
        let code = "fn main( { bad syntax }".to_string();
        let doc = cache.get_or_insert("file:///bad.vx", code, 1);

        assert_eq!(doc.version, 1);
        assert!(!doc.has_ast());
        assert!(!doc.parse_errors.is_empty());
    }

    #[test]
    fn test_uri_to_filename() {
        assert_eq!(uri_to_filename("file:///home/user/test.vx"), "test.vx");
        assert_eq!(uri_to_filename("file:///C:/Users/test.vx"), "test.vx");
        assert_eq!(uri_to_filename("invalid"), "unknown.vx");
    }
}
