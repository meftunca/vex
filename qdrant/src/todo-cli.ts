#!/usr/bin/env bun
/**
 * Vex Qdrant RAG - TODO CLI
 * Quick access to TODO items
 */

import SemanticSearcher from "./searcher.js";

async function main() {
  const args = process.argv.slice(2);

  const searcher = new SemanticSearcher();
  await searcher.initialize();

  if (args.length === 0 || args[0] === "list") {
    // List all TODOs
    console.log("📋 All TODO Items:\n");
    const results = await searcher.searchTodos(
      undefined,
      undefined,
      undefined,
      100
    );

    for (let i = 0; i < results.length; i++) {
      const todo = results[i].metadata;
      const status = todo.status === "completed" ? "✅" : "⏳";
      const priority = todo.priority || "medium";
      const priorityIcon =
        priority === "high" ? "🔴" : priority === "low" ? "🟢" : "🟡";

      console.log(`\n${status} [${i + 1}] ${todo.title}`);
      console.log(`   ${priorityIcon} Priority: ${priority}`);
      if (todo.files && todo.files.length > 0) {
        console.log(`   📄 Files: ${todo.files.join(", ")}`);
      }
      if (todo.description) {
        const desc = todo.description.split("\n")[0].substring(0, 80);
        console.log(
          `   💬 ${desc}${todo.description.length > 80 ? "..." : ""}`
        );
      }
    }

    console.log(`\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━`);
    console.log(`Total: ${results.length} TODOs`);
    return;
  }

  if (args[0] === "high" || args[0] === "medium" || args[0] === "low") {
    // Filter by priority
    const priority = args[0];
    console.log(`🎯 ${priority.toUpperCase()} Priority TODOs:\n`);
    const results = await searcher.searchTodos(
      undefined,
      priority,
      undefined,
      50
    );

    for (const result of results) {
      const todo = result.metadata;
      console.log(`⏳ ${todo.title}`);
      if (todo.description) {
        console.log(`   ${todo.description.split("\n")[0].substring(0, 100)}`);
      }
    }

    console.log(`\n Total: ${results.length} TODOs`);
    return;
  }

  // Search TODOs
  const query = args.join(" ");
  const limit = 10;

  console.log(`🔍 Searching TODOs: "${query}"\n`);

  const results = await searcher.searchTodos(
    query,
    undefined,
    undefined,
    limit
  );

  if (results.length === 0) {
    console.log("No TODOs found.");
    return;
  }

  for (let i = 0; i < results.length; i++) {
    const result = results[i];
    const todo = result.metadata;
    const status = todo.status === "completed" ? "✅" : "⏳";
    const priority = todo.priority || "medium";
    const priorityIcon =
      priority === "high" ? "🔴" : priority === "low" ? "🟢" : "🟡";

    console.log(
      `\n━━━ [${i + 1}/${
        results.length
      }] ${status} ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━`
    );
    console.log(
      `📊 Score: ${result.score.toFixed(
        3
      )} | ${priorityIcon} Priority: ${priority}`
    );
    console.log(`📝 ${todo.title}`);
    console.log(
      `─────────────────────────────────────────────────────────────`
    );

    if (todo.description) {
      console.log(todo.description.substring(0, 300));
      if (todo.description.length > 300) console.log("...");
    }

    if (todo.files && todo.files.length > 0) {
      console.log(`\n📄 Files: ${todo.files.join(", ")}`);
    }
  }

  console.log("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
}

main().catch(console.error);
