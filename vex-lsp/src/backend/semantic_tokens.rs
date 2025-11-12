// LSP Semantic Tokens for advanced syntax highlighting

use tower_lsp::lsp_types::*;

use super::VexBackend;

impl VexBackend {
    pub async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> tower_lsp::jsonrpc::Result<Option<SemanticTokensResult>> {
        let uri = params.text_document.uri.to_string();

        // Get document text
        let text = match self.documents.get(&uri) {
            Some(doc) => doc.clone(),
            None => return Ok(None),
        };

        // Get AST for semantic analysis
        let ast = match self.ast_cache.get(&uri) {
            Some(ast) => ast.clone(),
            None => return Ok(None),
        };

        // Generate semantic tokens
        let mut tokens = Vec::new();
        self.generate_semantic_tokens(&ast, &text, &mut tokens);

        Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data: tokens,
        })))
    }

    fn generate_semantic_tokens(
        &self,
        ast: &vex_ast::Program,
        source: &str,
        tokens: &mut Vec<SemanticToken>,
    ) {
        // Process each item in the AST
        for item in &ast.items {
            match item {
                vex_ast::Item::Function(func) => {
                    self.add_function_tokens(func, source, tokens);
                }
                vex_ast::Item::Struct(s) => {
                    self.add_struct_tokens(s, source, tokens);
                }
                vex_ast::Item::Enum(e) => {
                    self.add_enum_tokens(e, source, tokens);
                }
                vex_ast::Item::Const(c) => {
                    self.add_const_tokens(c, source, tokens);
                }
                vex_ast::Item::Contract(c) => {
                    self.add_trait_tokens(c, source, tokens);
                }
                vex_ast::Item::TraitImpl(impl_block) => {
                    self.add_trait_impl_tokens(impl_block, source, tokens);
                }
                vex_ast::Item::TypeAlias(alias) => {
                    self.add_type_alias_tokens(alias, source, tokens);
                }
                vex_ast::Item::Export(export) => {
                    self.add_export_tokens(export, source, tokens);
                }
                vex_ast::Item::Policy(_) => {
                    // TODO: Add policy token handling
                }
                vex_ast::Item::ExternBlock(_) => {
                    // TODO: Add extern block token handling
                }
            }
        }
    }

    fn add_function_tokens(
        &self,
        func: &vex_ast::Function,
        source: &str,
        tokens: &mut Vec<SemanticToken>,
    ) {
        // Function name
        if let Some(range) = self.find_token_range(source, &func.name) {
            tokens.push(SemanticToken {
                delta_line: range.start.line as u32,
                delta_start: range.start.character as u32,
                length: (range.end.character - range.start.character) as u32,
                token_type: self.token_type_to_index(SemanticTokenType::FUNCTION),
                token_modifiers_bitset: self
                    .token_modifiers_to_bitset(&[SemanticTokenModifier::DECLARATION]),
            });
        }

        // Parameters
        for param in &func.params {
            if let Some(range) = self.find_token_range(source, &param.name) {
                tokens.push(SemanticToken {
                    delta_line: range.start.line as u32,
                    delta_start: range.start.character as u32,
                    length: (range.end.character - range.start.character) as u32,
                    token_type: self.token_type_to_index(SemanticTokenType::PARAMETER),
                    token_modifiers_bitset: self
                        .token_modifiers_to_bitset(&[SemanticTokenModifier::DECLARATION]),
                });
            }
        }
    }

    fn add_struct_tokens(
        &self,
        s: &vex_ast::Struct,
        source: &str,
        tokens: &mut Vec<SemanticToken>,
    ) {
        // Struct name
        if let Some(range) = self.find_token_range(source, &s.name) {
            tokens.push(SemanticToken {
                delta_line: range.start.line as u32,
                delta_start: range.start.character as u32,
                length: (range.end.character - range.start.character) as u32,
                token_type: self.token_type_to_index(SemanticTokenType::STRUCT),
                token_modifiers_bitset: self
                    .token_modifiers_to_bitset(&[SemanticTokenModifier::DECLARATION]),
            });
        }

        // Fields
        for field in &s.fields {
            if let Some(range) = self.find_token_range(source, &field.name) {
                tokens.push(SemanticToken {
                    delta_line: range.start.line as u32,
                    delta_start: range.start.character as u32,
                    length: (range.end.character - range.start.character) as u32,
                    token_type: self.token_type_to_index(SemanticTokenType::PROPERTY),
                    token_modifiers_bitset: self
                        .token_modifiers_to_bitset(&[SemanticTokenModifier::DECLARATION]),
                });
            }
        }
    }

    fn add_enum_tokens(&self, e: &vex_ast::Enum, source: &str, tokens: &mut Vec<SemanticToken>) {
        // Enum name
        if let Some(range) = self.find_token_range(source, &e.name) {
            tokens.push(SemanticToken {
                delta_line: range.start.line as u32,
                delta_start: range.start.character as u32,
                length: (range.end.character - range.start.character) as u32,
                token_type: self.token_type_to_index(SemanticTokenType::ENUM),
                token_modifiers_bitset: self
                    .token_modifiers_to_bitset(&[SemanticTokenModifier::DECLARATION]),
            });
        }

        // Variants
        for variant in &e.variants {
            if let Some(range) = self.find_token_range(source, &variant.name) {
                tokens.push(SemanticToken {
                    delta_line: range.start.line as u32,
                    delta_start: range.start.character as u32,
                    length: (range.end.character - range.start.character) as u32,
                    token_type: self.token_type_to_index(SemanticTokenType::ENUM_MEMBER),
                    token_modifiers_bitset: self
                        .token_modifiers_to_bitset(&[SemanticTokenModifier::DECLARATION]),
                });
            }
        }
    }

    fn add_const_tokens(&self, c: &vex_ast::Const, source: &str, tokens: &mut Vec<SemanticToken>) {
        // Const name
        if let Some(range) = self.find_token_range(source, &c.name) {
            tokens.push(SemanticToken {
                delta_line: range.start.line as u32,
                delta_start: range.start.character as u32,
                length: (range.end.character - range.start.character) as u32,
                token_type: self.token_type_to_index(SemanticTokenType::VARIABLE),
                token_modifiers_bitset: self.token_modifiers_to_bitset(&[
                    SemanticTokenModifier::DECLARATION,
                    SemanticTokenModifier::READONLY,
                ]),
            });
        }
    }

    fn add_trait_tokens(&self, t: &vex_ast::Trait, source: &str, tokens: &mut Vec<SemanticToken>) {
        // Trait name
        if let Some(range) = self.find_token_range(source, &t.name) {
            tokens.push(SemanticToken {
                delta_line: range.start.line as u32,
                delta_start: range.start.character as u32,
                length: (range.end.character - range.start.character) as u32,
                token_type: self.token_type_to_index(SemanticTokenType::INTERFACE),
                token_modifiers_bitset: self
                    .token_modifiers_to_bitset(&[SemanticTokenModifier::DECLARATION]),
            });
        }

        // Trait methods
        for method in &t.methods {
            if let Some(range) = self.find_token_range(source, &method.name) {
                tokens.push(SemanticToken {
                    delta_line: range.start.line as u32,
                    delta_start: range.start.character as u32,
                    length: (range.end.character - range.start.character) as u32,
                    token_type: self.token_type_to_index(SemanticTokenType::METHOD),
                    token_modifiers_bitset: self
                        .token_modifiers_to_bitset(&[SemanticTokenModifier::DECLARATION]),
                });
            }
        }
    }

    fn add_trait_impl_tokens(
        &self,
        _impl_block: &vex_ast::TraitImpl,
        _source: &str,
        _tokens: &mut Vec<SemanticToken>,
    ) {
        // TODO: Implement trait impl block semantic tokens
    }

    fn add_import_tokens(
        &self,
        _import: &vex_ast::Import,
        _source: &str,
        _tokens: &mut Vec<SemanticToken>,
    ) {
        // TODO: Implement import semantic tokens
    }

    fn add_type_alias_tokens(
        &self,
        _alias: &vex_ast::TypeAlias,
        _source: &str,
        _tokens: &mut Vec<SemanticToken>,
    ) {
        // TODO: Implement type alias semantic tokens
    }

    fn add_export_tokens(
        &self,
        _export: &vex_ast::Export,
        _source: &str,
        _tokens: &mut Vec<SemanticToken>,
    ) {
        // TODO: Implement export semantic tokens
    }

    fn find_token_range(&self, source: &str, token: &str) -> Option<Range> {
        // Simple token range finder - finds first occurrence
        let lines: Vec<&str> = source.lines().collect();
        for (line_idx, line) in lines.iter().enumerate() {
            if let Some(char_idx) = line.find(token) {
                return Some(Range {
                    start: Position {
                        line: line_idx as u32,
                        character: char_idx as u32,
                    },
                    end: Position {
                        line: line_idx as u32,
                        character: (char_idx + token.len()) as u32,
                    },
                });
            }
        }
        None
    }

    fn token_type_to_index(&self, token_type: SemanticTokenType) -> u32 {
        // Use string comparison instead of pattern matching on constants
        if token_type == SemanticTokenType::NAMESPACE {
            0
        } else if token_type == SemanticTokenType::TYPE {
            1
        } else if token_type == SemanticTokenType::CLASS {
            2
        } else if token_type == SemanticTokenType::ENUM {
            3
        } else if token_type == SemanticTokenType::INTERFACE {
            4
        } else if token_type == SemanticTokenType::STRUCT {
            5
        } else if token_type == SemanticTokenType::TYPE_PARAMETER {
            6
        } else if token_type == SemanticTokenType::PARAMETER {
            7
        } else if token_type == SemanticTokenType::VARIABLE {
            8
        } else if token_type == SemanticTokenType::PROPERTY {
            9
        } else if token_type == SemanticTokenType::ENUM_MEMBER {
            10
        } else if token_type == SemanticTokenType::EVENT {
            11
        } else if token_type == SemanticTokenType::FUNCTION {
            12
        } else if token_type == SemanticTokenType::METHOD {
            13
        } else if token_type == SemanticTokenType::MACRO {
            14
        } else if token_type == SemanticTokenType::KEYWORD {
            15
        } else if token_type == SemanticTokenType::MODIFIER {
            16
        } else if token_type == SemanticTokenType::COMMENT {
            17
        } else if token_type == SemanticTokenType::STRING {
            18
        } else if token_type == SemanticTokenType::NUMBER {
            19
        } else if token_type == SemanticTokenType::REGEXP {
            20
        } else if token_type == SemanticTokenType::OPERATOR {
            21
        } else {
            0 // Default to namespace
        }
    }

    fn token_modifiers_to_bitset(&self, modifiers: &[SemanticTokenModifier]) -> u32 {
        let mut bitset = 0u32;
        for modifier in modifiers {
            let bit = if *modifier == SemanticTokenModifier::DECLARATION {
                0
            } else if *modifier == SemanticTokenModifier::DEFINITION {
                1
            } else if *modifier == SemanticTokenModifier::READONLY {
                2
            } else if *modifier == SemanticTokenModifier::STATIC {
                3
            } else if *modifier == SemanticTokenModifier::DEPRECATED {
                4
            } else if *modifier == SemanticTokenModifier::ABSTRACT {
                5
            } else if *modifier == SemanticTokenModifier::ASYNC {
                6
            } else if *modifier == SemanticTokenModifier::MODIFICATION {
                7
            } else if *modifier == SemanticTokenModifier::DOCUMENTATION {
                8
            } else if *modifier == SemanticTokenModifier::DEFAULT_LIBRARY {
                9
            } else {
                continue;
            };
            bitset |= 1 << bit;
        }
        bitset
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_tokens_basic() {
        // Test that semantic token types are properly defined
        assert_eq!(SemanticTokenType::FUNCTION.as_str(), "function");
        assert_eq!(SemanticTokenType::STRUCT.as_str(), "struct");
        assert_eq!(SemanticTokenType::TYPE.as_str(), "type");

        // Test that semantic token modifiers are properly defined
        assert_eq!(SemanticTokenModifier::DECLARATION.as_str(), "declaration");
        assert_eq!(SemanticTokenModifier::DEFINITION.as_str(), "definition");
    }
}
