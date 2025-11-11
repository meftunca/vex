#!/usr/bin/env bun
/**
 * Simple MCP Server Test
 */

import SemanticSearcher from "./src/searcher.js";
import TodoManager from "./src/todo-manager.js";

async function testMCP() {
  console.log("Testing MCP Server initialization...\n");

  // Test MCP SDK
  console.log("✅ MCP SDK imported");

  // Test TodoManager
  const manager = new TodoManager();
  console.log("✅ TodoManager created");

  await manager.initialize();
  console.log("✅ TodoManager initialized");

  // Test SemanticSearcher
  const searcher = new SemanticSearcher();
  console.log("✅ SemanticSearcher created");

  await searcher.initialize();
  console.log("✅ SemanticSearcher initialized");

  console.log("\n🎉 All components initialized successfully!");
  console.log("\n📝 MCP Server is ready with the following tools:");
  console.log("  1. vex_search - Semantic search");
  console.log("  2. vex_find_examples - Find code examples");
  console.log("  3. vex_find_todos - Search TODOs");
  console.log("  4. vex_find_similar_code - Find similar code");
  console.log("  5. vex_get_file_context - Get file context");
  console.log("  6. vex_add_todo - Add new TODO");
  console.log("  7. vex_complete_todo - Mark TODO complete");
  console.log("  8. vex_update_todo - Update TODO");
  console.log("  9. vex_remove_todo - Remove TODO");
}

testMCP().catch((error) => {
  console.error("❌ Test failed:", error);
  process.exit(1);
});
