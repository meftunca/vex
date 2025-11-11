/**
 * Vex Qdrant RAG System - Configuration
 */

import { config as dotenvConfig } from "dotenv";
import { dirname, resolve } from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

dotenvConfig({ path: resolve(__dirname, "../.env") });

export const collectionNames = [
  "vex_docs",
  "vex_code",
  "vex_todos",
  "vex_examples",
  "vex_tests",
] as const;
export type CollectionName = (typeof collectionNames)[number];

export interface VexRAGConfig {
  qdrant: {
    url: string;
    apiKey?: string;
  };
  ollama: {
    url: string;
    model: string;
    embeddingDim: number;
  };
  collections: {
    docs: string;
    code: string;
    todos: string;
    examples: string;
  };
  paths: {
    root: string;
    specs: string;
    docs: string;
    code: string[];
    examples: string;
    todoFile: string;
  };
  chunking: {
    size: number;
    overlap: number;
    maxSize: number;
  };
  search: {
    defaultLimit: number;
    scoreThreshold: number;
    hybridSearch: boolean;
  };
  mcp: {
    serverName: string;
    version: string;
    port: number;
  };
}

export const config: VexRAGConfig = {
  qdrant: {
    url: process.env.QDRANT_URL || "http://localhost:6333",
    apiKey: process.env.QDRANT_API_KEY,
  },
  ollama: {
    url: process.env.OLLAMA_URL || "http://localhost:11434",
    model: process.env.OLLAMA_MODEL || "nomic-embed-text:latest",
    embeddingDim: parseInt(process.env.EMBEDDING_DIM || "768"),
  },
  collections: {
    docs: process.env.COLLECTION_DOCS || "vex_docs",
    code: process.env.COLLECTION_CODE || "vex_code",
    todos: process.env.COLLECTION_TODOS || "vex_todos",
    examples: process.env.COLLECTION_EXAMPLES || "vex_examples",
  },
  paths: {
    root: resolve(__dirname, process.env.VEX_ROOT || "../.."),
    specs: resolve(__dirname, process.env.SPECS_DIR || "../../Specifications"),
    docs: resolve(__dirname, process.env.DOCS_DIR || "../../docs"),
    code: (
      process.env.CODE_DIRS ||
      "../../vex-libs,../../vex-compiler,../../vex-parser,../../vex-runtime"
    )
      .split(",")
      .map((d) => resolve(__dirname, d.trim())),
    examples: resolve(__dirname, process.env.EXAMPLES_DIR || "../../examples"),
    todoFile: resolve(__dirname, process.env.TODO_FILE || "../../TODO.md"),
  },
  chunking: {
    size: parseInt(process.env.CHUNK_SIZE || "1000"),
    overlap: parseInt(process.env.CHUNK_OVERLAP || "200"),
    maxSize: parseInt(process.env.MAX_CHUNK_SIZE || "2000"),
  },
  search: {
    defaultLimit: parseInt(process.env.DEFAULT_LIMIT || "5"),
    scoreThreshold: parseFloat(process.env.SCORE_THRESHOLD || "0.7"),
    hybridSearch: process.env.ENABLE_HYBRID_SEARCH === "true",
  },
  mcp: {
    serverName: process.env.MCP_SERVER_NAME || "vex-qdrant-rag",
    version: process.env.MCP_SERVER_VERSION || "1.0.0",
    port: parseInt(process.env.MCP_PORT || "3000"),
  },
};

export function getConfig() {
  return config;
}

export default config;
