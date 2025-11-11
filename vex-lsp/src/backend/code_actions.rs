// LSP Code Actions features

use tower_lsp::lsp_types::*;

use super::VexBackend;

impl VexBackend {
    pub async fn code_actions(
        &self,
        params: CodeActionParams,
    ) -> tower_lsp::jsonrpc::Result<Option<CodeActionResponse>> {
        let uri = params.text_document.uri.to_string();

        // Get document text
        let text = match self.documents.get(&uri) {
            Some(doc) => doc.clone(),
            None => return Ok(None),
        };

        let mut actions = Vec::new();

        // Parse the document to get AST
        match vex_parser::Parser::new(&text) {
            Ok(mut parser) => match parser.parse() {
                Ok(ast) => {
                    // Extract code actions from AST
                    self.extract_code_actions(&ast, &params, &mut actions);
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
        _ast: &vex_ast::Program,
        _params: &CodeActionParams,
        _actions: &mut Vec<CodeActionOrCommand>,
    ) {
        // TODO: Implement AST-based code actions
        // Examples:
        // - Add missing imports
        // - Implement missing methods
        // - Convert between constructor syntaxes
        // - Add trait implementations
    }

    fn extract_quick_fixes(
        &self,
        diagnostic: &Diagnostic,
        params: &CodeActionParams,
        actions: &mut Vec<CodeActionOrCommand>,
    ) {
        // Quick fixes based on diagnostic codes
        if let Some(code) = &diagnostic.code {
            match code {
                NumberOrString::String(code_str) => match code_str.as_str() {
                    "unused_variable" => {
                        // Suggest removing unused variable
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

    pub async fn code_action_resolve(
        &self,
        params: CodeAction,
    ) -> tower_lsp::jsonrpc::Result<CodeAction> {
        // For now, just return the action as-is
        // TODO: Implement code action resolution for more complex actions
        Ok(params)
    }
}
