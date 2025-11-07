#!/bin/bash
# Check Vex VS Code Extension Installation Status

EXTENSION_DIR="$HOME/.vscode/extensions/vex-language-0.2.0"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "🔍 Checking Vex VS Code Extension Status..."
echo ""

# Check if extension directory exists
if [ -L "$EXTENSION_DIR" ]; then
    echo "✅ Extension linked: $EXTENSION_DIR"
    echo "   Target: $(readlink $EXTENSION_DIR)"
    
    # Check if link is valid
    if [ -e "$EXTENSION_DIR" ]; then
        echo "   Status: 🟢 Valid symlink"
    else
        echo "   Status: 🔴 Broken symlink"
    fi
elif [ -d "$EXTENSION_DIR" ]; then
    echo "⚠️  Extension exists as directory (not symlink)"
    echo "   Path: $EXTENSION_DIR"
else
    echo "❌ Extension not installed"
    echo "   Expected: $EXTENSION_DIR"
fi

echo ""

# Check extension files
if [ -e "$EXTENSION_DIR/package.json" ]; then
    echo "✅ package.json found"
else
    echo "❌ package.json missing"
fi

if [ -e "$EXTENSION_DIR/out/extension.js" ]; then
    echo "✅ Compiled extension.js found"
else
    echo "❌ extension.js missing (run: cd client && npm run compile)"
fi

echo ""

# Check LSP binary
LSP_DEBUG="$HOME/.cargo/target/debug/vex-lsp"
LSP_RELEASE="$HOME/.cargo/target/release/vex-lsp"

if [ -f "$LSP_RELEASE" ]; then
    echo "✅ LSP server (release): $LSP_RELEASE"
    ls -lh "$LSP_RELEASE"
elif [ -f "$LSP_DEBUG" ]; then
    echo "✅ LSP server (debug): $LSP_DEBUG"
    ls -lh "$LSP_DEBUG"
else
    echo "❌ LSP server not found"
    echo "   Build: cargo build --release -p vex-lsp"
fi

echo ""

# Check if LSP is running
if pgrep -f vex-lsp > /dev/null; then
    echo "🟢 LSP server is running"
    ps aux | grep vex-lsp | grep -v grep
else
    echo "⚪ LSP server not running (will start when .vx file opened)"
fi

echo ""
echo "📋 Quick Actions:"
echo "  Install/Update: cd vscode-vex && ./install.sh"
echo "  Uninstall:      rm -rf $EXTENSION_DIR"
echo "  Reload VS Code: Cmd+Shift+P → 'Developer: Reload Window'"
echo "  Check logs:     VS Code → Output → 'Vex Language Server'"
