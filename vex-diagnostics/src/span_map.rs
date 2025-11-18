/// Span tracking without modifying AST
///
/// Uses stable string IDs for AST nodes.
/// Parser generates unique IDs when creating expressions/statements.
/// Codegen looks up spans using these IDs.
use crate::Span;
use std::collections::HashMap;

/// Global span tracker for AST nodes
#[derive(Debug, Default, Clone)]
pub struct SpanMap {
    /// Maps string IDs to their source spans
    spans: HashMap<String, Span>,
    /// Counter for generating unique IDs
    next_id: usize,
}

impl SpanMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Generate a unique ID
    pub fn generate_id(&mut self) -> String {
        let id = format!("span_{}", self.next_id);
        self.next_id += 1;
        id
    }

    /// Record span with an ID
    pub fn record(&mut self, id: String, span: Span) {
        self.spans.insert(id, span);
    }

    /// Get span by ID
    pub fn get(&self, id: &str) -> Option<&Span> {
        self.spans.get(id)
    }

    /// Get span or return unknown
    pub fn get_or_unknown(&self, id: &str) -> Span {
        self.get(id).cloned().unwrap_or_else(Span::unknown)
    }

    /// Clear all stored spans
    pub fn clear(&mut self) {
        self.spans.clear();
        self.next_id = 0;
    }

    /// Get statistics
    pub fn stats(&self) -> usize {
        self.spans.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_map() {
        let mut map = SpanMap::new();
        let span = Span::new("test.vx".to_string(), 10, 5, 2);

        let id = map.generate_id();
        map.record(id.clone(), span.clone());
        assert_eq!(map.get(&id), Some(&span));
    }

    #[test]
    fn test_multiple_nodes() {
        let mut map = SpanMap::new();

        let span1 = Span::new("test.vx".to_string(), 1, 1, 5);
        let span2 = Span::new("test.vx".to_string(), 2, 1, 5);

        let id1 = map.generate_id();
        let id2 = map.generate_id();

        map.record(id1.clone(), span1.clone());
        map.record(id2.clone(), span2.clone());

        assert_eq!(map.get(&id1), Some(&span1));
        assert_eq!(map.get(&id2), Some(&span2));
    }
}
