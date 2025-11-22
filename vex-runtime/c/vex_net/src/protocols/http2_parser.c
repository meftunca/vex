/*
 * http2_parser.c - Minimal HTTP/2 preface and frame header parser
 * Note: This is NOT a full HTTP/2 implementation
 */

#include "protocols/http2_parser.h"
#include <string.h>

/* HTTP/2 client connection preface */
static const char vex_http2_client_preface[VEX_HTTP2_PREFACE_LEN] =
    "PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

int vex_http2_is_preface(const uint8_t *buf, size_t len) {
    if (len < VEX_HTTP2_PREFACE_LEN) {
        return 0;
    }
    return memcmp(buf, vex_http2_client_preface, VEX_HTTP2_PREFACE_LEN) == 0;
}

int vex_http2_parse_frame_header(const uint8_t *buf, size_t len,
                                 vex_http2_frame_header_t *out)
{
    if (len < 9) {
        return -1; /* Truncated */
    }

    /* Parse 24-bit length (3 bytes, big-endian) */
    uint32_t length = ((uint32_t)buf[0] << 16) |
                      ((uint32_t)buf[1] << 8)  |
                      ((uint32_t)buf[2]);

    /* Parse type and flags */
    uint8_t type  = buf[3];
    uint8_t flags = buf[4];

    /* Parse 32-bit stream ID (4 bytes, big-endian), ignore reserved bit */
    uint32_t stream_id = ((uint32_t)buf[5] << 24) |
                         ((uint32_t)buf[6] << 16) |
                         ((uint32_t)buf[7] << 8)  |
                         ((uint32_t)buf[8]);
    stream_id &= 0x7FFFFFFFu; /* Clear reserved bit */

    out->length    = length;
    out->type      = type;
    out->flags     = flags;
    out->stream_id = stream_id;

    return 0; /* Success */
}
