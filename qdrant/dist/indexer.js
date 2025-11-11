/**
 * Vex Qdrant RAG System - Documentation Indexer
 */
import { readFileSync } from 'fs';
import { relative, extname, basename } from 'path';
import { glob } from 'glob';
import VexQdrantClient from './client.js';
import TextChunker from './chunker.js';
import config from './config.js';
export class DocumentationIndexer {
    client;
    constructor() {
        this.client = new VexQdrantClient();
    }
    /**
     * Index all Specifications
     */
    async indexSpecifications() {
        console.log('\n📚 Indexing Specifications...');
        await this.client.ensureCollection(config.collections.docs);
        const mdFiles = await glob('**/*.md', {
            cwd: config.paths.specs,
            absolute: true,
        });
        let totalChunks = 0;
        for (const file of mdFiles) {
            const chunks = await this.indexMarkdownFile(file, 'specification');
            totalChunks += chunks;
        }
        console.log(`✓ Indexed ${mdFiles.length} specification files (${totalChunks} chunks)`);
    }
    /**
     * Index all documentation files
     */
    async indexDocs() {
        console.log('\n📝 Indexing Documentation...');
        await this.client.ensureCollection(config.collections.docs);
        const mdFiles = await glob('**/*.md', {
            cwd: config.paths.docs,
            absolute: true,
        });
        let totalChunks = 0;
        for (const file of mdFiles) {
            const chunks = await this.indexMarkdownFile(file, 'documentation');
            totalChunks += chunks;
        }
        console.log(`✓ Indexed ${mdFiles.length} documentation files (${totalChunks} chunks)`);
    }
    /**
     * Index code files
     */
    async indexCode() {
        console.log('\n💻 Indexing Code...');
        await this.client.ensureCollection(config.collections.code);
        let totalFiles = 0;
        let totalChunks = 0;
        for (const codeDir of config.paths.code) {
            const vxFiles = await glob('**/*.{vx,rs}', {
                cwd: codeDir,
                absolute: true,
                ignore: ['**/target/**', '**/node_modules/**', '**/dist/**'],
            });
            for (const file of vxFiles) {
                const chunks = await this.indexCodeFile(file);
                totalChunks += chunks;
                totalFiles++;
            }
        }
        console.log(`✓ Indexed ${totalFiles} code files (${totalChunks} chunks)`);
    }
    /**
     * Index examples
     */
    async indexExamples() {
        console.log('\n📦 Indexing Examples...');
        await this.client.ensureCollection(config.collections.examples);
        const vxFiles = await glob('**/*.vx', {
            cwd: config.paths.examples,
            absolute: true,
        });
        let totalChunks = 0;
        for (const file of vxFiles) {
            const chunks = await this.indexExampleFile(file);
            totalChunks += chunks;
        }
        console.log(`✓ Indexed ${vxFiles.length} example files (${totalChunks} chunks)`);
    }
    /**
     * Index TODO items
     */
    async indexTodos() {
        console.log('\n✅ Indexing TODOs...');
        await this.client.ensureCollection(config.collections.todos);
        const content = readFileSync(config.paths.todoFile, 'utf-8');
        const todos = this.parseTodos(content);
        const points = [];
        for (let i = 0; i < todos.length; i++) {
            const todo = todos[i];
            const embedding = await this.client.generateEmbedding(`${todo.title}\n${todo.description}`);
            points.push({
                id: `todo-${i}`,
                vector: embedding,
                payload: {
                    type: 'todo',
                    title: todo.title,
                    description: todo.description,
                    status: todo.status,
                    files: todo.files,
                    priority: todo.priority,
                    dependencies: todo.dependencies,
                },
            });
        }
        await this.client.upsert(config.collections.todos, points);
        console.log(`✓ Indexed ${todos.length} TODO items`);
    }
    /**
     * Index a markdown file
     */
    async indexMarkdownFile(filePath, docType) {
        const content = readFileSync(filePath, 'utf-8');
        const relativePath = relative(config.paths.root, filePath);
        const chunks = TextChunker.chunkMarkdown(content, {
            type: docType,
            file: relativePath,
            fileName: basename(filePath),
        });
        const points = [];
        for (let i = 0; i < chunks.length; i++) {
            const chunk = chunks[i];
            const embedding = await this.client.generateEmbedding(chunk.text);
            points.push({
                id: `${docType}-${relativePath}-${i}`,
                vector: embedding,
                payload: {
                    ...chunk.metadata,
                    text: chunk.text,
                    charCount: chunk.text.length,
                },
            });
        }
        await this.client.upsert(config.collections.docs, points);
        return chunks.length;
    }
    /**
     * Index a code file
     */
    async indexCodeFile(filePath) {
        const content = readFileSync(filePath, 'utf-8');
        const relativePath = relative(config.paths.root, filePath);
        const ext = extname(filePath).slice(1);
        const chunks = TextChunker.chunkCode(content, ext, {
            type: 'code',
            file: relativePath,
            fileName: basename(filePath),
            language: ext === 'vx' ? 'vex' : ext,
        });
        const points = [];
        for (let i = 0; i < chunks.length; i++) {
            const chunk = chunks[i];
            const embedding = await this.client.generateEmbedding(chunk.text);
            points.push({
                id: `code-${relativePath}-${i}`,
                vector: embedding,
                payload: {
                    ...chunk.metadata,
                    text: chunk.text,
                    charCount: chunk.text.length,
                },
            });
        }
        await this.client.upsert(config.collections.code, points);
        return chunks.length;
    }
    /**
     * Index an example file
     */
    async indexExampleFile(filePath) {
        const content = readFileSync(filePath, 'utf-8');
        const relativePath = relative(config.paths.examples, filePath);
        const chunks = TextChunker.chunkCode(content, 'vex', {
            type: 'example',
            file: relativePath,
            fileName: basename(filePath),
            category: relativePath.split('/')[0],
        });
        const points = [];
        for (let i = 0; i < chunks.length; i++) {
            const chunk = chunks[i];
            const embedding = await this.client.generateEmbedding(chunk.text);
            points.push({
                id: `example-${relativePath}-${i}`,
                vector: embedding,
                payload: {
                    ...chunk.metadata,
                    text: chunk.text,
                    charCount: chunk.text.length,
                },
            });
        }
        await this.client.upsert(config.collections.examples, points);
        return chunks.length;
    }
    /**
     * Parse TODO items from markdown
     */
    parseTodos(content) {
        const todos = [];
        const lines = content.split('\n');
        let currentTodo = null;
        for (const line of lines) {
            const todoMatch = line.match(/^-\s+\[([ x])\]\s+(.+)$/);
            if (todoMatch) {
                if (currentTodo) {
                    todos.push(currentTodo);
                }
                currentTodo = {
                    title: todoMatch[2],
                    description: '',
                    status: todoMatch[1] === 'x' ? 'completed' : 'pending',
                    files: [],
                    priority: 'medium',
                    dependencies: [],
                };
            }
            else if (currentTodo && line.trim().startsWith('-')) {
                currentTodo.description += line.trim().substring(1).trim() + '\n';
                // Extract file references
                const fileMatch = line.match(/Files?:\s+([^\n]+)/i);
                if (fileMatch) {
                    currentTodo.files = fileMatch[1].split(',').map((f) => f.trim());
                }
            }
        }
        if (currentTodo) {
            todos.push(currentTodo);
        }
        return todos;
    }
    /**
     * Index everything
     */
    async indexAll() {
        console.log('🚀 Starting full indexing...\n');
        const startTime = Date.now();
        await this.indexSpecifications();
        await this.indexDocs();
        await this.indexCode();
        await this.indexExamples();
        await this.indexTodos();
        const duration = ((Date.now() - startTime) / 1000).toFixed(2);
        console.log(`\n✅ Indexing completed in ${duration}s`);
        // Show collection stats
        await this.showStats();
    }
    /**
     * Show collection statistics
     */
    async showStats() {
        console.log('\n📊 Collection Statistics:');
        for (const [name, collectionName] of Object.entries(config.collections)) {
            try {
                const count = await this.client.countPoints(collectionName);
                console.log(`  ${name}: ${count} points`);
            }
            catch (error) {
                console.log(`  ${name}: not indexed yet`);
            }
        }
    }
}
export default DocumentationIndexer;
//# sourceMappingURL=indexer.js.map