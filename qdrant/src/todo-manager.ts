#!/usr/bin/env bun
/**
 * Vex TODO Manager
 * Add, remove, and update TODO items
 */

import { randomUUID } from "crypto";
import { readFileSync, writeFileSync } from "fs";
import VexQdrantClient from "./client.js";
import config from "./config.js";

interface Todo {
  title: string;
  description: string;
  status: "pending" | "completed";
  files: string[];
  priority: "high" | "medium" | "low";
  dependencies: string[];
}

class TodoManager {
  private client: VexQdrantClient;
  private todoFilePath: string;

  constructor() {
    this.client = new VexQdrantClient();
    this.todoFilePath = config.paths.todoFile;
  }

  async initialize(): Promise<void> {
    await this.client.initialize();
    await this.client.ensureCollection(config.collections.todos);
  }

  /**
   * Parse TODO.md file
   */
  private parseTodos(content: string): Todo[] {
    const todos: Todo[] = [];
    const lines = content.split("\n");
    let currentTodo: Todo | null = null;

    for (const line of lines) {
      const todoMatch = line.match(/^-\s+\[([ x])\]\s+(.+)$/);
      if (todoMatch) {
        if (currentTodo) {
          todos.push(currentTodo);
        }
        currentTodo = {
          title: todoMatch[2],
          description: "",
          status: todoMatch[1] === "x" ? "completed" : "pending",
          files: [],
          priority: "medium",
          dependencies: [],
        };
      } else if (currentTodo && line.trim().startsWith("-")) {
        currentTodo.description += line.trim().substring(1).trim() + "\n";

        // Extract metadata
        const fileMatch = line.match(/Files?:\s+([^\n]+)/i);
        if (fileMatch) {
          currentTodo.files = fileMatch[1].split(",").map((f) => f.trim());
        }

        const priorityMatch = line.match(/Priority:\s+(high|medium|low)/i);
        if (priorityMatch) {
          currentTodo.priority = priorityMatch[1].toLowerCase() as any;
        }
      }
    }

    if (currentTodo) {
      todos.push(currentTodo);
    }

    return todos;
  }

  /**
   * Save TODOs back to file
   */
  private saveTodos(todos: Todo[]): void {
    let content = "# Todo List\n\n";

    for (const todo of todos) {
      const checkbox = todo.status === "completed" ? "[x]" : "[ ]";
      content += `- ${checkbox} ${todo.title}\n`;

      if (todo.description) {
        const lines = todo.description.trim().split("\n");
        for (const line of lines) {
          content += `  - ${line}\n`;
        }
      }

      content += "\n";
    }

    writeFileSync(this.todoFilePath, content, "utf-8");
  }

  /**
   * Re-index all TODOs in Qdrant
   */
  private async reindexTodos(todos: Todo[]): Promise<void> {
    // Delete collection and recreate
    await this.client.deleteCollection(config.collections.todos);
    await this.client.ensureCollection(config.collections.todos);

    const points = [];
    for (let i = 0; i < todos.length; i++) {
      const todo = todos[i];

      // Retry embedding generation
      let embedding;
      for (let retry = 0; retry < 3; retry++) {
        try {
          embedding = await this.client.generateEmbedding(
            `${todo.title}\n${todo.description}`
          );
          break;
        } catch (error) {
          if (retry === 2) throw error;
          console.log(`Retrying embedding (${retry + 1}/3)...`);
          await new Promise((r) => setTimeout(r, 1000));
        }
      }

      points.push({
        id: randomUUID(),
        vector: embedding!,
        payload: {
          type: "todo",
          title: todo.title,
          description: todo.description,
          status: todo.status,
          files: todo.files,
          priority: todo.priority,
          dependencies: todo.dependencies,
          idStr: `todo-${i}`,
        },
      });
    }

    if (points.length > 0) {
      await this.client.upsert(config.collections.todos, points);
    }
  }

  /**
   * Add a new TODO
   */
  async addTodo(
    title: string,
    description = "",
    priority: "high" | "medium" | "low" = "medium",
    files: string[] = []
  ): Promise<void> {
    const content = readFileSync(this.todoFilePath, "utf-8");
    const todos = this.parseTodos(content);

    const newTodo: Todo = {
      title,
      description,
      status: "pending",
      files,
      priority,
      dependencies: [],
    };

    todos.push(newTodo);
    this.saveTodos(todos);
    await this.reindexTodos(todos);

    console.log(`✅ Added TODO: ${title}`);
  }

  /**
   * Remove TODO by title (fuzzy match)
   */
  async removeTodo(titlePattern: string): Promise<void> {
    const content = readFileSync(this.todoFilePath, "utf-8");
    const todos = this.parseTodos(content);

    const pattern = titlePattern.toLowerCase();
    const index = todos.findIndex((t) =>
      t.title.toLowerCase().includes(pattern)
    );

    if (index === -1) {
      console.log(`❌ No TODO found matching: ${titlePattern}`);
      return;
    }

    const removed = todos[index];
    todos.splice(index, 1);

    this.saveTodos(todos);
    await this.reindexTodos(todos);

    console.log(`✅ Removed TODO: ${removed.title}`);
  }

  /**
   * Mark TODO as completed
   */
  async completeTodo(titlePattern: string): Promise<void> {
    const content = readFileSync(this.todoFilePath, "utf-8");
    const todos = this.parseTodos(content);

    const pattern = titlePattern.toLowerCase();
    const todo = todos.find((t) => t.title.toLowerCase().includes(pattern));

    if (!todo) {
      console.log(`❌ No TODO found matching: ${titlePattern}`);
      return;
    }

    todo.status = "completed";
    this.saveTodos(todos);
    await this.reindexTodos(todos);

    console.log(`✅ Completed TODO: ${todo.title}`);
  }

  /**
   * List all TODOs
   */
  async listTodos(): Promise<void> {
    const content = readFileSync(this.todoFilePath, "utf-8");
    const todos = this.parseTodos(content);

    console.log(`\n📋 Total TODOs: ${todos.length}\n`);

    const pending = todos.filter((t) => t.status === "pending");
    const completed = todos.filter((t) => t.status === "completed");

    console.log(`⏳ Pending: ${pending.length}`);
    for (const todo of pending) {
      const priorityIcon =
        todo.priority === "high" ? "🔴" : todo.priority === "low" ? "🟢" : "🟡";
      console.log(`   ${priorityIcon} ${todo.title}`);
    }

    console.log(`\n✅ Completed: ${completed.length}`);
    for (const todo of completed) {
      console.log(`   ✓ ${todo.title}`);
    }
  }

  /**
   * Get all TODOs as data (for MCP)
   */
  async getTodos(): Promise<Todo[]> {
    const content = readFileSync(this.todoFilePath, "utf-8");
    return this.parseTodos(content);
  }
}

export default TodoManager;

// CLI
if (process.argv[1]?.includes("todo-manager.ts")) {
  const args = process.argv.slice(2);
  const manager = new TodoManager();
  await manager.initialize();

  const command = args[0];

  if (command === "add") {
    const title = args.slice(1).join(" ");
    if (!title) {
      console.log("Usage: bun src/todo-manager.ts add <title>");
      process.exit(1);
    }
    await manager.addTodo(title);
  } else if (command === "remove") {
    const pattern = args.slice(1).join(" ");
    if (!pattern) {
      console.log("Usage: bun src/todo-manager.ts remove <title pattern>");
      process.exit(1);
    }
    await manager.removeTodo(pattern);
  } else if (command === "complete") {
    const pattern = args.slice(1).join(" ");
    if (!pattern) {
      console.log("Usage: bun src/todo-manager.ts complete <title pattern>");
      process.exit(1);
    }
    await manager.completeTodo(pattern);
  } else if (command === "list") {
    await manager.listTodos();
  } else {
    console.log(`
📝 Vex TODO Manager

Usage:
  bun src/todo-manager.ts add <title>           Add new TODO
  bun src/todo-manager.ts remove <pattern>      Remove TODO
  bun src/todo-manager.ts complete <pattern>    Mark as completed
  bun src/todo-manager.ts list                  List all TODOs

Examples:
  bun src/todo-manager.ts add "Implement Display trait"
  bun src/todo-manager.ts remove "Display"
  bun src/todo-manager.ts complete "Iterator"
  bun src/todo-manager.ts list
    `);
  }
}
