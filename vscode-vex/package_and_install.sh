#!/usr/bin/env bash
# Build and package VS Code extension, then install for stable and insiders
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VERSION=$(node -pe "require('./package.json').version")

# Ensure LSP server is built
cd "$SCRIPT_DIR/.."
if [ ! -f "$HOME/.cargo/target/release/vex-lsp" ]; then
  echo "Building LSP server..."
  cargo build --release -p vex-lsp
fi

# Build client
cd "$SCRIPT_DIR/client"
npm install
npm run compile

# Package extension
cd "$SCRIPT_DIR"
if ! command -v vsce >/dev/null; then
  echo "vsce not found, installing locally"
  npm install -g @vscode/vsce || true
fi

vsce package

VSIX_FILE="$SCRIPT_DIR/vex-language-$VERSION.vsix"
if [ -f "$VSIX_FILE" ]; then
  echo "Installing VSIX: $VSIX_FILE"
  if command -v code >/dev/null; then
    code --install-extension "$VSIX_FILE" --force || true
  fi
  if command -v code-insiders >/dev/null 2>&1; then
    code-insiders --install-extension "$VSIX_FILE" --force || true
  fi
else
  echo "VSIX not found: $VSIX_FILE"
  exit 1
fi

echo "Done: installed Vex extension ($VERSION) for stable and insiders."
