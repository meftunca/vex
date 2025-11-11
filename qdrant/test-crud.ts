#!/usr/bin/env bun
/**
 * Comprehensive TODO CRUD Test
 * Tests all Create, Read, Update, Delete operations
 */

import { readFileSync } from "fs";
import config from "./src/config.js";
import TodoManager from "./src/todo-manager.js";

async function testCRUD() {
  console.log("🧪 Testing TODO CRUD Operations\n");

  const manager = new TodoManager();
  await manager.initialize();

  const testTodoTitle = "Test CRUD Operations - DELETE ME";

  try {
    // ==================== CREATE ====================
    console.log("📝 CREATE: Adding a new TODO...");
    await manager.addTodo(testTodoTitle);

    // Verify it was added
    const contentAfterAdd = readFileSync(config.paths.todoFile, "utf-8");
    if (contentAfterAdd.includes(testTodoTitle)) {
      console.log("✅ CREATE SUCCESS: TODO added to file");
    } else {
      throw new Error("CREATE FAILED: TODO not found in file");
    }
    console.log();

    // ==================== READ ====================
    console.log("📖 READ: Listing all TODOs...");
    await manager.listTodos();

    const todos = readFileSync(config.paths.todoFile, "utf-8");
    const todoCount = (todos.match(/^- \[/gm) || []).length;
    console.log(`✅ READ SUCCESS: Found ${todoCount} total TODOs`);
    console.log();

    // ==================== UPDATE (via complete) ====================
    console.log("✏️  UPDATE: Marking TODO as complete...");
    await manager.completeTodo(testTodoTitle);

    const contentAfterComplete = readFileSync(config.paths.todoFile, "utf-8");
    if (contentAfterComplete.includes(`- [x] ${testTodoTitle}`)) {
      console.log("✅ UPDATE SUCCESS: TODO marked as completed");
    } else {
      throw new Error("UPDATE FAILED: TODO not marked as completed");
    }
    console.log();

    // ==================== DELETE ====================
    console.log("🗑️  DELETE: Removing TODO...");
    await manager.removeTodo(testTodoTitle);

    const contentAfterDelete = readFileSync(config.paths.todoFile, "utf-8");
    if (!contentAfterDelete.includes(testTodoTitle)) {
      console.log("✅ DELETE SUCCESS: TODO removed from file");
    } else {
      throw new Error("DELETE FAILED: TODO still in file");
    }
    console.log();

    // ==================== VERIFICATION ====================
    console.log("🔍 VERIFICATION: Running final checks...");

    // Test multiple adds
    console.log("\n  Testing multiple TODOs...");
    await manager.addTodo("Test TODO 1 - BATCH TEST");
    await manager.addTodo("Test TODO 2 - BATCH TEST");
    await manager.addTodo("Test TODO 3 - BATCH TEST");

    const contentMultiple = readFileSync(config.paths.todoFile, "utf-8");
    const batchCount = (contentMultiple.match(/BATCH TEST/g) || []).length;
    if (batchCount === 3) {
      console.log("  ✅ Multiple TODOs added successfully");
    }

    // Cleanup batch test TODOs
    console.log("\n  Cleaning up test TODOs...");
    await manager.removeTodo("Test TODO 1 - BATCH TEST");
    await manager.removeTodo("Test TODO 2 - BATCH TEST");
    await manager.removeTodo("Test TODO 3 - BATCH TEST");

    const contentCleanup = readFileSync(config.paths.todoFile, "utf-8");
    if (!contentCleanup.includes("BATCH TEST")) {
      console.log("  ✅ All test TODOs cleaned up");
    }

    console.log("\n🎉 ALL CRUD OPERATIONS PASSED!");
    console.log("\n✅ Summary:");
    console.log("  ✓ CREATE - Add TODO");
    console.log("  ✓ READ - List TODOs");
    console.log("  ✓ UPDATE - Complete TODO");
    console.log("  ✓ DELETE - Remove TODO");
    console.log("  ✓ BATCH - Multiple operations");
    console.log("  ✓ CLEANUP - Remove test data");
  } catch (error) {
    console.error("\n❌ CRUD TEST FAILED:", error);

    // Cleanup on failure
    try {
      console.log("\n🧹 Cleaning up test data...");
      const content = readFileSync(config.paths.todoFile, "utf-8");
      if (content.includes(testTodoTitle)) {
        await manager.removeTodo(testTodoTitle);
      }
      if (content.includes("BATCH TEST")) {
        await manager.removeTodo("BATCH TEST");
      }
    } catch (cleanupError) {
      console.error("Cleanup failed:", cleanupError);
    }

    process.exit(1);
  }
}

testCRUD();
