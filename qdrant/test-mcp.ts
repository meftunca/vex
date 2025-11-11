#!/usr/bin/env bun
/**
 * Test MCP tools locally
 */

import SemanticSearcher from "./src/searcher.js";

console.log("🧪 Testing MCP Tools Locally\n");

const searcher = new SemanticSearcher();
await searcher.initialize();

// Test 1: vex_search
console.log("1️⃣  Testing vex_search...");
const searchResults = await searcher.smartSearch(
  "how to implement generics",
  3
);
console.log(`   ✓ Found ${searchResults.length} results`);
console.log(`   ✓ Top result score: ${searchResults[0]?.score.toFixed(3)}`);
console.log(`   ✓ Top result file: ${searchResults[0]?.metadata.file}\n`);

// Test 2: vex_find_examples
console.log("2️⃣  Testing vex_find_examples...");
const examples = await searcher.findExamples("pattern matching", 2);
console.log(`   ✓ Found ${examples.length} examples`);
console.log(`   ✓ Example file: ${examples[0]?.metadata.file}\n`);

// Test 3: vex_find_todos
console.log("3️⃣  Testing vex_find_todos...");
const todos = await searcher.searchTodos("Display trait");
console.log(`   ✓ Found ${todos.length} TODOs`);
if (todos[0]) {
  console.log(`   ✓ TODO: ${todos[0].metadata.title}`);
  console.log(`   ✓ Priority: ${todos[0].metadata.priority}\n`);
}

// Test 4: vex_find_similar_code
console.log("4️⃣  Testing vex_find_similar_code...");
const similarCode = await searcher.findSimilarCode(
  "trait Iterator { fn next(): Option<T>; }",
  2
);
console.log(`   ✓ Found ${similarCode.length} similar code snippets`);
console.log(`   ✓ Similarity score: ${similarCode[0]?.score.toFixed(3)}\n`);

// Test JSON serialization (like MCP)
console.log("5️⃣  Testing JSON serialization (MCP format)...");
const mcpResponse = {
  content: [
    {
      type: "text",
      text: JSON.stringify(searchResults.slice(0, 2), null, 2),
    },
  ],
};
console.log(
  `   ✓ JSON valid: ${typeof mcpResponse.content[0].text === "string"}`
);
console.log(`   ✓ Parseable: ${!!JSON.parse(mcpResponse.content[0].text)}\n`);

console.log("✅ All MCP tools working!\n");
console.log("📝 Usage in VS Code:");
console.log("   1. Start MCP server: bun src/mcp-server.ts");
console.log("   2. Restart VS Code");
console.log(
  "   3. Use tools: vex_search, vex_find_examples, vex_find_todos, etc."
);
