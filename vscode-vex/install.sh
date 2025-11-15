#!/bin/bash
# Install Vex VS Code Extension via Symlink (with LSP support)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# Read version from package.json
VERSION=$(node -pe "require('./package.json').version")
EXTENSION_DIR_STABLE="$HOME/.vscode/extensions/vex-language-$VERSION"
EXTENSION_DIR_INSIDERS="$HOME/.vscode-insiders/extensions/vex-language-$VERSION"

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

# Ensure dependencies and build TypeScript client if needed
if [ ! -f "$SCRIPT_DIR/out/extension.js" ]; then
    echo "📦 Installing root and client dependencies and building TypeScript client..."
    cd "$SCRIPT_DIR" && npm ci
    cd "$SCRIPT_DIR/client" && npm ci && npm run compile
    if [ $? -ne 0 ]; then
        echo "❌ Client build failed!"
        exit 1
    fi
fi

# Remove existing stable/insiders extensions if present
if [ -L "$EXTENSION_DIR_STABLE" ] || [ -d "$EXTENSION_DIR_STABLE" ]; then
    echo "📦 Removing existing stable extension..."
    rm -rf "$EXTENSION_DIR_STABLE"
fi
if [ -L "$EXTENSION_DIR_INSIDERS" ] || [ -d "$EXTENSION_DIR_INSIDERS" ]; then
    echo "📦 Removing existing insiders extension..."
    rm -rf "$EXTENSION_DIR_INSIDERS"
fi

# Create symlink for both stable and insiders if 'code' path is not used
echo "🔗 Creating symlink for stable and insiders..."
ln -s "$SCRIPT_DIR" "$EXTENSION_DIR_STABLE"
ln -s "$SCRIPT_DIR" "$EXTENSION_DIR_INSIDERS"

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
    # Also attempt to install via vsix if available
    VSIX_FILE="$SCRIPT_DIR/vex-language-$VERSION.vsix"
    if [ -f "$VSIX_FILE" ]; then
        echo "📦 Installing VSIX for stable and insiders from $VSIX_FILE"
        if command -v code >/dev/null 2>&1; then
            code --install-extension "$VSIX_FILE" --force || true
        fi
        if command -v code-insiders >/dev/null 2>&1; then
            code-insiders --install-extension "$VSIX_FILE" --force || true
        fi
    fi
    echo "💡 To uninstall: rm -rf $EXTENSION_DIR"
else
    echo "❌ Installation failed!"
    exit 1
fi
