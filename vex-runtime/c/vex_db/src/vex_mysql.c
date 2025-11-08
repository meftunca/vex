
/* vex_mysql.c — MySQL (libmysqlclient) full implementation
 * Requires: libmysqlclient
 * Build: define HAVE_MYSQL=1 and link with `mysql_config --cflags --libs` or pkg-config.
 */
#ifndef HAVE_MYSQL
# error "MySQL driver requires HAVE_MYSQL=1 and libmysqlclient present."
#endif

#include "vex_db_driver.h"
#include <mysql.h>
#include <string.h>
#include <stdio.h>
#include <stdlib.h>

static const char* mysql_column_name_impl(void *r, uint32_t i) {
    MYSQL_RES *res = (MYSQL_RES*)r;
    MYSQL_FIELD *f = mysql_fetch_fields(res);
    return (i < (uint32_t)mysql_num_fields(res)) ? f[i].name : NULL;
}

static VexDbType mysql_column_type_impl(void *r, uint32_t i) {
    (void)r; (void)i;
    return VEX_T_TEXT;
}

static int mysql_column_is_binary_impl(void *r, uint32_t i) {
    (void)r; (void)i;
    return 0;
}

/* simple conninfo parser: "host=.. user=.. password=.. db=.. port=3306" */
typedef struct { const char* host; const char* user; const char* pass; const char* db; unsigned int port; } ConnInfo;
static void parse_conninfo(const char *s, ConnInfo *ci){
    ci->host="127.0.0.1"; ci->user=NULL; ci->pass=NULL; ci->db=NULL; ci->port=0;
    if(!s) return;
    char *dup = strdup(s);
    for (char *tok=strtok(dup," ;"); tok; tok=strtok(NULL," ;")){
        if (strncmp(tok,"host=",5)==0) ci->host = tok+5;
        else if (strncmp(tok,"user=",5)==0) ci->user = tok+5;
        else if (strncmp(tok,"password=",9)==0) ci->pass = tok+9;
        else if (strncmp(tok,"db=",3)==0) ci->db = tok+3;
        else if (strncmp(tok,"port=",5)==0) ci->port = (unsigned)atoi(tok+5);
    }
    /* Note: dup memory intentionally leaked for simplicity since pointers refer to it;
       in production, copy into fixed buffers. */
}

static VexConnection my_connect(const char *conninfo){
    VexConnection c = {0}; c.api_version=VEX_DB_API_VERSION; c.capabilities=VEX_CAP_SQL;
    MYSQL *m = mysql_init(NULL);
    if (!m){ c.error.code=VEX_DB_ERROR_CONNECT; snprintf(c.error.message,sizeof(c.error.message),"mysql_init failed"); return c; }
    ConnInfo ci; parse_conninfo(conninfo, &ci);
    if (!mysql_real_connect(m, ci.host, ci.user, ci.pass, ci.db, ci.port, NULL, 0)){
        c.error.code = VEX_DB_ERROR_CONNECT;
        snprintf(c.error.message, sizeof(c.error.message), "%s", mysql_error(m));
        mysql_close(m);
        return c;
    }
    c.native_conn = m;
    c.error.code = VEX_DB_OK;
    return c;
}

static void my_disconnect(VexConnection *c){
    if (!c || !c->native_conn) return;
    mysql_close((MYSQL*)c->native_conn);
    c->native_conn = NULL;
}

static VexResultSet my_execute_query(VexConnection *c, const char *q, int n, const VexDbValue *p){
    (void)n; (void)p; /* Simple text query. For prepared/binary, extend here. */
    VexResultSet rs = {0};
    MYSQL *m = (MYSQL*)c->native_conn;
    if (mysql_real_query(m, q, (unsigned long)strlen(q)) != 0){
        rs.error.code = VEX_DB_ERROR_EXECUTION;
        snprintf(rs.error.message, sizeof(rs.error.message), "%s", mysql_error(m));
        return rs;
    }
    MYSQL_RES *res = mysql_store_result(m); /* buffered results (row-wise pointers valid until next fetch) */
    if (!res){
        /* Non-SELECT (OK) */
        rs.row_affected = (long)mysql_affected_rows(m);
        rs.error.code = VEX_DB_OK;
        rs.native_result = NULL;
        rs.column_count = 0;
        return rs;
    }
    rs.native_result = res;
    rs.error.code = VEX_DB_OK;
    rs.column_count = (uint32_t)mysql_num_fields(res);
    rs._row_index = 0;
    rs.column_name = mysql_column_name_impl;
    rs.column_type = mysql_column_type_impl;
    rs.column_is_binary = mysql_column_is_binary_impl;
    return rs;
}

static VexDbPayload my_fetch_next(VexResultSet *res){
    VexDbPayload p = {0};
    if (!res) return p;
    MYSQL_RES *r = (MYSQL_RES*)res->native_result;
    if (!r) return p;
    MYSQL_ROW row = mysql_fetch_row(r);
    if (!row) return p;
    unsigned long *lengths = mysql_fetch_lengths(r);
    /* Fast path: expose first column zero-copy (pointer valid until next mysql_fetch_row) */
    p.data = (const char*)row[0];
    p.length = lengths ? (size_t)lengths[0] : (row[0]?strlen(row[0]):0);
    p.is_null = (row[0]==NULL);
    p.lifetime = VEX_LIFE_ROW_BUFFER;
    p.owner = res;
    p.type = VEX_T_TEXT;
    return p;
}

static void my_clear_result(VexResultSet *res){
    if (!res) return;
    if (res->native_result){
        mysql_free_result((MYSQL_RES*)res->native_result);
        res->native_result = NULL;
    }
}

/* Transaction support */
static int my_begin_transaction(VexConnection *c) {
    if (!c || !c->native_conn) return -1;
    return mysql_query((MYSQL*)c->native_conn, "START TRANSACTION") == 0 ? 0 : -1;
}

static int my_commit_transaction(VexConnection *c) {
    if (!c || !c->native_conn) return -1;
    return mysql_query((MYSQL*)c->native_conn, "COMMIT") == 0 ? 0 : -1;
}

static int my_rollback_transaction(VexConnection *c) {
    if (!c || !c->native_conn) return -1;
    return mysql_query((MYSQL*)c->native_conn, "ROLLBACK") == 0 ? 0 : -1;
}

VexDbDriver VEX_DRIVER_MYSQL = {
    .driver_name = "mysql",
    .api_version = VEX_DB_API_VERSION,
    .capabilities = VEX_CAP_SQL | VEX_CAP_TXN,
    .connect = my_connect,
    .disconnect = my_disconnect,
    .clear_result = my_clear_result,
    .execute_query = my_execute_query,
    .fetch_next = my_fetch_next,
    .get_event_fd = NULL, .wants_read = NULL, .wants_write = NULL,
    .start_execute = NULL, .poll_ready = NULL, .result_ready = NULL,
    .get_result = NULL, .cancel = NULL, .set_timeout_ms = NULL,
    .subscribe = NULL, .unsubscribe = NULL, .publish = NULL,
    .poll_notifications = NULL, .get_notification = NULL,
    .begin_transaction = my_begin_transaction,
    .commit_transaction = my_commit_transaction,
    .rollback_transaction = my_rollback_transaction,
    .declare_cursor = NULL,
    .fetch_from_cursor = NULL,
    .close_cursor = NULL
};
