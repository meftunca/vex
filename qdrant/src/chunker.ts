/**
 * Vex Qdrant RAG System - Text Chunker
 */

import config from "./config.js";

export interface Chunk {
  text: string;
  metadata: Record<string, any>;
}

export class TextChunker {
  /**
   * Split text into chunks with overlap
   */
  static chunkText(
    text: string,
    metadata: Record<string, any> = {},
    chunkSize: number = config.chunking.size,
    overlap: number = config.chunking.overlap
  ): Chunk[] {
    const chunks: Chunk[] = [];
    const lines = text.split("\n");
    let currentChunk: string[] = [];
    let currentSize = 0;

    for (const line of lines) {
      const lineSize = line.length;

      if (currentSize + lineSize > chunkSize && currentChunk.length > 0) {
        // Save current chunk
        chunks.push({
          text: currentChunk.join("\n"),
          metadata: { ...metadata, chunkIndex: chunks.length },
        });

        // Keep overlap
        const overlapLines = Math.floor(
          (overlap / chunkSize) * currentChunk.length
        );
        currentChunk = currentChunk.slice(-overlapLines);
        currentSize = currentChunk.reduce((sum, l) => sum + l.length, 0);
      }

      currentChunk.push(line);
      currentSize += lineSize;

      // Prevent chunks from being too large
      if (currentSize > config.chunking.maxSize) {
        chunks.push({
          text: currentChunk.join("\n"),
          metadata: { ...metadata, chunkIndex: chunks.length },
        });
        currentChunk = [];
        currentSize = 0;
      }
    }

    // Add remaining chunk
    if (currentChunk.length > 0) {
      chunks.push({
        text: currentChunk.join("\n"),
        metadata: { ...metadata, chunkIndex: chunks.length },
      });
    }

    return chunks;
  }

  /**
   * Chunk markdown file with section awareness
   */
  static chunkMarkdown(
    content: string,
    metadata: Record<string, any> = {}
  ): Chunk[] {
    const chunks: Chunk[] = [];
    const sections = this.splitByHeaders(content);

    for (const section of sections) {
      const sectionChunks = this.chunkText(section.content, {
        ...metadata,
        section: section.header,
        level: section.level,
      });
      chunks.push(...sectionChunks);
    }

    return chunks;
  }

  /**
   * Split markdown by headers
   */
  private static splitByHeaders(
    content: string
  ): Array<{ header: string; level: number; content: string }> {
    const sections: Array<{ header: string; level: number; content: string }> =
      [];
    const lines = content.split("\n");
    let currentHeader = "";
    let currentLevel = 0;
    let currentContent: string[] = [];

    for (const line of lines) {
      const headerMatch = line.match(/^(#{1,6})\s+(.+)$/);

      if (headerMatch) {
        // Save previous section (with content)
        if (currentContent.length > 1) {
          // More than just header
          sections.push({
            header: currentHeader,
            level: currentLevel,
            content: currentContent.join("\n"),
          });
        }

        // Start new section
        currentLevel = headerMatch[1].length;
        currentHeader = headerMatch[2];
        currentContent = [line]; // Include header in content
      } else {
        currentContent.push(line);
      }
    }

    // Add last section (with content)
    if (currentContent.length > 1) {
      sections.push({
        header: currentHeader,
        level: currentLevel,
        content: currentContent.join("\n"),
      });
    }

    return sections;
  }

  /**
   * Chunk code with function/block awareness
   */
  static chunkCode(
    code: string,
    language: string,
    metadata: Record<string, any> = {}
  ): Chunk[] {
    // Simple line-based chunking for now
    // TODO: Add AST-based chunking for better semantic boundaries
    return this.chunkText(code, { ...metadata, language, type: "code" });
  }
}

export default TextChunker;
