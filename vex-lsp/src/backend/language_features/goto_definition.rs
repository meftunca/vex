//! This module contains the implementation of the `goto_definition` language feature.
use tower_lsp::lsp_types::*;

use crate::backend::{language_features::helpers::*, VexBackend};

impl VexBackend {
    pub async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> tower_lsp::jsonrpc::Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        // Get document text
        let text = match self.documents.get(&uri.to_string()) {
            Some(t) => t.clone(),
            None => return Ok(None),
        };

        // Get the word at cursor position
        let word = get_word_at_position(&text, position);
        if word.is_empty() {
            return Ok(None);
        }

        // Get the AST
        let ast = match self.ast_cache.get(&uri.to_string()) {
            Some(ast) => ast.clone(),
            None => return Ok(None),
        };

        // Find definition location by searching for the symbol
        if let Some(location) = self.find_definition_location(&ast, &word, &text) {
            return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                uri: uri.clone(),
                range: location,
            })));
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
                vex_ast::Item::Enum(e) if e.name == word => {
                    return find_pattern_in_source(text, &format!("enum {}", word));
                }
                vex_ast::Item::Const(c) if c.name == word => {
                    return find_pattern_in_source(text, &format!("const {}", word));
                }
                vex_ast::Item::Contract(contract) if contract.name == word => {
                    return find_pattern_in_source(text, &format!("contract {}", word));
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
}
