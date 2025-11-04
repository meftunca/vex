#!/bin/bash
# Install Vex VS Code Extension via Symlink

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXTENSION_DIR="$HOME/.vscode/extensions/vex-language-0.2.0"

echo "🔧 Installing Vex Language Support for VS Code..."

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
    echo "2. Open any .vx file to see syntax highlighting"
    echo "3. Try snippets: type 'main', 'fn', 'struct', etc."
    echo ""
    echo "💡 To uninstall: rm -rf $EXTENSION_DIR"
else
    echo "❌ Installation failed!"
    exit 1
fi
