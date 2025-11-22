#ifndef VEX_WEBSOCKET_PARSER_H
#define VEX_WEBSOCKET_PARSER_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* WebSocket opcodes (RFC 6455) */
typedef enum {
    VEX_WS_OPCODE_CONTINUATION = 0x0,
    VEX_WS_OPCODE_TEXT         = 0x1,
    VEX_WS_OPCODE_BINARY       = 0x2,
    VEX_WS_OPCODE_CLOSE        = 0x8,
    VEX_WS_OPCODE_PING         = 0x9,
    VEX_WS_OPCODE_PONG         = 0xA
} vex_ws_opcode_t;

/* WebSocket close codes */
typedef enum {
    VEX_WS_CLOSE_NORMAL            = 1000,
    VEX_WS_CLOSE_GOING_AWAY        = 1001,
    VEX_WS_CLOSE_PROTOCOL_ERROR    = 1002,
    VEX_WS_CLOSE_UNSUPPORTED_DATA  = 1003,
    VEX_WS_CLOSE_NO_STATUS         = 1005,
    VEX_WS_CLOSE_ABNORMAL          = 1006,
    VEX_WS_CLOSE_INVALID_PAYLOAD   = 1007,
    VEX_WS_CLOSE_POLICY_VIOLATION  = 1008,
    VEX_WS_CLOSE_MESSAGE_TOO_BIG   = 1009,
    VEX_WS_CLOSE_MANDATORY_EXT     = 1010,
    VEX_WS_CLOSE_INTERNAL_ERROR    = 1011
} vex_ws_close_code_t;

/* WebSocket frame */
typedef struct {
    int             fin;           /* Final fragment */
    int             rsv1;          /* Reserved bit 1 */
    int             rsv2;          /* Reserved bit 2 */
    int             rsv3;          /* Reserved bit 3 */
    vex_ws_opcode_t opcode;        /* Opcode */
    int             masked;        /* Payload masked */
    uint64_t        payload_len;   /* Payload length */
    uint8_t         mask_key[4];   /* Masking key (if masked) */
    const uint8_t  *payload;       /* Payload data (points into buffer) */
} vex_ws_frame_t;

/* Error codes */
#define VEX_WS_OK               0
#define VEX_WS_ERR_TRUNCATED   -1
#define VEX_WS_ERR_INVALID     -2
#define VEX_WS_ERR_TOO_LARGE   -3

/**
 * Parse WebSocket frame
 * @param data Frame data
 * @param len Data length
 * @param frame Output frame structure
 * @param consumed Output: bytes consumed
 * @return VEX_WS_OK on success, negative on error
 */
int vex_ws_parse_frame(const uint8_t *data, size_t len,
                       vex_ws_frame_t *frame, size_t *consumed);

/**
 * Unmask WebSocket payload (in-place)
 * @param payload Payload data (will be modified)
 * @param len Payload length
 * @param mask_key Masking key (4 bytes)
 */
void vex_ws_unmask_payload(uint8_t *payload, size_t len, const uint8_t mask_key[4]);

/**
 * Validate WebSocket upgrade request
 * @param req HTTP request (from http_parser.h)
 * @return 1 if valid upgrade, 0 otherwise
 */
int vex_ws_validate_upgrade(const void *http_request);

/**
 * Build WebSocket accept key from client key
 * @param client_key Client Sec-WebSocket-Key header value
 * @param client_key_len Length of client key
 * @param accept_key Output buffer (must be at least 29 bytes)
 * @return VEX_WS_OK on success, negative on error
 */
int vex_ws_build_accept_key(const char *client_key, size_t client_key_len,
                            char *accept_key);

#ifdef __cplusplus
}
#endif

#endif /* VEX_WEBSOCKET_PARSER_H */
