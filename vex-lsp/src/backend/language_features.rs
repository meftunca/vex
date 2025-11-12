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

        // Get document text for word extraction
        let text = match self.documents.get(&uri) {
            Some(t) => t.clone(),
            None => return Ok(None),
        };

        // Get word at cursor position
        let word = self.get_word_at_position(&text, position);
        if word.is_empty() {
            return Ok(None);
        }

        // Simple hover: show the word that was found
        Ok(Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("**Symbol**: `{}`\n\n*Vex Language*", word),
            }),
            range: None,
        }))
    }

    pub async fn completion(
        &self,
        params: CompletionParams,
    ) -> tower_lsp::jsonrpc::Result<Option<CompletionResponse>> {
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

    pub async fn signature_help(
        &self,
        params: SignatureHelpParams,
    ) -> tower_lsp::jsonrpc::Result<Option<SignatureHelp>> {
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

    pub async fn references(
        &self,
        params: ReferenceParams,
    ) -> tower_lsp::jsonrpc::Result<Option<Vec<Location>>> {
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

    pub async fn rename(
        &self,
        params: RenameParams,
    ) -> tower_lsp::jsonrpc::Result<Option<WorkspaceEdit>> {
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

    pub async fn workspace_symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> tower_lsp::jsonrpc::Result<Option<Vec<SymbolInformation>>> {
        let query = params.query;
        let mut symbols = Vec::new();

        // Iterate through all documents in the workspace
        for uri in self.documents.iter() {
            let uri_str = uri.key().clone();
            let text = uri.value().clone();

            // Get AST for this document
            if let Some(ast) = self.ast_cache.get(&uri_str) {
                // Extract symbols from AST
                self.extract_workspace_symbols(&ast, &uri_str, &query, &mut symbols);
            }
        }

        Ok(Some(symbols))
    }

    pub async fn folding_range(
        &self,
        params: FoldingRangeParams,
    ) -> tower_lsp::jsonrpc::Result<Option<Vec<FoldingRange>>> {
        let uri = params.text_document.uri;
        let text = match self.documents.get(&uri.to_string()) {
            Some(t) => t.clone(),
            None => return Ok(None),
        };

        let mut folding_ranges = Vec::new();
        self.extract_folding_ranges(&text, &mut folding_ranges);

        Ok(Some(folding_ranges))
    }

    fn extract_folding_ranges(&self, text: &str, folding_ranges: &mut Vec<FoldingRange>) {
        let lines: Vec<&str> = text.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            let line_num = i as u32;
            let trimmed = line.trim();

            // Function definitions
            if trimmed.starts_with("fn ") {
                // Find the opening brace
                if let Some(opening_brace_pos) = self.find_matching_brace(&lines, i, '{') {
                    folding_ranges.push(FoldingRange {
                        start_line: line_num,
                        start_character: Some(0),
                        end_line: opening_brace_pos,
                        end_character: Some(0),
                        kind: Some(FoldingRangeKind::Region),
                        collapsed_text: Some(format!("fn ...")),
                    });
                }
            }
            // Struct definitions
            else if trimmed.starts_with("struct ") {
                // Find the opening brace
                if let Some(opening_brace_pos) = self.find_matching_brace(&lines, i, '{') {
                    folding_ranges.push(FoldingRange {
                        start_line: line_num,
                        start_character: Some(0),
                        end_line: opening_brace_pos,
                        end_character: Some(0),
                        kind: Some(FoldingRangeKind::Region),
                        collapsed_text: Some(format!("struct ...")),
                    });
                }
            }
            // Enum definitions
            else if trimmed.starts_with("enum ") {
                // Find the opening brace
                if let Some(opening_brace_pos) = self.find_matching_brace(&lines, i, '{') {
                    folding_ranges.push(FoldingRange {
                        start_line: line_num,
                        start_character: Some(0),
                        end_line: opening_brace_pos,
                        end_character: Some(0),
                        kind: Some(FoldingRangeKind::Region),
                        collapsed_text: Some(format!("enum ...")),
                    });
                }
            }
            // Trait definitions
            else if trimmed.starts_with("trait ") {
                // Find the opening brace
                if let Some(opening_brace_pos) = self.find_matching_brace(&lines, i, '{') {
                    folding_ranges.push(FoldingRange {
                        start_line: line_num,
                        start_character: Some(0),
                        end_line: opening_brace_pos,
                        end_character: Some(0),
                        kind: Some(FoldingRangeKind::Region),
                        collapsed_text: Some(format!("trait ...")),
                    });
                }
            }
            // Impl blocks
            else if trimmed.starts_with("impl ") {
                // Find the opening brace
                if let Some(opening_brace_pos) = self.find_matching_brace(&lines, i, '{') {
                    folding_ranges.push(FoldingRange {
                        start_line: line_num,
                        start_character: Some(0),
                        end_line: opening_brace_pos,
                        end_character: Some(0),
                        kind: Some(FoldingRangeKind::Region),
                        collapsed_text: Some(format!("impl ...")),
                    });
                }
            }
            // If statements
            else if trimmed.starts_with("if ") && trimmed.ends_with("{") {
                // Find the matching closing brace
                if let Some(closing_brace_pos) = self.find_matching_brace(&lines, i, '}') {
                    folding_ranges.push(FoldingRange {
                        start_line: line_num,
                        start_character: Some(0),
                        end_line: closing_brace_pos,
                        end_character: Some(0),
                        kind: Some(FoldingRangeKind::Region),
                        collapsed_text: Some(format!("if ...")),
                    });
                }
            }
            // While loops
            else if trimmed.starts_with("while ") && trimmed.ends_with("{") {
                // Find the matching closing brace
                if let Some(closing_brace_pos) = self.find_matching_brace(&lines, i, '}') {
                    folding_ranges.push(FoldingRange {
                        start_line: line_num,
                        start_character: Some(0),
                        end_line: closing_brace_pos,
                        end_character: Some(0),
                        kind: Some(FoldingRangeKind::Region),
                        collapsed_text: Some(format!("while ...")),
                    });
                }
            }
            // For loops
            else if trimmed.starts_with("for ") && trimmed.ends_with("{") {
                // Find the matching closing brace
                if let Some(closing_brace_pos) = self.find_matching_brace(&lines, i, '}') {
                    folding_ranges.push(FoldingRange {
                        start_line: line_num,
                        start_character: Some(0),
                        end_line: closing_brace_pos,
                        end_character: Some(0),
                        kind: Some(FoldingRangeKind::Region),
                        collapsed_text: Some(format!("for ...")),
                    });
                }
            }
            // Match expressions
            else if trimmed.starts_with("match ") && trimmed.ends_with("{") {
                // Find the matching closing brace
                if let Some(closing_brace_pos) = self.find_matching_brace(&lines, i, '}') {
                    folding_ranges.push(FoldingRange {
                        start_line: line_num,
                        start_character: Some(0),
                        end_line: closing_brace_pos,
                        end_character: Some(0),
                        kind: Some(FoldingRangeKind::Region),
                        collapsed_text: Some(format!("match ...")),
                    });
                }
            }
        }
    }

    fn find_matching_brace(
        &self,
        lines: &[&str],
        start_line: usize,
        target_brace: char,
    ) -> Option<u32> {
        let mut brace_count = 0;
        let mut found_opening = false;

        for (i, line) in lines.iter().enumerate().skip(start_line) {
            for ch in line.chars() {
                if ch == '{' {
                    brace_count += 1;
                    found_opening = true;
                } else if ch == '}' {
                    brace_count -= 1;
                    if found_opening && brace_count == 0 {
                        return Some(i as u32);
                    }
                }
            }
        }

        None
    }

    // Helper methods for language features

    fn get_completion_context(&self, text: &str, position: Position) -> CompletionContext {
        let lines: Vec<&str> = text.lines().collect();
        let line_idx = position.line as usize;
        let char_idx = position.character as usize;

        if line_idx >= lines.len() {
            return CompletionContext {
                before_cursor: String::new(),
                after_dot: false,
            };
        }

        let line = lines[line_idx];
        if char_idx > line.len() {
            return CompletionContext {
                before_cursor: line.to_string(),
                after_dot: false,
            };
        }

        let before_cursor = &line[..char_idx];
        let after_dot = before_cursor.ends_with('.');

        CompletionContext {
            before_cursor: before_cursor.to_string(),
            after_dot,
        }
    }

    fn type_to_string(&self, ty: &vex_ast::Type) -> String {
        match ty {
            vex_ast::Type::I8 => "i8".to_string(),
            vex_ast::Type::I16 => "i16".to_string(),
            vex_ast::Type::I32 => "i32".to_string(),
            vex_ast::Type::I64 => "i64".to_string(),
            vex_ast::Type::I128 => "i128".to_string(),
            vex_ast::Type::U8 => "u8".to_string(),
            vex_ast::Type::U16 => "u16".to_string(),
            vex_ast::Type::U32 => "u32".to_string(),
            vex_ast::Type::U64 => "u64".to_string(),
            vex_ast::Type::F16 => "f16".to_string(),
            vex_ast::Type::F32 => "f32".to_string(),
            vex_ast::Type::F64 => "f64".to_string(),
            vex_ast::Type::Bool => "bool".to_string(),
            vex_ast::Type::String => "string".to_string(),
            vex_ast::Type::Byte => "byte".to_string(),
            vex_ast::Type::Error => "error".to_string(),
            vex_ast::Type::Nil => "nil".to_string(),
            vex_ast::Type::Named(name) => name.clone(),
            vex_ast::Type::Generic { name, type_args } => {
                if type_args.is_empty() {
                    name.clone()
                } else {
                    format!(
                        "{}<{}>",
                        name,
                        type_args
                            .iter()
                            .map(|t| self.type_to_string(t))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
            }
            vex_ast::Type::Array(element, _) => format!("[{}]", self.type_to_string(element)),
            vex_ast::Type::Slice(element, is_mutable) => {
                if *is_mutable {
                    format!("&mut [{}]", self.type_to_string(element))
                } else {
                    format!("&[{}]", self.type_to_string(element))
                }
            }
            vex_ast::Type::Reference(inner, is_mutable) => {
                if *is_mutable {
                    format!("&mut {}", self.type_to_string(inner))
                } else {
                    format!("&{}", self.type_to_string(inner))
                }
            }
            vex_ast::Type::Tuple(types) => {
                format!(
                    "({})",
                    types
                        .iter()
                        .map(|t| self.type_to_string(t))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            vex_ast::Type::Function {
                params,
                return_type,
            } => {
                let params_str = params
                    .iter()
                    .map(|p| self.type_to_string(p))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("fn({}): {}", params_str, self.type_to_string(return_type))
            }
            vex_ast::Type::Never => "!".to_string(),
            vex_ast::Type::Infer(_) => "_".to_string(),
            vex_ast::Type::Unit => "()".to_string(),
            vex_ast::Type::RawPtr { inner, is_const } => {
                if *is_const {
                    format!("*const {}", self.type_to_string(inner))
                } else {
                    format!("*{}", self.type_to_string(inner))
                }
            }
            vex_ast::Type::Option(inner) => format!("Option<{}>", self.type_to_string(inner)),
            vex_ast::Type::Result(ok_type, err_type) => format!(
                "Result<{}, {}>",
                self.type_to_string(ok_type),
                self.type_to_string(err_type)
            ),
            vex_ast::Type::Vec(inner) => format!("Vec<{}>", self.type_to_string(inner)),
            vex_ast::Type::Box(inner) => format!("Box<{}>", self.type_to_string(inner)),
            vex_ast::Type::Channel(inner) => format!("Channel<{}>", self.type_to_string(inner)),
            // Handle other variants with defaults
            _ => format!("{:?}", ty), // Fallback for unhandled variants
        }
    }

    fn get_word_at_position(&self, text: &str, position: Position) -> String {
        let lines: Vec<&str> = text.lines().collect();
        let line_idx = position.line as usize;
        let char_idx = position.character as usize;

        if line_idx >= lines.len() {
            return String::new();
        }

        let line = lines[line_idx];
        if char_idx >= line.len() {
            return String::new();
        }

        // Find word boundaries
        let mut start = char_idx;
        let mut end = char_idx;

        // Move start backwards to find word start
        while start > 0 && line.chars().nth(start - 1).unwrap_or(' ').is_alphanumeric() {
            start -= 1;
        }

        // Move end forwards to find word end
        while end < line.len() && line.chars().nth(end).unwrap_or(' ').is_alphanumeric() {
            end += 1;
        }

        if start < end {
            line[start..end].to_string()
        } else {
            String::new()
        }
    }

    fn find_definition_location(
        &self,
        ast: &vex_ast::Program,
        word: &str,
        text: &str,
    ) -> Option<Range> {
        // Simple implementation: search for function/struct/enum definitions
        for item in &ast.items {
            match item {
                vex_ast::Item::Function(func) if func.name == word => {
                    return self.find_pattern_in_source(text, &format!("fn {}", word));
                }
                vex_ast::Item::Struct(s) if s.name == word => {
                    return self.find_pattern_in_source(text, &format!("struct {}", word));
                }
                vex_ast::Item::Enum(e) if e.name == word => {
                    return self.find_pattern_in_source(text, &format!("enum {}", word));
                }
                vex_ast::Item::Const(c) if c.name == word => {
                    return self.find_pattern_in_source(text, &format!("const {}", word));
                }
                _ => {}
            }
        }
        None
    }

    fn find_function_call_context(
        &self,
        text: &str,
        position: Position,
    ) -> Option<(String, usize)> {
        let lines: Vec<&str> = text.lines().collect();
        let line_idx = position.line as usize;
        let char_idx = position.character as usize;

        if line_idx >= lines.len() {
            return None;
        }

        let line = lines[line_idx];
        if char_idx >= line.len() {
            return None;
        }

        // Look backwards from cursor to find function call
        let mut paren_count = 0;
        let mut param_start = char_idx;
        let mut func_name_end = char_idx;

        for i in (0..char_idx).rev() {
            let ch = line.chars().nth(i).unwrap_or(' ');
            match ch {
                ')' => paren_count += 1,
                '(' => {
                    if paren_count == 0 {
                        // Found the opening paren
                        param_start = i + 1;
                        func_name_end = i;
                        break;
                    } else {
                        paren_count -= 1;
                    }
                }
                ',' if paren_count == 0 => {
                    // Parameter separator
                    param_start = i + 1;
                }
                _ => {}
            }
        }

        if func_name_end > 0 {
            // Extract function name
            let mut name_start = func_name_end;
            while name_start > 0
                && line
                    .chars()
                    .nth(name_start - 1)
                    .unwrap_or(' ')
                    .is_alphanumeric()
            {
                name_start -= 1;
            }

            if name_start < func_name_end {
                let func_name = line[name_start..func_name_end].to_string();

                // Count parameters before cursor
                let params_before = &line[param_start..char_idx];
                let param_index = params_before.chars().filter(|&c| c == ',').count();

                return Some((func_name, param_index));
            }
        }

        None
    }

    fn find_pattern_in_source(&self, text: &str, pattern: &str) -> Option<Range> {
        let lines: Vec<&str> = text.lines().collect();

        for (line_idx, line) in lines.iter().enumerate() {
            if let Some(col_idx) = line.find(pattern) {
                return Some(Range {
                    start: Position {
                        line: line_idx as u32,
                        character: col_idx as u32,
                    },
                    end: Position {
                        line: line_idx as u32,
                        character: (col_idx + pattern.len()) as u32,
                    },
                });
            }
        }

        None
    }

    fn extract_workspace_symbols(
        &self,
        ast: &vex_ast::Program,
        uri: &str,
        query: &str,
        symbols: &mut Vec<SymbolInformation>,
    ) {
        for item in &ast.items {
            match item {
                vex_ast::Item::Struct(struct_def) => {
                    if query.is_empty() || struct_def.name.contains(query) {
                        symbols.push(SymbolInformation {
                            name: struct_def.name.clone(),
                            kind: SymbolKind::STRUCT,
                            location: Location {
                                uri: Url::parse(uri).unwrap(),
                                range: Range {
                                    start: Position {
                                        line: 0,
                                        character: 0,
                                    }, // TODO: Get actual position
                                    end: Position {
                                        line: 0,
                                        character: 0,
                                    },
                                },
                            },
                            container_name: None,
                            deprecated: None,
                            tags: Some(vec![]),
                        });
                    }
                }
                vex_ast::Item::Enum(enum_def) => {
                    if query.is_empty() || enum_def.name.contains(query) {
                        symbols.push(SymbolInformation {
                            name: enum_def.name.clone(),
                            kind: SymbolKind::ENUM,
                            location: Location {
                                uri: Url::parse(uri).unwrap(),
                                range: Range {
                                    start: Position {
                                        line: 0,
                                        character: 0,
                                    }, // TODO: Get actual position
                                    end: Position {
                                        line: 0,
                                        character: 0,
                                    },
                                },
                            },
                            container_name: None,
                            deprecated: None,
                            tags: Some(vec![]),
                        });
                    }
                }
                vex_ast::Item::Function(func_def) => {
                    if query.is_empty() || func_def.name.contains(query) {
                        symbols.push(SymbolInformation {
                            name: func_def.name.clone(),
                            kind: SymbolKind::FUNCTION,
                            location: Location {
                                uri: Url::parse(uri).unwrap(),
                                range: Range {
                                    start: Position {
                                        line: 0,
                                        character: 0,
                                    }, // TODO: Get actual position
                                    end: Position {
                                        line: 0,
                                        character: 0,
                                    },
                                },
                            },
                            container_name: None,
                            deprecated: None,
                            tags: Some(vec![]),
                        });
                    }
                }
                vex_ast::Item::Contract(contract_def) => {
                    if query.is_empty() || contract_def.name.contains(query) {
                        symbols.push(SymbolInformation {
                            name: contract_def.name.clone(),
                            kind: SymbolKind::INTERFACE,
                            location: Location {
                                uri: Url::parse(uri).unwrap(),
                                range: Range {
                                    start: Position {
                                        line: 0,
                                        character: 0,
                                    }, // TODO: Get actual position
                                    end: Position {
                                        line: 0,
                                        character: 0,
                                    },
                                },
                            },
                            container_name: None,
                            deprecated: None,
                            tags: Some(vec![]),
                        });
                    }
                }
                vex_ast::Item::TraitImpl(impl_def) => {
                    // For trait impl blocks, we can extract method symbols
                    for method in &impl_def.methods {
                        if query.is_empty() || method.name.contains(query) {
                            symbols.push(SymbolInformation {
                                name: method.name.clone(),
                                kind: SymbolKind::METHOD,
                                location: Location {
                                    uri: Url::parse(uri).unwrap(),
                                    range: Range {
                                        start: Position {
                                            line: 0,
                                            character: 0,
                                        }, // TODO: Get actual position
                                        end: Position {
                                            line: 0,
                                            character: 0,
                                        },
                                    },
                                },
                                container_name: Some(format!("{:?}", impl_def.for_type)),
                                deprecated: None,
                                tags: Some(vec![]),
                            });
                        }
                    }
                }
                _ => {}
            }
        }
    }

    pub async fn type_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> tower_lsp::jsonrpc::Result<Option<GotoDefinitionResponse>> {
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .to_string();
        let position = params.text_document_position_params.position;

        // Get the AST for this document
        let ast = match self.ast_cache.get(&uri) {
            Some(ast) => ast.clone(),
            None => return Ok(None),
        };

        // Get document text for word extraction
        let text = match self.documents.get(&uri) {
            Some(t) => t.clone(),
            None => return Ok(None),
        };

        // Get word at cursor position
        let word = self.get_word_at_position(&text, position);
        if word.is_empty() {
            return Ok(None);
        }

        // Search for type definitions (struct, enum, trait) in the current file
        let mut locations = Vec::new();

        for (line_idx, line) in text.lines().enumerate() {
            let trimmed = line.trim();

            // Check for struct definitions
            if trimmed.starts_with("struct ") && trimmed.contains(&word) {
                if let Some(name_start) = trimmed.find("struct ") {
                    let name_end = trimmed[name_start + 7..]
                        .find(' ')
                        .unwrap_or(trimmed.len() - name_start - 7);
                    let struct_name = &trimmed[name_start + 7..name_start + 7 + name_end];
                    if struct_name == word {
                        locations.push(Location {
                            uri: params
                                .text_document_position_params
                                .text_document
                                .uri
                                .clone(),
                            range: Range {
                                start: Position {
                                    line: line_idx as u32,
                                    character: (name_start + 7) as u32,
                                },
                                end: Position {
                                    line: line_idx as u32,
                                    character: (name_start + 7 + name_end) as u32,
                                },
                            },
                        });
                    }
                }
            }
            // Check for enum definitions
            else if trimmed.starts_with("enum ") && trimmed.contains(&word) {
                if let Some(name_start) = trimmed.find("enum ") {
                    let name_end = trimmed[name_start + 5..]
                        .find(' ')
                        .unwrap_or(trimmed.len() - name_start - 5);
                    let enum_name = &trimmed[name_start + 5..name_start + 5 + name_end];
                    if enum_name == word {
                        locations.push(Location {
                            uri: params
                                .text_document_position_params
                                .text_document
                                .uri
                                .clone(),
                            range: Range {
                                start: Position {
                                    line: line_idx as u32,
                                    character: (name_start + 5) as u32,
                                },
                                end: Position {
                                    line: line_idx as u32,
                                    character: (name_start + 5 + name_end) as u32,
                                },
                            },
                        });
                    }
                }
            }
            // Check for trait definitions
            else if trimmed.starts_with("trait ") && trimmed.contains(&word) {
                if let Some(name_start) = trimmed.find("trait ") {
                    let name_end = trimmed[name_start + 6..]
                        .find(' ')
                        .unwrap_or(trimmed.len() - name_start - 6);
                    let trait_name = &trimmed[name_start + 6..name_start + 6 + name_end];
                    if trait_name == word {
                        locations.push(Location {
                            uri: params
                                .text_document_position_params
                                .text_document
                                .uri
                                .clone(),
                            range: Range {
                                start: Position {
                                    line: line_idx as u32,
                                    character: (name_start + 6) as u32,
                                },
                                end: Position {
                                    line: line_idx as u32,
                                    character: (name_start + 6 + name_end) as u32,
                                },
                            },
                        });
                    }
                }
            }
        }

        if locations.is_empty() {
            Ok(None)
        } else {
            Ok(Some(GotoDefinitionResponse::Array(locations)))
        }
    }

    pub async fn implementation(
        &self,
        params: GotoDefinitionParams,
    ) -> tower_lsp::jsonrpc::Result<Option<GotoDefinitionResponse>> {
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .to_string();
        let position = params.text_document_position_params.position;

        // Get the AST for this document
        let ast = match self.ast_cache.get(&uri) {
            Some(ast) => ast.clone(),
            None => return Ok(None),
        };

        // Get document text for word extraction
        let text = match self.documents.get(&uri) {
            Some(t) => t.clone(),
            None => return Ok(None),
        };

        // Get word at cursor position
        let word = self.get_word_at_position(&text, position);
        if word.is_empty() {
            return Ok(None);
        }

        // Search for implementations (impl blocks) in the current file
        let mut locations = Vec::new();

        for (line_idx, line) in text.lines().enumerate() {
            let trimmed = line.trim();

            // Check for impl blocks
            if trimmed.starts_with("impl ") && trimmed.contains(&word) {
                locations.push(Location {
                    uri: params
                        .text_document_position_params
                        .text_document
                        .uri
                        .clone(),
                    range: Range {
                        start: Position {
                            line: line_idx as u32,
                            character: 0,
                        },
                        end: Position {
                            line: line_idx as u32,
                            character: line.len() as u32,
                        },
                    },
                });
            }
            // Also check for trait implementations like "impl Trait for Type"
            else if trimmed.starts_with("impl ")
                && trimmed.contains(" for ")
                && trimmed.contains(&word)
            {
                locations.push(Location {
                    uri: params
                        .text_document_position_params
                        .text_document
                        .uri
                        .clone(),
                    range: Range {
                        start: Position {
                            line: line_idx as u32,
                            character: 0,
                        },
                        end: Position {
                            line: line_idx as u32,
                            character: line.len() as u32,
                        },
                    },
                });
            }
        }

        if locations.is_empty() {
            Ok(None)
        } else {
            Ok(Some(GotoDefinitionResponse::Array(locations)))
        }
    }
}

// Helper struct for completion context
#[derive(Debug)]
struct CompletionContext {
    before_cursor: String,
    after_dot: bool,
}
