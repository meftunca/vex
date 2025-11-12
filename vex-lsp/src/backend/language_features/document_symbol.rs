//! This module contains the implementation of the `document_symbol` language feature.
use tower_lsp::lsp_types::*;

use crate::backend::{language_features::helpers::*, VexBackend};

impl VexBackend {
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
                    if let Some(range) = find_pattern_in_source(&text, &format!("fn {}", func.name))
                    {
                        let params_str = func
                            .params
                            .iter()
                            .map(|p| format!("{}: {}", p.name, type_to_string(&p.ty)))
                            .collect::<Vec<_>>()
                            .join(", ");

                        let return_str = if let Some(ret) = &func.return_type {
                            format!(": {}", type_to_string(ret))
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
                        find_pattern_in_source(&text, &format!("struct {}", s.name))
                    {
                        let mut children = Vec::new();

                        // Add struct fields as children
                        for field in &s.fields {
                            if let Some(field_range) = find_pattern_in_source(&text, &field.name) {
                                #[allow(deprecated)]
                                children.push(DocumentSymbol {
                                    name: field.name.clone(),
                                    detail: Some(type_to_string(&field.ty)),
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
                    if let Some(range) = find_pattern_in_source(&text, &format!("enum {}", e.name))
                    {
                        let mut children = Vec::new();

                        // Add enum variants as children
                        for variant in &e.variants {
                            if let Some(variant_range) =
                                find_pattern_in_source(&text, &variant.name)
                            {
                                // Format multi-value tuple variant types
                                let detail = if variant.data.is_empty() {
                                    None
                                } else if variant.data.len() == 1 {
                                    Some(type_to_string(&variant.data[0]))
                                } else {
                                    Some(format!(
                                        "({})",
                                        variant
                                            .data
                                            .iter()
                                            .map(|t| type_to_string(t))
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
                    if let Some(range) = find_pattern_in_source(&text, &format!("const {}", c.name))
                    {
                        let type_str = if let Some(ty) = &c.ty {
                            type_to_string(ty)
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
}
