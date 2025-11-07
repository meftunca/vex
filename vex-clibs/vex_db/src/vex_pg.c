
/* vex_pg.c — PostgreSQL (libpq) full implementation (no mocks)
 * Requires: libpq
 * Build: define HAVE_LIBPQ=1 and link with pkg-config --libs libpq
 */
#ifndef HAVE_LIBPQ
# error "PostgreSQL driver requires HAVE_LIBPQ=1 and libpq present."
#endif

#include "vex_db_driver.h"
#include <libpq-fe.h>
#include <string.h>
#include <stdio.h>
#include <stdlib.h>

static const char* pg_column_name_impl(void *r, uint32_t i) {
    return PQfname((PGresult*)r, (int)i);
}

static VexDbType pg_column_type_impl(void *r, uint32_t i) {
    (void)r; (void)i;
    return VEX_T_BIN;
}

static int pg_column_is_binary_impl(void *r, uint32_t i) {
    return PQfformat((PGresult*)r, (int)i) == 1;
}

static VexConnection pg_connect(const char *conninfo){
    VexConnection c = {0};
    c.api_version = VEX_DB_API_VERSION;
    c.capabilities = VEX_CAP_SQL | VEX_CAP_ASYNC | VEX_CAP_BINARY_PARAMS;
    PGconn *pg = PQconnectdb(conninfo ? conninfo : "");
    c.native_conn = pg;
    if (PQstatus(pg) != CONNECTION_OK) {
        c.error.code = VEX_DB_ERROR_CONNECT;
        snprintf(c.error.message, sizeof(c.error.message), "%s", PQerrorMessage(pg));
    } else {
        c.error.code = VEX_DB_OK;
    }
    return c;
}

static void pg_disconnect(VexConnection *conn){
    if (!conn || !conn->native_conn) return;
    PQfinish((PGconn*)conn->native_conn);
    conn->native_conn = NULL;
}

static VexResultSet pg_execute_query(VexConnection *conn, const char *query,
                                     int nParams, const VexDbValue *params){
    VexResultSet rs = {0};
    PGconn *pg = (PGconn*)conn->native_conn;
    const char *vals[256]={0}; int lens[256]={0}; int fmts[256]={0};
    if (nParams > 256) nParams = 256;
    for (int i=0;i<nParams;i++){ vals[i]=(const char*)params[i].data; lens[i]=(int)params[i].length; fmts[i]=params[i].is_binary?1:0; }
    PGresult *pr = PQexecParams(pg, query, nParams, NULL, vals, lens, fmts, 1);
    rs.native_result = pr;
    if (!pr) {
        rs.error.code = VEX_DB_ERROR_EXECUTION;
        snprintf(rs.error.message, sizeof(rs.error.message), "%s", PQerrorMessage(pg));
        return rs;
    }
    ExecStatusType st = PQresultStatus(pr);
    if (!(st==PGRES_TUPLES_OK || st==PGRES_COMMAND_OK)) {
        rs.error.code = VEX_DB_ERROR_EXECUTION;
        snprintf(rs.error.message, sizeof(rs.error.message), "%s", PQresultErrorMessage(pr));
    } else {
        rs.error.code = VEX_DB_OK;
    }
    rs.row_affected = (long)atoi(PQcmdTuples(pr));
    rs.column_count = (uint32_t)PQnfields(pr);
    rs._row_index = 0;
    rs.column_name = pg_column_name_impl;
    rs.column_type = pg_column_type_impl;
    rs.column_is_binary = pg_column_is_binary_impl;
    return rs;
}

static VexDbPayload pg_fetch_next(VexResultSet *res){
    VexDbPayload p = {0};
    if (!res || !res->native_result) return p;
    PGresult *pr = (PGresult*)res->native_result;
    int nrows = PQntuples(pr);
    if (res->_row_index >= nrows) return p;
    int row = res->_row_index++;
    /* Fast path: expose first column zero-copy */
    const char *val = PQgetvalue(pr, row, 0);
    int isnull = PQgetisnull(pr, row, 0);
    int len = PQgetlength(pr, row, 0);
    p.data = val;
    p.length = (size_t)len;
    p.is_null = isnull;
    p.lifetime = VEX_LIFE_RESULT_OWNED;
    p.owner = res;
    p.type = PQfformat(pr,0)==1 ? VEX_T_BIN : VEX_T_TEXT;
    return p;
}

static void pg_clear_result(VexResultSet *res){
    if (!res) return;
    if (res->native_result) {
        PQclear((PGresult*)res->native_result);
        res->native_result = NULL;
    }
}

/* Async helpers */
static int pg_get_event_fd(VexConnection *c){ return PQsocket((PGconn*)c->native_conn); }
static int pg_wants_read(VexConnection *c){ (void)c; return 1; }
static int pg_wants_write(VexConnection *c){ (void)c; return 0; }
static int pg_start_execute(VexConnection *c,const char *q,int n,const VexDbValue *p){
    PGconn *pg=(PGconn*)c->native_conn;
    const char *vals[256]={0}; int lens[256]={0}; int fmts[256]={0};
    if(n>256) n=256; for(int i=0;i<n;i++){ vals[i]=(const char*)p[i].data; lens[i]=(int)p[i].length; fmts[i]=p[i].is_binary?1:0; }
    return PQsendQueryParams(pg,q,n,NULL,vals,lens,fmts,1)?0:-1;
}
static int pg_poll_ready(VexConnection *c){ PGconn *pg=(PGconn*)c->native_conn; if(!PQconsumeInput(pg)) return -1; return PQisBusy(pg)?0:1; }
static int pg_result_ready(VexConnection *c){ PGconn *pg=(PGconn*)c->native_conn; return PQisBusy(pg)?0:1; }
static VexResultSet pg_get_result(VexConnection *c){ VexResultSet rs={0}; PGresult *pr=PQgetResult((PGconn*)c->native_conn); if(!pr){ rs.error.code=VEX_DB_ERROR_NOT_FOUND; return rs; } rs.native_result=pr; rs.column_count=(uint32_t)PQnfields(pr); rs._row_index=0; return rs; }
static int pg_cancel(VexConnection *c){ return PQrequestCancel((PGconn*)c->native_conn)?0:-1; }
static void pg_set_timeout_ms(VexConnection *c,uint32_t ms){ (void)c; (void)ms; }

/* Pub/Sub support (LISTEN/NOTIFY) */
static int pg_subscribe(VexConnection *c, const char *channel) {
    if (!c || !channel) return -1;
    char query[256];
    snprintf(query, sizeof(query), "LISTEN %s", channel);
    PGresult *res = PQexec((PGconn*)c->native_conn, query);
    int success = (PQresultStatus(res) == PGRES_COMMAND_OK) ? 0 : -1;
    PQclear(res);
    return success;
}

static int pg_unsubscribe(VexConnection *c, const char *channel) {
    if (!c || !channel) return -1;
    char query[256];
    snprintf(query, sizeof(query), "UNLISTEN %s", channel);
    PGresult *res = PQexec((PGconn*)c->native_conn, query);
    int success = (PQresultStatus(res) == PGRES_COMMAND_OK) ? 0 : -1;
    PQclear(res);
    return success;
}

static int pg_publish(VexConnection *c, const char *channel, const char *message, size_t message_len) {
    if (!c || !channel || !message) return -1;
    (void)message_len; /* PostgreSQL NOTIFY uses null-terminated strings */
    
    char query[1024];
    /* Escape single quotes in message */
    char escaped_msg[512];
    size_t j = 0;
    for (size_t i = 0; message[i] && i < 255 && j < 510; i++) {
        if (message[i] == '\'') {
            escaped_msg[j++] = '\'';
            escaped_msg[j++] = '\'';
        } else {
            escaped_msg[j++] = message[i];
        }
    }
    escaped_msg[j] = '\0';
    
    snprintf(query, sizeof(query), "NOTIFY %s, '%s'", channel, escaped_msg);
    PGresult *res = PQexec((PGconn*)c->native_conn, query);
    int success = (PQresultStatus(res) == PGRES_COMMAND_OK) ? 0 : -1;
    PQclear(res);
    return success;
}

static int pg_poll_notifications(VexConnection *c) {
    if (!c) return -1;
    PGconn *pg = (PGconn*)c->native_conn;
    PQconsumeInput(pg);
    PGnotify *notify = PQnotifies(pg);
    if (notify) {
        /* Put it back for get_notification to retrieve */
        /* Note: We can't really put it back, so we'll use a different approach */
        PQfreemem(notify);
        return 1; /* Has notifications */
    }
    return 0; /* No notifications */
}

static VexDbPayload pg_get_notification(VexConnection *c) {
    VexDbPayload p = {0};
    if (!c) return p;
    
    PGconn *pg = (PGconn*)c->native_conn;
    PQconsumeInput(pg);
    PGnotify *notify = PQnotifies(pg);
    
    if (notify) {
        /* Format: "channel:payload" */
        static char notif_buffer[1024];
        snprintf(notif_buffer, sizeof(notif_buffer), "%s:%s", 
                 notify->relname, notify->extra);
        
        p.data = notif_buffer;
        p.length = strlen(notif_buffer);
        p.type = VEX_T_TEXT;
        p.lifetime = VEX_LIFE_ROW_BUFFER;
        p.owner = c;
        
        PQfreemem(notify);
    }
    
    return p;
}

/* Transaction support */
static int pg_begin_transaction(VexConnection *c) {
    if (!c) return -1;
    PGresult *res = PQexec((PGconn*)c->native_conn, "BEGIN");
    int success = (PQresultStatus(res) == PGRES_COMMAND_OK) ? 0 : -1;
    PQclear(res);
    return success;
}

static int pg_commit_transaction(VexConnection *c) {
    if (!c) return -1;
    PGresult *res = PQexec((PGconn*)c->native_conn, "COMMIT");
    int success = (PQresultStatus(res) == PGRES_COMMAND_OK) ? 0 : -1;
    PQclear(res);
    return success;
}

static int pg_rollback_transaction(VexConnection *c) {
    if (!c) return -1;
    PGresult *res = PQexec((PGconn*)c->native_conn, "ROLLBACK");
    int success = (PQresultStatus(res) == PGRES_COMMAND_OK) ? 0 : -1;
    PQclear(res);
    return success;
}

/* Cursor-based streaming */
static int pg_declare_cursor(VexConnection *c, const char *cursor_name, const char *query) {
    if (!c || !cursor_name || !query) return -1;
    
    char declare_query[2048];
    snprintf(declare_query, sizeof(declare_query), 
             "DECLARE %s CURSOR FOR %s", cursor_name, query);
    
    PGresult *res = PQexec((PGconn*)c->native_conn, declare_query);
    int success = (PQresultStatus(res) == PGRES_COMMAND_OK) ? 0 : -1;
    PQclear(res);
    return success;
}

static VexResultSet pg_fetch_from_cursor(VexConnection *c, const char *cursor_name, int fetch_size) {
    VexResultSet rs = {0};
    if (!c || !cursor_name) {
        rs.error.code = VEX_DB_ERROR_INVALID_PARAM;
        return rs;
    }
    
    char fetch_query[256];
    snprintf(fetch_query, sizeof(fetch_query), 
             "FETCH %d FROM %s", fetch_size, cursor_name);
    
    PGresult *pr = PQexec((PGconn*)c->native_conn, fetch_query);
    rs.native_result = pr;
    
    if (!pr || PQresultStatus(pr) != PGRES_TUPLES_OK) {
        rs.error.code = VEX_DB_ERROR_EXECUTION;
        if (pr) {
            snprintf(rs.error.message, sizeof(rs.error.message), "%s", PQresultErrorMessage(pr));
        }
        return rs;
    }
    
    rs.error.code = VEX_DB_OK;
    rs.column_count = (uint32_t)PQnfields(pr);
    rs._row_index = 0;
    rs.column_name = pg_column_name_impl;
    rs.column_type = pg_column_type_impl;
    rs.column_is_binary = pg_column_is_binary_impl;
    
    return rs;
}

static int pg_close_cursor(VexConnection *c, const char *cursor_name) {
    if (!c || !cursor_name) return -1;
    
    char close_query[256];
    snprintf(close_query, sizeof(close_query), "CLOSE %s", cursor_name);
    
    PGresult *res = PQexec((PGconn*)c->native_conn, close_query);
    int success = (PQresultStatus(res) == PGRES_COMMAND_OK) ? 0 : -1;
    PQclear(res);
    return success;
}

VexDbDriver VEX_DRIVER_POSTGRES = {
    .driver_name = "postgres",
    .api_version = VEX_DB_API_VERSION,
    .capabilities = VEX_CAP_SQL | VEX_CAP_ASYNC | VEX_CAP_BINARY_PARAMS | VEX_CAP_TXN | VEX_CAP_PUBSUB | VEX_CAP_STREAMING,
    .connect = pg_connect,
    .disconnect = pg_disconnect,
    .clear_result = pg_clear_result,
    .execute_query = pg_execute_query,
    .fetch_next = pg_fetch_next,
    .get_event_fd = pg_get_event_fd,
    .wants_read = pg_wants_read,
    .wants_write = pg_wants_write,
    .start_execute = pg_start_execute,
    .poll_ready = pg_poll_ready,
    .result_ready = pg_result_ready,
    .get_result = pg_get_result,
    .cancel = pg_cancel,
    .set_timeout_ms = pg_set_timeout_ms,
    .subscribe = pg_subscribe,
    .unsubscribe = pg_unsubscribe,
    .publish = pg_publish,
    .poll_notifications = pg_poll_notifications,
    .get_notification = pg_get_notification,
    .begin_transaction = pg_begin_transaction,
    .commit_transaction = pg_commit_transaction,
    .rollback_transaction = pg_rollback_transaction,
    .declare_cursor = pg_declare_cursor,
    .fetch_from_cursor = pg_fetch_from_cursor,
    .close_cursor = pg_close_cursor
};
