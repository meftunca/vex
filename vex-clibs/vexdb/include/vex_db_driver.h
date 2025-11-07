/**
 * vex_db_driver.h
 * Universal DB access interface for Vex via C function pointers.
 * - Zero-abstraction/zero-allocation (as feasible)
 * - Atomic operations
 * - LLVM LTO friendly (no vtables; plain function pointers)
 */
#ifndef VEX_DB_DRIVER_H
#define VEX_DB_DRIVER_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define VEX_DB_API_VERSION 1u

typedef enum {
    VEX_DB_OK = 0,
    VEX_DB_ERROR_CONNECT,
    VEX_DB_ERROR_EXECUTION,
    VEX_DB_ERROR_NOT_FOUND,
    VEX_DB_ERROR_INVALID_PARAM,
    VEX_DB_ERROR_UNKNOWN
} VexDbStatus;

typedef struct {
    VexDbStatus code;
    char message[128];  /* null-terminated; keeps zero-alloc by being inline */
} VexDbError;

typedef enum {
    VEX_T_NULL=0, VEX_T_BOOL, VEX_T_I64, VEX_T_F64,
    VEX_T_TEXT,  /* UTF-8 */
    VEX_T_BIN,   /* arbitrary bytes (BSON/bytea) */
    VEX_T_JSON
} VexDbType;

/* Zero-copy payload returned from fetch_next (lifetime tied to result set) */
typedef enum {
    VEX_LIFE_RESULT_OWNED = 0,
    VEX_LIFE_ROW_BUFFER,
    VEX_LIFE_DRIVER_ARENA
} VexDbLifetime;

typedef struct {
    const char *data;   /* may be text or binary */
    size_t length;      /* in bytes */
    int is_null;
    VexDbLifetime lifetime;
    const void *owner;  /* result/cursor it is tied to */
    VexDbType type;     /* optional hint/type */
} VexDbPayload;

typedef struct {
    const void *data;
    size_t length;
    VexDbType type;
    int is_binary;
} VexDbValue;

typedef struct {
    void *native_conn;   /* e.g., PGconn*, mongoc_client_t* */
    VexDbError error;
    uint32_t api_version;
    uint32_t capabilities; /* see flags below */
} VexConnection;

typedef struct {
    void *native_result; /* e.g., PGresult*, mongoc_cursor_t* */
    long row_affected;
    VexDbError error;
    uint32_t column_count;
    /* optional helpers; may be NULL if unknown */
    const char* (*column_name)(void *native_result, uint32_t idx);
    VexDbType  (*column_type)(void *native_result, uint32_t idx);
    int        (*column_is_binary)(void *native_result, uint32_t idx);
} VexResultSet;

typedef enum {
    VEX_CAP_SQL            = 1u << 0,
    VEX_CAP_DOC_FIND       = 1u << 1,
    VEX_CAP_ASYNC          = 1u << 2,
    VEX_CAP_BINARY_PARAMS  = 1u << 3,
    VEX_CAP_TXN            = 1u << 4,
    VEX_CAP_PIPELINE       = 1u << 5
} VexDbCapabilities;

typedef struct VexDbDriver {
    const char *driver_name;
    uint32_t api_version;   /* = VEX_DB_API_VERSION */
    uint32_t capabilities;  /* bitmask */

    /* Atomic lifecycle */
    VexConnection (*connect)(const char *conninfo);
    void (*disconnect)(VexConnection *conn);
    void (*clear_result)(VexResultSet *res);

    /* Core query ops */
    VexResultSet (*execute_query)(
        VexConnection *conn,
        const char *query,
        int nParams,
        const VexDbValue *params
    );

    VexResultSet (*find_doc)(
        VexConnection *conn,
        const char *collection_name,
        const VexDbPayload filter_payload
    ); /* may be NULL for pure SQL drivers */

    VexDbPayload (*fetch_next)(VexResultSet *res);

    /* --- OPTIONAL ASYNC SURFACE (may be NULL if unsupported) ---
     *
     * These map 1:1 to libpq and libmongoc patterns.
     * See src/vex_pg.c and src/vex_mongo.c for detailed usage comments.
     */
    int  (*get_event_fd)(VexConnection *conn);   /* -1 if not available */
    int  (*wants_read)(VexConnection *conn);
    int  (*wants_write)(VexConnection *conn);

    int  (*start_execute)(VexConnection *conn,
                          const char *query, int nParams, const VexDbValue *params);
    int  (*poll_ready)(VexConnection *conn);     /* 1 ready, 0 not-yet, <0 error */
    int  (*result_ready)(VexConnection *conn);   /* 1 has result, 0 none */
    VexResultSet (*get_result)(VexConnection *conn);

    int  (*cancel)(VexConnection *conn);
    void (*set_timeout_ms)(VexConnection *conn, uint32_t ms);
} VexDbDriver;

/* Externs to link statically and allow LTO to inline through a const pointer */
extern VexDbDriver VEX_DRIVER_POSTGRES;
extern VexDbDriver VEX_DRIVER_MONGODB;

#ifdef __cplusplus
}
#endif

#endif /* VEX_DB_DRIVER_H */
