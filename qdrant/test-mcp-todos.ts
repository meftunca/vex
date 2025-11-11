#!/usr/bin/env bun
/**
 * Test MCP Server TODO functions
 */

import TodoManager from "./src/todo-manager.js";

async function testTodoOperations() {
  console.log("🧪 Testing TODO Manager Operations\n");

  const manager = new TodoManager();
  await manager.initialize();

  try {
    // Test 1: Add a TODO
    console.log("📝 Test 1: Adding a TODO...");
    await manager.addTodo("Test TODO - MCP Integration");
    console.log("✅ Add test passed\n");

    // Test 2: List TODOs
    console.log("📋 Test 2: Listing TODOs...");
    await manager.listTodos();
    console.log("✅ List test passed\n");

    // Test 3: Complete a TODO
    console.log("✓ Test 3: Completing a TODO...");
    await manager.completeTodo("Test TODO - MCP Integration");
    console.log("✅ Complete test passed\n");

    // Test 4: Remove a TODO
    console.log("🗑️  Test 4: Removing a TODO...");
    await manager.removeTodo("Test TODO - MCP Integration");
    console.log("✅ Remove test passed\n");

    console.log("🎉 All tests passed!");
  } catch (error) {
    console.error("❌ Test failed:", error);
    process.exit(1);
  }
}

testTodoOperations();
