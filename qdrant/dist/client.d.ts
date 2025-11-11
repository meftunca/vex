/**
 * Vex Qdrant RAG System - Qdrant Client
 */
export declare class VexQdrantClient {
    private qdrant;
    private ollama;
    constructor();
    /**
     * Initialize Qdrant client
     */
    initialize(): Promise<void>;
    /**
     * Initialize all collections
     */
    initializeCollections(): Promise<void>;
    /**
     * Generate embeddings using Ollama
     */
    generateEmbedding(text: string): Promise<number[]>;
    /**
     * Create collection if it doesn't exist
     */
    ensureCollection(collectionName: string): Promise<void>;
    /**
     * Upsert points to collection
     */
    upsert(collectionName: string, points: Array<{
        id: string | number;
        vector: number[];
        payload: Record<string, any>;
    }>): Promise<void>;
    /**
     * Search in collection
     */
    search(collectionName: string, query: string, limit?: number, filter?: Record<string, any>, scoreThreshold?: number): Promise<{
        id: string | number;
        version: number;
        score: number;
        payload?: Record<string, unknown> | {
            [key: string]: unknown;
        } | null | undefined;
        vector?: Record<string, unknown> | number[] | number[][] | {
            [key: string]: number[] | number[][] | {
                indices: number[];
                values: number[];
            } | undefined;
        } | null | undefined;
        shard_key?: string | number | Record<string, unknown> | null | undefined;
        order_value?: number | Record<string, unknown> | null | undefined;
    }[]>;
    /**
     * Delete collection
     */
    deleteCollection(collectionName: string): Promise<void>;
    /**
     * Get collection info
     */
    getCollectionInfo(collectionName: string): Promise<{
        status: "green" | "yellow" | "grey" | "red";
        optimizer_status: "ok" | {
            error: string;
        };
        vectors_count?: number | null | undefined;
        indexed_vectors_count?: number | null | undefined;
        points_count?: number | null | undefined;
        segments_count: number;
        config: {
            params: {
                vectors?: {
                    size: number;
                    distance: "Cosine" | "Euclid" | "Dot" | "Manhattan";
                    hnsw_config?: Record<string, unknown> | {
                        m?: number | null | undefined;
                        ef_construct?: number | null | undefined;
                        full_scan_threshold?: number | null | undefined;
                        max_indexing_threads?: number | null | undefined;
                        on_disk?: boolean | null | undefined;
                        payload_m?: number | null | undefined;
                    } | null | undefined;
                    quantization_config?: Record<string, unknown> | {
                        scalar: {
                            type: "int8";
                            quantile?: number | null | undefined;
                            always_ram?: boolean | null | undefined;
                        };
                    } | {
                        product: {
                            compression: "x4" | "x8" | "x16" | "x32" | "x64";
                            always_ram?: boolean | null | undefined;
                        };
                    } | {
                        binary: {
                            always_ram?: boolean | null | undefined;
                            encoding?: Record<string, unknown> | "one_bit" | "two_bits" | "one_and_half_bits" | null | undefined;
                            query_encoding?: Record<string, unknown> | "default" | "binary" | "scalar4bits" | "scalar8bits" | null | undefined;
                        };
                    } | null | undefined;
                    on_disk?: boolean | null | undefined;
                    datatype?: Record<string, unknown> | "float32" | "uint8" | "float16" | null | undefined;
                    multivector_config?: Record<string, unknown> | {
                        comparator: "max_sim";
                    } | null | undefined;
                } | {
                    [key: string]: {
                        size: number;
                        distance: "Cosine" | "Euclid" | "Dot" | "Manhattan";
                        hnsw_config?: Record<string, unknown> | {
                            m?: number | null | undefined;
                            ef_construct?: number | null | undefined;
                            full_scan_threshold?: number | null | undefined;
                            max_indexing_threads?: number | null | undefined;
                            on_disk?: boolean | null | undefined;
                            payload_m?: number | null | undefined;
                        } | null | undefined;
                        quantization_config?: Record<string, unknown> | {
                            scalar: {
                                type: "int8";
                                quantile?: number | null | undefined;
                                always_ram?: boolean | null | undefined;
                            };
                        } | {
                            product: {
                                compression: "x4" | "x8" | "x16" | "x32" | "x64";
                                always_ram?: boolean | null | undefined;
                            };
                        } | {
                            binary: {
                                always_ram?: boolean | null | undefined;
                                encoding?: Record<string, unknown> | "one_bit" | "two_bits" | "one_and_half_bits" | null | undefined;
                                query_encoding?: Record<string, unknown> | "default" | "binary" | "scalar4bits" | "scalar8bits" | null | undefined;
                            };
                        } | null | undefined;
                        on_disk?: boolean | null | undefined;
                        datatype?: Record<string, unknown> | "float32" | "uint8" | "float16" | null | undefined;
                        multivector_config?: Record<string, unknown> | {
                            comparator: "max_sim";
                        } | null | undefined;
                    } | undefined;
                } | undefined;
                shard_number?: number | undefined;
                sharding_method?: Record<string, unknown> | "auto" | "custom" | null | undefined;
                replication_factor?: number | undefined;
                write_consistency_factor?: number | undefined;
                read_fan_out_factor?: number | null | undefined;
                on_disk_payload?: boolean | undefined;
                sparse_vectors?: {
                    [key: string]: {
                        index?: Record<string, unknown> | {
                            full_scan_threshold?: number | null | undefined;
                            on_disk?: boolean | null | undefined;
                            datatype?: Record<string, unknown> | "float32" | "uint8" | "float16" | null | undefined;
                        } | null | undefined;
                        modifier?: Record<string, unknown> | "none" | "idf" | null | undefined;
                    } | undefined;
                } | null | undefined;
            };
            hnsw_config: {
                m: number;
                ef_construct: number;
                full_scan_threshold: number;
                max_indexing_threads?: number | undefined;
                on_disk?: boolean | null | undefined;
                payload_m?: number | null | undefined;
            };
            optimizer_config: {
                deleted_threshold: number;
                vacuum_min_vector_number: number;
                default_segment_number: number;
                max_segment_size?: number | null | undefined;
                memmap_threshold?: number | null | undefined;
                indexing_threshold?: number | null | undefined;
                flush_interval_sec: number;
                max_optimization_threads?: number | null | undefined;
            };
            wal_config?: Record<string, unknown> | {
                wal_capacity_mb: number;
                wal_segments_ahead: number;
            } | null | undefined;
            quantization_config?: Record<string, unknown> | {
                scalar: {
                    type: "int8";
                    quantile?: number | null | undefined;
                    always_ram?: boolean | null | undefined;
                };
            } | {
                product: {
                    compression: "x4" | "x8" | "x16" | "x32" | "x64";
                    always_ram?: boolean | null | undefined;
                };
            } | {
                binary: {
                    always_ram?: boolean | null | undefined;
                    encoding?: Record<string, unknown> | "one_bit" | "two_bits" | "one_and_half_bits" | null | undefined;
                    query_encoding?: Record<string, unknown> | "default" | "binary" | "scalar4bits" | "scalar8bits" | null | undefined;
                };
            } | null | undefined;
            strict_mode_config?: Record<string, unknown> | {
                enabled?: boolean | null | undefined;
                max_query_limit?: number | null | undefined;
                max_timeout?: number | null | undefined;
                unindexed_filtering_retrieve?: boolean | null | undefined;
                unindexed_filtering_update?: boolean | null | undefined;
                search_max_hnsw_ef?: number | null | undefined;
                search_allow_exact?: boolean | null | undefined;
                search_max_oversampling?: number | null | undefined;
                upsert_max_batchsize?: number | null | undefined;
                max_collection_vector_size_bytes?: number | null | undefined;
                read_rate_limit?: number | null | undefined;
                write_rate_limit?: number | null | undefined;
                max_collection_payload_size_bytes?: number | null | undefined;
                max_points_count?: number | null | undefined;
                filter_max_conditions?: number | null | undefined;
                condition_max_size?: number | null | undefined;
                multivector_config?: Record<string, unknown> | {
                    [key: string]: {
                        max_vectors?: number | null | undefined;
                    } | undefined;
                } | null | undefined;
                sparse_config?: Record<string, unknown> | {
                    [key: string]: {
                        max_length?: number | null | undefined;
                    } | undefined;
                } | null | undefined;
            } | null | undefined;
        };
        payload_schema: {
            [key: string]: {
                data_type: "keyword" | "integer" | "float" | "geo" | "text" | "bool" | "datetime" | "uuid";
                params?: Record<string, unknown> | {
                    type: "keyword";
                    is_tenant?: boolean | null | undefined;
                    on_disk?: boolean | null | undefined;
                } | {
                    type: "integer";
                    lookup?: boolean | null | undefined;
                    range?: boolean | null | undefined;
                    is_principal?: boolean | null | undefined;
                    on_disk?: boolean | null | undefined;
                } | {
                    type: "float";
                    is_principal?: boolean | null | undefined;
                    on_disk?: boolean | null | undefined;
                } | {
                    type: "geo";
                    on_disk?: boolean | null | undefined;
                } | {
                    type: "text";
                    tokenizer?: "prefix" | "whitespace" | "word" | "multilingual" | undefined;
                    min_token_len?: number | null | undefined;
                    max_token_len?: number | null | undefined;
                    lowercase?: boolean | null | undefined;
                    phrase_matching?: boolean | null | undefined;
                    stopwords?: Record<string, unknown> | "arabic" | "azerbaijani" | "basque" | "bengali" | "catalan" | "chinese" | "danish" | "dutch" | "english" | "finnish" | "french" | "german" | "greek" | "hebrew" | "hinglish" | "hungarian" | "indonesian" | "italian" | "japanese" | "kazakh" | "nepali" | "norwegian" | "portuguese" | "romanian" | "russian" | "slovene" | "spanish" | "swedish" | "tajik" | "turkish" | {
                        languages?: ("arabic" | "azerbaijani" | "basque" | "bengali" | "catalan" | "chinese" | "danish" | "dutch" | "english" | "finnish" | "french" | "german" | "greek" | "hebrew" | "hinglish" | "hungarian" | "indonesian" | "italian" | "japanese" | "kazakh" | "nepali" | "norwegian" | "portuguese" | "romanian" | "russian" | "slovene" | "spanish" | "swedish" | "tajik" | "turkish")[] | null | undefined;
                        custom?: string[] | null | undefined;
                    } | null | undefined;
                    on_disk?: boolean | null | undefined;
                    stemmer?: Record<string, unknown> | {
                        type: "snowball";
                        language: "arabic" | "danish" | "dutch" | "english" | "finnish" | "french" | "german" | "greek" | "hungarian" | "italian" | "norwegian" | "portuguese" | "romanian" | "russian" | "spanish" | "swedish" | "turkish" | "armenian" | "tamil";
                    } | null | undefined;
                } | {
                    type: "bool";
                    on_disk?: boolean | null | undefined;
                } | {
                    type: "datetime";
                    is_principal?: boolean | null | undefined;
                    on_disk?: boolean | null | undefined;
                } | {
                    type: "uuid";
                    is_tenant?: boolean | null | undefined;
                    on_disk?: boolean | null | undefined;
                } | null | undefined;
                points: number;
            } | undefined;
        };
    }>;
    /**
     * Count points in collection
     */
    countPoints(collectionName: string): Promise<number>;
}
export default VexQdrantClient;
//# sourceMappingURL=client.d.ts.map