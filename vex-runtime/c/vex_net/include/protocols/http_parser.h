#ifndef VEX_HTTP_PARSER_H
#define VEX_HTTP_PARSER_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Maximum headers to parse */
#ifndef VEX_HTTP_MAX_HEADERS
#define VEX_HTTP_MAX_HEADERS 32
#endif

/* HTTP status codes */
#define VEX_HTTP_OK               0
#define VEX_HTTP_ERR_TRUNCATED   -1  /* Need more data (streaming) */
#define VEX_HTTP_ERR_BAD_REQUEST -2  /* Malformed request */
#define VEX_HTTP_ERR_TOO_MANY_HDR -3 /* Too many headers */

/* HTTP header (zero-copy, points into buffer) */
typedef struct {
    const char *name;
    uint16_t    name_len;
    const char *value;
    uint16_t    value_len;
} vex_http_header_t;

/* HTTP request line */
typedef struct {
    const char *method;
    size_t      method_len;
    const char *uri;
    size_t      uri_len;
    int         http_major;
    int         http_minor;
} vex_http_request_line_t;

/* Complete HTTP request */
typedef struct {
    vex_http_request_line_t request_line;
    vex_http_header_t       headers[VEX_HTTP_MAX_HEADERS];
    size_t                  header_count;
    const char             *body;
    size_t                  body_len;
} vex_http_request_t;

/* Streaming parser internal state (opaque) */
typedef enum {
    VEX_HTTP_PARSER_REQ_LINE,
    VEX_HTTP_PARSER_HEADERS,
    VEX_HTTP_PARSER_BODY_CL,
    VEX_HTTP_PARSER_BODY_CHUNK_SIZE,
    VEX_HTTP_PARSER_BODY_CHUNK_DATA,
    VEX_HTTP_PARSER_BODY_CHUNK_TRAILERS,
    VEX_HTTP_PARSER_DONE,
    VEX_HTTP_PARSER_ERROR
} vex_http_stream_state_t;

typedef struct {
    vex_http_request_t     *req;
    vex_http_stream_state_t state;
    size_t                  pos;
    size_t                  content_length;
    int                     has_content_length;
    int                     is_chunked;
    size_t                  body_start;
    size_t                  body_written;
    size_t                  chunk_bytes_remaining;
} vex_http_stream_t;

/* Connection wrapper (for pipelining/keep-alive) */
typedef struct {
    vex_http_stream_t parser;
} vex_http_connection_t;

/* ========== Streaming API ========== */

/**
 * Initialize streaming HTTP parser
 */
void vex_http_stream_init(vex_http_stream_t *st, vex_http_request_t *req);

/**
 * Execute streaming parse
 * 
 * @param st Streaming parser state
 * @param buf Buffer (must remain valid between calls)
 * @param len Current buffer length
 * @param bytes_parsed Optional output: bytes consumed
 * @return VEX_HTTP_OK if done, VEX_HTTP_ERR_TRUNCATED if need more data, negative on error
 */
int vex_http_stream_execute(vex_http_stream_t *st, const char *buf, size_t len,
                            size_t *bytes_parsed);

/**
 * Check if parser is done
 */
int vex_http_stream_is_done(const vex_http_stream_t *st);

/* ========== One-Shot API ========== */

/**
 * Parse complete HTTP request from buffer
 * 
 * @param buf Request buffer
 * @param len Buffer length
 * @param out Output request structure
 * @return VEX_HTTP_OK on success, negative on error
 */
int vex_http_parse(const char *buf, size_t len, vex_http_request_t *out);

/* ========== Connection Wrapper (Pipelining) ========== */

/**
 * Initialize HTTP connection for pipelining
 */
void vex_http_connection_init(vex_http_connection_t *conn,
                              vex_http_request_t *first_req);

/**
 * Prepare for next pipelined request
 */
void vex_http_connection_next_request(vex_http_connection_t *conn,
                                      vex_http_request_t *next_req);

#ifdef __cplusplus
}
#endif

#endif /* VEX_HTTP_PARSER_H */
