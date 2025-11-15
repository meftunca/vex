#!/bin/bash
# Uninstall Vex VS Code Extension

VERSION=$(node -pe "require('./package.json').version")
EXTENSION_DIR_STABLE="$HOME/.vscode/extensions/vex-language-$VERSION"
EXTENSION_DIR_INSIDERS="$HOME/.vscode-insiders/extensions/vex-language-$VERSION"

echo "🗑️  Uninstalling Vex Language Support..."

if [ -L "$EXTENSION_DIR_STABLE" ] || [ -d "$EXTENSION_DIR_STABLE" ]; then
    rm -rf "$EXTENSION_DIR_STABLE"
    echo "Removed stable extension"
fi
if [ -L "$EXTENSION_DIR_INSIDERS" ] || [ -d "$EXTENSION_DIR_INSIDERS" ]; then
    rm -rf "$EXTENSION_DIR_INSIDERS"
    echo "✅ Successfully uninstalled!"
    echo "🔄 Reload VS Code to complete removal"
else
    echo "⚠️  Extension not found"
fi
