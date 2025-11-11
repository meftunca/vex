/**
 * Vex Qdrant RAG System - Semantic Searcher
 */

import QdrantClient from "./client.js";
import { CollectionName, getConfig } from "./config.js";

export interface SearchOptions {
  collection: CollectionName;
  query: string;
  limit?: number;
  filter?: Record<string, any>;
  scoreThreshold?: number;
}

export interface SearchResult {
  content: string;
  metadata: Record<string, any>;
  score: number;
}

class SemanticSearcher {
  private client: QdrantClient;
  private config: ReturnType<typeof getConfig>;

  constructor() {
    this.client = new QdrantClient();
    this.config = getConfig();
  }

  async initialize(): Promise<void> {
    await this.client.initialize();
  }

  /**
   * Search in a single collection
   */
  async search(options: SearchOptions): Promise<SearchResult[]> {
    const {
      collection,
      query,
      limit = 10,
      filter,
      scoreThreshold = 0.5,
    } = options;

    const results = await this.client.search(
      collection,
      query,
      limit,
      filter,
      scoreThreshold
    );

    return results.map((r) => {
      const payload = (r.payload as any) || {};
      return {
        content: payload.text || "",
        metadata: {
          file: payload.file || payload.fileName || "unknown",
          type: payload.type || "unknown",
          ...payload,
        },
        score: r.score || 0,
      };
    });
  }

  /**
   * Multi-collection search with collection priority
   */
  async multiSearch(
    query: string,
    collections: CollectionName[],
    limit = 5
  ): Promise<Map<CollectionName, SearchResult[]>> {
    const results = new Map<CollectionName, SearchResult[]>();

    for (const collection of collections) {
      const collectionResults = await this.search({
        collection,
        query,
        limit,
      });
      results.set(collection, collectionResults);
    }

    return results;
  }

  /**
   * Smart search that routes queries to appropriate collections
   */
  async smartSearch(query: string, limit = 10): Promise<SearchResult[]> {
    const queryLower = query.toLowerCase();
    let targetCollections: CollectionName[];

    // Route based on query intent
    if (queryLower.includes("todo") || queryLower.includes("fixme")) {
      targetCollections = ["vex_todos"];
    } else if (
      queryLower.includes("example") ||
      queryLower.includes("how to")
    ) {
      targetCollections = ["vex_examples", "vex_docs"];
    } else if (queryLower.includes("test") || queryLower.includes("spec")) {
      targetCollections = ["vex_tests", "vex_docs"];
    } else if (
      queryLower.includes("implement") ||
      queryLower.includes("code")
    ) {
      targetCollections = ["vex_code", "vex_examples"];
    } else {
      // Default: search all collections with priority weighting
      targetCollections = ["vex_docs", "vex_code", "vex_examples"];
    }

    const multiResults = await this.multiSearch(
      query,
      targetCollections,
      limit
    );

    // Flatten and sort by score
    const allResults: SearchResult[] = [];
    for (const [collection, results] of multiResults) {
      // Add collection boost
      const boost = this.getCollectionBoost(collection, queryLower);
      allResults.push(
        ...results.map((r) => ({
          ...r,
          score: r.score * boost,
          metadata: { ...r.metadata, collection },
        }))
      );
    }

    allResults.sort((a, b) => b.score - a.score);
    return allResults.slice(0, limit);
  }

  /**
   * Search TODOs with filters
   */
  async searchTodos(
    query?: string,
    priority?: string,
    category?: string,
    limit = 20
  ): Promise<SearchResult[]> {
    let searchQuery = query || "";
    let filter: Record<string, any> | undefined;

    // Build Qdrant filter
    const mustConditions: any[] = [];

    if (priority) {
      mustConditions.push({
        key: "priority",
        match: { value: priority },
      });
    }

    if (category) {
      mustConditions.push({
        key: "category",
        match: { value: category },
      });
    }

    if (mustConditions.length > 0) {
      filter = { must: mustConditions };
    }

    // If no query provided, search for all TODOs
    if (!searchQuery) {
      searchQuery = "TODO";
    }

    return this.search({
      collection: "vex_todos",
      query: searchQuery,
      limit,
      filter,
    });
  }

  /**
   * Get collection boost based on query intent
   */
  private getCollectionBoost(
    collection: CollectionName,
    query: string
  ): number {
    const boosts: Record<CollectionName, number> = {
      vex_docs: 1.2, // Default priority
      vex_code: 1.0,
      vex_todos: 1.1,
      vex_examples: 1.15,
      vex_tests: 0.9,
    };

    // Query-specific boosts
    if (query.includes("example") && collection === "vex_examples") {
      return 1.5;
    }
    if (query.includes("test") && collection === "vex_tests") {
      return 1.4;
    }
    if (query.includes("todo") && collection === "vex_todos") {
      return 1.6;
    }

    return boosts[collection] || 1.0;
  }

  /**
   * Find similar code snippets
   */
  async findSimilarCode(
    codeSnippet: string,
    limit = 5
  ): Promise<SearchResult[]> {
    return this.search({
      collection: "vex_code",
      query: codeSnippet,
      limit,
    });
  }

  /**
   * Find relevant examples for a feature
   */
  async findExamples(feature: string, limit = 5): Promise<SearchResult[]> {
    return this.search({
      collection: "vex_examples",
      query: feature,
      limit,
    });
  }

  /**
   * Get context for a specific file
   */
  async getFileContext(filePath: string, limit = 3): Promise<SearchResult[]> {
    return this.search({
      collection: "vex_code",
      query: filePath,
      limit,
      filter: { file: filePath },
    });
  }
}

export default SemanticSearcher;
