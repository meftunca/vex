
/* vex_mongo.c — MongoDB (libmongoc) full implementation
 * Requires: libmongoc-1.0
 * Build: define HAVE_MONGO=1 and link with pkg-config --libs libmongoc-1.0
 */
#ifndef HAVE_MONGO
#error "MongoDB driver requires HAVE_MONGO=1 and libmongoc present."
#endif

#include "vex_db_driver.h"
#include "../../vex.h"
#include <mongoc/mongoc.h>
#include <string.h>
#include <stdio.h>
#include <stdlib.h>

typedef struct
{
    mongoc_client_t *client;
    mongoc_database_t *database;
    char db_name[128];
} MongoContext;

typedef struct
{
    mongoc_cursor_t *cursor;
    const bson_t *current_doc;
    char *json_str;
    size_t json_len;
} MongoResultContext;

static void mongo_set_error(VexDbError *err, VexDbStatus code, const char *msg)
{
    err->code = code;
    snprintf(err->message, sizeof(err->message), "%s", msg ? msg : "unknown error");
}

static VexConnection mongo_connect(const char *conninfo)
{
    VexConnection c = {0};
    c.api_version = VEX_DB_API_VERSION;
    c.capabilities = VEX_CAP_ASYNC; // MongoDB is document-oriented, not SQL

    // Initialize MongoDB driver (idempotent)
    static int mongo_initialized = 0;
    if (!mongo_initialized)
    {
        mongoc_init();
        mongo_initialized = 1;
    }

    // Parse connection string
    // Format: "mongodb://host:port/database" or just "database"
    MongoContext *ctx = (MongoContext *)vex_malloc(sizeof(MongoContext));
    if (!ctx)
    {
        mongo_set_error(&c.error, VEX_DB_ERROR_CONNECT, "out of memory");
        return c;
    }

    // Default to local MongoDB
    const char *uri = conninfo && strlen(conninfo) > 0 ? conninfo : "mongodb://localhost:27017";

    // Extract database name if in URI format
    const char *last_slash = strrchr(uri, '/');
    if (last_slash && *(last_slash + 1))
    {
        snprintf(ctx->db_name, sizeof(ctx->db_name), "%s", last_slash + 1);
    }
    else
    {
        snprintf(ctx->db_name, sizeof(ctx->db_name), "test");
    }

    bson_error_t error;
    ctx->client = mongoc_client_new(uri);

    if (!ctx->client)
    {
        mongo_set_error(&c.error, VEX_DB_ERROR_CONNECT, "failed to create MongoDB client");
        vex_free(ctx);
        return c;
    }

    // Test connection with ping
    bson_t *command = BCON_NEW("ping", BCON_INT32(1));
    bson_t reply;
    bool ok = mongoc_client_command_simple(ctx->client, "admin", command, NULL, &reply, &error);
    bson_destroy(command);
    bson_destroy(&reply);

    if (!ok)
    {
        mongo_set_error(&c.error, VEX_DB_ERROR_CONNECT, error.message);
        mongoc_client_destroy(ctx->client);
        vex_free(ctx);
        return c;
    }

    ctx->database = mongoc_client_get_database(ctx->client, ctx->db_name);
    c.native_conn = ctx;
    c.error.code = VEX_DB_OK;

    return c;
}

static void mongo_disconnect(VexConnection *conn)
{
    if (!conn || !conn->native_conn)
        return;

    MongoContext *ctx = (MongoContext *)conn->native_conn;
    if (ctx->database)
        mongoc_database_destroy(ctx->database);
    if (ctx->client)
        mongoc_client_destroy(ctx->client);
    vex_free(ctx);
    conn->native_conn = NULL;
}

static void mongo_clear_result(VexResultSet *res)
{
    if (!res || !res->native_result)
        return;

    MongoResultContext *mres = (MongoResultContext *)res->native_result;
    if (mres->cursor)
        mongoc_cursor_destroy(mres->cursor);
    if (mres->json_str)
        vex_free(mres->json_str);
    vex_free(mres);
    res->native_result = NULL;
}

// MongoDB query format: "collection.find({\"field\":\"value\"})"
// or "collection.aggregate([{\"$match\":{...}}])"
static VexResultSet mongo_execute_query(VexConnection *conn, const char *query,
                                        int nParams, const VexDbValue *params)
{
    VexResultSet rs = {0};

    if (!conn || !conn->native_conn || !query)
    {
        mongo_set_error(&rs.error, VEX_DB_ERROR_INVALID_PARAM, "invalid parameters");
        return rs;
    }

    MongoContext *ctx = (MongoContext *)conn->native_conn;

    // Parse query: "collection.operation(args)"
    char collection_name[128] = {0};
    char operation[32] = {0};
    const char *args_start = NULL;

    const char *dot = strchr(query, '.');
    if (!dot)
    {
        mongo_set_error(&rs.error, VEX_DB_ERROR_INVALID_PARAM,
                        "query format: collection.find({...}) or collection.aggregate([...])");
        return rs;
    }

    size_t coll_len = dot - query;
    if (coll_len >= sizeof(collection_name))
        coll_len = sizeof(collection_name) - 1;
    memcpy(collection_name, query, coll_len);
    collection_name[coll_len] = '\0';

    const char *paren = strchr(dot + 1, '(');
    if (!paren)
    {
        mongo_set_error(&rs.error, VEX_DB_ERROR_INVALID_PARAM, "missing '(' in query");
        return rs;
    }

    size_t op_len = paren - (dot + 1);
    if (op_len >= sizeof(operation))
        op_len = sizeof(operation) - 1;
    memcpy(operation, dot + 1, op_len);
    operation[op_len] = '\0';

    args_start = paren + 1;
    const char *args_end = strrchr(args_start, ')');
    if (!args_end)
    {
        mongo_set_error(&rs.error, VEX_DB_ERROR_INVALID_PARAM, "missing ')' in query");
        return rs;
    }

    char args[2048] = {0};
    size_t args_len = args_end - args_start;
    if (args_len >= sizeof(args))
        args_len = sizeof(args) - 1;
    memcpy(args, args_start, args_len);
    args[args_len] = '\0';

    // Get collection
    mongoc_collection_t *collection = mongoc_client_get_collection(ctx->client, ctx->db_name, collection_name);
    if (!collection)
    {
        mongo_set_error(&rs.error, VEX_DB_ERROR_EXECUTION, "failed to get collection");
        return rs;
    }

    MongoResultContext *mres = (MongoResultContext *)vex_malloc(sizeof(MongoResultContext));
    if (!mres)
    {
        mongoc_collection_destroy(collection);
        mongo_set_error(&rs.error, VEX_DB_ERROR_EXECUTION, "out of memory");
        return rs;
    }

    // Parse BSON query
    bson_error_t error;
    bson_t *query_doc = NULL;

    if (strlen(args) == 0 || strcmp(args, "{}") == 0)
    {
        query_doc = bson_new();
    }
    else
    {
        query_doc = bson_new_from_json((const uint8_t *)args, args_len, &error);
        if (!query_doc)
        {
            vex_free(mres);
            mongoc_collection_destroy(collection);
            mongo_set_error(&rs.error, VEX_DB_ERROR_INVALID_PARAM, error.message);
            return rs;
        }
    }

    // Execute operation
    if (strcmp(operation, "find") == 0)
    {
        mres->cursor = mongoc_collection_find_with_opts(collection, query_doc, NULL, NULL);
    }
    else if (strcmp(operation, "aggregate") == 0)
    {
        mres->cursor = mongoc_collection_aggregate(collection, MONGOC_QUERY_NONE, query_doc, NULL, NULL);
    }
    else
    {
        bson_destroy(query_doc);
        vex_free(mres);
        mongoc_collection_destroy(collection);
        mongo_set_error(&rs.error, VEX_DB_ERROR_UNSUPPORTED,
                        "unsupported operation (supported: find, aggregate)");
        return rs;
    }

    bson_destroy(query_doc);
    mongoc_collection_destroy(collection);

    if (!mres->cursor)
    {
        vex_free(mres);
        mongo_set_error(&rs.error, VEX_DB_ERROR_EXECUTION, "failed to create cursor");
        return rs;
    }

    rs.native_result = mres;
    rs.column_count = 1; // MongoDB returns JSON documents
    rs.error.code = VEX_DB_OK;
    rs._row_index = 0;

    return rs;
}

static VexDbPayload mongo_fetch_next(VexResultSet *res)
{
    VexDbPayload p = {0};

    if (!res || !res->native_result)
        return p;

    MongoResultContext *mres = (MongoResultContext *)res->native_result;

    if (!mongoc_cursor_next(mres->cursor, &mres->current_doc))
    {
        // End of results or error
        bson_error_t error;
        if (mongoc_cursor_error(mres->cursor, &error))
        {
            // Error occurred
            mongo_set_error(&res->error, VEX_DB_ERROR_EXECUTION, error.message);
        }
        return p; // End of results
    }

    // Free previous JSON string
    if (mres->json_str)
    {
        vex_free(mres->json_str);
        mres->json_str = NULL;
    }

    // Convert BSON to JSON
    mres->json_str = bson_as_canonical_extended_json(mres->current_doc, &mres->json_len);

    if (!mres->json_str)
    {
        mongo_set_error(&res->error, VEX_DB_ERROR_EXECUTION, "failed to convert BSON to JSON");
        return p;
    }

    p.data = mres->json_str;
    p.length = mres->json_len;
    p.is_null = 0;
    p.type = VEX_T_JSON;
    p.lifetime = VEX_LIFE_ROW_BUFFER;
    p.owner = mres->cursor;

    res->_row_index++;

    return p;
}

static const char *mongo_column_name(void *native_result, uint32_t idx)
{
    (void)native_result;
    return idx == 0 ? "document" : NULL;
}

static VexDbType mongo_column_type(void *native_result, uint32_t idx)
{
    (void)native_result;
    return idx == 0 ? VEX_T_JSON : VEX_T_NULL;
}

static int mongo_column_is_binary(void *native_result, uint32_t idx)
{
    (void)native_result;
    (void)idx;
    return 0;
}

/* Transaction support for MongoDB (sessions) - simplified */
static int mongo_begin_transaction(VexConnection *c)
{
    /* MongoDB transactions require sessions - simplified for now */
    (void)c;
    return -1; /* Not implemented yet - requires session handling */
}

static int mongo_commit_transaction(VexConnection *c)
{
    (void)c;
    return -1; /* Not implemented yet */
}

static int mongo_rollback_transaction(VexConnection *c)
{
    (void)c;
    return -1; /* Not implemented yet */
}

VexDbDriver VEX_DRIVER_MONGO = {
    .driver_name = "MongoDB",
    .api_version = VEX_DB_API_VERSION,
    .capabilities = VEX_CAP_ASYNC,

    .connect = mongo_connect,
    .disconnect = mongo_disconnect,
    .clear_result = mongo_clear_result,
    .execute_query = mongo_execute_query,
    .fetch_next = mongo_fetch_next,

    // Async operations not implemented for MongoDB
    .get_event_fd = NULL,
    .wants_read = NULL,
    .wants_write = NULL,
    .start_execute = NULL,
    .poll_ready = NULL,
    .result_ready = NULL,
    .get_result = NULL,
    .cancel = NULL,
    .set_timeout_ms = NULL,

    // Pub/Sub not supported
    .subscribe = NULL,
    .unsubscribe = NULL,
    .publish = NULL,
    .poll_notifications = NULL,
    .get_notification = NULL,

    // Transaction support (requires sessions - not yet implemented)
    .begin_transaction = mongo_begin_transaction,
    .commit_transaction = mongo_commit_transaction,
    .rollback_transaction = mongo_rollback_transaction,

    // Cursor support not needed (MongoDB has built-in cursor)
    .declare_cursor = NULL,
    .fetch_from_cursor = NULL,
    .close_cursor = NULL};
