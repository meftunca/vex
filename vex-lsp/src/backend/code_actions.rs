// LSP Code Actions features

use std::sync::Arc;
use tower_lsp::lsp_types::*;

use super::VexBackend;

impl VexBackend {
    pub async fn code_actions(
        &self,
        params: CodeActionParams,
    ) -> tower_lsp::jsonrpc::Result<Option<CodeActionResponse>> {
        let uri_str = params.text_document.uri.to_string();
        let doc = match self.documents.get(&uri_str) {
            Some(doc) => Arc::clone(doc.value()),
            None => return Ok(None),
        };

        let mut actions = Vec::new();

        // Parse the document to get AST
        match vex_parser::Parser::new(&doc) {
            Ok(mut parser) => match parser.parse() {
                Ok(ast) => {
                    // Extract code actions from AST
                    self.extract_code_actions(&ast, &params, &doc, &mut actions);
                }
                Err(_) => {
                    // If parsing fails, still provide basic actions
                }
            },
            Err(_) => {
                // If parsing fails, still provide basic actions
            }
        }

        // Add quick fixes for diagnostics
        for diagnostic in &params.context.diagnostics {
            self.extract_quick_fixes(diagnostic, &params, &mut actions);
        }

        Ok(Some(actions))
    }

    fn extract_code_actions(
        &self,
        ast: &vex_ast::Program,
        params: &CodeActionParams,
        text: &str,
        actions: &mut Vec<CodeActionOrCommand>,
    ) {
        // Suggest missing imports
        self.suggest_missing_imports(ast, params, text, actions);

        // Suggest mutability fixes
        self.suggest_mutability_fixes(ast, params, text, actions);

        // Suggest trait implementations
        self.suggest_trait_implementations(ast, params, actions);
    }

    fn suggest_missing_imports(
        &self,
        _ast: &vex_ast::Program,
        params: &CodeActionParams,
        _text: &str,
        actions: &mut Vec<CodeActionOrCommand>,
    ) {
        // For now, provide common imports that might be missing
        // TODO: Analyze AST to determine actually missing imports

        let common_imports = [
            ("Vec", "use std::vec::Vec;"),
            ("HashMap", "use std::collections::HashMap;"),
            ("Option", "use std::option::Option;"),
            ("Result", "use std::result::Result;"),
            ("String", "use std::string::String;"),
            ("Box", "use std::boxed::Box;"),
        ];

        for (type_name, import_stmt) in &common_imports {
            // Add import at the top of the file
            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: format!("Add import for {}", type_name),
                kind: Some(CodeActionKind::QUICKFIX),
                diagnostics: None,
                edit: Some(WorkspaceEdit {
                    changes: Some(std::collections::HashMap::from([(
                        params.text_document.uri.clone(),
                        vec![TextEdit {
                            range: Range {
                                start: Position {
                                    line: 0,
                                    character: 0,
                                },
                                end: Position {
                                    line: 0,
                                    character: 0,
                                },
                            },
                            new_text: format!("{}\n", import_stmt),
                        }],
                    )])),
                    document_changes: None,
                    change_annotations: None,
                }),
                command: None,
                is_preferred: Some(false),
                disabled: None,
                data: None,
            }));
        }
    }

    fn suggest_mutability_fixes(
        &self,
        _ast: &vex_ast::Program,
        params: &CodeActionParams,
        text: &str,
        actions: &mut Vec<CodeActionOrCommand>,
    ) {
        // Get text around the cursor
        let lines: Vec<&str> = text.lines().collect();
        let line_idx = params.range.start.line as usize;

        if line_idx < lines.len() {
            let line = lines[line_idx];

            // Check for immutable variable assignments that might need mutability
            if line.contains("let ") && line.contains("=") && !line.contains("let!") {
                // Look for common patterns that suggest mutability is needed
                if line.contains(".push(") || line.contains(".insert(") || line.contains(".remove(")
                {
                    actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                        title: "Make variable mutable (let → let!)".to_string(),
                        kind: Some(CodeActionKind::QUICKFIX),
                        diagnostics: None,
                        edit: Some(WorkspaceEdit {
                            changes: Some(std::collections::HashMap::from([(
                                params.text_document.uri.clone(),
                                vec![TextEdit {
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
                                    new_text: line.replace("let ", "let! "),
                                }],
                            )])),
                            document_changes: None,
                            change_annotations: None,
                        }),
                        command: None,
                        is_preferred: Some(true),
                        disabled: None,
                        data: None,
                    }));
                }
            }
        }
    }

    fn suggest_trait_implementations(
        &self,
        ast: &vex_ast::Program,
        params: &CodeActionParams,
        actions: &mut Vec<CodeActionOrCommand>,
    ) {
        // Look for structs that might benefit from common trait implementations
        for item in &ast.items {
            match item {
                vex_ast::Item::Struct(struct_def) => {
                    // Suggest Debug implementation
                    if !self.has_trait_implementation(ast, &struct_def.name, "Debug") {
                        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                            title: format!("Implement Debug for {}", struct_def.name),
                            kind: Some(CodeActionKind::REFACTOR),
                            diagnostics: None,
                            edit: Some(WorkspaceEdit {
                                changes: Some(std::collections::HashMap::from([(
                                    params.text_document.uri.clone(),
                                    vec![TextEdit {
                                        range: Range {
                                            start: params.range.end,
                                            end: params.range.end,
                                        },
                                        new_text: format!(
                                            "\nimpl Debug for {} {{\n    fn debug(&self, f: &mut Formatter) -> Result<(), Error> {{\n        // TODO: implement debug formatting\n        Ok(())\n    }}\n}}\n",
                                            struct_def.name
                                        ),
                                    }],
                                )])),
                                document_changes: None,
                                change_annotations: None,
                            }),
                            command: None,
                            is_preferred: Some(false),
                            disabled: None,
                            data: None,
                        }));
                    }

                    // Suggest Clone implementation
                    if !self.has_trait_implementation(ast, &struct_def.name, "Clone") {
                        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                            title: format!("Implement Clone for {}", struct_def.name),
                            kind: Some(CodeActionKind::REFACTOR),
                            diagnostics: None,
                            edit: Some(WorkspaceEdit {
                                changes: Some(std::collections::HashMap::from([(
                                    params.text_document.uri.clone(),
                                    vec![TextEdit {
                                        range: Range {
                                            start: params.range.end,
                                            end: params.range.end,
                                        },
                                        new_text: format!(
                                            "\nimpl Clone for {} {{\n    fn clone(&self) -> Self {{\n        // TODO: implement clone\n        Self {{ /* fields */ }}\n    }}\n}}\n",
                                            struct_def.name
                                        ),
                                    }],
                                )])),
                                document_changes: None,
                                change_annotations: None,
                            }),
                            command: None,
                            is_preferred: Some(false),
                            disabled: None,
                            data: None,
                        }));
                    }
                }
                _ => {}
            }
        }
    }

    fn has_trait_implementation(
        &self,
        ast: &vex_ast::Program,
        type_name: &str,
        trait_name: &str,
    ) -> bool {
        // Check if the type already has the trait implemented
        for item in &ast.items {
            if let vex_ast::Item::TraitImpl(impl_block) = item {
                if impl_block.trait_name == trait_name
                    && matches!(&impl_block.for_type, vex_ast::Type::Named(t) if t == type_name)
                {
                    return true;
                }
            }
        }
        false
    }

    fn extract_quick_fixes(
        &self,
        diagnostic: &Diagnostic,
        params: &CodeActionParams,
        actions: &mut Vec<CodeActionOrCommand>,
    ) {
        // Get document text for context
        let uri_str = params.text_document.uri.to_string();
        let text = self
            .documents
            .get(&uri_str)
            .map(|r| r.value().clone())
            .unwrap_or_default();
        let lines: Vec<&str> = text.lines().collect();

        // Quick fixes based on diagnostic codes
        if let Some(code) = &diagnostic.code {
            match code {
                NumberOrString::String(code_str) => match code_str.as_str() {
                    "W0001" => {
                        // Unused variable - suggest prefixing with underscore
                        if let Some(var_name) =
                            Self::extract_variable_from_message(&diagnostic.message)
                        {
                            let new_name = format!("_{}", var_name);

                            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                                title: format!("Rename to `{}`", new_name),
                                kind: Some(CodeActionKind::QUICKFIX),
                                diagnostics: Some(vec![diagnostic.clone()]),
                                edit: Some(WorkspaceEdit {
                                    changes: Some(std::collections::HashMap::from([(
                                        params.text_document.uri.clone(),
                                        vec![TextEdit {
                                            range: diagnostic.range,
                                            new_text: new_name,
                                        }],
                                    )])),
                                    document_changes: None,
                                    change_annotations: None,
                                }),
                                command: None,
                                is_preferred: Some(true),
                                disabled: None,
                                data: None,
                            }));

                            // Also suggest removing if it's truly unused
                            let line_idx = diagnostic.range.start.line as usize;
                            if line_idx < lines.len() {
                                let line = lines[line_idx];
                                if line.trim().starts_with("let ") {
                                    actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                                        title: "Remove unused variable".to_string(),
                                        kind: Some(CodeActionKind::QUICKFIX),
                                        diagnostics: Some(vec![diagnostic.clone()]),
                                        edit: Some(WorkspaceEdit {
                                            changes: Some(std::collections::HashMap::from([(
                                                params.text_document.uri.clone(),
                                                vec![TextEdit {
                                                    range: Range {
                                                        start: Position {
                                                            line: diagnostic.range.start.line,
                                                            character: 0,
                                                        },
                                                        end: Position {
                                                            line: diagnostic.range.start.line + 1,
                                                            character: 0,
                                                        },
                                                    },
                                                    new_text: String::new(),
                                                }],
                                            )])),
                                            document_changes: None,
                                            change_annotations: None,
                                        }),
                                        command: None,
                                        is_preferred: Some(false),
                                        disabled: None,
                                        data: None,
                                    }));
                                }
                            }
                        }
                    }
                    "E0594" | "E0101" | "E0102" => {
                        // Cannot assign to immutable - suggest making mutable
                        if let Some(var_name) =
                            Self::extract_variable_from_message(&diagnostic.message)
                        {
                            let line_idx = diagnostic.range.start.line as usize;
                            if line_idx < lines.len() {
                                let line = lines[line_idx];
                                if let Some(col) = line.find(&format!("let {}", var_name)) {
                                    // Insert ! after "let"
                                    actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                                        title: format!("Make `{}` mutable", var_name),
                                        kind: Some(CodeActionKind::QUICKFIX),
                                        diagnostics: Some(vec![diagnostic.clone()]),
                                        edit: Some(WorkspaceEdit {
                                            changes: Some(std::collections::HashMap::from([(
                                                params.text_document.uri.clone(),
                                                vec![TextEdit {
                                                    range: Range {
                                                        start: Position {
                                                            line: line_idx as u32,
                                                            character: (col + 3) as u32, // "let".len()
                                                        },
                                                        end: Position {
                                                            line: line_idx as u32,
                                                            character: (col + 3) as u32,
                                                        },
                                                    },
                                                    new_text: "!".to_string(),
                                                }],
                                            )])),
                                            document_changes: None,
                                            change_annotations: None,
                                        }),
                                        command: None,
                                        is_preferred: Some(true),
                                        disabled: None,
                                        data: None,
                                    }));
                                }
                            }
                        }
                    }
                    "W0002" | "W0003" | "W0004" | "W0005" => {
                        // Dead code - suggest removing or making public
                        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                            title: "Remove dead code".to_string(),
                            kind: Some(CodeActionKind::QUICKFIX),
                            diagnostics: Some(vec![diagnostic.clone()]),
                            edit: Some(WorkspaceEdit {
                                changes: Some(std::collections::HashMap::from([(
                                    params.text_document.uri.clone(),
                                    vec![TextEdit {
                                        range: diagnostic.range,
                                        new_text: String::new(),
                                    }],
                                )])),
                                document_changes: None,
                                change_annotations: None,
                            }),
                            command: None,
                            is_preferred: Some(false),
                            disabled: None,
                            data: None,
                        }));
                    }
                    "unused_variable" => {
                        // Legacy code - kept for compatibility
                        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                            title: "Remove unused variable".to_string(),
                            kind: Some(CodeActionKind::QUICKFIX),
                            diagnostics: Some(vec![diagnostic.clone()]),
                            edit: Some(WorkspaceEdit {
                                changes: Some(std::collections::HashMap::from([(
                                    params.text_document.uri.clone(),
                                    vec![TextEdit {
                                        range: diagnostic.range,
                                        new_text: String::new(),
                                    }],
                                )])),
                                document_changes: None,
                                change_annotations: None,
                            }),
                            command: None,
                            is_preferred: Some(true),
                            disabled: None,
                            data: None,
                        }));
                    }
                    "missing_import" => {
                        // Suggest adding import
                        // TODO: Implement import suggestion
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    /// Extract variable name from diagnostic message
    /// "unused variable: `x`" → Some("x")
    fn extract_variable_from_message(message: &str) -> Option<String> {
        let start = message.find('`')?;
        let end = message[start + 1..].find('`')?;
        Some(message[start + 1..start + 1 + end].to_string())
    }

    pub async fn code_action_resolve(
        &self,
        params: CodeAction,
    ) -> tower_lsp::jsonrpc::Result<CodeAction> {
        // For now, just return the action as-is
        // TODO: Implement code action resolution for more complex actions
        Ok(params)
    }
}
