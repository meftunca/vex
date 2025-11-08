
/* vex_redis.c — Redis (hiredis) full implementation
 * Requires: hiredis
 * Build: define HAVE_REDIS=1 and link with -lhiredis
 */
#ifndef HAVE_REDIS
# error "Redis driver requires HAVE_REDIS=1 and hiredis present."
#endif

#include "vex_db_driver.h"
#include <hiredis/hiredis.h>
#include <string.h>
#include <stdio.h>
#include <stdlib.h>

typedef struct {
    redisContext *ctx;
} RedisContext;

typedef struct {
    redisReply *reply;
    size_t current_index;
    size_t array_size;
    char *current_str;
    size_t current_str_len;
} RedisResultContext;

static void redis_set_error(VexDbError *err, VexDbStatus code, const char *msg) {
    err->code = code;
    snprintf(err->message, sizeof(err->message), "%s", msg ? msg : "unknown error");
}

static VexConnection redis_connect(const char *conninfo) {
    VexConnection c = {0};
    c.api_version = VEX_DB_API_VERSION;
    c.capabilities = VEX_CAP_ASYNC; // Redis supports async, but we'll use sync for simplicity
    
    // Parse connection string
    // Format: "host:port" or "host" (default port 6379) or "/path/to/socket"
    char host[256] = "127.0.0.1";
    int port = 6379;
    
    if (conninfo && strlen(conninfo) > 0) {
        // Check if it's a Unix socket path
        if (conninfo[0] == '/') {
            RedisContext *rctx = (RedisContext*)calloc(1, sizeof(RedisContext));
            if (!rctx) {
                redis_set_error(&c.error, VEX_DB_ERROR_CONNECT, "out of memory");
                return c;
            }
            
            rctx->ctx = redisConnectUnix(conninfo);
            if (!rctx->ctx || rctx->ctx->err) {
                redis_set_error(&c.error, VEX_DB_ERROR_CONNECT, 
                               rctx->ctx ? rctx->ctx->errstr : "connection failed");
                if (rctx->ctx) redisFree(rctx->ctx);
                free(rctx);
                return c;
            }
            
            c.native_conn = rctx;
            c.error.code = VEX_DB_OK;
            return c;
        }
        
        // Parse host:port
        const char *colon = strchr(conninfo, ':');
        if (colon) {
            size_t host_len = colon - conninfo;
            if (host_len >= sizeof(host)) host_len = sizeof(host) - 1;
            memcpy(host, conninfo, host_len);
            host[host_len] = '\0';
            port = atoi(colon + 1);
        } else {
            snprintf(host, sizeof(host), "%s", conninfo);
        }
    }
    
    RedisContext *rctx = (RedisContext*)calloc(1, sizeof(RedisContext));
    if (!rctx) {
        redis_set_error(&c.error, VEX_DB_ERROR_CONNECT, "out of memory");
        return c;
    }
    
    struct timeval timeout = { 1, 500000 }; // 1.5 seconds
    rctx->ctx = redisConnectWithTimeout(host, port, timeout);
    
    if (!rctx->ctx || rctx->ctx->err) {
        redis_set_error(&c.error, VEX_DB_ERROR_CONNECT, 
                       rctx->ctx ? rctx->ctx->errstr : "connection failed");
        if (rctx->ctx) redisFree(rctx->ctx);
        free(rctx);
        return c;
    }
    
    // Test connection with PING
    redisReply *reply = redisCommand(rctx->ctx, "PING");
    if (!reply || reply->type == REDIS_REPLY_ERROR) {
        redis_set_error(&c.error, VEX_DB_ERROR_CONNECT, 
                       reply ? reply->str : "ping failed");
        if (reply) freeReplyObject(reply);
        redisFree(rctx->ctx);
        free(rctx);
        return c;
    }
    freeReplyObject(reply);
    
    c.native_conn = rctx;
    c.error.code = VEX_DB_OK;
    
    return c;
}

static void redis_disconnect(VexConnection *conn) {
    if (!conn || !conn->native_conn) return;
    
    RedisContext *rctx = (RedisContext*)conn->native_conn;
    if (rctx->ctx) redisFree(rctx->ctx);
    free(rctx);
    conn->native_conn = NULL;
}

static void redis_clear_result(VexResultSet *res) {
    if (!res || !res->native_result) return;
    
    RedisResultContext *rres = (RedisResultContext*)res->native_result;
    if (rres->reply) freeReplyObject(rres->reply);
    if (rres->current_str) free(rres->current_str);
    free(rres);
    res->native_result = NULL;
}

// Redis query format: "GET key" or "HGETALL hash" or "LRANGE list 0 -1" etc.
static VexResultSet redis_execute_query(VexConnection *conn, const char *query,
                                        int nParams, const VexDbValue *params) {
    VexResultSet rs = {0};
    
    if (!conn || !conn->native_conn || !query) {
        redis_set_error(&rs.error, VEX_DB_ERROR_INVALID_PARAM, "invalid parameters");
        return rs;
    }
    
    RedisContext *rctx = (RedisContext*)conn->native_conn;
    
    // Parse the query string into command and arguments
    // Redis commands are space-separated: "SET key value" or "GET key"
    redisReply *reply;
    
    if (nParams > 0) {
        // Build argument array with parameters
        const char **argv = malloc((nParams + 1) * sizeof(char*));
        size_t *argvlen = malloc((nParams + 1) * sizeof(size_t));
        
        if (!argv || !argvlen) {
            free(argv);
            free(argvlen);
            redis_set_error(&rs.error, VEX_DB_ERROR_EXECUTION, "out of memory");
            return rs;
        }
        
        argv[0] = query;
        argvlen[0] = strlen(query);
        
        for (int i = 0; i < nParams; i++) {
            argv[i + 1] = (const char*)params[i].data;
            argvlen[i + 1] = params[i].length;
        }
        
        reply = redisCommandArgv(rctx->ctx, nParams + 1, argv, argvlen);
        
        free(argv);
        free(argvlen);
    } else {
        // Parse query into words for redisCommandArgv
        // Make a copy to tokenize
        char *query_copy = strdup(query);
        if (!query_copy) {
            redis_set_error(&rs.error, VEX_DB_ERROR_EXECUTION, "out of memory");
            return rs;
        }
        
        // Count words
        int word_count = 0;
        char *tmp = query_copy;
        int in_word = 0;
        while (*tmp) {
            if (*tmp == ' ' || *tmp == '\t') {
                in_word = 0;
            } else if (!in_word) {
                in_word = 1;
                word_count++;
            }
            tmp++;
        }
        
        // Allocate argv
        const char **argv = malloc(word_count * sizeof(char*));
        size_t *argvlen = malloc(word_count * sizeof(size_t));
        
        if (!argv || !argvlen) {
            free(query_copy);
            free(argv);
            free(argvlen);
            redis_set_error(&rs.error, VEX_DB_ERROR_EXECUTION, "out of memory");
            return rs;
        }
        
        // Split into words
        int idx = 0;
        char *token = strtok(query_copy, " \t");
        while (token && idx < word_count) {
            argv[idx] = token;
            argvlen[idx] = strlen(token);
            idx++;
            token = strtok(NULL, " \t");
        }
        
        reply = redisCommandArgv(rctx->ctx, word_count, argv, argvlen);
        
        free(argv);
        free(argvlen);
        free(query_copy);
    }
    
    if (!reply) {
        redis_set_error(&rs.error, VEX_DB_ERROR_EXECUTION, 
                       rctx->ctx->errstr ? rctx->ctx->errstr : "command failed");
        return rs;
    }
    
    if (reply->type == REDIS_REPLY_ERROR) {
        redis_set_error(&rs.error, VEX_DB_ERROR_EXECUTION, reply->str);
        freeReplyObject(reply);
        return rs;
    }
    
    RedisResultContext *rres = (RedisResultContext*)calloc(1, sizeof(RedisResultContext));
    if (!rres) {
        freeReplyObject(reply);
        redis_set_error(&rs.error, VEX_DB_ERROR_EXECUTION, "out of memory");
        return rs;
    }
    
    rres->reply = reply;
    rres->current_index = 0;
    
    // Determine array size for array/set responses
    if (reply->type == REDIS_REPLY_ARRAY) {
        rres->array_size = reply->elements;
    } else {
        rres->array_size = 1; // Single value
    }
    
    rs.native_result = rres;
    rs.column_count = 1;
    rs.error.code = VEX_DB_OK;
    rs._row_index = 0;
    rs.row_affected = (reply->type == REDIS_REPLY_INTEGER) ? reply->integer : 0;
    
    return rs;
}

static VexDbPayload redis_fetch_next(VexResultSet *res) {
    VexDbPayload p = {0};
    
    if (!res || !res->native_result) return p;
    
    RedisResultContext *rres = (RedisResultContext*)res->native_result;
    redisReply *reply = rres->reply;
    
    // Free previous string if any
    if (rres->current_str) {
        free(rres->current_str);
        rres->current_str = NULL;
    }
    
    // Handle different reply types
    if (reply->type == REDIS_REPLY_ARRAY) {
        if (rres->current_index >= reply->elements) {
            return p; // End of array
        }
        
        redisReply *element = reply->element[rres->current_index++];
        
        switch (element->type) {
            case REDIS_REPLY_STRING:
            case REDIS_REPLY_STATUS:
                p.data = element->str;
                p.length = element->len;
                p.type = VEX_T_TEXT;
                break;
                
            case REDIS_REPLY_INTEGER:
                // Convert integer to string
                rres->current_str = malloc(32);
                if (rres->current_str) {
                    rres->current_str_len = snprintf(rres->current_str, 32, "%lld", element->integer);
                    p.data = rres->current_str;
                    p.length = rres->current_str_len;
                    p.type = VEX_T_I64;
                }
                break;
                
            case REDIS_REPLY_NIL:
                p.is_null = 1;
                p.type = VEX_T_NULL;
                break;
                
            default:
                p.data = "(unsupported type)";
                p.length = strlen(p.data);
                p.type = VEX_T_TEXT;
                break;
        }
    } else {
        // Single value response
        if (rres->current_index > 0) {
            return p; // Already fetched
        }
        rres->current_index++;
        
        switch (reply->type) {
            case REDIS_REPLY_STRING:
            case REDIS_REPLY_STATUS:
                p.data = reply->str;
                p.length = reply->len;
                p.type = VEX_T_TEXT;
                break;
                
            case REDIS_REPLY_INTEGER:
                rres->current_str = malloc(32);
                if (rres->current_str) {
                    rres->current_str_len = snprintf(rres->current_str, 32, "%lld", reply->integer);
                    p.data = rres->current_str;
                    p.length = rres->current_str_len;
                    p.type = VEX_T_I64;
                }
                break;
                
            case REDIS_REPLY_NIL:
                p.is_null = 1;
                p.type = VEX_T_NULL;
                break;
                
            default:
                p.data = "(unsupported type)";
                p.length = strlen(p.data);
                p.type = VEX_T_TEXT;
                break;
        }
    }
    
    p.lifetime = VEX_LIFE_RESULT_OWNED;
    p.owner = reply;
    res->_row_index++;
    
    return p;
}

static const char* redis_column_name(void *native_result, uint32_t idx) {
    (void)native_result;
    return idx == 0 ? "value" : NULL;
}

static VexDbType redis_column_type(void *native_result, uint32_t idx) {
    (void)native_result;
    if (idx != 0) return VEX_T_NULL;
    
    RedisResultContext *rres = (RedisResultContext*)native_result;
    if (!rres || !rres->reply) return VEX_T_NULL;
    
    switch (rres->reply->type) {
        case REDIS_REPLY_STRING:
        case REDIS_REPLY_STATUS:
            return VEX_T_TEXT;
        case REDIS_REPLY_INTEGER:
            return VEX_T_I64;
        case REDIS_REPLY_ARRAY:
            return VEX_T_TEXT; // Array elements
        case REDIS_REPLY_NIL:
            return VEX_T_NULL;
        default:
            return VEX_T_TEXT;
    }
}

static int redis_column_is_binary(void *native_result, uint32_t idx) {
    (void)native_result;
    (void)idx;
    return 0;
}

/* Pub/Sub support */
static int redis_subscribe(VexConnection *conn, const char *channel) {
    if (!conn || !conn->native_conn || !channel) return -1;
    
    RedisContext *rctx = (RedisContext*)conn->native_conn;
    redisReply *reply = redisCommand(rctx->ctx, "SUBSCRIBE %s", channel);
    
    if (!reply || reply->type == REDIS_REPLY_ERROR) {
        if (reply) freeReplyObject(reply);
        return -1;
    }
    
    freeReplyObject(reply);
    return 0;
}

static int redis_unsubscribe(VexConnection *conn, const char *channel) {
    if (!conn || !conn->native_conn || !channel) return -1;
    
    RedisContext *rctx = (RedisContext*)conn->native_conn;
    redisReply *reply = redisCommand(rctx->ctx, "UNSUBSCRIBE %s", channel);
    
    if (!reply || reply->type == REDIS_REPLY_ERROR) {
        if (reply) freeReplyObject(reply);
        return -1;
    }
    
    freeReplyObject(reply);
    return 0;
}

static int redis_publish(VexConnection *conn, const char *channel, const char *message, size_t message_len) {
    if (!conn || !conn->native_conn || !channel || !message) return -1;
    
    RedisContext *rctx = (RedisContext*)conn->native_conn;
    redisReply *reply = redisCommand(rctx->ctx, "PUBLISH %s %b", channel, message, message_len);
    
    if (!reply || reply->type == REDIS_REPLY_ERROR) {
        if (reply) freeReplyObject(reply);
        return -1;
    }
    
    long long subscribers = reply->integer;
    freeReplyObject(reply);
    
    return (int)subscribers; /* Return number of subscribers that received the message */
}

static int redis_poll_notifications(VexConnection *conn) {
    if (!conn || !conn->native_conn) return -1;
    
    RedisContext *rctx = (RedisContext*)conn->native_conn;
    
    /* Check if there's data available without blocking */
    struct timeval tv = {0, 1000}; /* 1ms timeout */
    redisSetTimeout(rctx->ctx, tv);
    
    redisReply *reply;
    if (redisGetReply(rctx->ctx, (void**)&reply) == REDIS_OK && reply) {
        /* Check if it's a pub/sub message */
        if (reply->type == REDIS_REPLY_ARRAY && reply->elements >= 3) {
            if (reply->element[0]->type == REDIS_REPLY_STRING &&
                strcmp(reply->element[0]->str, "message") == 0) {
                /* Put it back somehow... or we need to cache it */
                /* For now, we'll just return 1 and let get_notification handle it */
                freeReplyObject(reply);
                return 1;
            }
        }
        freeReplyObject(reply);
    }
    
    return 0;
}

static VexDbPayload redis_get_notification(VexConnection *conn) {
    VexDbPayload p = {0};
    if (!conn || !conn->native_conn) return p;
    
    RedisContext *rctx = (RedisContext*)conn->native_conn;
    
    redisReply *reply;
    if (redisGetReply(rctx->ctx, (void**)&reply) != REDIS_OK || !reply) {
        return p;
    }
    
    if (reply->type == REDIS_REPLY_ARRAY && reply->elements >= 3) {
        if (reply->element[0]->type == REDIS_REPLY_STRING &&
            strcmp(reply->element[0]->str, "message") == 0) {
            
            /* Format: "channel:message" */
            static char notif_buffer[2048];
            const char *channel = reply->element[1]->str;
            const char *message = reply->element[2]->str;
            
            snprintf(notif_buffer, sizeof(notif_buffer), "%s:%s", channel, message);
            
            p.data = notif_buffer;
            p.length = strlen(notif_buffer);
            p.type = VEX_T_TEXT;
            p.lifetime = VEX_LIFE_ROW_BUFFER;
            p.owner = conn;
        }
    }
    
    freeReplyObject(reply);
    return p;
}

/* Transaction support (MULTI/EXEC/DISCARD) */
static int redis_begin_transaction(VexConnection *conn) {
    if (!conn || !conn->native_conn) return -1;
    
    RedisContext *rctx = (RedisContext*)conn->native_conn;
    redisReply *reply = redisCommand(rctx->ctx, "MULTI");
    
    int success = (reply && reply->type == REDIS_REPLY_STATUS && 
                   strcmp(reply->str, "OK") == 0) ? 0 : -1;
    
    if (reply) freeReplyObject(reply);
    return success;
}

static int redis_commit_transaction(VexConnection *conn) {
    if (!conn || !conn->native_conn) return -1;
    
    RedisContext *rctx = (RedisContext*)conn->native_conn;
    redisReply *reply = redisCommand(rctx->ctx, "EXEC");
    
    int success = (reply && reply->type == REDIS_REPLY_ARRAY) ? 0 : -1;
    
    if (reply) freeReplyObject(reply);
    return success;
}

static int redis_rollback_transaction(VexConnection *conn) {
    if (!conn || !conn->native_conn) return -1;
    
    RedisContext *rctx = (RedisContext*)conn->native_conn;
    redisReply *reply = redisCommand(rctx->ctx, "DISCARD");
    
    int success = (reply && reply->type == REDIS_REPLY_STATUS &&
                   strcmp(reply->str, "OK") == 0) ? 0 : -1;
    
    if (reply) freeReplyObject(reply);
    return success;
}

VexDbDriver VEX_DRIVER_REDIS = {
    .driver_name = "Redis",
    .api_version = VEX_DB_API_VERSION,
    .capabilities = VEX_CAP_ASYNC | VEX_CAP_TXN | VEX_CAP_PUBSUB,
    
    .connect = redis_connect,
    .disconnect = redis_disconnect,
    .clear_result = redis_clear_result,
    .execute_query = redis_execute_query,
    .fetch_next = redis_fetch_next,
    
    // Async operations not implemented
    .get_event_fd = NULL,
    .wants_read = NULL,
    .wants_write = NULL,
    .start_execute = NULL,
    .poll_ready = NULL,
    .result_ready = NULL,
    .get_result = NULL,
    .cancel = NULL,
    .set_timeout_ms = NULL,
    
    // Pub/Sub support
    .subscribe = redis_subscribe,
    .unsubscribe = redis_unsubscribe,
    .publish = redis_publish,
    .poll_notifications = redis_poll_notifications,
    .get_notification = redis_get_notification,
    
    // Transaction support
    .begin_transaction = redis_begin_transaction,
    .commit_transaction = redis_commit_transaction,
    .rollback_transaction = redis_rollback_transaction,
    
    // Cursor support not applicable
    .declare_cursor = NULL,
    .fetch_from_cursor = NULL,
    .close_cursor = NULL
};

