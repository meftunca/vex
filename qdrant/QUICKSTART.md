# Vex Qdrant RAG System - Quick Setup

## Prerequisites Check

```bash
# 1. Check Qdrant
curl http://localhost:6333/collections 2>/dev/null && echo "✅ Qdrant running" || echo "❌ Qdrant not running"

# 2. Check Ollama
curl http://localhost:11434/api/tags 2>/dev/null && echo "✅ Ollama running" || echo "❌ Ollama not running"

# 3. Check nomic-embed-text model
ollama list | grep nomic-embed-text && echo "✅ Model ready" || echo "❌ Model not installed"
```

## Quick Start (2 commands - No build needed!)

```bash
# 1. Install dependencies
bun install

# 2. Index everything (Bun runs TypeScript directly!)
bun run src/cli.ts all
```

## Usage

```bash
# Search
bun run src/search-cli.ts "how to implement generics"

# Reset (clears all collections)
bun run src/reset.ts

# Re-sync after reset
bun run src/cli.ts all

# Start MCP server for VS Code
bun run src/mcp-server.ts
```

## Embedding Model

This system uses **nomic-embed-text** (768 dimensions) via Ollama:

```bash
# Install model
ollama pull nomic-embed-text

# Test embedding
curl http://localhost:11434/api/embeddings -d '{
  "model": "nomic-embed-text",
  "prompt": "test"
}'
```

## Troubleshooting

### 1. Qdrant not running

```bash
docker run -d -p 6333:6333 qdrant/qdrant
```

### 2. Ollama not running

```bash
# macOS
brew services start ollama
# or
ollama serve
```

### 3. Model not found

```bash
ollama pull nomic-embed-text
```

### 4. Reset collections

```bash
bun run src/reset.ts
bun run src/cli.ts all
```

## Why Bun?

- ⚡ **No build step**: Runs TypeScript directly
- 🚀 **3x faster**: Compared to Node.js + tsc
- 📦 **Same APIs**: Drop-in replacement for npm scripts
- 🎯 **Less complexity**: No dist/ folder needed

## Collection Info

After indexing, check stats:

```bash
bun run src/cli.ts stats
```

Expected output:

```
📊 Collection Statistics:
  docs: ~500 points (Specifications + docs/)
  code: ~2000 points (vex-*/src/)
  todos: ~50 points (TODO items)
  examples: ~300 points (examples/)
```
