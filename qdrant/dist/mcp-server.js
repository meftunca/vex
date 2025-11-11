#!/usr/bin/env node
/**
 * Vex Qdrant RAG System - MCP Server
 * Model Context Protocol server for VS Code integration
 */
import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import { CallToolRequestSchema, ListToolsRequestSchema, } from '@modelcontextprotocol/sdk/types.js';
import SemanticSearcher from './searcher.js';
const searcher = new SemanticSearcher();
const server = new Server({
    name: 'vex-qdrant-rag',
    version: '1.0.0',
}, {
    capabilities: {
        tools: {},
    },
});
// List available tools
server.setRequestHandler(ListToolsRequestSchema, async () => {
    return {
        tools: [
            {
                name: 'vex_search',
                description: 'Semantic search across Vex documentation, code, examples, and TODOs. Automatically routes queries to appropriate collections.',
                inputSchema: {
                    type: 'object',
                    properties: {
                        query: {
                            type: 'string',
                            description: 'The search query (e.g., "how to implement generics", "TODO for type system")',
                        },
                        limit: {
                            type: 'number',
                            description: 'Maximum number of results (default: 10)',
                            default: 10,
                        },
                    },
                    required: ['query'],
                },
            },
            {
                name: 'vex_find_examples',
                description: 'Find code examples for a specific Vex feature or pattern',
                inputSchema: {
                    type: 'object',
                    properties: {
                        feature: {
                            type: 'string',
                            description: 'The feature or pattern to find examples for (e.g., "async/await", "pattern matching", "generics")',
                        },
                        limit: {
                            type: 'number',
                            description: 'Maximum number of examples (default: 5)',
                            default: 5,
                        },
                    },
                    required: ['feature'],
                },
            },
            {
                name: 'vex_find_todos',
                description: 'Search and filter TODO items from the Vex codebase',
                inputSchema: {
                    type: 'object',
                    properties: {
                        query: {
                            type: 'string',
                            description: 'Optional search query for TODOs',
                        },
                        priority: {
                            type: 'string',
                            description: 'Filter by priority (high, medium, low)',
                            enum: ['high', 'medium', 'low'],
                        },
                        category: {
                            type: 'string',
                            description: 'Filter by category (e.g., "type-system", "parser", "stdlib")',
                        },
                        limit: {
                            type: 'number',
                            description: 'Maximum number of TODOs (default: 20)',
                            default: 20,
                        },
                    },
                },
            },
            {
                name: 'vex_find_similar_code',
                description: 'Find similar code snippets in the Vex codebase',
                inputSchema: {
                    type: 'object',
                    properties: {
                        code: {
                            type: 'string',
                            description: 'The code snippet to find similar examples for',
                        },
                        limit: {
                            type: 'number',
                            description: 'Maximum number of results (default: 5)',
                            default: 5,
                        },
                    },
                    required: ['code'],
                },
            },
            {
                name: 'vex_get_file_context',
                description: 'Get relevant context for a specific file in the Vex codebase',
                inputSchema: {
                    type: 'object',
                    properties: {
                        filePath: {
                            type: 'string',
                            description: 'Path to the file (relative or absolute)',
                        },
                        limit: {
                            type: 'number',
                            description: 'Maximum number of context chunks (default: 3)',
                            default: 3,
                        },
                    },
                    required: ['filePath'],
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
            case 'vex_search': {
                const { query, limit = 10 } = args;
                const results = await searcher.smartSearch(query, limit);
                return {
                    content: [
                        {
                            type: 'text',
                            text: JSON.stringify(results, null, 2),
                        },
                    ],
                };
            }
            case 'vex_find_examples': {
                const { feature, limit = 5 } = args;
                const results = await searcher.findExamples(feature, limit);
                return {
                    content: [
                        {
                            type: 'text',
                            text: JSON.stringify(results, null, 2),
                        },
                    ],
                };
            }
            case 'vex_find_todos': {
                const { query, priority, category, limit = 20 } = args;
                const results = await searcher.searchTodos(query, priority, category, limit);
                return {
                    content: [
                        {
                            type: 'text',
                            text: JSON.stringify(results, null, 2),
                        },
                    ],
                };
            }
            case 'vex_find_similar_code': {
                const { code, limit = 5 } = args;
                const results = await searcher.findSimilarCode(code, limit);
                return {
                    content: [
                        {
                            type: 'text',
                            text: JSON.stringify(results, null, 2),
                        },
                    ],
                };
            }
            case 'vex_get_file_context': {
                const { filePath, limit = 3 } = args;
                const results = await searcher.getFileContext(filePath, limit);
                return {
                    content: [
                        {
                            type: 'text',
                            text: JSON.stringify(results, null, 2),
                        },
                    ],
                };
            }
            default:
                throw new Error(`Unknown tool: ${name}`);
        }
    }
    catch (error) {
        const errorMessage = error instanceof Error ? error.message : String(error);
        return {
            content: [
                {
                    type: 'text',
                    text: JSON.stringify({ error: errorMessage }, null, 2),
                },
            ],
            isError: true,
        };
    }
});
async function main() {
    console.error('Initializing Vex Qdrant RAG system...');
    await searcher.initialize();
    console.error('Vex Qdrant RAG MCP server ready');
    const transport = new StdioServerTransport();
    await server.connect(transport);
}
main().catch((error) => {
    console.error('Fatal error:', error);
    process.exit(1);
});
//# sourceMappingURL=mcp-server.js.map