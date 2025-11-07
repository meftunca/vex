# Building Vex VS Code Extension with LSP

## Prerequisites

```bash
# Install Node.js and npm (if not already installed)
# On macOS:
brew install node

# Install vsce (VS Code Extension Manager)
npm install -g @vscode/vsce
```

## Build Steps

### 1. Build LSP Server (Rust)

```bash
cd /Users/mapletechnologies/Desktop/big_projects/vex_lang
cargo build --release -p vex-lsp
```

This creates: `~/.cargo/target/release/vex-lsp`

### 2. Build VS Code Extension (TypeScript)

```bash
cd vscode-vex/client
npm install
npm run compile
```

This compiles TypeScript to: `vscode-vex/out/extension.js`

### 3. Package Extension

```bash
cd vscode-vex
vsce package
```

This creates: `vex-language-0.2.0.vsix`

### 4. Install Extension

**Option A: From VSIX (Recommended)**

```bash
code --install-extension vex-language-0.2.0.vsix
```

**Option B: Development Mode**

```bash
# From VS Code:
# 1. Press F5 or Run > Start Debugging
# 2. Opens new Extension Development Host window
# 3. Open a .vx file to activate LSP
```

### 5. Verify LSP is Running

1. Open any `.vx` file
2. Check "Output" panel → Select "Vex Language Server"
3. Should see: `Starting Vex Language Server...`

## Quick Rebuild

After changes:

```bash
# Rebuild LSP server
cargo build -p vex-lsp

# Rebuild client
cd vscode-vex/client && npm run compile

# Restart VS Code extension (Cmd+Shift+P → "Reload Window")
# Or use command: "Vex: Restart Language Server"
```

## Troubleshooting

**LSP not starting?**

- Check binary exists: `ls ~/.cargo/target/debug/vex-lsp`
- Build in debug mode: `cargo build -p vex-lsp`
- Check logs: VS Code Output panel → "Vex Language Server"

**Syntax errors not showing?**

- Open a `.vx` file with syntax errors
- Wait 1-2 seconds for parsing
- Check if red squiggles appear

**Extension not activating?**

- Open Developer Tools: Help → Toggle Developer Tools
- Check Console for errors
- Verify `out/extension.js` exists

## Current Status

✅ LSP server builds successfully
✅ Client TypeScript structure ready
✅ Extension activation configured
⚠️ Need to run: `npm install` in client/
⚠️ Need to compile TypeScript
⚠️ Need to test in VS Code
