/**
 * Vex Qdrant RAG System - Text Chunker
 */
export interface Chunk {
    text: string;
    metadata: Record<string, any>;
}
export declare class TextChunker {
    /**
     * Split text into chunks with overlap
     */
    static chunkText(text: string, metadata?: Record<string, any>, chunkSize?: number, overlap?: number): Chunk[];
    /**
     * Chunk markdown file with section awareness
     */
    static chunkMarkdown(content: string, metadata?: Record<string, any>): Chunk[];
    /**
     * Split markdown by headers
     */
    private static splitByHeaders;
    /**
     * Chunk code with function/block awareness
     */
    static chunkCode(code: string, language: string, metadata?: Record<string, any>): Chunk[];
}
export default TextChunker;
//# sourceMappingURL=chunker.d.ts.map