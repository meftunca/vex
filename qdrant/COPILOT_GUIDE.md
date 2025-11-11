# 🎯 Vex RAG Quick Reference (For Copilot)

## What's Indexed

- ✅ **1,023 Specification chunks** - Complete language reference (Specifications/)
- ✅ **359 Code examples** - Working .vx files with patterns (examples/)
- ✅ **97 TODO items** - Structured development tasks (TODO.md)
- ⏳ **Code chunks** - Rust implementation (vex-compiler/, vex-parser/, etc.)

## 🔄 Keep RAG Synced

**IMPORTANT: After updating specs/docs/TODOs, re-index!**

```bash
cd qdrant

# After updating Specifications/*.md
bun src/cli.ts specs

# After adding examples/*.vx
bun src/cli.ts examples

# After modifying TODO.md
bun src/cli.ts todos

# Full re-index (takes ~2 min)
bun src/cli.ts all
```

## Quick Search Commands

```bash
# General search (searches all collections)
bun src/search-cli.ts "your query" [limit]

# TODO-specific search
bun src/todo-cli.ts "keyword"           # Search TODOs
bun src/todo-cli.ts list                # List all TODOs
bun src/todo-cli.ts high                # High priority TODOs

# TODO management
bun src/todo-simple.ts add "New TODO"
bun src/todo-simple.ts remove "pattern"
bun src/todo-simple.ts complete "pattern"

# Examples
bun src/search-cli.ts "generics" 3
bun src/search-cli.ts "async await" 2
bun src/todo-cli.ts "Display trait"
```

## MCP Tools (Use in Copilot Chat)

### 1. `vex_search` - Smart semantic search

Automatically routes to appropriate collections.

**Example prompts:**

- "Use vex_search to find how generics work"
- "Search for async/await implementation"
- "Find pattern matching examples"

### 2. `vex_find_examples` - Code examples only

Searches only in examples/ directory.

**Example prompts:**

- "Use vex_find_examples to show trait implementation"
- "Find examples of error handling"

### 3. `vex_find_todos` - TODO items

Searches structured TODO list with filters.

**Example prompts:**

- "Use vex_find_todos to find Display trait TODO"
- "Show high priority TODOs"
- "Find TODOs related to iterators"

### 4. `vex_find_similar_code` - Code similarity

Finds similar code patterns.

**Example:**

```
Use vex_find_similar_code with:
trait Iterator { fn next(): Option<T>; }
```

### 5. `vex_get_file_context` - File-specific context

Gets relevant chunks for a specific file.

**Example:**

- "Use vex_get_file_context for vex-parser/src/parser.rs"

## Search Result Quality

- **Score > 0.8**: Highly relevant (exact match)
- **Score 0.6-0.8**: Good match (related content)
- **Score < 0.6**: Weak match (consider refining query)

## Collection Strategy

| Collection   | Content                   | Use When                              |
| ------------ | ------------------------- | ------------------------------------- |
| vex_docs     | Specifications (23 files) | Need language reference, syntax rules |
| vex_examples | Code samples (290 files)  | Need working code patterns            |
| vex_todos    | TODO items (96 tasks)     | Planning development, checking status |

## Common Queries

### Language Features

```bash
# Traits
bun src/search-cli.ts "trait implementation syntax" 3

# Generics
bun src/search-cli.ts "generic functions and structs" 3

# Pattern Matching
bun src/search-cli.ts "match expression exhaustiveness" 2

# Memory Management
bun src/search-cli.ts "ownership and borrowing" 3
```

### Development Tasks

```bash
# Check Display trait status
bun src/todo-cli.ts "Display"

# Iterator adapters
bun src/todo-cli.ts "Iterator map filter"

# Find high priority items
bun src/todo-cli.ts high
```

### Code Examples

```bash
# Async patterns
bun src/search-cli.ts "async function example" 3

# Error handling
bun src/search-cli.ts "Result type usage" 2

# Traits with generics
bun src/search-cli.ts "generic trait implementation" 3
```

## Tips for Better Results

1. **Be specific**: "trait implementation" > "traits"
2. **Use code terms**: "Iterator.map()" > "map function"
3. **Check score**: Re-query if top result < 0.6
4. **Combine searches**: General search + TODO search
5. **File names help**: Results show exact file paths

## Maintenance

```bash
# Check what's indexed
bun src/cli.ts stats

# Re-index (if files changed)
bun src/cli.ts specs     # Just specifications
bun src/cli.ts examples  # Just examples
bun src/cli.ts todos     # Just TODOs
bun src/cli.ts all       # Everything

# Reset and start fresh
bun src/reset.ts
bun src/cli.ts all
```

## Output Format

Search results show:

- 📊 **Score**: Relevance (0-1)
- 📁 **Collection**: Source (docs/examples/todos)
- 📄 **File**: Exact file path
- 📝 **Content**: First 400 chars (with context)

TODO results show:

- ✅/⏳ **Status**: Completed or pending
- 🔴🟡🟢 **Priority**: high/medium/low
- 📝 **Title**: Task description
- 📄 **Files**: Related files
- 💬 **Description**: Implementation details

## Quick Diagnostic

```bash
# Test everything works
bun test.ts              # System health
bun test-mcp.ts          # MCP tools test

# Should see:
# ✓ Qdrant: http://localhost:6333
# ✓ Ollama: http://localhost:11434
# ✓ Model: nomic-embed-text (768 dim)
# ✓ Collections: 4 active
```

## Performance

- **Embedding**: ~100ms per query (Ollama)
- **Search**: ~50ms per collection
- **Total latency**: ~200-300ms typical
- **Accuracy**: 70-95% depending on query specificity

## Troubleshooting

**No results found:**

- Try broader terms
- Check collection stats: `bun src/cli.ts stats`
- Re-index if needed

**Low scores (<0.5):**

- Query too vague
- Try code-specific terms
- Use exact function/type names

**Wrong collection:**

- Use specific tools (vex_find_examples, vex_find_todos)
- Add context keywords ("example", "TODO", "spec")
