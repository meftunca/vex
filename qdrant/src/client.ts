/**
 * Vex Qdrant RAG System - Qdrant Client
 */

import { QdrantClient } from "@qdrant/js-client-rest";
import { Ollama } from "ollama";
import config from "./config.js";

export class VexQdrantClient {
  private qdrant: QdrantClient;
  private ollama: Ollama;

  constructor() {
    this.qdrant = new QdrantClient({
      url: config.qdrant.url,
      apiKey: config.qdrant.apiKey,
    });

    this.ollama = new Ollama({
      host: config.ollama.url,
    });
  }

  /**
   * Initialize Qdrant client
   */
  async initialize(): Promise<void> {
    await this.initializeCollections();
  }

  /**
   * Initialize all collections
   */
  async initializeCollections(): Promise<void> {
    const collections = [
      config.collections.docs,
      config.collections.code,
      config.collections.todos,
      config.collections.examples,
      "vex_tests",
    ];

    for (const collection of collections) {
      await this.ensureCollection(collection);
    }
  }

  /**
   * Generate embeddings using Ollama
   */
  async generateEmbedding(text: string): Promise<number[]> {
    const response = await this.ollama.embeddings({
      model: config.ollama.model,
      prompt: text,
    });
    return response.embedding;
  }

  /**
   * Create collection if it doesn't exist
   */
  async ensureCollection(collectionName: string): Promise<void> {
    try {
      await this.qdrant.getCollection(collectionName);
      console.log(`✓ Collection "${collectionName}" already exists`);
    } catch (error) {
      console.log(`Creating collection "${collectionName}"...`);
      await this.qdrant.createCollection(collectionName, {
        vectors: {
          size: config.ollama.embeddingDim,
          distance: "Cosine",
        },
      });
      console.log(`✓ Collection "${collectionName}" created`);
    }
  }

  /**
   * Upsert points to collection
   */
  async upsert(
    collectionName: string,
    points: Array<{
      id: string;
      vector: number[];
      payload: Record<string, any>;
    }>
  ): Promise<void> {
    await this.qdrant.upsert(collectionName, {
      wait: true,
      points,
    });
  }

  /**
   * Search in collection
   */
  async search(
    collectionName: string,
    query: string,
    limit: number = config.search.defaultLimit,
    filter?: Record<string, any>,
    scoreThreshold: number = config.search.scoreThreshold
  ) {
    const embedding = await this.generateEmbedding(query);

    return await this.qdrant.search(collectionName, {
      vector: embedding,
      limit,
      filter,
      score_threshold: scoreThreshold,
    });
  }

  /**
   * Delete collection
   */
  async deleteCollection(collectionName: string): Promise<void> {
    try {
      await this.qdrant.deleteCollection(collectionName);
      console.log(`✓ Collection "${collectionName}" deleted`);
    } catch (error) {
      console.log(`Collection "${collectionName}" doesn't exist`);
    }
  }

  /**
   * Get collection info
   */
  async getCollectionInfo(collectionName: string) {
    return await this.qdrant.getCollection(collectionName);
  }

  /**
   * Count points in collection
   */
  async countPoints(collectionName: string): Promise<number> {
    const info = await this.qdrant.getCollection(collectionName);
    return info.points_count || 0;
  }
}

export default VexQdrantClient;
