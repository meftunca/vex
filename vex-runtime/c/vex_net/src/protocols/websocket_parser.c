/*
 * websocket_parser.c - WebSocket Protocol Parser (RFC 6455)
 */

#include "protocols/websocket_parser.h"
#include "protocols/simd_utils.h"
#include <string.h>

/* ========== Frame Parsing ========== */

int vex_ws_parse_frame(const uint8_t *data, size_t len,
                       vex_ws_frame_t *frame, size_t *consumed)
{
    if (len < 2) {
        return VEX_WS_ERR_TRUNCATED;
    }

    size_t pos = 0;

    /* Byte 0: FIN, RSV, opcode */
    uint8_t byte0 = data[pos++];
    frame->fin    = (byte0 & 0x80) != 0;
    frame->rsv1   = (byte0 & 0x40) != 0;
    frame->rsv2   = (byte0 & 0x20) != 0;
    frame->rsv3   = (byte0 & 0x10) != 0;
    frame->opcode = (vex_ws_opcode_t)(byte0 & 0x0F);

    /* Validate opcode */
    if (frame->opcode > 0xA || 
        (frame->opcode > 0x2 && frame->opcode < 0x8)) {
        return VEX_WS_ERR_INVALID;
    }

    /* Byte 1: MASK, payload length */
    uint8_t byte1 = data[pos++];
    frame->masked      = (byte1 & 0x80) != 0;
    uint8_t len_field  = byte1 & 0x7F;

    /* Parse payload length */
    if (len_field < 126) {
        frame->payload_len = len_field;
    } else if (len_field == 126) {
        if (pos + 2 > len) {
            return VEX_WS_ERR_TRUNCATED;
        }
        frame->payload_len = ((uint64_t)data[pos] << 8) | data[pos + 1];
        pos += 2;
    } else { /* len_field == 127 */
        if (pos + 8 > len) {
            return VEX_WS_ERR_TRUNCATED;
        }
        frame->payload_len = 0;
        for (int i = 0; i < 8; ++i) {
            frame->payload_len = (frame->payload_len << 8) | data[pos + i];
        }
        pos += 8;
    }

    /* Parse masking key */
    if (frame->masked) {
        if (pos + 4 > len) {
            return VEX_WS_ERR_TRUNCATED;
        }
        memcpy(frame->mask_key, data + pos, 4);
        pos += 4;
    }

    /* Check if payload is available */
    if (pos + frame->payload_len > len) {
        return VEX_WS_ERR_TRUNCATED;
    }

    frame->payload = data + pos;
    pos += frame->payload_len;

    *consumed = pos;
    return VEX_WS_OK;
}

/* ========== Payload Unmasking ========== */

void vex_ws_unmask_payload(uint8_t *payload, size_t len, const uint8_t mask_key[4]) {
    vex_simd_xor_stream(payload, len, mask_key);
}

/* ========== Upgrade Validation ========== */

int vex_ws_validate_upgrade(const void *http_request) {
    /* This would require including http_parser.h and checking headers */
    /* For now, stub implementation */
    (void)http_request;
    return 0;  /* TODO: Implement proper validation */
}

/* ========== Accept Key Generation ========== */

/* WebSocket GUID (RFC 6455) */
static const char *WS_GUID = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

int vex_ws_build_accept_key(const char *client_key, size_t client_key_len,
                            char *accept_key)
{
    /* TODO: Implement SHA-1 hash + base64 encode */
    /* For now, stub implementation */
    (void)client_key;
    (void)client_key_len;
    (void)WS_GUID;
    (void)accept_key;
    return VEX_WS_ERR_INVALID;  /* Not implemented */
}
