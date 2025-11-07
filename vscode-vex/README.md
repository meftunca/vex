# Vex Language - VS Code Extension

**Status:** ✅ Globally Installed  
**Extension ID:** `meftunca.vex-language`  
**Version:** 0.2.0

## 🚀 Quick Start

### Check Status

```bash
./check_status.sh
```

### Install/Update

```bash
./install.sh
```

### Reload VS Code

```
Cmd+Shift+P → "Developer: Reload Window"
```

---

## 📚 Documentation

- **[GLOBAL_INSTALLATION.md](./GLOBAL_INSTALLATION.md)** - Complete installation guide
- **[LSP_STATUS.md](./LSP_STATUS.md)** - LSP implementation status
- **[BUILD.md](./BUILD.md)** - Build instructions

---

## ✨ Features

### Current (v0.2.0)

- ✅ **Syntax Highlighting** - Full Vex v0.9 syntax
- ✅ **Language Server (LSP)** - Real-time diagnostics with exact positions
- ✅ **Code Snippets** - `main`, `fn`, `struct`, `trait`, etc.
- ✅ **Vex Dark Theme** - Custom color scheme
- ✅ **Commands** - Restart Language Server

### Coming Soon

- 🚧 Hover type information
- 🚧 Go to definition
- 🚧 Auto-completion
- 🚧 Code formatting

---

## 🔍 Current Installation

```bash
# Extension location (symlink)
~/.vscode/extensions/vex-language-0.2.0
→ /Users/mapletechnologies/Desktop/big_projects/vex_lang/vscode-vex

# LSP server
~/.cargo/target/release/vex-lsp (3.2 MB)

# Status
✅ Extension globally installed
✅ TypeScript compiled (out/extension.js)
✅ LSP server built
⏳ Waiting for .vx file to activate
```

---

## 📋 Quick Commands

```bash
# Development
cd client && npm run compile          # Recompile TypeScript
cargo build --release -p vex-lsp      # Rebuild LSP server

# Testing
code test.vx                          # Open test file
code --list-extensions | grep vex     # Verify installation

# Maintenance
./check_status.sh                     # Check everything
./uninstall.sh                        # Remove extension
```

---

## 🧪 Test It

```bash
# 1. Create test file
cat > test_syntax_error.vx << 'EOF'
fn main(): i32 {
    let x = "missing semicolon"
    return 0;
}
EOF

# 2. Open in VS Code
code test_syntax_error.vx

# 3. Expected result
# Line 2: Red squiggle at "missing semicolon"
# Message: "Expected ';' after let statement"
# Position: test_syntax_error.vx:2:32
```

---

## 🔧 Development Mode

Since extension is **symlinked**, changes reflect immediately:

```bash
# 1. Edit source
vim client/src/extension.ts

# 2. Compile
cd client && npm run compile

# 3. Reload VS Code
# Cmd+Shift+P → "Developer: Reload Window"
```

No reinstall needed! 🎉

---

## 📊 Architecture

```
vscode-vex/                           # Extension source (symlinked)
├── package.json                      # Extension manifest
├── client/src/extension.ts           # TypeScript client
├── out/extension.js                  # Compiled output
├── syntaxes/vex.tmLanguage.json     # Syntax rules
├── snippets/vex.json                # Code snippets
└── themes/vex-dark.json             # Color theme

~/.cargo/target/release/vex-lsp      # Rust LSP server
├── Parse .vx files
├── Generate diagnostics
└── Communicate via stdio

VS Code
├── Loads extension from ~/.vscode/extensions/
├── Spawns vex-lsp process
├── Shows diagnostics as red squiggles
└── Sends hover/completion requests
```

---

## 🐛 Troubleshooting

### Extension not found?

```bash
ls -la ~/.vscode/extensions/ | grep vex
# If missing: ./install.sh
```

### LSP not starting?

```bash
ls -lh ~/.cargo/target/release/vex-lsp
# If missing: cargo build --release -p vex-lsp
```

### Syntax highlighting not working?

```bash
# Check language mode (bottom right)
# Should show: "Vex"
# Manually set: Cmd+Shift+P → "Change Language Mode" → "Vex"
```

### Changes not reflecting?

```bash
cd client && npm run compile
# Then reload VS Code
```

---

## 📝 Version History

### v0.2.0 (Current)

- ✅ LSP integration with exact error positions
- ✅ Parser span tracking (file:line:column)
- ✅ Real-time diagnostics
- ✅ Symlink installation

### v0.1.0

- ✅ Basic syntax highlighting
- ✅ Code snippets
- ✅ Vex Dark theme

---

## 🎯 Next Steps

1. **Test in VS Code** - Open any `.vx` file
2. **Check Output panel** - "Vex Language Server"
3. **Try syntax errors** - Should show red squiggles
4. **Test snippets** - Type `main` and press Tab

---

**Need Help?**

- Check: `./check_status.sh`
- Read: `GLOBAL_INSTALLATION.md`
- Logs: VS Code → Output → "Vex Language Server"

**Ready to use!** 🚀
