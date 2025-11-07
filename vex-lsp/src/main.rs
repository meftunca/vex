// Vex Language Server (LSP)
// Provides IDE features: diagnostics, completion, hover, etc.

use dashmap::DashMap;
use std::sync::Arc;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

mod backend;
mod diagnostics;
mod symbol_resolver;

pub use backend::VexBackend;

#[tokio::main]
async fn main() {
    // Initialize logging to a file
    let log_file = tracing_appender::rolling::daily("/tmp", "vex-lsp.log");
    let (non_blocking_writer, _guard) = tracing_appender::non_blocking(log_file);
    tracing_subscriber::fmt()
        .with_writer(non_blocking_writer)
        .with_max_level(tracing::Level::INFO)
        .with_ansi(false) // Disable ANSI colors in log file
        .init();

    tracing::info!("Starting Vex Language Server...");

    // Create LSP service
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| VexBackend::new(client));

    Server::new(stdin, stdout, socket).serve(service).await;
}
