/**
 * Vex Qdrant RAG System - Documentation Indexer
 */
export declare class DocumentationIndexer {
    private client;
    constructor();
    /**
     * Index all Specifications
     */
    indexSpecifications(): Promise<void>;
    /**
     * Index all documentation files
     */
    indexDocs(): Promise<void>;
    /**
     * Index code files
     */
    indexCode(): Promise<void>;
    /**
     * Index examples
     */
    indexExamples(): Promise<void>;
    /**
     * Index TODO items
     */
    indexTodos(): Promise<void>;
    /**
     * Index a markdown file
     */
    private indexMarkdownFile;
    /**
     * Index a code file
     */
    private indexCodeFile;
    /**
     * Index an example file
     */
    private indexExampleFile;
    /**
     * Parse TODO items from markdown
     */
    private parseTodos;
    /**
     * Index everything
     */
    indexAll(): Promise<void>;
    /**
     * Show collection statistics
     */
    showStats(): Promise<void>;
}
export default DocumentationIndexer;
//# sourceMappingURL=indexer.d.ts.map