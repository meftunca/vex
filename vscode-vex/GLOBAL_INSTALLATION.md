# Vex VS Code Extension - Global Installation Guide

## ✅ Current Status

Your extension is **already globally installed** via symlink:

```bash
~/.vscode/extensions/vex-language-0.2.0 → vex_lang/vscode-vex
```

VS Code recognizes it as: `meftunca.vex-language`

---

## 🔍 Check Installation

```bash
cd /Users/mapletechnologies/Desktop/big_projects/vex_lang/vscode-vex
./check_status.sh
```

**Expected output:**

- ✅ Extension linked
- ✅ package.json found
- ✅ Compiled extension.js found
- ✅ LSP server (release/debug)

---

## 🚀 How It Works

### 1. Symlink Structure

```
~/.vscode/extensions/
└── vex-language-0.2.0/          # Symlink
    → /path/to/vex_lang/vscode-vex/   # Your dev folder
        ├── package.json          # Extension manifest
        ├── out/extension.js      # Compiled TypeScript
        ├── syntaxes/            # Syntax highlighting
        ├── snippets/            # Code snippets
        └── themes/              # Color themes
```

**Benefits:**

- ✅ Edit source → Changes instantly reflected
- ✅ No need to reinstall after changes
- ✅ Single source of truth
- ✅ Works across all VS Code windows

### 2. Extension Discovery

VS Code automatically loads extensions from:

1. **System:** `/Applications/Visual Studio Code.app/Contents/Resources/app/extensions/`
2. **User (Global):** `~/.vscode/extensions/` ← **Your extension here!**
3. **Workspace:** `.vscode/extensions/`

---

## 📦 Installation Methods

### Method 1: Symlink (Current - Recommended for Development)

**Pros:**

- ✅ Instant updates
- ✅ No reinstall needed
- ✅ Easy debugging

**Cons:**

- ⚠️ Requires source code present

```bash
cd vscode-vex
./install.sh
```

**What it does:**

1. Builds LSP server (if missing)
2. Compiles TypeScript client
3. Creates symlink: `~/.vscode/extensions/vex-language-0.2.0`
4. Points to your dev folder

### Method 2: VSIX Package (For Distribution)

**Pros:**

- ✅ Portable
- ✅ No source needed
- ✅ Can publish to marketplace

**Cons:**

- ⚠️ Must rebuild after changes

```bash
# 1. Install vsce (VS Code Extension Manager)
npm install -g @vscode/vsce

# 2. Package extension
cd vscode-vex
vsce package
# Creates: vex-language-0.2.0.vsix

# 3. Install VSIX
code --install-extension vex-language-0.2.0.vsix

# Or via UI:
# Extensions → ... → Install from VSIX
```

### Method 3: Marketplace (Future)

**Requirements:**

- Publisher account
- Extension ID: `meftunca.vex-language`

```bash
# Publish to marketplace
vsce publish
```

---

## 🔄 Update Extension

### If Using Symlink (Current Setup)

**No reinstall needed!** Just edit and reload:

```bash
# 1. Edit source files
vim vscode-vex/client/src/extension.ts

# 2. Recompile TypeScript
cd vscode-vex/client
npm run compile

# 3. Reload VS Code
# Cmd+Shift+P → "Developer: Reload Window"
```

**For LSP changes:**

```bash
# Rebuild LSP server
cargo build --release -p vex-lsp

# Copy to debug location (if needed)
cp ~/.cargo/target/release/vex-lsp ~/.cargo/target/debug/vex-lsp

# Restart LSP in VS Code
# Cmd+Shift+P → "Vex: Restart Language Server"
```

### If Using VSIX

```bash
# 1. Make changes
# 2. Rebuild package
vsce package
# 3. Reinstall
code --install-extension vex-language-0.2.0.vsix --force
```

---

## 🗑️ Uninstall

### Remove Symlink

```bash
rm -rf ~/.vscode/extensions/vex-language-0.2.0
```

### Via VS Code UI

1. Extensions panel (Cmd+Shift+X)
2. Find "Vex Language Support"
3. Click gear → Uninstall

### Via Command Line

```bash
code --uninstall-extension meftunca.vex-language
```

---

## 🧪 Testing

### Test 1: Extension Loads

```bash
# 1. Check if recognized
code --list-extensions | grep vex
# Should show: meftunca.vex-language

# 2. Open any .vx file
code test.vx

# 3. Check activation
# Output panel → "Vex Language Server"
# Should see: "Starting Vex Language Server..."
```

### Test 2: LSP Diagnostics

```bash
# Create test file with syntax error
cat > test_error.vx << 'EOF'
fn main(): i32 {
    let broken = "test"  // Missing semicolon
    return 0;
}
EOF

# Open in VS Code
code test_error.vx

# Expected: Red squiggle at line 2, column 24
```

### Test 3: Syntax Highlighting

```bash
# Open any .vx file
code examples/01_basics/hello.vx

# Should see:
# - Keywords colored (fn, let, return)
# - Strings highlighted
# - Comments grayed out
```

---

## 🐛 Troubleshooting

### Extension Not Found

**Symptom:** `code --list-extensions` doesn't show vex

**Solution:**

```bash
# Check symlink
ls -la ~/.vscode/extensions/ | grep vex

# Recreate if missing
cd vscode-vex && ./install.sh

# Reload VS Code
```

### LSP Not Starting

**Symptom:** No "Vex Language Server" in Output panel

**Check 1: Binary exists**

```bash
ls -lh ~/.cargo/target/debug/vex-lsp
ls -lh ~/.cargo/target/release/vex-lsp
```

**Check 2: Extension activated**

```bash
# Open Developer Tools
# Help → Toggle Developer Tools
# Console should show: "Vex Language Extension activating..."
```

**Check 3: File association**

```bash
# Ensure file ends with .vx
mv test.txt test.vx
```

### Syntax Highlighting Not Working

**Symptom:** All text same color

**Solution:**

```bash
# 1. Check language ID
# Bottom right of VS Code → Should show "Vex"

# 2. Manually set language
# Cmd+Shift+P → "Change Language Mode" → "Vex"

# 3. Reload extension
cd vscode-vex && ./install.sh
```

### Changes Not Reflecting

**Symptom:** Edited extension.ts but no effect

**Solution:**

```bash
# 1. Recompile TypeScript
cd vscode-vex/client
npm run compile

# 2. Check output
ls -lh ../out/extension.js
# Should have recent timestamp

# 3. Hard reload VS Code
# Cmd+Shift+P → "Developer: Reload Window"
```

---

## 📊 Extension Info

```bash
# Location
~/.vscode/extensions/vex-language-0.2.0

# Publisher
meftunca

# Extension ID
meftunca.vex-language

# Version
0.2.0

# Activation Events
- onLanguage:vex

# Main Entry Point
out/extension.js

# LSP Binary Locations (searched in order)
1. ~/.cargo/target/debug/vex-lsp
2. ~/.cargo/target/release/vex-lsp
3. {workspace}/target/debug/vex-lsp
4. {workspace}/target/release/vex-lsp
```

---

## 🔧 Development Workflow

### Daily Development

```bash
# 1. Edit extension code
vim vscode-vex/client/src/extension.ts

# 2. Compile
cd vscode-vex/client && npm run compile

# 3. Reload VS Code
# Cmd+Shift+P → "Developer: Reload Window"

# 4. Test
code test.vx
```

### LSP Changes

```bash
# 1. Edit LSP server
vim vex-lsp/src/backend.rs

# 2. Rebuild
cargo build --release -p vex-lsp

# 3. Restart LSP
# Cmd+Shift+P → "Vex: Restart Language Server"
```

### Add New Feature

```bash
# 1. Edit package.json (add command/config)
vim vscode-vex/package.json

# 2. Implement in extension.ts
vim vscode-vex/client/src/extension.ts

# 3. Compile & reload
cd vscode-vex/client && npm run compile
# Then reload VS Code

# 4. Test new feature
```

---

## 📝 Quick Reference

```bash
# Check status
./check_status.sh

# Install/Update
./install.sh

# Uninstall
rm -rf ~/.vscode/extensions/vex-language-0.2.0

# Rebuild TypeScript
cd client && npm run compile

# Rebuild LSP
cargo build --release -p vex-lsp

# Restart LSP (in VS Code)
Cmd+Shift+P → "Vex: Restart Language Server"

# Reload VS Code
Cmd+Shift+P → "Developer: Reload Window"

# Package for distribution
vsce package

# View logs
VS Code → Output → "Vex Language Server"
```

---

## ✅ Verification Checklist

- [x] Symlink exists: `~/.vscode/extensions/vex-language-0.2.0`
- [x] VS Code recognizes: `meftunca.vex-language`
- [x] package.json present
- [x] extension.js compiled
- [x] LSP binary built
- [ ] **Extension activates on .vx file** (test in VS Code)
- [ ] **Syntax highlighting works** (test in VS Code)
- [ ] **LSP diagnostics show** (test with syntax error)

---

**Current Status:** ✅ **Extension Globally Installed via Symlink**  
**Ready for:** Development, Testing, Daily Use  
**Distribution:** Use `vsce package` when ready to share
