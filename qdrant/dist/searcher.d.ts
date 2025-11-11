/**
 * Vex Qdrant RAG System - Semantic Searcher
 */
import { CollectionName } from './config.js';
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
declare class SemanticSearcher {
    private client;
    private config;
    constructor();
    initialize(): Promise<void>;
    /**
     * Search in a single collection
     */
    search(options: SearchOptions): Promise<SearchResult[]>;
    /**
     * Multi-collection search with collection priority
     */
    multiSearch(query: string, collections: CollectionName[], limit?: number): Promise<Map<CollectionName, SearchResult[]>>;
    /**
     * Smart search that routes queries to appropriate collections
     */
    smartSearch(query: string, limit?: number): Promise<SearchResult[]>;
    /**
     * Search for TODO items with filtering
     */
    searchTodos(query?: string, priority?: string, category?: string, limit?: number): Promise<SearchResult[]>;
    /**
     * Get collection boost based on query intent
     */
    private getCollectionBoost;
    /**
     * Find similar code snippets
     */
    findSimilarCode(codeSnippet: string, limit?: number): Promise<SearchResult[]>;
    /**
     * Find relevant examples for a feature
     */
    findExamples(feature: string, limit?: number): Promise<SearchResult[]>;
    /**
     * Get context for a specific file
     */
    getFileContext(filePath: string, limit?: number): Promise<SearchResult[]>;
}
export default SemanticSearcher;
//# sourceMappingURL=searcher.d.ts.map