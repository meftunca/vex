# Vex LSP Quick Start Guide

## 🚀 Getting Started

### Prerequisites

```bash
# Install Rust nightly (for async)
rustup update

# Install Node.js & TypeScript
brew install node
npm install -g typescript

# Install VS Code Extension dependencies
cd vscode-ext/client
npm install
```

### Build & Run

```bash
# 1. Build LSP server
cargo build --release --bin vex-lsp

# 2. Compile TypeScript client
cd vscode-ext/client
npm run compile

# 3. Create symlink
cd ../server
ln -sf ../../target/release/vex-lsp vex-lsp

# 4. Install extension (already done)
cd ..
./install.sh

# 5. Reload VS Code
# Press: Cmd+Shift+P -> "Developer: Reload Window"
```

## 🔧 Development Workflow

### Terminal 1: LSP Server (Watch Mode)

```bash
cargo watch -x 'build --release --bin vex-lsp'
```

### Terminal 2: VS Code Extension (Watch Mode)

```bash
cd vscode-ext/client
npm run watch
```

### Terminal 3: Test

```bash
# Open test file in VS Code
code examples/01_basics/hello_world.vx

# Or use LSP test suite
cargo test -p vex-lsp
```

## 🐛 Debugging

### Debug LSP Server

```bash
# Enable logging
RUST_LOG=debug cargo run --bin vex-lsp

# Or attach debugger
lldb target/debug/vex-lsp
```

### Debug VS Code Extension

1. Open `vscode-ext/` in VS Code
2. Press F5 (Launch Extension)
3. New VS Code window opens with extension loaded
4. Open Vex file and test features

## 📝 Implementation Checklist

### Phase 1: Minimal LSP (Start Here)

- [ ] Create `vex-lsp/` crate
- [ ] Add dependencies (tower-lsp, tokio)
- [ ] Implement basic server skeleton
- [ ] Handle `initialize` request
- [ ] Handle `shutdown` request
- [ ] Test: Server starts and responds

### Phase 2: Diagnostics

- [ ] Integrate vex-parser
- [ ] Parse on document open/change
- [ ] Convert parse errors to LSP diagnostics
- [ ] Send diagnostics to client
- [ ] Test: Syntax errors show in VS Code

### Phase 3: Hover

- [ ] Build symbol table from AST
- [ ] Implement position -> symbol lookup
- [ ] Format type information
- [ ] Return hover response
- [ ] Test: Hover shows "i32" on variable

### Phase 4: Completion

- [ ] Collect keywords, types, functions
- [ ] Implement context-aware filtering
- [ ] Return completion items
- [ ] Test: Type "fn" and see suggestions

### Phase 5: Go to Definition

- [ ] Track symbol definitions in AST
- [ ] Map positions to definitions
- [ ] Return definition location
- [ ] Test: Ctrl+Click jumps to function

## 🧪 Test Cases

### Diagnostics

```vex
// test_diagnostics.vx
fn main( {  // Missing closing paren - should error
    let x = ;  // Missing value - should error
}
```

### Hover

```vex
// test_hover.vx
fn main(): i32 {
    let x = 42;  // Hover on 'x' -> "let x: i32"
    return x;
}
```

### Completion

```vex
// test_completion.vx
fn main(): i32 {
    let x = 42;
    x.  // Should suggest methods
    return 0;
}
```

### Go to Definition

```vex
// test_definition.vx
fn add(a: i32, b: i32): i32 {
    return a + b;
}

fn main(): i32 {
    return add(1, 2);  // Ctrl+Click 'add' -> jumps to definition
}
```

## 📊 Performance Targets

| Feature     | Target Latency | Current |
| ----------- | -------------- | ------- |
| Diagnostics | < 500ms        | TBD     |
| Hover       | < 100ms        | TBD     |
| Completion  | < 200ms        | TBD     |
| Go to Def   | < 50ms         | TBD     |

## 🎯 Next Steps

1. **Today**: Read tower-lsp docs, create vex-lsp crate
2. **Tomorrow**: Implement basic server skeleton
3. **Week 1**: Get diagnostics working
4. **Week 2**: Add hover support
5. **Month 1**: Complete Phase 1-3
6. **Month 2**: Complete Phase 4-5

## 📚 Learning Resources

- [LSP Tutorial](https://code.visualstudio.com/api/language-extensions/language-server-extension-guide)
- [tower-lsp Examples](https://github.com/ebkalderon/tower-lsp/tree/master/examples)
- [rust-analyzer Architecture](https://github.com/rust-lang/rust-analyzer/blob/master/docs/dev/architecture.md)
