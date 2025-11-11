#!/usr/bin/env node
/**
 * Vex Qdrant RAG System - MCP Server
 * Model Context Protocol server for VS Code integration
 */

import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
} from "@modelcontextprotocol/sdk/types.js";
import SemanticSearcher from "./searcher.js";
import TodoManager from "./todo-manager.js";

const searcher = new SemanticSearcher();
const todoManager = new TodoManager();

const server = new Server(
  {
    name: "vex-qdrant-rag",
    version: "1.0.0",
  },
  {
    capabilities: {
      tools: {},
    },
  }
);

// List available tools
server.setRequestHandler(ListToolsRequestSchema, async () => {
  return {
    tools: [
      {
        name: "vex_search",
        description:
          "Semantic search across Vex documentation, code, examples, and TODOs. Automatically routes queries to appropriate collections.",
        inputSchema: {
          type: "object",
          properties: {
            query: {
              type: "string",
              description:
                'The search query (e.g., "how to implement generics", "TODO for type system")',
            },
            limit: {
              type: "number",
              description: "Maximum number of results (default: 10)",
              default: 10,
            },
          },
          required: ["query"],
        },
      },
      {
        name: "vex_find_examples",
        description: "Find code examples for a specific Vex feature or pattern",
        inputSchema: {
          type: "object",
          properties: {
            feature: {
              type: "string",
              description:
                'The feature or pattern to find examples for (e.g., "async/await", "pattern matching", "generics")',
            },
            limit: {
              type: "number",
              description: "Maximum number of examples (default: 5)",
              default: 5,
            },
          },
          required: ["feature"],
        },
      },
      {
        name: "vex_find_todos",
        description: "Search and filter TODO items from the Vex codebase",
        inputSchema: {
          type: "object",
          properties: {
            query: {
              type: "string",
              description: "Optional search query for TODOs",
            },
            priority: {
              type: "string",
              description: "Filter by priority (high, medium, low)",
              enum: ["high", "medium", "low"],
            },
            category: {
              type: "string",
              description:
                'Filter by category (e.g., "type-system", "parser", "stdlib")',
            },
            limit: {
              type: "number",
              description: "Maximum number of TODOs (default: 20)",
              default: 20,
            },
          },
        },
      },
      {
        name: "vex_find_similar_code",
        description: "Find similar code snippets in the Vex codebase",
        inputSchema: {
          type: "object",
          properties: {
            code: {
              type: "string",
              description: "The code snippet to find similar examples for",
            },
            limit: {
              type: "number",
              description: "Maximum number of results (default: 5)",
              default: 5,
            },
          },
          required: ["code"],
        },
      },
      {
        name: "vex_get_file_context",
        description:
          "Get relevant context for a specific file in the Vex codebase",
        inputSchema: {
          type: "object",
          properties: {
            filePath: {
              type: "string",
              description: "Path to the file (relative or absolute)",
            },
            limit: {
              type: "number",
              description: "Maximum number of context chunks (default: 3)",
              default: 3,
            },
          },
          required: ["filePath"],
        },
      },
      {
        name: "vex_add_todo",
        description:
          "Add a new TODO item to TODO.md with proper formatting and categorization",
        inputSchema: {
          type: "object",
          properties: {
            title: {
              type: "string",
              description: "TODO title (e.g., 'Loop syntax support')",
            },
            description: {
              type: "string",
              description: "Detailed description with context, files, priority",
            },
            priority: {
              type: "string",
              description: "Priority level",
              enum: ["critical", "high", "medium", "low"],
              default: "medium",
            },
            category: {
              type: "string",
              description: "Category section to add to",
              enum: [
                "critical-bugs",
                "medium-priority",
                "low-priority",
                "completed",
              ],
              default: "medium-priority",
            },
            estimatedTime: {
              type: "string",
              description: 'Time estimate (e.g., "2 hours", "1 day")',
            },
          },
          required: ["title", "description"],
        },
      },
      {
        name: "vex_complete_todo",
        description:
          "Mark a TODO item as complete by moving it to completed section with timestamp",
        inputSchema: {
          type: "object",
          properties: {
            title: {
              type: "string",
              description: "TODO title to mark as complete",
            },
          },
          required: ["title"],
        },
      },
      {
        name: "vex_update_todo",
        description: "Update an existing TODO item's description or priority",
        inputSchema: {
          type: "object",
          properties: {
            title: {
              type: "string",
              description: "TODO title to update",
            },
            newDescription: {
              type: "string",
              description: "New description (optional)",
            },
            newPriority: {
              type: "string",
              description: "New priority (optional)",
              enum: ["critical", "high", "medium", "low"],
            },
          },
          required: ["title"],
        },
      },
      {
        name: "vex_remove_todo",
        description: "Remove a TODO item completely from TODO.md",
        inputSchema: {
          type: "object",
          properties: {
            title: {
              type: "string",
              description: "TODO title to remove",
            },
          },
          required: ["title"],
        },
      },
    ],
  };
});

// Handle tool calls
server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { name, arguments: args } = request.params;

  try {
    switch (name) {
      case "vex_search": {
        const { query, limit = 10 } = args as { query: string; limit?: number };
        const results = await searcher.smartSearch(query, limit);

        return {
          content: [
            {
              type: "text",
              text: JSON.stringify(results, null, 2),
            },
          ],
        };
      }

      case "vex_find_examples": {
        const { feature, limit = 5 } = args as {
          feature: string;
          limit?: number;
        };
        const results = await searcher.findExamples(feature, limit);

        return {
          content: [
            {
              type: "text",
              text: JSON.stringify(results, null, 2),
            },
          ],
        };
      }

      case "vex_find_todos": {
        const {
          query,
          priority,
          category,
          limit = 20,
        } = args as {
          query?: string;
          priority?: string;
          category?: string;
          limit?: number;
        };
        const results = await searcher.searchTodos(
          query,
          priority,
          category,
          limit
        );

        return {
          content: [
            {
              type: "text",
              text: JSON.stringify(results, null, 2),
            },
          ],
        };
      }

      case "vex_find_similar_code": {
        const { code, limit = 5 } = args as { code: string; limit?: number };
        const results = await searcher.findSimilarCode(code, limit);

        return {
          content: [
            {
              type: "text",
              text: JSON.stringify(results, null, 2),
            },
          ],
        };
      }

      case "vex_get_file_context": {
        const { filePath, limit = 3 } = args as {
          filePath: string;
          limit?: number;
        };
        const results = await searcher.getFileContext(filePath, limit);

        return {
          content: [
            {
              type: "text",
              text: JSON.stringify(results, null, 2),
            },
          ],
        };
      }

      case "vex_add_todo": {
        const {
          title,
          description,
          priority = "medium",
          category = "medium-priority",
          estimatedTime,
        } = args as {
          title: string;
          description: string;
          priority?: string;
          category?: string;
          estimatedTime?: string;
        };

        // Format description with metadata
        let fullDesc = description;
        if (priority) fullDesc += `\n- Priority: ${priority}`;
        if (estimatedTime) fullDesc += `\n- Estimated: ${estimatedTime}`;

        await todoManager.addTodo(title);

        return {
          content: [
            {
              type: "text",
              text: JSON.stringify(
                {
                  success: true,
                  message: `✅ TODO added: ${title}`,
                  title,
                  priority,
                  category,
                },
                null,
                2
              ),
            },
          ],
        };
      }

      case "vex_complete_todo": {
        const { title } = args as { title: string };
        await todoManager.completeTodo(title);

        return {
          content: [
            {
              type: "text",
              text: JSON.stringify(
                {
                  success: true,
                  message: `✅ TODO completed: ${title}`,
                },
                null,
                2
              ),
            },
          ],
        };
      }

      case "vex_update_todo": {
        const { title, newDescription, newPriority } = args as {
          title: string;
          newDescription?: string;
          newPriority?: string;
        };

        // Get current todos
        const todos = await todoManager.getTodos();
        const existing = todos.find((t) =>
          t.title.toLowerCase().includes(title.toLowerCase())
        );

        if (!existing) {
          return {
            content: [
              {
                type: "text",
                text: JSON.stringify(
                  {
                    success: false,
                    error: `TODO not found: ${title}`,
                  },
                  null,
                  2
                ),
              },
            ],
          };
        }

        // Update via remove + add with new info
        await todoManager.removeTodo(existing.title);
        await todoManager.addTodo(
          existing.title + (newDescription ? ` ${newDescription}` : "")
        );

        return {
          content: [
            {
              type: "text",
              text: JSON.stringify(
                {
                  success: true,
                  message: `✅ TODO updated: ${existing.title}`,
                  oldTitle: existing.title,
                  updated: {
                    description: newDescription,
                    priority: newPriority,
                  },
                },
                null,
                2
              ),
            },
          ],
        };
      }

      case "vex_remove_todo": {
        const { title } = args as { title: string };
        await todoManager.removeTodo(title);

        return {
          content: [
            {
              type: "text",
              text: JSON.stringify(
                {
                  success: true,
                  message: `✅ TODO removed: ${title}`,
                },
                null,
                2
              ),
            },
          ],
        };
      }

      default:
        throw new Error(`Unknown tool: ${name}`);
    }
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    return {
      content: [
        {
          type: "text",
          text: JSON.stringify({ error: errorMessage }, null, 2),
        },
      ],
      isError: true,
    };
  }
});

async function main() {
  console.error("Initializing Vex Qdrant RAG system...");
  await searcher.initialize();
  await todoManager.initialize();
  console.error("Vex Qdrant RAG MCP server ready");

  const transport = new StdioServerTransport();
  await server.connect(transport);
}

main().catch((error) => {
  console.error("Fatal error:", error);
  process.exit(1);
});
