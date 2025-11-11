#!/usr/bin/env node
/**
 * Vex Qdrant RAG System - Search CLI
 */

import SemanticSearcher from "./searcher.js";

async function main() {
  const args = process.argv.slice(2);

  if (args.length === 0) {
    console.error("Usage: search-cli <query> [limit]");
    console.error('Example: search-cli "how to implement generics" 5');
    process.exit(1);
  }

  const query = args[0];
  const limit = args[1] ? parseInt(args[1], 10) : 10;

  console.log(`🔍 Searching: "${query}"\n`);

  const searcher = new SemanticSearcher();
  await searcher.initialize();

  const results = await searcher.smartSearch(query, limit);

  if (results.length === 0) {
    console.log("No results found.");
    return;
  }

  console.log(`Found ${results.length} results:\n`);

  for (let i = 0; i < results.length; i++) {
    const result = results[i];
    const collection = result.metadata.collection || "unknown";
    const file = result.metadata.file || result.metadata.fileName || "unknown";
    const type = result.metadata.type || "";

    console.log(
      `\n━━━ [${i + 1}/${
        results.length
      }] ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━`
    );
    console.log(
      `📊 Score: ${result.score.toFixed(3)} | Collection: ${collection}`
    );
    console.log(`📄 File: ${file}${type ? ` (${type})` : ""}`);
    console.log(
      `─────────────────────────────────────────────────────────────`
    );

    // Show first 400 chars of content
    const preview = result.content.substring(0, 400);
    console.log(
      preview + (result.content.length > 400 ? "\n\n... (truncated)" : "")
    );
  }

  console.log("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
}

main().catch(console.error);
