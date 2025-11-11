#!/usr/bin/env bun
/**
 * Vex Qdrant RAG - Test Script
 * Quick validation of all features
 */

console.log("🧪 Testing Vex Qdrant RAG System\n");

// Test 1: Config loading
console.log("1️⃣  Testing configuration...");
const { config } = await import("./src/config.ts");
console.log(`   ✓ Qdrant URL: ${config.qdrant.url}`);
console.log(`   ✓ Ollama URL: ${config.ollama.url}`);
console.log(`   ✓ Model: ${config.ollama.model}`);
console.log(`   ✓ Embedding dim: ${config.ollama.embeddingDim}\n`);

// Test 2: Client
console.log("2️⃣  Testing Qdrant client...");
const { default: VexQdrantClient } = await import("./src/client.ts");
const client = new VexQdrantClient();

try {
  // Test Qdrant connection
  const collections = [
    "vex_docs",
    "vex_code",
    "vex_todos",
    "vex_examples",
    "vex_tests",
  ];
  for (const collection of collections) {
    try {
      const count = await client.countPoints(collection);
      console.log(`   ✓ ${collection}: ${count} points`);
    } catch (e) {
      console.log(
        `   ⏭️  ${collection}: not found (will be created on first sync)`
      );
    }
  }
} catch (error) {
  console.log(`   ❌ Error: ${error.message}`);
  console.log(
    `   💡 Make sure Qdrant is running: docker run -p 6333:6333 qdrant/qdrant`
  );
}

console.log("\n3️⃣  Testing Ollama embedding...");
try {
  const embedding = await client.generateEmbedding("test");
  console.log(`   ✓ Generated embedding: ${embedding.length} dimensions`);
} catch (error) {
  console.log(`   ❌ Error: ${error.message}`);
  console.log(`   💡 Make sure Ollama is running: ollama serve`);
  console.log(`   💡 And model is installed: ollama pull nomic-embed-text`);
}

console.log("\n✅ Test complete!");
console.log("\n📝 Next steps:");
console.log("   - Start Qdrant: docker run -p 6333:6333 qdrant/qdrant");
console.log("   - Start Ollama: ollama serve");
console.log("   - Pull model: ollama pull nomic-embed-text");
console.log("   - Index docs: bun run src/cli.ts all");
console.log('   - Search: bun run src/search-cli.ts "your query"');
