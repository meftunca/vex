/**
 * Vex Qdrant RAG System - Configuration
 */
export declare const collectionNames: readonly ["vex_docs", "vex_code", "vex_todos", "vex_examples", "vex_tests"];
export type CollectionName = typeof collectionNames[number];
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
export declare const config: VexRAGConfig;
export declare function getConfig(): VexRAGConfig;
export default config;
//# sourceMappingURL=config.d.ts.map