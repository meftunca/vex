#ifndef VEX_HTTP2_PARSER_H
#define VEX_HTTP2_PARSER_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* HTTP/2 client preface length */
#define VEX_HTTP2_PREFACE_LEN 24

/* HTTP/2 frame header (9 bytes) */
typedef struct {
    uint32_t length;     /* 24-bit payload length */
    uint8_t  type;       /* Frame type */
    uint8_t  flags;      /* Flags */
    uint32_t stream_id;  /* 31-bit stream identifier */
} vex_http2_frame_header_t;

/**
 * Check if buffer starts with HTTP/2 client preface
 * Preface: "PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n"
 * 
 * @param buf Buffer to check
 * @param len Buffer length
 * @return 1 if preface matches, 0 otherwise
 */
int vex_http2_is_preface(const uint8_t *buf, size_t len);

/**
 * Parse HTTP/2 frame header (9 bytes)
 * 
 * @param buf Buffer containing frame header
 * @param len Buffer length (must be >= 9)
 * @param out Output frame header struct
 * @return 0 on success, -1 if truncated
 */
int vex_http2_parse_frame_header(const uint8_t *buf, size_t len,
                                 vex_http2_frame_header_t *out);

#ifdef __cplusplus
}
#endif

#endif /* VEX_HTTP2_PARSER_H */
