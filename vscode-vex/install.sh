#!/bin/bash
# Install Vex VS Code Extension via Symlink (with LSP support)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXTENSION_DIR="$HOME/.vscode-insiders/extensions/vex-language-0.9.2"

echo "🔧 Installing Vex Language Support for VS Code..."

# Build LSP server if not present
LSP_DEBUG="$HOME/.cargo/target/debug/vex-lsp"
LSP_RELEASE="$HOME/.cargo/target/release/vex-lsp"

if [ ! -f "$LSP_DEBUG" ] && [ ! -f "$LSP_RELEASE" ]; then
    echo "🦀 Building LSP server..."
    cd "$SCRIPT_DIR/.." && cargo build -p vex-lsp
    if [ $? -ne 0 ]; then
        echo "❌ LSP build failed!"
        exit 1
    fi
fi

# Build TypeScript client if needed
if [ ! -f "$SCRIPT_DIR/out/extension.js" ]; then
    echo "📦 Building TypeScript client..."
    cd "$SCRIPT_DIR/client" && npm install && npm run compile
    if [ $? -ne 0 ]; then
        echo "❌ Client build failed!"
        exit 1
    fi
fi

# Remove existing extension if present
if [ -L "$EXTENSION_DIR" ] || [ -d "$EXTENSION_DIR" ]; then
    echo "📦 Removing existing extension..."
    rm -rf "$EXTENSION_DIR"
fi

# Create symlink
echo "🔗 Creating symlink..."
ln -s "$SCRIPT_DIR" "$EXTENSION_DIR"

if [ $? -eq 0 ]; then
    echo "✅ Successfully installed!"
    echo ""
    echo "📋 Next steps:"
    echo "1. Reload VS Code window (Cmd+Shift+P -> 'Developer: Reload Window')"
    echo "2. Open any .vx file to activate LSP"
    echo "3. Check 'Output' -> 'Vex Language Server' for diagnostics"
    echo "4. Try syntax errors - they should show red squiggles!"
    echo ""
    echo "🔧 Commands:"
    echo "  - Vex: Restart Language Server (Cmd+Shift+P)"
    echo ""
    echo "💡 LSP binary: $(ls -1 $HOME/.cargo/target/*/vex-lsp 2>/dev/null | head -1)"
    echo "💡 To uninstall: rm -rf $EXTENSION_DIR"
else
    echo "❌ Installation failed!"
    exit 1
fi
