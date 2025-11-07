
/* vex_sqlite.c — SQLite (sqlite3) full implementation
 * Requires: sqlite3
 * Build: define HAVE_SQLITE=1 and link with -lsqlite3 (or pkg-config).
 */
#ifndef HAVE_SQLITE
# error "SQLite driver requires HAVE_SQLITE=1 and sqlite3 present."
#endif

#include "vex_db_driver.h"
#include <sqlite3.h>
#include <string.h>
#include <stdio.h>
#include <stdlib.h>

typedef struct {
    sqlite3 *db;
} SqliteConn;

typedef struct {
    sqlite3_stmt *stmt;
    int done;
} SqliteResult;

static const char* sqlite_column_name_impl(void *r, uint32_t i) {
    return sqlite3_column_name((sqlite3_stmt*)r, (int)i);
}

static VexDbType sqlite_column_type_impl(void *r, uint32_t i) {
    int t = sqlite3_column_type((sqlite3_stmt*)r, (int)i);
    switch (t) {
        case SQLITE_INTEGER: return VEX_T_I64;
        case SQLITE_FLOAT: return VEX_T_F64;
        case SQLITE_BLOB: return VEX_T_BIN;
        case SQLITE_TEXT: return VEX_T_TEXT;
        default: return VEX_T_NULL;
    }
}

static int sqlite_column_is_binary_impl(void *r, uint32_t i) {
    (void)r; (void)i;
    return 0;
}

static VexConnection sq_connect(const char *conninfo){
    VexConnection c = {0};
    c.api_version=VEX_DB_API_VERSION;
    c.capabilities=VEX_CAP_SQL;
    
    sqlite3 *db=NULL;
    int rc = sqlite3_open_v2(conninfo?conninfo:":memory:", &db, SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE, NULL);
    if (rc != SQLITE_OK){
        c.error.code = VEX_DB_ERROR_CONNECT;
        snprintf(c.error.message, sizeof(c.error.message), "sqlite open: %s", sqlite3_errstr(rc));
        if (db) sqlite3_close(db);
        return c;
    }
    
    // Store the sqlite3* directly (not using SqliteConn wrapper for simplicity)
    c.native_conn = db;
    c.error.code = VEX_DB_OK;
    return c;
}

static void sq_disconnect(VexConnection *c){
    if (!c || !c->native_conn) return;
    sqlite3_close((sqlite3*)c->native_conn);
    c->native_conn = NULL;
}

static VexResultSet sq_execute_query(VexConnection *c, const char *q, int n, const VexDbValue *p){
    (void)n; (void)p;
    VexResultSet rs = {0};
    sqlite3 *db = (sqlite3*)c->native_conn;
    sqlite3_stmt *stmt = NULL;
    int rc = sqlite3_prepare_v2(db, q, -1, &stmt, NULL);
    if (rc != SQLITE_OK){
        rs.error.code = VEX_DB_ERROR_EXECUTION;
        snprintf(rs.error.message, sizeof(rs.error.message), "sqlite prepare: %s", sqlite3_errmsg(db));
        return rs;
    }
    /* Note: bind parameters can be added here based on VexDbValue if needed */
    
    /* For DDL/DML statements (CREATE, INSERT, UPDATE, DELETE), we need to step once
     * to execute them. For SELECT, step will be called in fetch_next. */
    int col_count = sqlite3_column_count(stmt);
    if (col_count == 0) {
        /* DDL/DML - execute now */
        rc = sqlite3_step(stmt);
        if (rc != SQLITE_DONE && rc != SQLITE_ROW) {
            rs.error.code = VEX_DB_ERROR_EXECUTION;
            snprintf(rs.error.message, sizeof(rs.error.message), "sqlite exec: %s", sqlite3_errmsg(db));
            sqlite3_finalize(stmt);
            return rs;
        }
        /* Statement executed, finalize it */
        sqlite3_finalize(stmt);
        rs.native_result = NULL;
        rs.error.code = VEX_DB_OK;
        rs.column_count = 0;
        rs._row_index = 0;
    } else {
        /* SELECT - keep statement for fetch_next */
        rs.native_result = stmt;
        rs.error.code = VEX_DB_OK;
        rs.column_count = (uint32_t)col_count;
        rs._row_index = 0;
        rs.column_name = sqlite_column_name_impl;
        rs.column_type = sqlite_column_type_impl;
        rs.column_is_binary = sqlite_column_is_binary_impl;
    }
    return rs;
}

static VexDbPayload sq_fetch_next(VexResultSet *res){
    VexDbPayload p = {0};
    if (!res || !res->native_result) return p;
    sqlite3_stmt *st = (sqlite3_stmt*)res->native_result;
    int rc = sqlite3_step(st);
    if (rc == SQLITE_ROW){
        /* Expose first column pointer (valid until next step/reset) */
        int t = sqlite3_column_type(st, 0);
        if (t == SQLITE_NULL){
            p.data = NULL; p.length=0; p.is_null=1; p.lifetime=VEX_LIFE_ROW_BUFFER; p.owner=res; p.type=VEX_T_NULL; return p;
        } else if (t == SQLITE_BLOB){
            const void *b = sqlite3_column_blob(st, 0);
            int len = sqlite3_column_bytes(st, 0);
            p.data = (const char*)b; p.length=(size_t)len; p.is_null=0; p.lifetime=VEX_LIFE_ROW_BUFFER; p.owner=res; p.type=VEX_T_BIN; return p;
        } else {
            const unsigned char *txt = sqlite3_column_text(st, 0);
            int len = sqlite3_column_bytes(st, 0);
            p.data = (const char*)txt; p.length=(size_t)len; p.is_null=0; p.lifetime=VEX_LIFE_ROW_BUFFER; p.owner=res; p.type=VEX_T_TEXT; return p;
        }
    } else if (rc == SQLITE_DONE){
        /* finalize to release memory */
        sqlite3_finalize(st);
        res->native_result = NULL;
        return p;
    } else {
        /* error */
        p.data = NULL;
        return p;
    }
}

static void sq_clear_result(VexResultSet *res){
    if (!res) return;
    if (res->native_result){
        sqlite3_finalize((sqlite3_stmt*)res->native_result);
        res->native_result = NULL;
    }
}

/* Transaction support */
static int sq_begin_transaction(VexConnection *c) {
    if (!c || !c->native_conn) return -1;
    char *err_msg = NULL;
    int rc = sqlite3_exec((sqlite3*)c->native_conn, "BEGIN", NULL, NULL, &err_msg);
    if (err_msg) sqlite3_free(err_msg);
    return (rc == SQLITE_OK) ? 0 : -1;
}

static int sq_commit_transaction(VexConnection *c) {
    if (!c || !c->native_conn) return -1;
    char *err_msg = NULL;
    int rc = sqlite3_exec((sqlite3*)c->native_conn, "COMMIT", NULL, NULL, &err_msg);
    if (err_msg) sqlite3_free(err_msg);
    return (rc == SQLITE_OK) ? 0 : -1;
}

static int sq_rollback_transaction(VexConnection *c) {
    if (!c || !c->native_conn) return -1;
    char *err_msg = NULL;
    int rc = sqlite3_exec((sqlite3*)c->native_conn, "ROLLBACK", NULL, NULL, &err_msg);
    if (err_msg) sqlite3_free(err_msg);
    return (rc == SQLITE_OK) ? 0 : -1;
}

VexDbDriver VEX_DRIVER_SQLITE = {
    .driver_name = "sqlite",
    .api_version = VEX_DB_API_VERSION,
    .capabilities = VEX_CAP_SQL | VEX_CAP_TXN,
    .connect = sq_connect,
    .disconnect = sq_disconnect,
    .clear_result = sq_clear_result,
    .execute_query = sq_execute_query,
    .fetch_next = sq_fetch_next,
    .get_event_fd = NULL, .wants_read = NULL, .wants_write = NULL,
    .start_execute = NULL, .poll_ready = NULL, .result_ready = NULL,
    .get_result = NULL, .cancel = NULL, .set_timeout_ms = NULL,
    .subscribe = NULL, .unsubscribe = NULL, .publish = NULL,
    .poll_notifications = NULL, .get_notification = NULL,
    .begin_transaction = sq_begin_transaction,
    .commit_transaction = sq_commit_transaction,
    .rollback_transaction = sq_rollback_transaction,
    .declare_cursor = NULL,
    .fetch_from_cursor = NULL,
    .close_cursor = NULL
};
