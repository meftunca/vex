# Vex VS Code Extension - Quick Setup Guide

## 🚀 One-Command Install

```bash
cd /Users/mapletechnologies/Desktop/big_projects/vex_lang/vscode-vex
./install.sh
```

Then in VS Code:

1. `Cmd+Shift+P` → `Developer: Reload Window`
2. Open any `.vx` file
3. Check `Output` → `Vex Language Server`

## ✨ Features

### Current (v0.2.0)

- ✅ **Syntax Highlighting** - Full Vex v0.9 syntax
- ✅ **Language Server** - Real-time diagnostics
- ✅ **Code Snippets** - `main`, `fn`, `struct`, etc.
- ✅ **Theme** - Vex Dark theme
- ✅ **Auto-completion** (basic)

### Coming Soon

- 🚧 Type information on hover
- 🚧 Go to definition
- 🚧 Find all references
- 🚧 Code formatting

## 📦 What Gets Installed

```
~/.vscode/extensions/vex-language-0.2.0/  (symlink)
  ├── syntaxes/           # Syntax highlighting
  ├── snippets/           # Code snippets
  ├── themes/             # Color theme
  ├── out/extension.js    # TypeScript client (compiled)
  └── client/             # Source files

~/.cargo/target/debug/vex-lsp            # LSP server binary
```

## 🔧 Development Commands

```bash
# Rebuild LSP server
cargo build -p vex-lsp

# Rebuild extension client
cd vscode-vex/client && npm run compile

# Reinstall
cd vscode-vex && ./install.sh

# Restart LSP in VS Code
Cmd+Shift+P → "Vex: Restart Language Server"
```

## 🧪 Testing

```bash
# Open test file with syntax errors
code vscode-vex/test_lsp.vx

# Should see red squiggles on:
# - Line 9: Missing semicolon
# - Line 12: Undefined variable
# - Line 18: Missing function body
```

## 📊 Status

See [LSP_STATUS.md](./LSP_STATUS.md) for detailed implementation progress.

**Current Phase**: LSP Foundation Complete ✅  
**Next Phase**: Parser Span Integration (syntax error positions)

## 🐛 Troubleshooting

**LSP not starting?**

```bash
# Check binary exists
ls -lh ~/.cargo/target/debug/vex-lsp

# Check VS Code logs
# Output panel → "Vex Language Server"
```

**No syntax highlighting?**

```bash
# Verify extension installed
ls -lh ~/.vscode/extensions/ | grep vex

# Reload VS Code
Cmd+Shift+P → "Developer: Reload Window"
```

**Need to uninstall?**

```bash
rm -rf ~/.vscode/extensions/vex-language-0.2.0
```

---

**Built with**: Rust (LSP server) + TypeScript (VS Code client)  
**Version**: 0.2.0 (LSP Foundation)  
**License**: MIT
