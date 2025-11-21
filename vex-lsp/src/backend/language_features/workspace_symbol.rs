//! This module contains the implementation of the `workspace_symbol` language feature.
#![allow(deprecated)]
use tower_lsp::lsp_types::*;

use crate::backend::VexBackend;

impl VexBackend {
    pub async fn workspace_symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> tower_lsp::jsonrpc::Result<Option<Vec<SymbolInformation>>> {
        let query = params.query;
        let mut symbols = Vec::new();

        // Iterate through all documents in the workspace
        for uri in self.documents.iter() {
            let uri_str = uri.key().clone();
            let _text = uri.value().clone();

            // Get AST and SpanMap for this document from cache
            if let Some(cached_doc) = self.document_cache.get(&uri_str) {
                if let Some(ast) = &cached_doc.ast {
                    // Extract symbols from AST using SpanMap for accurate positions
                    if let Err(e) = self.extract_workspace_symbols(
                        ast,
                        &cached_doc.span_map,
                        &uri_str,
                        &query,
                        &mut symbols,
                    ) {
                        eprintln!("Error extracting workspace symbols for {}: {}", uri_str, e);
                        // Continue with other documents
                    }
                }
            }
        }

        Ok(Some(symbols))
    }

    pub fn extract_workspace_symbols(
        &self,
        ast: &vex_ast::Program,
        span_map: &vex_diagnostics::SpanMap,
        uri: &str,
        query: &str,
        symbols: &mut Vec<SymbolInformation>,
    ) -> Result<(), String> {
        for item in &ast.items {
            match item {
                vex_ast::Item::Struct(struct_def) => {
                    if query.is_empty() || struct_def.name.contains(query) {
                        let parsed_uri =
                            Url::parse(uri).map_err(|e| format!("Invalid URI '{}': {}", uri, e))?;

                        let location =
                            self.get_location(uri, struct_def.span_id.as_deref(), span_map)?;

                        symbols.push(SymbolInformation {
                            name: struct_def.name.clone(),
                            kind: SymbolKind::STRUCT,
                            location,
                            container_name: None,
                            #[allow(deprecated)]
                            deprecated: None,
                            tags: Some(vec![]),
                        });
                    }
                }
                vex_ast::Item::Enum(enum_def) => {
                    if query.is_empty() || enum_def.name.contains(query) {
                        let location =
                            self.get_location(uri, enum_def.span_id.as_deref(), span_map)?;

                        symbols.push(SymbolInformation {
                            name: enum_def.name.clone(),
                            kind: SymbolKind::ENUM,
                            location,
                            container_name: None,
                            deprecated: None,
                            tags: Some(vec![]),
                        });
                    }
                }
                vex_ast::Item::Function(func_def) => {
                    if query.is_empty() || func_def.name.contains(query) {
                        let location =
                            self.get_location(uri, func_def.span_id.as_deref(), span_map)?;

                        symbols.push(SymbolInformation {
                            name: func_def.name.clone(),
                            kind: SymbolKind::FUNCTION,
                            location,
                            container_name: None,
                            deprecated: None,
                            tags: Some(vec![]),
                        });
                    }
                }
                vex_ast::Item::Contract(contract_def) => {
                    if query.is_empty() || contract_def.name.contains(query) {
                        let location =
                            self.get_location(uri, contract_def.span_id.as_deref(), span_map)?;

                        symbols.push(SymbolInformation {
                            name: contract_def.name.clone(),
                            kind: SymbolKind::INTERFACE,
                            location,
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
                            let location =
                                self.get_location(uri, method.span_id.as_deref(), span_map)?;

                            symbols.push(SymbolInformation {
                                name: method.name.clone(),
                                kind: SymbolKind::METHOD,
                                location,
                                container_name: Some(format!("{:?}", impl_def.for_type)),
                                deprecated: None,
                                tags: Some(vec![]),
                            });
                        }
                    }
                }
                vex_ast::Item::TypeAlias(alias_def) => {
                    if query.is_empty() || alias_def.name.contains(query) {
                        let location =
                            self.get_location(uri, alias_def.span_id.as_deref(), span_map)?;

                        symbols.push(SymbolInformation {
                            name: alias_def.name.clone(),
                            kind: SymbolKind::CLASS, // TypeAlias maps to Class or Interface usually
                            location,
                            container_name: None,
                            deprecated: None,
                            tags: Some(vec![]),
                        });
                    }
                }
                vex_ast::Item::Const(const_def) => {
                    if query.is_empty() || const_def.name.contains(query) {
                        let location =
                            self.get_location(uri, const_def.span_id.as_deref(), span_map)?;

                        symbols.push(SymbolInformation {
                            name: const_def.name.clone(),
                            kind: SymbolKind::CONSTANT,
                            location,
                            container_name: None,
                            deprecated: None,
                            tags: Some(vec![]),
                        });
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn get_location(
        &self,
        uri: &str,
        span_id: Option<&str>,
        span_map: &vex_diagnostics::SpanMap,
    ) -> Result<Location, String> {
        let parsed_uri = Url::parse(uri).map_err(|e| format!("Invalid URI '{}': {}", uri, e))?;

        if let Some(id) = span_id {
            if let Some(span) = span_map.get(id) {
                return Ok(Location {
                    uri: parsed_uri,
                    range: Range {
                        start: Position {
                            line: (span.line.saturating_sub(1)) as u32,
                            character: (span.column.saturating_sub(1)) as u32,
                        },
                        end: Position {
                            line: (span.line.saturating_sub(1)) as u32,
                            character: (span.column.saturating_sub(1) + span.length) as u32,
                        },
                    },
                });
            }
        }

        // Fallback to 0,0 if no span info
        Ok(Location {
            uri: parsed_uri,
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
        })
    }
}
