// Core LSP Backend structures and initialization

use dashmap::DashMap;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::module_resolver::ModuleResolver;

// Simple document cache for incremental parsing
#[derive(Debug)]
pub struct DocumentCache {
    // Placeholder - implement incremental parsing later
}

impl DocumentCache {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&self, _uri: &str, _text: String, _version: i32) -> CachedDocument {
        // Placeholder implementation
        CachedDocument {
            parse_errors: Vec::new(),
            ast: None,
        }
    }

    pub fn remove(&self, _uri: &str) {
        // Placeholder implementation
    }
}

#[derive(Debug)]
pub struct CachedDocument {
    pub parse_errors: Vec<vex_diagnostics::Diagnostic>,
    pub ast: Option<vex_ast::Program>,
}

pub struct VexBackend {
    pub client: Client,
    /// Document cache with incremental parsing
    pub document_cache: Arc<DocumentCache>,
    /// Document text cache (Arc for cheap cloning)
    pub documents: Arc<DashMap<String, Arc<String>>>,
    /// Parsed AST cache (Arc for cheap cloning)
    pub ast_cache: Arc<DashMap<String, Arc<vex_ast::Program>>>,
    /// Debounce tasks for did_change events (prevents UI freezing)
    pub debounce_tasks: Arc<RwLock<HashMap<String, tokio::task::JoinHandle<()>>>>,
    /// Module resolver for import path resolution
    pub module_resolver: Arc<ModuleResolver>,
    /// Workspace root path
    pub workspace_root: Option<PathBuf>,
}

impl VexBackend {
    pub fn new(client: Client) -> Self {
        // Default workspace root to current directory
        let workspace_root = std::env::current_dir().ok();
        let resolver = workspace_root
            .as_ref()
            .map(|root| ModuleResolver::new(root.clone()))
            .unwrap_or_else(|| ModuleResolver::new(PathBuf::from(".")));

        Self {
            client,
            document_cache: Arc::new(DocumentCache::new()),
            documents: Arc::new(DashMap::new()),
            ast_cache: Arc::new(DashMap::new()),
            debounce_tasks: Arc::new(RwLock::new(HashMap::new())),
            module_resolver: Arc::new(resolver),
            workspace_root,
        }
    }

    /// Set workspace root (called from initialize)
    pub fn set_workspace_root(&mut self, root: PathBuf) {
        self.workspace_root = Some(root.clone());
        self.module_resolver = Arc::new(ModuleResolver::new(root));
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for VexBackend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        // Extract workspace root from initialization params
        if let Some(root_uri) = params.root_uri {
            if let Ok(root_path) = root_uri.to_file_path() {
                tracing::info!("Workspace root: {:?}", root_path);
                // Note: Can't mutate self here, will need refactoring to use interior mutability
                // For now, resolver is created in new() with current_dir
            }
        }

        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "vex-language-server".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string()]),
                    ..Default::default()
                }),
                definition_provider: Some(OneOf::Left(true)),
                signature_help_provider: Some(SignatureHelpOptions {
                    trigger_characters: Some(vec!["(".to_string()]),
                    retrigger_characters: Some(vec![",".to_string()]),
                    ..Default::default()
                }),
                document_symbol_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Right(RenameOptions {
                    prepare_provider: Some(false),
                    work_done_progress_options: Default::default(),
                })),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            work_done_progress_options: Default::default(),
                            legend: SemanticTokensLegend {
                                token_types: vec![
                                    SemanticTokenType::NAMESPACE,
                                    SemanticTokenType::TYPE,
                                    SemanticTokenType::CLASS,
                                    SemanticTokenType::ENUM,
                                    SemanticTokenType::INTERFACE,
                                    SemanticTokenType::STRUCT,
                                    SemanticTokenType::TYPE_PARAMETER,
                                    SemanticTokenType::PARAMETER,
                                    SemanticTokenType::VARIABLE,
                                    SemanticTokenType::PROPERTY,
                                    SemanticTokenType::ENUM_MEMBER,
                                    SemanticTokenType::EVENT,
                                    SemanticTokenType::FUNCTION,
                                    SemanticTokenType::METHOD,
                                    SemanticTokenType::MACRO,
                                    SemanticTokenType::KEYWORD,
                                    SemanticTokenType::MODIFIER,
                                    SemanticTokenType::COMMENT,
                                    SemanticTokenType::STRING,
                                    SemanticTokenType::NUMBER,
                                    SemanticTokenType::REGEXP,
                                    SemanticTokenType::OPERATOR,
                                ],
                                token_modifiers: vec![
                                    SemanticTokenModifier::DECLARATION,
                                    SemanticTokenModifier::DEFINITION,
                                    SemanticTokenModifier::READONLY,
                                    SemanticTokenModifier::STATIC,
                                    SemanticTokenModifier::DEPRECATED,
                                    SemanticTokenModifier::ABSTRACT,
                                    SemanticTokenModifier::ASYNC,
                                    SemanticTokenModifier::MODIFICATION,
                                    SemanticTokenModifier::DOCUMENTATION,
                                    SemanticTokenModifier::DEFAULT_LIBRARY,
                                ],
                            },
                            range: Some(false),
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                            ..Default::default()
                        },
                    ),
                ),
                workspace_symbol_provider: Some(OneOf::Left(true)),
                // Enable formatting/range formatting
                document_formatting_provider: Some(OneOf::Left(true)),
                document_range_formatting_provider: Some(OneOf::Left(true)),
                folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),
                type_definition_provider: Some(TypeDefinitionProviderCapability::Simple(true)),
                implementation_provider: Some(ImplementationProviderCapability::Simple(true)),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Vex Language Server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        tracing::info!("LSP server shutting down");

        // Abort all pending debounce tasks to prevent hanging
        let tasks = self.debounce_tasks.write().await;
        for (uri, task) in tasks.iter() {
            tracing::debug!("Aborting debounce task for: {}", uri);
            task.abort();
        }
        drop(tasks);

        // Clear all caches
        self.documents.clear();
        self.ast_cache.clear();
        self.module_resolver.clear_cache();

        tracing::info!("LSP server shutdown complete");
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.did_open(params).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.did_change(params).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.did_close(params).await;
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        self.hover(params).await
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        self.completion(params).await
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        self.goto_definition(params).await
    }

    async fn signature_help(&self, params: SignatureHelpParams) -> Result<Option<SignatureHelp>> {
        self.signature_help(params).await
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        self.document_symbol(params).await
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        self.references(params).await
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        self.rename(params).await
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        self.formatting(params).await
    }

    async fn range_formatting(
        &self,
        params: DocumentRangeFormattingParams,
    ) -> Result<Option<Vec<TextEdit>>> {
        self.range_formatting(params).await
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        self.code_actions(params).await
    }

    async fn code_action_resolve(&self, params: CodeAction) -> Result<CodeAction> {
        self.code_action_resolve(params).await
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        self.semantic_tokens_full(params).await
    }

    async fn symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Option<Vec<SymbolInformation>>> {
        self.workspace_symbol(params).await
    }

    async fn folding_range(&self, params: FoldingRangeParams) -> Result<Option<Vec<FoldingRange>>> {
        self.folding_range(params).await
    }
}
