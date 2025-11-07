/* vex_mongo.c — MongoDB driver (mock + async commentary)
 * For demonstration, this is a MOCK that outlines how a real libmongoc
 * integration should look. It compiles without any external deps.
 */
#include "vex_db_driver.h"
#include <string.h>
#include <stdlib.h>
#include <stdio.h>

typedef struct {
    int idx, total;
    const char *docs[2];
} MockCursor;

static VexConnection mongo_connect(const char *conninfo) {
    (void)conninfo;
    VexConnection c = {0};
    c.api_version = VEX_DB_API_VERSION;
    c.capabilities = VEX_CAP_DOC_FIND;
    c.native_conn = (void*)0x2;
    c.error.code = VEX_DB_OK;
    return c;
}

static void mongo_disconnect(VexConnection *conn) {
    if (!conn) return;
    conn->native_conn = NULL;
}

static VexResultSet mongo_execute_query(VexConnection *conn, const char *query,
                                        int nParams, const VexDbValue *params) {
    (void)conn; (void)query; (void)nParams; (void)params;
    VexResultSet rs = {0};
    rs.error.code = VEX_DB_ERROR_UNKNOWN;
    snprintf(rs.error.message, sizeof(rs.error.message),
             "execute_query unsupported on mongodb driver (mock). Use find_doc.");
    return rs;
}

static VexResultSet mongo_find_doc(VexConnection *conn, const char *collection,
                                   const VexDbPayload filter) {
    (void)conn; (void)collection; (void)filter;
    VexResultSet rs = {0};
    MockCursor *mc = (MockCursor*)calloc(1, sizeof(MockCursor));
    mc->total = 2;
    mc->docs[0] = "{\"_id\":1,\"name\":\"alice\"}";
    mc->docs[1] = "{\"_id\":2,\"name\":\"bob\"}";
    rs.native_result = mc;
    rs.column_count = 1;
    rs.error.code = VEX_DB_OK;
    return rs;
}

static VexDbPayload mongo_fetch_next(VexResultSet *res) {
    VexDbPayload p = {0};
    if (!res || !res->native_result) return p;
    MockCursor *mc = (MockCursor*)res->native_result;
    if (mc->idx >= mc->total) return p;
    const char *doc = mc->docs[mc->idx++];
    p.data = doc;
    p.length = strlen(doc);
    p.is_null = 0;
    p.lifetime = VEX_LIFE_RESULT_OWNED;
    p.owner = res;
    p.type = VEX_T_JSON;
    return p;
}

static void mongo_clear_result(VexResultSet *res) {
    if (!res) return;
    if (res->native_result) {
        free(res->native_result);
        res->native_result = NULL;
    }
}

/* --- ASYNC Commentary (libmongoc) ---
 * A real libmongoc integration would:
 *  - create client via mongoc_client_new()
 *  - get collection via mongoc_client_get_collection()
 *  - create cursor with mongoc_collection_find_with_opts()
 *  - Not all libmongoc flows are fd-pollable in the same way as libpq.
 *    If you can obtain pollfds via mongoc_client_get_pollfds() you can expose
 *    get_event_fd/wants_read/wants_write; otherwise leave them NULL.
 *  - fetch via mongoc_cursor_next(); BSON pointers are valid until next call.
 *    That matches VEX_LIFE_RESULT_OWNED lifetime.
 */

static int noasync(VexConnection *c){(void)c; return -1;}
static void noasync_void(VexConnection *c, uint32_t ms){(void)c;(void)ms;}
static VexResultSet noasync_res(VexConnection *c){(void)c; VexResultSet r={0}; r.error.code=VEX_DB_ERROR_UNKNOWN; return r;}

VexDbDriver VEX_DRIVER_MONGODB = {
    .driver_name = "mongodb",
    .api_version = VEX_DB_API_VERSION,
    .capabilities = VEX_CAP_DOC_FIND,
    .connect = mongo_connect,
    .disconnect = mongo_disconnect,
    .clear_result = mongo_clear_result,
    .execute_query = mongo_execute_query,
    .find_doc = mongo_find_doc,
    .fetch_next = mongo_fetch_next,
    .get_event_fd = noasync,
    .wants_read = noasync,
    .wants_write = noasync,
    .start_execute = (int(*)(VexConnection*, const char*, int, const VexDbValue*))noasync,
    .poll_ready = noasync,
    .result_ready = noasync,
    .get_result = noasync_res,
    .cancel = noasync,
    .set_timeout_ms = noasync_void
};
