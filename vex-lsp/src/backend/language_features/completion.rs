//! This module contains the implementation of the `completion` language feature.
use tower_lsp::lsp_types::*;

use crate::backend::{language_features::helpers::*, VexBackend};

impl VexBackend {
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

        let context = params.context;

        // Only provide struct field suggestions if the trigger was `.`
        let after_dot = context.map_or(false, |ctx| {
            ctx.trigger_kind == CompletionTriggerKind::TRIGGER_CHARACTER
                && ctx.trigger_character.as_deref() == Some(".")
        });

        // Keywords
        let keywords = vec![
            ("fn", CompletionItemKind::KEYWORD, "function declaration"),
            ("let", CompletionItemKind::KEYWORD, "immutable variable"),
            ("let!", CompletionItemKind::KEYWORD, "mutable variable"),
            ("const", CompletionItemKind::KEYWORD, "constant"),
            ("struct", CompletionItemKind::KEYWORD, "struct definition"),
            ("enum", CompletionItemKind::KEYWORD, "enum definition"),
            (
                "contract",
                CompletionItemKind::KEYWORD,
                "contract definition",
            ),
            ("impl", CompletionItemKind::KEYWORD, "implementation block"),
            ("type", CompletionItemKind::KEYWORD, "type alias"),
            (
                "extern",
                CompletionItemKind::KEYWORD,
                "external declaration",
            ),
            ("policy", CompletionItemKind::KEYWORD, "policy definition"),
            ("if", CompletionItemKind::KEYWORD, "conditional"),
            ("else", CompletionItemKind::KEYWORD, "else clause"),
            ("elif", CompletionItemKind::KEYWORD, "else if clause"),
            ("match", CompletionItemKind::KEYWORD, "pattern matching"),
            ("for", CompletionItemKind::KEYWORD, "for loop"),
            ("while", CompletionItemKind::KEYWORD, "while loop"),
            ("loop", CompletionItemKind::KEYWORD, "infinite loop"),
            ("in", CompletionItemKind::KEYWORD, "in keyword"),
            ("return", CompletionItemKind::KEYWORD, "return statement"),
            ("break", CompletionItemKind::KEYWORD, "break loop"),
            ("continue", CompletionItemKind::KEYWORD, "continue loop"),
            ("defer", CompletionItemKind::KEYWORD, "defer statement"),
            ("select", CompletionItemKind::KEYWORD, "select statement"),
            ("async", CompletionItemKind::KEYWORD, "async function"),
            ("await", CompletionItemKind::KEYWORD, "await expression"),
            ("go", CompletionItemKind::KEYWORD, "goroutine"),
            ("gpu", CompletionItemKind::KEYWORD, "GPU launch"),
            ("launch", CompletionItemKind::KEYWORD, "launch keyword"),
            ("export", CompletionItemKind::KEYWORD, "export symbol"),
            ("import", CompletionItemKind::KEYWORD, "import module"),
            ("from", CompletionItemKind::KEYWORD, "from import"),
            ("as", CompletionItemKind::KEYWORD, "as keyword"),
            ("with", CompletionItemKind::KEYWORD, "with clause"),
        ];

        // Built-in types
        let builtin_types = vec![
            ("i8", CompletionItemKind::STRUCT, "8-bit integer"),
            ("i16", CompletionItemKind::STRUCT, "16-bit integer"),
            ("i32", CompletionItemKind::STRUCT, "32-bit integer"),
            ("i64", CompletionItemKind::STRUCT, "64-bit integer"),
            ("i128", CompletionItemKind::STRUCT, "128-bit integer"),
            ("u8", CompletionItemKind::STRUCT, "unsigned 8-bit integer"),
            ("u16", CompletionItemKind::STRUCT, "unsigned 16-bit integer"),
            ("u32", CompletionItemKind::STRUCT, "unsigned 32-bit integer"),
            ("u64", CompletionItemKind::STRUCT, "unsigned 64-bit integer"),
            (
                "u128",
                CompletionItemKind::STRUCT,
                "unsigned 128-bit integer",
            ),
            ("f16", CompletionItemKind::STRUCT, "16-bit float"),
            ("f32", CompletionItemKind::STRUCT, "32-bit float"),
            ("f64", CompletionItemKind::STRUCT, "64-bit float"),
            ("bool", CompletionItemKind::STRUCT, "boolean"),
            ("string", CompletionItemKind::STRUCT, "string type"),
            ("byte", CompletionItemKind::STRUCT, "byte type"),
            ("error", CompletionItemKind::STRUCT, "error type"),
            ("nil", CompletionItemKind::STRUCT, "nil type"),
            ("Vec", CompletionItemKind::STRUCT, "dynamic array"),
            ("Box", CompletionItemKind::STRUCT, "heap allocation"),
            ("Option", CompletionItemKind::ENUM, "optional value"),
            ("Result", CompletionItemKind::ENUM, "result type"),
            ("HashMap", CompletionItemKind::STRUCT, "hash map"),
            ("Channel", CompletionItemKind::STRUCT, "MPSC channel"),
            ("Future", CompletionItemKind::STRUCT, "async future"),
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
                            .map(|p| format!("{}: {}", p.name, type_to_string(&p.ty)))
                            .collect::<Vec<_>>()
                            .join(", ");
                        let return_str = if let Some(ret) = &func.return_type {
                            type_to_string(ret)
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
                        if after_dot {
                            for field in &s.fields {
                                items.push(CompletionItem {
                                    label: field.name.clone(),
                                    kind: Some(CompletionItemKind::FIELD),
                                    detail: Some(format!(
                                        "{}: {}",
                                        field.name,
                                        type_to_string(&field.ty)
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
                            type_to_string(ty)
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
}
