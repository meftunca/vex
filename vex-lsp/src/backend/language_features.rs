// LSP Language features (hover, completion, goto_definition, etc.)

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use super::VexBackend;

impl VexBackend {
    pub async fn hover(&self, params: HoverParams) -> tower_lsp::jsonrpc::Result<Option<Hover>> {
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .to_string();
        let position = params.text_document_position_params.position;

        // Get the AST for this document
        let ast = match self.ast_cache.get(&uri) {
            Some(ast) => ast.clone(),
            None => return Ok(None), // Document not parsed yet
        };

        // Create symbol resolver
        let mut resolver = // crate::symbol_resolver::SymbolResolver::new();
        resolver.extract_symbols(&ast);

        // Find symbol at cursor position (LSP uses 0-indexed lines)
        let line = (position.line + 1) as usize;
        let column = (position.character + 1) as usize;

        if let Some(symbol) = resolver.find_symbol_at(line, column) {
            // Format hover content based on symbol kind
            let mut content = String::new();

            // Add kind badge
            let kind_str = match symbol.kind {
                // crate::symbol_resolver::SymbolKind::Function => "function",
                // crate::symbol_resolver::SymbolKind::Variable => "variable",
                // crate::symbol_resolver::SymbolKind::Parameter => "parameter",
                // crate::symbol_resolver::SymbolKind::Struct => "struct",
                // crate::symbol_resolver::SymbolKind::Enum => "enum",
                // crate::symbol_resolver::SymbolKind::Trait => "trait",
                // crate::symbol_resolver::SymbolKind::Field => "field",
                // crate::symbol_resolver::SymbolKind::Method => "method",
            };

            content.push_str(&format!("**{}** `{}`\n\n", kind_str, symbol.name));

            // Add type information
            if let Some(type_info) = &symbol.type_info {
                content.push_str("```vex\n");
                content.push_str(type_info);
                content.push_str("\n```\n\n");
            }

            // Add documentation if available
            if let Some(doc) = &symbol.documentation {
                content.push_str("---\n\n");
                content.push_str(doc);
                content.push_str("\n");
            }

            return Ok(Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: content,
                }),
                range: None,
            }));
        }

        // Fallback: show generic hover info
        Ok(Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "*Vex Language*\n\nNo symbol information available at this position."
                    .to_string(),
            }),
            range: None,
        }))
    }

    pub async fn completion(&self, params: CompletionParams) -> tower_lsp::jsonrpc::Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let mut items = Vec::new();

        // Get document text
        let text = match self.documents.get(&uri.to_string()) {
            Some(t) => t.clone(),
            None => return Ok(None),
        };

        // Get context (what's before cursor)
        let context = self.get_completion_context(&text, position);

        // Keywords
        let keywords = vec![
            ("fn", CompletionItemKind::KEYWORD, "function declaration"),
            ("let", CompletionItemKind::KEYWORD, "immutable variable"),
            ("let!", CompletionItemKind::KEYWORD, "mutable variable"),
            ("const", CompletionItemKind::KEYWORD, "constant"),
            ("struct", CompletionItemKind::KEYWORD, "struct definition"),
            ("enum", CompletionItemKind::KEYWORD, "enum definition"),
            ("trait", CompletionItemKind::KEYWORD, "trait definition"),
            ("impl", CompletionItemKind::KEYWORD, "implementation block"),
            ("if", CompletionItemKind::KEYWORD, "conditional"),
            ("else", CompletionItemKind::KEYWORD, "else clause"),
            ("match", CompletionItemKind::KEYWORD, "pattern matching"),
            ("for", CompletionItemKind::KEYWORD, "for loop"),
            ("while", CompletionItemKind::KEYWORD, "while loop"),
            ("loop", CompletionItemKind::KEYWORD, "infinite loop"),
            ("return", CompletionItemKind::KEYWORD, "return statement"),
            ("break", CompletionItemKind::KEYWORD, "break loop"),
            ("continue", CompletionItemKind::KEYWORD, "continue loop"),
            ("defer", CompletionItemKind::KEYWORD, "defer statement"),
            ("import", CompletionItemKind::KEYWORD, "import module"),
            ("export", CompletionItemKind::KEYWORD, "export symbol"),
            (
                "extern",
                CompletionItemKind::KEYWORD,
                "external declaration",
            ),
            ("async", CompletionItemKind::KEYWORD, "async function"),
            ("await", CompletionItemKind::KEYWORD, "await expression"),
        ];

        // Built-in types
        let builtin_types = vec![
            ("i8", CompletionItemKind::STRUCT, "8-bit integer"),
            ("i16", CompletionItemKind::STRUCT, "16-bit integer"),
            ("i32", CompletionItemKind::STRUCT, "32-bit integer"),
            ("i64", CompletionItemKind::STRUCT, "64-bit integer"),
            ("u8", CompletionItemKind::STRUCT, "unsigned 8-bit integer"),
            ("u16", CompletionItemKind::STRUCT, "unsigned 16-bit integer"),
            ("u32", CompletionItemKind::STRUCT, "unsigned 32-bit integer"),
            ("u64", CompletionItemKind::STRUCT, "unsigned 64-bit integer"),
            ("f32", CompletionItemKind::STRUCT, "32-bit float"),
            ("f64", CompletionItemKind::STRUCT, "64-bit float"),
            ("bool", CompletionItemKind::STRUCT, "boolean"),
            ("string", CompletionItemKind::STRUCT, "string type"),
            ("Vec", CompletionItemKind::STRUCT, "vector collection"),
            ("Box", CompletionItemKind::STRUCT, "heap-allocated value"),
            ("Option", CompletionItemKind::ENUM, "optional value"),
            ("Result", CompletionItemKind::ENUM, "result type"),
            ("HashMap", CompletionItemKind::STRUCT, "hash map"),
        ];

        // Add keywords and types
        for (label, kind, doc) in keywords.iter().chain(builtin_types.iter()) {
            items.push(CompletionItem {
                label: label.to_string(),
                kind: Some(*kind),
                detail: Some(doc.to_string()),
                ..Default::default()
            });
        }

        // Get AST for context-aware suggestions
        if let Some(ast) = self.ast_cache.get(&uri.to_string()) {
            // Add functions from AST
            for item in &ast.items {
                match item {
                    vex_ast::Item::Function(func) => {
                        let params_str = func
                            .params
                            .iter()
                            .map(|p| format!("{}: {}", p.name, self.type_to_string(&p.ty)))
                            .collect::<Vec<_>>()
                            .join(", ");
                        let return_str = if let Some(ret) = &func.return_type {
                            self.type_to_string(ret)
                        } else {
                            "()".to_string()
                        };

                        items.push(CompletionItem {
                            label: func.name.clone(),
                            kind: Some(CompletionItemKind::FUNCTION),
                            detail: Some(format!(
                                "fn {}({}): {}",
                                func.name, params_str, return_str
                            )),
                            insert_text: Some(format!("{}()", func.name)),
                            insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                            ..Default::default()
                        });
                    }
                    vex_ast::Item::Struct(s) => {
                        items.push(CompletionItem {
                            label: s.name.clone(),
                            kind: Some(CompletionItemKind::STRUCT),
                            detail: Some(format!("struct {}", s.name)),
                            ..Default::default()
                        });

                        // If cursor is after ".", suggest struct fields
                        if context.after_dot {
                            for field in &s.fields {
                                items.push(CompletionItem {
                                    label: field.name.clone(),
                                    kind: Some(CompletionItemKind::FIELD),
                                    detail: Some(format!(
                                        "{}: {}",
                                        field.name,
                                        self.type_to_string(&field.ty)
                                    )),
                                    ..Default::default()
                                });
                            }
                        }
                    }
                    vex_ast::Item::Enum(e) => {
                        items.push(CompletionItem {
                            label: e.name.clone(),
                            kind: Some(CompletionItemKind::ENUM),
                            detail: Some(format!("enum {}", e.name)),
                            ..Default::default()
                        });

                        // Suggest enum variants
                        for variant in &e.variants {
                            items.push(CompletionItem {
                                label: variant.name.clone(),
                                kind: Some(CompletionItemKind::ENUM_MEMBER),
                                detail: Some(format!("{}::{}", e.name, variant.name)),
                                ..Default::default()
                            });
                        }
                    }

                    vex_ast::Item::Const(c) => {
                        let ty_str = if let Some(ty) = &c.ty {
                            self.type_to_string(ty)
                        } else {
                            "inferred".to_string()
                        };
                        items.push(CompletionItem {
                            label: c.name.clone(),
                            kind: Some(CompletionItemKind::CONSTANT),
                            detail: Some(format!("const {}: {}", c.name, ty_str)),
                            ..Default::default()
                        });
                    }
                    _ => {}
                }
            }
        }

        Ok(Some(CompletionResponse::Array(items)))
    }

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
        let word = self.get_word_at_position(&text, position);
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

    pub async fn signature_help(&self, params: SignatureHelpParams) -> tower_lsp::jsonrpc::Result<Option<SignatureHelp>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        // Get document text
        let text = match self.documents.get(&uri.to_string()) {
            Some(t) => t.clone(),
            None => return Ok(None),
        };

        // Get the AST
        let ast = match self.ast_cache.get(&uri.to_string()) {
            Some(ast) => ast.clone(),
            None => return Ok(None),
        };

        // Find function call context at cursor
        if let Some((func_name, param_index)) = self.find_function_call_context(&text, position) {
            // Search for function in AST
            for item in &ast.items {
                if let vex_ast::Item::Function(func) = item {
                    if func.name == func_name {
                        // Build signature label
                        let params_str = func
                            .params
                            .iter()
                            .map(|p| format!("{}: {}", p.name, self.type_to_string(&p.ty)))
                            .collect::<Vec<_>>()
                            .join(", ");

                        let return_str = if let Some(ret) = &func.return_type {
                            format!(": {}", self.type_to_string(ret))
                        } else {
                            String::new()
                        };

                        let label = format!("{}({}){}", func.name, params_str, return_str);

                        // Build parameter information
                        let parameters: Vec<ParameterInformation> = func
                            .params
                            .iter()
                            .map(|p| {
                                let param_label =
                                    format!("{}: {}", p.name, self.type_to_string(&p.ty));
                                ParameterInformation {
                                    label: ParameterLabel::Simple(param_label),
                                    documentation: None,
                                }
                            })
                            .collect();

                        return Ok(Some(SignatureHelp {
                            signatures: vec![SignatureInformation {
                                label,
                                documentation: None,
                                parameters: Some(parameters),
                                active_parameter: Some(param_index as u32),
                            }],
                            active_signature: Some(0),
                            active_parameter: Some(param_index as u32),
                        }));
                    }
                }
            }
        }

        Ok(None)
    }

    pub async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> tower_lsp::jsonrpc::Result<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri;

        // Get the AST
        let ast = match self.ast_cache.get(&uri.to_string()) {
            Some(ast) => ast.clone(),
            None => return Ok(None),
        };

        // Get document text for position calculation
        let text = match self.documents.get(&uri.to_string()) {
            Some(t) => t.clone(),
            None => return Ok(None),
        };

        let mut symbols = Vec::new();

        // Iterate through all items in the AST
        for item in &ast.items {
            match item {
                vex_ast::Item::Function(func) => {
                    // Find function position in source
                    if let Some(range) =
                        self.find_pattern_in_source(&text, &format!("fn {}", func.name))
                    {
                        let params_str = func
                            .params
                            .iter()
                            .map(|p| format!("{}: {}", p.name, self.type_to_string(&p.ty)))
                            .collect::<Vec<_>>()
                            .join(", ");

                        let return_str = if let Some(ret) = &func.return_type {
                            format!(": {}", self.type_to_string(ret))
                        } else {
                            String::new()
                        };

                        #[allow(deprecated)]
                        symbols.push(DocumentSymbol {
                            name: func.name.clone(),
                            detail: Some(format!("({}){}", params_str, return_str)),
                            kind: SymbolKind::FUNCTION,
                            tags: None,
                            deprecated: None,
                            range,
                            selection_range: range,
                            children: None,
                        });
                    }
                }
                vex_ast::Item::Struct(s) => {
                    if let Some(range) =
                        self.find_pattern_in_source(&text, &format!("struct {}", s.name))
                    {
                        let mut children = Vec::new();

                        // Add struct fields as children
                        for field in &s.fields {
                            if let Some(field_range) =
                                self.find_pattern_in_source(&text, &field.name)
                            {
                                #[allow(deprecated)]
                                children.push(DocumentSymbol {
                                    name: field.name.clone(),
                                    detail: Some(self.type_to_string(&field.ty)),
                                    kind: SymbolKind::FIELD,
                                    tags: None,
                                    deprecated: None,
                                    range: field_range,
                                    selection_range: field_range,
                                    children: None,
                                });
                            }
                        }

                        #[allow(deprecated)]
                        symbols.push(DocumentSymbol {
                            name: s.name.clone(),
                            detail: Some(format!("struct with {} fields", s.fields.len())),
                            kind: SymbolKind::STRUCT,
                            tags: None,
                            deprecated: None,
                            range,
                            selection_range: range,
                            children: if children.is_empty() {
                                None
                            } else {
                                Some(children)
                            },
                        });
                    }
                }
                vex_ast::Item::Enum(e) => {
                    if let Some(range) =
                        self.find_pattern_in_source(&text, &format!("enum {}", e.name))
                    {
                        let mut children = Vec::new();

                        // Add enum variants as children
                        for variant in &e.variants {
                            if let Some(variant_range) =
                                self.find_pattern_in_source(&text, &variant.name)
                            {
                                // Format multi-value tuple variant types
                                let detail = if variant.data.is_empty() {
                                    None
                                } else if variant.data.len() == 1 {
                                    Some(self.type_to_string(&variant.data[0]))
                                } else {
                                    Some(format!(
                                        "({})",
                                        variant
                                            .data
                                            .iter()
                                            .map(|t| self.type_to_string(t))
                                            .collect::<Vec<_>>()
                                            .join(", ")
                                    ))
                                };

                                #[allow(deprecated)]
                                children.push(DocumentSymbol {
                                    name: variant.name.clone(),
                                    detail,
                                    kind: SymbolKind::ENUM_MEMBER,
                                    tags: None,
                                    deprecated: None,
                                    range: variant_range,
                                    selection_range: variant_range,
                                    children: None,
                                });
                            }
                        }

                        #[allow(deprecated)]
                        symbols.push(DocumentSymbol {
                            name: e.name.clone(),
                            detail: Some(format!("enum with {} variants", e.variants.len())),
                            kind: SymbolKind::ENUM,
                            tags: None,
                            deprecated: None,
                            range,
                            selection_range: range,
                            children: if children.is_empty() {
                                None
                            } else {
                                Some(children)
                            },
                        });
                    }
                }

                vex_ast::Item::Const(c) => {
                    if let Some(range) =
                        self.find_pattern_in_source(&text, &format!("const {}", c.name))
                    {
                        let type_str = if let Some(ty) = &c.ty {
                            self.type_to_string(ty)
                        } else {
                            "inferred".to_string()
                        };

                        #[allow(deprecated)]
                        symbols.push(DocumentSymbol {
                            name: c.name.clone(),
                            detail: Some(type_str),
                            kind: SymbolKind::CONSTANT,
                            tags: None,
                            deprecated: None,
                            range,
                            selection_range: range,
                            children: None,
                        });
                    }
                }
                _ => {}
            }
        }

        if symbols.is_empty() {
            Ok(None)
        } else {
            Ok(Some(DocumentSymbolResponse::Nested(symbols)))
        }
    }

    pub async fn references(&self, params: ReferenceParams) -> tower_lsp::jsonrpc::Result<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        // Get document text
        let text = match self.documents.get(&uri.to_string()) {
            Some(t) => t.clone(),
            None => return Ok(None),
        };

        // Get the word at cursor position
        let word = self.get_word_at_position(&text, position);
        if word.is_empty() {
            return Ok(None);
        }

        let mut locations = Vec::new();
        let lines: Vec<&str> = text.lines().collect();

        // Search for all occurrences of the word in the document
        for (line_idx, line) in lines.iter().enumerate() {
            let mut start_pos = 0;
            while let Some(col_idx) = line[start_pos..].find(&word) {
                let actual_col = start_pos + col_idx;

                // Check if this is a whole word match (not part of another word)
                let before_ok = actual_col == 0
                    || !line
                        .chars()
                        .nth(actual_col - 1)
                        .unwrap_or(' ')
                        .is_alphanumeric();
                let after_pos = actual_col + word.len();
                let after_ok = after_pos >= line.len()
                    || !line.chars().nth(after_pos).unwrap_or(' ').is_alphanumeric();

                if before_ok && after_ok {
                    locations.push(Location {
                        uri: uri.clone(),
                        range: Range {
                            start: Position {
                                line: line_idx as u32,
                                character: actual_col as u32,
                            },
                            end: Position {
                                line: line_idx as u32,
                                character: (actual_col + word.len()) as u32,
                            },
                        },
                    });
                }

                start_pos = actual_col + 1;
            }
        }

        if locations.is_empty() {
            Ok(None)
        } else {
            Ok(Some(locations))
        }
    }

    pub async fn rename(&self, params: RenameParams) -> tower_lsp::jsonrpc::Result<Option<WorkspaceEdit>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let new_name = params.new_name;

        // Get document text
        let text = match self.documents.get(&uri.to_string()) {
            Some(t) => t.clone(),
            None => return Ok(None),
        };

        // Get the word at cursor position (old name)
        let old_name = self.get_word_at_position(&text, position);
        if old_name.is_empty() {
            return Ok(None);
        }

        let mut edits = Vec::new();
        let lines: Vec<&str> = text.lines().collect();

        // Find all occurrences and create text edits
        for (line_idx, line) in lines.iter().enumerate() {
            let mut start_pos = 0;
            while let Some(col_idx) = line[start_pos..].find(&old_name) {
                let actual_col = start_pos + col_idx;

                // Check if this is a whole word match
                let before_ok = actual_col == 0
                    || !line
                        .chars()
                        .nth(actual_col - 1)
                        .unwrap_or(' ')
                        .is_alphanumeric();
                let after_pos = actual_col + old_name.len();
                let after_ok = after_pos >= line.len()
                    || !line.chars().nth(after_pos).unwrap_or(' ').is_alphanumeric();

                if before_ok && after_ok {
                    edits.push(TextEdit {
                        range: Range {
                            start: Position {
                                line: line_idx as u32,
                                character: actual_col as u32,
                            },
                            end: Position {
                                line: line_idx as u32,
                                character: (actual_col + old_name.len()) as u32,
                            },
                        },
                        new_text: new_name.clone(),
                    });
                }

                start_pos = actual_col + 1;
            }
        }

        if edits.is_empty() {
            return Ok(None);
        }

        // Create workspace edit
        let mut changes = std::collections::HashMap::new();
        changes.insert(uri, edits);

        Ok(Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        }))
    }
}
