/**
 * Vex Qdrant RAG System - Semantic Searcher
 */
import QdrantClient from './client.js';
import { getConfig } from './config.js';
class SemanticSearcher {
    client;
    config;
    constructor() {
        this.client = new QdrantClient();
        this.config = getConfig();
    }
    async initialize() {
        await this.client.initialize();
    }
    /**
     * Search in a single collection
     */
    async search(options) {
        const { collection, query, limit = 10, filter, scoreThreshold = 0.5 } = options;
        const results = await this.client.search(collection, query, limit, filter, scoreThreshold);
        return results.map(r => ({
            content: r.payload?.text || '',
            metadata: r.payload?.metadata || {},
            score: r.score || 0
        }));
    }
    /**
     * Multi-collection search with collection priority
     */
    async multiSearch(query, collections, limit = 5) {
        const results = new Map();
        for (const collection of collections) {
            const collectionResults = await this.search({
                collection,
                query,
                limit
            });
            results.set(collection, collectionResults);
        }
        return results;
    }
    /**
     * Smart search that routes queries to appropriate collections
     */
    async smartSearch(query, limit = 10) {
        const queryLower = query.toLowerCase();
        let targetCollections;
        // Route based on query intent
        if (queryLower.includes('todo') || queryLower.includes('fixme')) {
            targetCollections = ['vex_todos'];
        }
        else if (queryLower.includes('example') || queryLower.includes('how to')) {
            targetCollections = ['vex_examples', 'vex_docs'];
        }
        else if (queryLower.includes('test') || queryLower.includes('spec')) {
            targetCollections = ['vex_tests', 'vex_docs'];
        }
        else if (queryLower.includes('implement') || queryLower.includes('code')) {
            targetCollections = ['vex_code', 'vex_examples'];
        }
        else {
            // Default: search all collections with priority weighting
            targetCollections = ['vex_docs', 'vex_code', 'vex_examples'];
        }
        const multiResults = await this.multiSearch(query, targetCollections, limit);
        // Flatten and sort by score
        const allResults = [];
        for (const [collection, results] of multiResults) {
            // Add collection boost
            const boost = this.getCollectionBoost(collection, queryLower);
            allResults.push(...results.map(r => ({
                ...r,
                score: r.score * boost,
                metadata: { ...r.metadata, collection }
            })));
        }
        allResults.sort((a, b) => b.score - a.score);
        return allResults.slice(0, limit);
    }
    /**
     * Search for TODO items with filtering
     */
    async searchTodos(query, priority, category, limit = 20) {
        const filter = {};
        if (priority) {
            filter.priority = priority;
        }
        if (category) {
            filter.category = category;
        }
        if (query) {
            return this.search({
                collection: 'vex_todos',
                query,
                limit,
                filter: Object.keys(filter).length > 0 ? filter : undefined
            });
        }
        else {
            // Just filter, no semantic search
            const client = this.client;
            const results = await client.client.scroll('vex_todos', {
                filter,
                limit
            });
            return results.points.map((p) => ({
                content: p.payload.text,
                metadata: p.payload.metadata || {},
                score: 1.0
            }));
        }
    }
    /**
     * Get collection boost based on query intent
     */
    getCollectionBoost(collection, query) {
        const boosts = {
            vex_docs: 1.2, // Default priority
            vex_code: 1.0,
            vex_todos: 1.1,
            vex_examples: 1.15,
            vex_tests: 0.9
        };
        // Query-specific boosts
        if (query.includes('example') && collection === 'vex_examples') {
            return 1.5;
        }
        if (query.includes('test') && collection === 'vex_tests') {
            return 1.4;
        }
        if (query.includes('todo') && collection === 'vex_todos') {
            return 1.6;
        }
        return boosts[collection] || 1.0;
    }
    /**
     * Find similar code snippets
     */
    async findSimilarCode(codeSnippet, limit = 5) {
        return this.search({
            collection: 'vex_code',
            query: codeSnippet,
            limit
        });
    }
    /**
     * Find relevant examples for a feature
     */
    async findExamples(feature, limit = 5) {
        return this.search({
            collection: 'vex_examples',
            query: feature,
            limit
        });
    }
    /**
     * Get context for a specific file
     */
    async getFileContext(filePath, limit = 3) {
        return this.search({
            collection: 'vex_code',
            query: filePath,
            limit,
            filter: { file: filePath }
        });
    }
}
export default SemanticSearcher;
//# sourceMappingURL=searcher.js.map