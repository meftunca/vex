#!/bin/bash
# Uninstall Vex VS Code Extension

EXTENSION_DIR="$HOME/.vscode/extensions/vex-language-0.9.2"

echo "🗑️  Uninstalling Vex Language Support..."

if [ -L "$EXTENSION_DIR" ] || [ -d "$EXTENSION_DIR" ]; then
    rm -rf "$EXTENSION_DIR"
    echo "✅ Successfully uninstalled!"
    echo "🔄 Reload VS Code to complete removal"
else
    echo "⚠️  Extension not found"
fi
