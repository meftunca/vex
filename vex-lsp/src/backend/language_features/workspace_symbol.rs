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

            // Get AST for this document
            if let Some(ast) = self.ast_cache.get(&uri_str) {
                // Extract symbols from AST
                if let Err(e) = self.extract_workspace_symbols(&ast, &uri_str, &query, &mut symbols) {
                    eprintln!("Error extracting workspace symbols for {}: {}", uri_str, e);
                    // Continue with other documents
                }
            }
        }

        Ok(Some(symbols))
    }

    pub fn extract_workspace_symbols(
        &self,
        ast: &vex_ast::Program,
        uri: &str,
        query: &str,
        symbols: &mut Vec<SymbolInformation>,
    ) -> Result<(), String> {
        for item in &ast.items {
            match item {
                vex_ast::Item::Struct(struct_def) => {
                    if query.is_empty() || struct_def.name.contains(query) {
                        let parsed_uri = Url::parse(uri)
                            .map_err(|e| format!("Invalid URI '{}': {}", uri, e))?;
                        symbols.push(SymbolInformation {
                            name: struct_def.name.clone(),
                            kind: SymbolKind::STRUCT,
                            location: Location {
                                uri: parsed_uri,
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
                            #[allow(deprecated)]
                            deprecated: None,
                            tags: Some(vec![]),
                        });
                    }
                }
                vex_ast::Item::Enum(enum_def) => {
                    if query.is_empty() || enum_def.name.contains(query) {
                        let parsed_uri = Url::parse(uri)
                            .map_err(|e| format!("Invalid URI '{}': {}", uri, e))?;
                        symbols.push(SymbolInformation {
                            name: enum_def.name.clone(),
                            kind: SymbolKind::ENUM,
                            location: Location {
                                uri: parsed_uri,
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
                        let parsed_uri = Url::parse(uri)
                            .map_err(|e| format!("Invalid URI '{}': {}", uri, e))?;
                        symbols.push(SymbolInformation {
                            name: func_def.name.clone(),
                            kind: SymbolKind::FUNCTION,
                            location: Location {
                                uri: parsed_uri,
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
                        let parsed_uri = Url::parse(uri)
                            .map_err(|e| format!("Invalid URI '{}': {}", uri, e))?;
                        symbols.push(SymbolInformation {
                            name: contract_def.name.clone(),
                            kind: SymbolKind::INTERFACE,
                            location: Location {
                                uri: parsed_uri,
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
                            let parsed_uri = Url::parse(uri)
                                .map_err(|e| format!("Invalid URI '{}': {}", uri, e))?;
                            symbols.push(SymbolInformation {
                                name: method.name.clone(),
                                kind: SymbolKind::METHOD,
                                location: Location {
                                    uri: parsed_uri,
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
        Ok(())
    }
}
