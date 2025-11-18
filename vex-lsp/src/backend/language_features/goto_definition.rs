//! This module contains the implementation of the `goto_definition` language feature.
use std::sync::Arc;
use tower_lsp::lsp_types::*;

use crate::backend::{language_features::helpers::*, VexBackend};

impl VexBackend {
    pub async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> tower_lsp::jsonrpc::Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let uri_str = uri.to_string();

        // Get document text
        let text = match self.documents.get(&uri_str) {
            Some(t) => Arc::clone(t.value()),
            None => return Ok(None),
        };

        // Get the token at cursor position (supports operator overload "op+" style names)
        let word = get_token_at_position(&text, position);
        // Detect a receiver if this is a dotted call: Counter.new()
        let receiver = get_receiver_at_position(&text, position);
        if word.is_empty() {
            return Ok(None);
        }

        // Check if cursor is on an import statement
        if let Some(import_location) = self.resolve_import_at_position(&uri, &text, position) {
            return Ok(Some(GotoDefinitionResponse::Scalar(import_location)));
        }

        // Get the AST
        let ast = match self.ast_cache.get(&uri_str) {
            Some(ast) => Arc::clone(ast.value()),
            None => return Ok(None),
        };

        // Find definition location by searching for the symbol (local)
        if let Some(location) = self.find_definition_location(&ast, &word, &text) {
            return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                uri: uri.clone(),
                range: location,
            })));
        }

        // If not found locally, search workspace ASTs
        if let Some((uri_str, range)) = self.find_definition_location_workspace(&word) {
            return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                uri: Url::parse(&uri_str).unwrap_or(uri.clone()),
                range,
            })));
        }

        // If we have a receiver and the token is a method name, try method resolution patterns.
        if let Some(recv) = receiver {
            // First, check for inline methods in the struct declaration
            if let Some(location) = self.find_method_definition_in_struct(&ast, &recv, &word, &text)
            {
                return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                    uri: uri.clone(),
                    range: location,
                })));
            }

            // Next, check trait impls or external method patterns (receiver_method)
            let free_fn = format!("fn {}_{}", recv.to_lowercase(), word);
            if let Some(location) = find_pattern_in_source(&text, &free_fn) {
                return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                    uri: uri.clone(),
                    range: location,
                })));
            }
        }

        Ok(None)
    }

    pub fn find_definition_location(
        &self,
        ast: &vex_ast::Program,
        word: &str,
        text: &str,
    ) -> Option<Range> {
        // Simple implementation: search for function/struct/enum definitions
        for item in &ast.items {
            match item {
                vex_ast::Item::Function(func) if func.name == word => {
                    return find_pattern_in_source(text, &format!("fn {}", word));
                }
                vex_ast::Item::Struct(s) if s.name == word => {
                    return find_pattern_in_source(text, &format!("struct {}", word));
                }
                // Also check inline struct methods (e.g., fn op+(...))
                vex_ast::Item::Struct(s) => {
                    for m in &s.methods {
                        if m.name == word {
                            return find_pattern_in_source(text, &format!("fn {}", word));
                        }
                    }
                }
                vex_ast::Item::Enum(e) if e.name == word => {
                    return find_pattern_in_source(text, &format!("enum {}", word));
                }
                // Inline trait impls (impl Trait for Type { fn op+() {} })
                vex_ast::Item::TraitImpl(impl_) => {
                    for m in &impl_.methods {
                        if m.name == word {
                            return find_pattern_in_source(text, &format!("fn {}", word));
                        }
                    }
                }
                vex_ast::Item::Const(c) if c.name == word => {
                    return find_pattern_in_source(text, &format!("const {}", word));
                }
                vex_ast::Item::Contract(contract) if contract.name == word => {
                    return find_pattern_in_source(text, &format!("contract {}", word));
                }
                // Also check contract methods
                vex_ast::Item::Contract(contract) => {
                    for m in &contract.methods {
                        if m.name == word {
                            return find_pattern_in_source(text, &format!("fn {}", word));
                        }
                    }
                }
                vex_ast::Item::TypeAlias(alias) if alias.name == word => {
                    return find_pattern_in_source(text, &format!("type {}", word));
                }
                vex_ast::Item::Policy(policy) if policy.name == word => {
                    return find_pattern_in_source(text, &format!("policy {}", word));
                }
                _ => {}
            }
        }
        None
    }

    pub fn find_definition_location_workspace(&self, word: &str) -> Option<(String, Range)> {
        for entry in self.ast_cache.iter() {
            let uri = entry.key();
            let ast = entry.value();
            for item in &ast.items {
                match item {
                    vex_ast::Item::Function(func) if func.name == word => {
                        if let Some(text) = self.documents.get(uri) {
                            if let Some(range) =
                                find_pattern_in_source(&text, &format!("fn {}", word))
                            {
                                return Some((uri.clone(), range));
                            }
                        } else {
                            return Some((
                                uri.clone(),
                                Range {
                                    start: Position {
                                        line: 0,
                                        character: 0,
                                    },
                                    end: Position {
                                        line: 0,
                                        character: 1,
                                    },
                                },
                            ));
                        }
                    }
                    vex_ast::Item::Struct(s) if s.name == word => {
                        if let Some(text) = self.documents.get(uri) {
                            if let Some(range) =
                                find_pattern_in_source(&text, &format!("struct {}", word))
                            {
                                return Some((uri.clone(), range));
                            }
                        }
                    }
                    vex_ast::Item::Enum(e) if e.name == word => {
                        if let Some(text) = self.documents.get(uri) {
                            if let Some(range) =
                                find_pattern_in_source(&text, &format!("enum {}", word))
                            {
                                return Some((uri.clone(), range));
                            }
                        }
                    }
                    vex_ast::Item::Const(c) if c.name == word => {
                        if let Some(text) = self.documents.get(uri) {
                            if let Some(range) =
                                find_pattern_in_source(&text, &format!("const {}", word))
                            {
                                return Some((uri.clone(), range));
                            }
                        }
                    }
                    vex_ast::Item::Contract(contract) if contract.name == word => {
                        if let Some(text) = self.documents.get(uri) {
                            if let Some(range) =
                                find_pattern_in_source(&text, &format!("contract {}", word))
                            {
                                return Some((uri.clone(), range));
                            }
                        }
                    }
                    vex_ast::Item::TypeAlias(alias) if alias.name == word => {
                        if let Some(text) = self.documents.get(uri) {
                            if let Some(range) =
                                find_pattern_in_source(&text, &format!("type {}", word))
                            {
                                return Some((uri.clone(), range));
                            }
                        }
                    }
                    vex_ast::Item::Policy(policy) if policy.name == word => {
                        if let Some(text) = self.documents.get(uri) {
                            if let Some(range) =
                                find_pattern_in_source(&text, &format!("policy {}", word))
                            {
                                return Some((uri.clone(), range));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        None
    }

    pub fn find_method_definition_in_struct(
        &self,
        ast: &vex_ast::Program,
        struct_name: &str,
        method_name: &str,
        text: &str,
    ) -> Option<Range> {
        for item in &ast.items {
            if let vex_ast::Item::Struct(s) = item {
                if s.name == struct_name {
                    for m in &s.methods {
                        if m.name == method_name {
                            return find_pattern_in_source(text, &format!("fn {}", method_name));
                        }
                    }
                }
            }
        }
        None
    }

    /// Resolve import statement at cursor position
    fn resolve_import_at_position(
        &self,
        uri: &tower_lsp::lsp_types::Url,
        text: &str,
        position: Position,
    ) -> Option<Location> {
        let line_idx = position.line as usize;
        let lines: Vec<&str> = text.lines().collect();

        if line_idx >= lines.len() {
            return None;
        }

        let line = lines[line_idx];

        // Check if line contains import statement
        if !line.trim().starts_with("import ") && !line.trim().starts_with("from ") {
            return None;
        }

        // Extract import path from line
        // Examples: "import std.io", "from std.collections import Vec"
        let import_path = if line.contains("import ") {
            line.split("import ")
                .nth(1)?
                .split_whitespace()
                .next()?
                .trim_end_matches(';')
        } else {
            return None;
        };

        // Get current file path for relative imports
        let current_file = uri.to_file_path().ok();

        // Resolve import path to file
        let resolved_path = self
            .module_resolver
            .resolve_import(import_path, current_file.as_deref())?;

        // Convert to file URI
        let file_uri = tower_lsp::lsp_types::Url::from_file_path(&resolved_path).ok()?;

        Some(Location {
            uri: file_uri,
            range: Range::default(), // Jump to start of file
        })
    }
}
