# Vex Language Server Protocol (LSP) Implementation Plan

## 📋 Overview

Implement a full-featured Language Server Protocol server for Vex in Rust, integrated with VS Code extension.

## 🏗️ Architecture

```
vex-lsp/                          # Rust LSP server (new crate)
├── Cargo.toml
└── src/
    ├── main.rs                   # LSP server entry point
    ├── server.rs                 # Main server implementation
    ├── handlers/
    │   ├── hover.rs              # Type info on hover
    │   ├── diagnostics.rs        # Syntax/semantic errors
    │   ├── completion.rs         # Auto-complete
    │   ├── definition.rs         # Go to definition
    │   ├── references.rs         # Find all references
    │   └── formatting.rs         # Code formatting
    ├── analyzer/
    │   ├── mod.rs                # Semantic analysis
    │   ├── type_checker.rs       # Type inference
    │   └── scope.rs              # Symbol resolution
    └── protocol/
        ├── types.rs              # LSP type definitions
        └── messages.rs           # LSP message handling

vscode-ext/                       # VS Code client extension
├── package.json                  # Add LSP activation
├── client/                       # TypeScript client (new)
│   ├── src/
│   │   ├── extension.ts         # Main extension entry
│   │   └── client.ts            # LSP client wrapper
│   ├── package.json
│   └── tsconfig.json
└── server/                       # Points to vex-lsp binary
    └── vex-lsp                   # Symlink to ../target/release/vex-lsp
```

## 🎯 Features to Implement

### Phase 1: Core LSP (Week 1-2)

- [ ] **Hover**: Show type information, function signatures, documentation
- [ ] **Diagnostics**: Real-time syntax errors from vex-parser
- [ ] **Document Symbols**: Outline view (structs, functions, traits)
- [ ] **Workspace Symbols**: Global symbol search

### Phase 2: Semantic Analysis (Week 3-4)

- [ ] **Type Checking**: Integrate vex-compiler type checker
- [ ] **Borrow Checker Integration**: Show borrow errors as diagnostics
- [ ] **Semantic Highlighting**: Color based on semantic meaning
- [ ] **Inlay Hints**: Show inferred types, parameter names

### Phase 3: Navigation (Week 5)

- [ ] **Go to Definition**: Jump to function/type/variable declaration
- [ ] **Find References**: Find all usages of symbol
- [ ] **Go to Implementation**: Jump to trait impl
- [ ] **Call Hierarchy**: Show caller/callee tree

### Phase 4: Refactoring (Week 6)

- [ ] **Code Completion**: Context-aware suggestions
  - Keywords, types, functions
  - Struct fields, enum variants
  - Import suggestions
- [ ] **Rename Symbol**: Rename across workspace
- [ ] **Code Actions**: Quick fixes
  - Add missing import
  - Implement missing trait method
  - Convert `let` to `let!`

### Phase 5: Advanced (Week 7-8)

- [ ] **Signature Help**: Parameter hints while typing
- [ ] **Code Lens**: Inline run/test buttons
- [ ] **Formatting**: rustfmt-like formatter for Vex
- [ ] **Folding Ranges**: Smart code folding
- [ ] **Selection Range**: Smart expand/shrink selection

## 🛠️ Technology Stack

### Rust LSP Server

```toml
[dependencies]
tower-lsp = "0.20"           # LSP framework
tokio = "1.0"                 # Async runtime
serde = "1.0"                 # Serialization
serde_json = "1.0"            # JSON protocol
lsp-types = "0.95"            # LSP type definitions
ropey = "1.6"                 # Rope data structure for text
dashmap = "5.5"               # Concurrent HashMap

# Vex compiler crates (already exists)
vex-lexer = { path = "../vex-lexer" }
vex-parser = { path = "../vex-parser" }
vex-ast = { path = "../vex-ast" }
vex-compiler = { path = "../vex-compiler" }
```

### TypeScript VS Code Client

```json
{
  "dependencies": {
    "vscode-languageclient": "^9.0.0",
    "vscode": "^1.85.0"
  },
  "devDependencies": {
    "@types/node": "^20.0.0",
    "@types/vscode": "^1.85.0",
    "typescript": "^5.3.0"
  }
}
```

## 📝 Implementation Steps

### Step 1: Setup Project Structure

```bash
# Create LSP crate
cargo new --lib vex-lsp
cd vex-lsp
cargo add tower-lsp tokio lsp-types serde serde_json ropey dashmap

# Add to workspace Cargo.toml
[workspace]
members = [
    "vex-lexer",
    "vex-parser",
    "vex-ast",
    "vex-compiler",
    "vex-cli",
    "vex-runtime",
    "vex-lsp"  # NEW
]
```

### Step 2: Minimal LSP Server

```rust
// vex-lsp/src/main.rs
use tower_lsp::{LspService, Server};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

struct VexLanguageServer {
    client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for VexLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions::default(),
                )),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        // TODO: Implement hover
        Ok(None)
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| VexLanguageServer { client });

    Server::new(stdin, stdout, socket).serve(service).await;
}
```

### Step 3: VS Code Extension Client

```typescript
// vscode-ext/client/src/extension.ts
import * as path from "path";
import { workspace, ExtensionContext } from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/node";

let client: LanguageClient;

export function activate(context: ExtensionContext) {
  const serverPath = path.join(context.extensionPath, "server", "vex-lsp");

  const serverOptions: ServerOptions = {
    run: { command: serverPath },
    debug: { command: serverPath },
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "vex" }],
    synchronize: {
      fileEvents: workspace.createFileSystemWatcher("**/*.vx"),
    },
  };

  client = new LanguageClient(
    "vexLanguageServer",
    "Vex Language Server",
    serverOptions,
    clientOptions
  );

  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
```

### Step 4: Update package.json

```json
{
  "main": "./client/out/extension.js",
  "activationEvents": ["onLanguage:vex"],
  "scripts": {
    "compile": "tsc -b",
    "watch": "tsc -b -w",
    "build-server": "cd ../vex-lsp && cargo build --release"
  }
}
```

## 🔄 Integration with Existing Crates

### Use vex-parser for Diagnostics

```rust
// vex-lsp/src/handlers/diagnostics.rs
use vex_parser::Parser;
use lsp_types::Diagnostic;

pub fn parse_and_diagnose(source: &str, uri: &Url) -> Vec<Diagnostic> {
    let mut parser = Parser::new(source);
    match parser.parse_program() {
        Ok(_) => vec![],  // No errors
        Err(errors) => errors.iter().map(|e| {
            Diagnostic {
                range: error_to_range(e),
                severity: Some(DiagnosticSeverity::ERROR),
                message: e.to_string(),
                source: Some("vex-parser".to_string()),
                ..Default::default()
            }
        }).collect()
    }
}
```

### Use vex-compiler for Type Checking

```rust
// vex-lsp/src/analyzer/type_checker.rs
use vex_compiler::borrow_checker::BorrowChecker;

pub fn check_borrows(ast: &Program) -> Vec<Diagnostic> {
    let mut checker = BorrowChecker::new();
    checker.check(ast)
        .map_err(|errors| errors_to_diagnostics(errors))
        .unwrap_or_default()
}
```

## 📊 State Management

```rust
// vex-lsp/src/server.rs
use dashmap::DashMap;
use std::sync::Arc;

pub struct ServerState {
    // Document cache (URI -> content)
    documents: Arc<DashMap<Url, String>>,

    // AST cache (URI -> parsed AST)
    asts: Arc<DashMap<Url, Program>>,

    // Symbol table (workspace symbols)
    symbols: Arc<DashMap<String, SymbolInfo>>,
}
```

## 🧪 Testing Strategy

1. **Unit Tests**: Each handler in isolation
2. **Integration Tests**: Full LSP protocol flow
3. **VS Code Integration**: Manual testing in real editor
4. **Performance**: Benchmark large files (1000+ lines)

## 📦 Distribution

### Development

```bash
# Build LSP server
cargo build --release --bin vex-lsp

# Symlink in extension
cd vscode-ext/server
ln -s ../../target/release/vex-lsp vex-lsp

# Install extension (existing install.sh)
cd ../
./install.sh
```

### Production

- Package as `.vsix` with bundled binary
- Separate binaries for Linux/macOS/Windows
- Auto-download on first run if missing

## 🎯 Success Criteria

- [ ] Hover shows correct types (90%+ accuracy)
- [ ] Diagnostics appear within 500ms of typing
- [ ] Go to definition works for all symbols
- [ ] No crashes on invalid syntax
- [ ] Memory usage < 100MB per workspace
- [ ] Works on 10,000+ line codebases

## 📅 Timeline

- **Week 1-2**: Core LSP + Diagnostics
- **Week 3-4**: Type checking + Hover
- **Week 5**: Navigation features
- **Week 6**: Code completion + Refactoring
- **Week 7-8**: Advanced features + Polish

**Total: ~2 months part-time development**

## 🔗 References

- [LSP Specification](https://microsoft.github.io/language-server-protocol/)
- [tower-lsp Documentation](https://docs.rs/tower-lsp/)
- [vscode-languageclient](https://www.npmjs.com/package/vscode-languageclient)
- [rust-analyzer](https://github.com/rust-lang/rust-analyzer) (similar implementation)
