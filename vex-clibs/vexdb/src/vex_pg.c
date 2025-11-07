/* vex_pg.c — PostgreSQL driver (libpq-backed if HAVE_LIBPQ else mock)
 * This file is written to compile in two modes:
 *  1) MOCK (default): no libpq dependency; returns synthetic rows.
 *  2) REAL: build with -DHAVE_LIBPQ and link against libpq.
 *
 * Async usage is documented below in comments (libpq mapping).
 */
#include "vex_db_driver.h"
#include <string.h>
#include <stdio.h>
#include <stdlib.h>

/* ---------------------- MOCK BACKEND (default) ---------------------- */
#ifndef HAVE_LIBPQ

typedef struct {
    /* simple in-memory table: two rows, two columns */
    int idx;
    int total;
    const char *rows[2][2];
} MockPgResult;

static VexConnection pg_connect(const char *conninfo) {
    (void)conninfo;
    VexConnection c = {0};
    c.api_version = VEX_DB_API_VERSION;
    c.capabilities = VEX_CAP_SQL;
    c.native_conn = (void*)0x1; /* non-NULL to look connected */
    c.error.code = VEX_DB_OK;
    c.error.message[0] = 0;
    return c;
}

static void pg_disconnect(VexConnection *conn) {
    if (!conn) return;
    conn->native_conn = NULL;
}

static VexResultSet pg_execute_query(VexConnection *conn, const char *query,
                                     int nParams, const VexDbValue *params) {
    (void)conn; (void)query; (void)nParams; (void)params;
    VexResultSet rs = {0};
    MockPgResult *mr = (MockPgResult*)calloc(1, sizeof(MockPgResult));
    mr->idx = 0;
    mr->total = 2;
    mr->rows[0][0] = "1";     mr->rows[0][1] = "hello";
    mr->rows[1][0] = "2";     mr->rows[1][1] = "world";
    rs.native_result = mr;
    rs.row_affected = 2;
    rs.error.code = VEX_DB_OK;
    rs.column_count = 2;
    rs.column_name = NULL;
    rs.column_type = NULL;
    rs.column_is_binary = NULL;
    return rs;
}

static VexDbPayload pg_fetch_next(VexResultSet *res) {
    VexDbPayload p = {0};
    if (!res || !res->native_result) return p;
    MockPgResult *mr = (MockPgResult*)res->native_result;
    if (mr->idx >= mr->total) return p;
    /* For the demo, we encode the row as a simple CSV-like "id,name" string. */
    static char buf[64];
    snprintf(buf, sizeof(buf), "%s,%s", mr->rows[mr->idx][0], mr->rows[mr->idx][1]);
    mr->idx++;
    p.data = buf;
    p.length = strlen(buf);
    p.is_null = 0;
    p.lifetime = VEX_LIFE_RESULT_OWNED; /* tied to res (here static, but ok for mock) */
    p.owner = res;
    p.type = VEX_T_TEXT;
    return p;
}

static void pg_clear_result(VexResultSet *res) {
    if (!res) return;
    if (res->native_result) {
        free(res->native_result);
        res->native_result = NULL;
    }
}

/* find_doc is not supported for SQL; return NOT_FOUND or empty cursor */
static VexResultSet pg_find_doc(VexConnection *conn, const char *collection,
                                const VexDbPayload filter) {
    (void)conn; (void)collection; (void)filter;
    VexResultSet rs = {0};
    rs.error.code = VEX_DB_ERROR_UNKNOWN;
    snprintf(rs.error.message, sizeof(rs.error.message),
             "find_doc unsupported on postgres driver (mock).");
    return rs;
}

/* Async optional API: not available in mock mode */
static int pg_noasync_int(VexConnection *c){ (void)c; return -1; }
static void pg_noasync_void(VexConnection *c, uint32_t ms){ (void)c; (void)ms; }
static VexResultSet pg_noasync_getres(VexConnection *c){
    (void)c; VexResultSet r = {0}; r.error.code = VEX_DB_ERROR_UNKNOWN; return r;
}

VexDbDriver VEX_DRIVER_POSTGRES = {
    .driver_name = "postgres",
    .api_version = VEX_DB_API_VERSION,
    .capabilities = VEX_CAP_SQL,
    .connect = pg_connect,
    .disconnect = pg_disconnect,
    .clear_result = pg_clear_result,
    .execute_query = pg_execute_query,
    .find_doc = pg_find_doc,
    .fetch_next = pg_fetch_next,
    .get_event_fd = pg_noasync_int,
    .wants_read = pg_noasync_int,
    .wants_write = pg_noasync_int,
    .start_execute = (int(*)(VexConnection*, const char*, int, const VexDbValue*))pg_noasync_int,
    .poll_ready = pg_noasync_int,
    .result_ready = pg_noasync_int,
    .get_result = pg_noasync_getres,
    .cancel = pg_noasync_int,
    .set_timeout_ms = pg_noasync_void
};

#else /* ---------------------- REAL libpq BACKEND ---------------------- */

#include <libpq-fe.h>

static VexConnection pg_connect(const char *conninfo) {
    VexConnection c = {0};
    c.api_version = VEX_DB_API_VERSION;
    c.capabilities = VEX_CAP_SQL | VEX_CAP_ASYNC | VEX_CAP_BINARY_PARAMS;
    PGconn *pg = PQconnectdb(conninfo);
    c.native_conn = pg;
    if (PQstatus(pg) != CONNECTION_OK) {
        c.error.code = VEX_DB_ERROR_CONNECT;
        snprintf(c.error.message, sizeof(c.error.message), "%s", PQerrorMessage(pg));
    } else {
        c.error.code = VEX_DB_OK;
    }
    return c;
}

static void pg_disconnect(VexConnection *conn) {
    if (!conn || !conn->native_conn) return;
    PQfinish((PGconn*)conn->native_conn);
    conn->native_conn = NULL;
}

static VexResultSet pg_execute_query(VexConnection *conn, const char *query,
                                     int nParams, const VexDbValue *params) {
    VexResultSet rs = {0};
    PGconn *pg = (PGconn*)conn->native_conn;
    const char *paramValues[256] = {0};
    int paramLengths[256] = {0};
    int paramFormats[256] = {0};
    if (nParams > 256) nParams = 256;
    for (int i=0;i<nParams;i++){
        paramValues[i] = (const char*)params[i].data;
        paramLengths[i] = (int)params[i].length;
        paramFormats[i] = params[i].is_binary ? 1 : 0;
    }
    PGresult *pr = PQexecParams(pg, query, nParams, NULL, paramValues, paramLengths,
                                paramFormats, 1 /* result in binary if possible */);
    rs.native_result = pr;
    rs.error.code = VEX_DB_OK;
    if (!pr) {
        rs.error.code = VEX_DB_ERROR_EXECUTION;
        snprintf(rs.error.message, sizeof(rs.error.message), "%s", PQerrorMessage(pg));
        return rs;
    }
    ExecStatusType st = PQresultStatus(pr);
    if (!(st == PGRES_TUPLES_OK || st == PGRES_COMMAND_OK)) {
        rs.error.code = VEX_DB_ERROR_EXECUTION;
        snprintf(rs.error.message, sizeof(rs.error.message), "%s", PQresultErrorMessage(pr));
    }
    rs.row_affected = (long)atoi(PQcmdTuples(pr));
    rs.column_count = (uint32_t)PQnfields(pr);
    rs.column_name = [](void *r, uint32_t i)->const char*{ return PQfname((PGresult*)r, (int)i); };
    rs.column_type = [](void *r, uint32_t i)->VexDbType{
        /* Map a few common OIDs; for brevity we mark binary/text generically */
        (void)r; (void)i; return VEX_T_BIN;
    };
    rs.column_is_binary = [](void *r, uint32_t i)->int{
        return PQfformat((PGresult*)r, (int)i) == 1;
    };
    return rs;
}

static VexDbPayload pg_fetch_next(VexResultSet *res) {
    static int row = -1;
    VexDbPayload p = {0};
    if (!res || !res->native_result) return p;
    PGresult *pr = (PGresult*)res->native_result;
    int nrows = PQntuples(pr);
    int ncols = PQnfields(pr);
    if (row < 0) row = 0;
    if (row >= nrows) { row = -1; return p; }
    /* For demo: return the first column of current row as payload.
     * In your Vex binding, you will likely expose column-wise accessors.
     */
    const char *val = PQgetvalue(pr, row, 0);
    int isnull = PQgetisnull(pr, row, 0);
    int len = PQgetlength(pr, row, 0);
    p.data = val;
    p.length = (size_t)len;
    p.is_null = isnull;
    p.lifetime = VEX_LIFE_RESULT_OWNED;
    p.owner = res;
    p.type = PQfformat(pr,0)==1 ? VEX_T_BIN : VEX_T_TEXT;
    row++;
    return p;
}

static void pg_clear_result(VexResultSet *res) {
    if (!res) return;
    if (res->native_result) {
        PQclear((PGresult*)res->native_result);
        res->native_result = NULL;
    }
}

/* find_doc unsupported on postgres */
static VexResultSet pg_find_doc(VexConnection *conn, const char *collection,
                                const VexDbPayload filter) {
    (void)conn; (void)collection; (void)filter;
    VexResultSet rs = {0};
    rs.error.code = VEX_DB_ERROR_UNKNOWN;
    snprintf(rs.error.message, sizeof(rs.error.message),
             "find_doc unsupported on postgres driver.");
    return rs;
}

/* --------- ASYNC MAPPING (commented guide) ---------
 *  get_event_fd(conn)  -> PQsocket(PGconn*)
 *  wants_read/write    -> libpq does reads mostly; writes when flushing
 *  start_execute       -> PQsendQueryParams(...)
 *  poll_ready          -> call PQconsumeInput(), then PQisBusy()==0 => ready
 *  result_ready        -> returns 1 if PQgetResult() would return non-NULL (peek via PQisBusy)
 *  get_result          -> returns a VexResultSet wrapping PQgetResult()
 *  cancel              -> PQrequestCancel()
 *  set_timeout_ms      -> store timeout locally; enforce in your poll loop
 */
static int pg_get_event_fd(VexConnection *c) {
    PGconn *pg = (PGconn*)c->native_conn;
    return PQsocket(pg);
}
static int pg_wants_read(VexConnection *c){ (void)c; return 1; }
static int pg_wants_write(VexConnection *c){ (void)c; return 0; }

static int pg_start_execute(VexConnection *c, const char *q, int n, const VexDbValue *p) {
    PGconn *pg = (PGconn*)c->native_conn;
    const char *vals[256]={0}; int lens[256]={0}; int fmts[256]={0};
    if (n>256) n=256;
    for (int i=0;i<n;i++){ vals[i]=(const char*)p[i].data; lens[i]=(int)p[i].length; fmts[i]=p[i].is_binary?1:0; }
    if (!PQsendQueryParams(pg, q, n, NULL, vals, lens, fmts, 1)) return -1;
    return 0;
}

static int pg_poll_ready(VexConnection *c) {
    PGconn *pg = (PGconn*)c->native_conn;
    if (!PQconsumeInput(pg)) return -1;
    return PQisBusy(pg) ? 0 : 1;
}

static int pg_result_ready(VexConnection *c) {
    PGconn *pg = (PGconn*)c->native_conn;
    return PQisBusy(pg) ? 0 : 1;
}

static VexResultSet pg_get_result(VexConnection *c) {
    VexResultSet rs = {0};
    PGconn *pg = (PGconn*)c->native_conn;
    PGresult *pr = PQgetResult(pg);
    if (!pr) { rs.error.code = VEX_DB_ERROR_NOT_FOUND; return rs; }
    rs.native_result = pr;
    rs.error.code = VEX_DB_OK;
    rs.column_count = (uint32_t)PQnfields(pr);
    return rs;
}

static int pg_cancel(VexConnection *c) {
    PGconn *pg = (PGconn*)c->native_conn;
    return PQrequestCancel(pg) ? 0 : -1;
}

static void pg_set_timeout_ms(VexConnection *c, uint32_t ms) {
    (void)c; (void)ms; /* store in a side-structure if needed */
}

VexDbDriver VEX_DRIVER_POSTGRES = {
    .driver_name = "postgres",
    .api_version = VEX_DB_API_VERSION,
    .capabilities = VEX_CAP_SQL | VEX_CAP_ASYNC | VEX_CAP_BINARY_PARAMS,
    .connect = pg_connect,
    .disconnect = pg_disconnect,
    .clear_result = pg_clear_result,
    .execute_query = pg_execute_query,
    .find_doc = pg_find_doc,
    .fetch_next = pg_fetch_next,
    .get_event_fd = pg_get_event_fd,
    .wants_read = pg_wants_read,
    .wants_write = pg_wants_write,
    .start_execute = pg_start_execute,
    .poll_ready = pg_poll_ready,
    .result_ready = pg_result_ready,
    .get_result = pg_get_result,
    .cancel = pg_cancel,
    .set_timeout_ms = pg_set_timeout_ms
};

#endif /* HAVE_LIBPQ */
