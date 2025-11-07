
/**
 * vex_db_driver.h — Universal DB access for Vex via C.
 * Full implementations for PostgreSQL/MySQL/SQLite (no mock data).
 * This header intentionally keeps the surface minimal and zero-copy friendly.
 */
#ifndef VEX_DB_DRIVER_H
#define VEX_DB_DRIVER_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define VEX_DB_API_VERSION 2u

typedef enum {
    VEX_DB_OK = 0,
    VEX_DB_ERROR_CONNECT,
    VEX_DB_ERROR_EXECUTION,
    VEX_DB_ERROR_NOT_FOUND,
    VEX_DB_ERROR_INVALID_PARAM,
    VEX_DB_ERROR_UNSUPPORTED,
    VEX_DB_ERROR_UNKNOWN
} VexDbStatus;

typedef struct {
    VexDbStatus code;
    char message[160];
} VexDbError;

typedef enum {
    VEX_T_NULL=0, VEX_T_BOOL, VEX_T_I64, VEX_T_F64,
    VEX_T_TEXT, VEX_T_BIN, VEX_T_JSON
} VexDbType;

typedef enum {
    VEX_LIFE_RESULT_OWNED = 0, /* memory valid while result exists (e.g., libpq) */
    VEX_LIFE_ROW_BUFFER,       /* valid until next row fetch (e.g., mysql/sqlite) */
    VEX_LIFE_DRIVER_ARENA
} VexDbLifetime;

/* Generic zero-copy payload (often "first column" view for simple fast path) */
typedef struct {
    const char *data;
    size_t length;
    int is_null;
    VexDbLifetime lifetime;
    const void *owner;
    VexDbType type;
} VexDbPayload;

/* Typed parameter */
typedef struct {
    const void *data;
    size_t length;
    VexDbType type;
    int is_binary;
} VexDbValue;

typedef struct {
    void *native_conn;
    VexDbError error;
    uint32_t api_version;
    uint32_t capabilities;
} VexConnection;

typedef struct {
    void *native_result; /* PGresult* | MYSQL_RES* | (sqlite3_stmt*) */
    long row_affected;
    VexDbError error;
    uint32_t column_count;
    const char* (*column_name)(void *native_result, uint32_t idx);
    VexDbType  (*column_type)(void *native_result, uint32_t idx);
    int        (*column_is_binary)(void *native_result, uint32_t idx);
    /* Internal cursor row index (for drivers that need it) */
    int _row_index;
} VexResultSet;

typedef enum {
    VEX_CAP_SQL            = 1u << 0,
    VEX_CAP_ASYNC          = 1u << 2,
    VEX_CAP_BINARY_PARAMS  = 1u << 3,
    VEX_CAP_TXN            = 1u << 4,
    VEX_CAP_PUBSUB         = 1u << 5,  /* PostgreSQL LISTEN/NOTIFY, Redis Pub/Sub */
    VEX_CAP_STREAMING      = 1u << 6   /* Cursor-based streaming */
} VexDbCapabilities;

typedef struct VexDbDriver {
    const char *driver_name;
    uint32_t api_version;
    uint32_t capabilities;

    VexConnection (*connect)(const char *conninfo);
    void (*disconnect)(VexConnection *conn);
    void (*clear_result)(VexResultSet *res);

    VexResultSet (*execute_query)(
        VexConnection *conn,
        const char *query,
        int nParams,
        const VexDbValue *params
    );

    VexDbPayload (*fetch_next)(VexResultSet *res);

    /* Optional async (only PostgreSQL implemented in this pack) */
    int  (*get_event_fd)(VexConnection *conn);
    int  (*wants_read)(VexConnection *conn);
    int  (*wants_write)(VexConnection *conn);
    int  (*start_execute)(VexConnection *conn, const char *query, int nParams, const VexDbValue *params);
    int  (*poll_ready)(VexConnection *conn);
    int  (*result_ready)(VexConnection *conn);
    VexResultSet (*get_result)(VexConnection *conn);
    int  (*cancel)(VexConnection *conn);
    void (*set_timeout_ms)(VexConnection *conn, uint32_t ms);
    
    /* Pub/Sub support (PostgreSQL LISTEN/NOTIFY, Redis Pub/Sub) */
    int  (*subscribe)(VexConnection *conn, const char *channel);
    int  (*unsubscribe)(VexConnection *conn, const char *channel);
    int  (*publish)(VexConnection *conn, const char *channel, const char *message, size_t message_len);
    int  (*poll_notifications)(VexConnection *conn); /* Check for pending notifications */
    VexDbPayload (*get_notification)(VexConnection *conn); /* Get next notification */
    
    /* Transaction support */
    int  (*begin_transaction)(VexConnection *conn);
    int  (*commit_transaction)(VexConnection *conn);
    int  (*rollback_transaction)(VexConnection *conn);
    
    /* Cursor-based streaming (for large result sets) */
    int  (*declare_cursor)(VexConnection *conn, const char *cursor_name, const char *query);
    VexResultSet (*fetch_from_cursor)(VexConnection *conn, const char *cursor_name, int fetch_size);
    int  (*close_cursor)(VexConnection *conn, const char *cursor_name);
} VexDbDriver;

/* Extern drivers */
extern VexDbDriver VEX_DRIVER_POSTGRES;
extern VexDbDriver VEX_DRIVER_MYSQL;
extern VexDbDriver VEX_DRIVER_SQLITE;
extern VexDbDriver VEX_DRIVER_MONGO;
extern VexDbDriver VEX_DRIVER_REDIS;

#ifdef __cplusplus
}
#endif
#endif /* VEX_DB_DRIVER_H */
