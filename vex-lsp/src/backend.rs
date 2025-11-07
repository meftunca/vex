// LSP Backend - Handles all LSP requests

use dashmap::DashMap;
use std::sync::Arc;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use vex_parser::Parser;

pub struct VexBackend {
    client: Client,
    /// Document cache: URI -> source code
    documents: Arc<DashMap<String, String>>,
    /// Parsed AST cache: URI -> AST
    ast_cache: Arc<DashMap<String, vex_ast::Program>>,
}

impl VexBackend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(DashMap::new()),
            ast_cache: Arc::new(DashMap::new()),
        }
    }

    /// Parse document and return diagnostics
    async fn parse_and_diagnose(&self, uri: &str, text: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Extract filename from URI
        let filename = uri.split('/').last().unwrap_or("unknown.vx");

        // Create parser with filename for better error reporting
        let parser = match vex_parser::Parser::new_with_file(filename, text) {
            Ok(p) => p,
            Err(e) => {
                // Convert parser creation error to diagnostic
                diagnostics.push(self.parse_error_to_diagnostic(&e, text));
                return diagnostics;
            }
        };

        // Parse the document
        let mut parser = parser;
        match parser.parse_file() {
            Ok(program) => {
                // Store parsed AST
                self.ast_cache.insert(uri.to_string(), program);

                // TODO: Run type checker and borrow checker
                // For now, no errors if parsing succeeds
            }
            Err(parse_error) => {
                // Convert parse error to LSP diagnostic with accurate span
                diagnostics.push(self.parse_error_to_diagnostic(&parse_error, text));
            }
        }

        diagnostics
    }

    /// Convert Vex ParseError to LSP Diagnostic with accurate position
    fn parse_error_to_diagnostic(
        &self,
        error: &vex_parser::ParseError,
        _source: &str,
    ) -> Diagnostic {
        use vex_parser::ParseError;

        match error {
            ParseError::SyntaxError { location, message } => {
                let range = Range {
                    start: Position {
                        line: location.line.saturating_sub(1) as u32,
                        character: location.column.saturating_sub(1) as u32,
                    },
                    end: Position {
                        line: location.line.saturating_sub(1) as u32,
                        character: (location.column + location.length).saturating_sub(1) as u32,
                    },
                };

                Diagnostic {
                    range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::String("E0001".to_string())),
                    source: Some("vex".to_string()),
                    message: message.clone(),
                    ..Default::default()
                }
            }
            ParseError::LexerError(msg) => Diagnostic {
                range: Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: 0,
                        character: 1,
                    },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                code: Some(NumberOrString::String("E0001".to_string())),
                source: Some("vex".to_string()),
                message: format!("Lexer error: {}", msg),
                ..Default::default()
            },
        }
    }

    /// Publish diagnostics to client
    async fn publish_diagnostics(&self, uri: Url, diagnostics: Vec<Diagnostic>) {
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for VexBackend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        tracing::info!("LSP client connected");

        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "vex-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![".".to_string(), ":".to_string()]),
                    ..Default::default()
                }),
                definition_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        tracing::info!("LSP server initialized");
        self.client
            .log_message(MessageType::INFO, "Vex LSP server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        tracing::info!("LSP server shutting down");
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        let text = params.text_document.text.clone();

        tracing::info!("Document opened: {}", uri);

        // Store document
        self.documents.insert(uri.clone(), text.clone());

        // Parse and send diagnostics
        let diagnostics = self.parse_and_diagnose(&uri, &text).await;
        self.publish_diagnostics(params.text_document.uri, diagnostics)
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.to_string();

        if let Some(change) = params.content_changes.first() {
            let text = change.text.clone();

            tracing::info!("Document changed: {}", uri);

            // Update document
            self.documents.insert(uri.clone(), text.clone());

            // Re-parse and send diagnostics
            let diagnostics = self.parse_and_diagnose(&uri, &text).await;
            self.publish_diagnostics(params.text_document.uri, diagnostics)
                .await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        tracing::info!("Document closed: {}", uri);

        // Remove from cache
        self.documents.remove(&uri);
        self.ast_cache.remove(&uri);
    }

    async fn hover(&self, _params: HoverParams) -> Result<Option<Hover>> {
        // TODO: Implement hover information
        // For now, return placeholder
        Ok(Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String(
                "Vex Language - Hover info coming soon!".to_string(),
            )),
            range: None,
        }))
    }

    async fn completion(&self, _params: CompletionParams) -> Result<Option<CompletionResponse>> {
        // TODO: Implement smart completion
        // For now, return basic keywords
        let keywords = vec![
            "fn", "let", "let!", "const", "struct", "enum", "trait", "impl", "if", "else", "match",
            "for", "while", "loop", "return", "break", "continue", "i32", "i64", "f32", "f64",
            "bool", "string", "Vec", "Box", "Option", "Result",
        ];

        let items: Vec<CompletionItem> = keywords
            .into_iter()
            .map(|keyword| CompletionItem {
                label: keyword.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                ..Default::default()
            })
            .collect();

        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn goto_definition(
        &self,
        _params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        // TODO: Implement go-to-definition
        Ok(None)
    }
}
