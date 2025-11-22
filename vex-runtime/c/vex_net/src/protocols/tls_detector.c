/*
 * tls_detector.c - TLS Protocol Detector & ClientHello Parser
 */

#include "protocols/tls_detector.h"
#include <string.h>

/* Helper: Read 16-bit big-endian */
static inline uint16_t read_be16(const uint8_t *p) {
    return (uint16_t)((uint16_t)p[0] << 8 | (uint16_t)p[1]);
}

int vex_tls_is_handshake(const uint8_t *buf, size_t len) {
    if (len < 5) return 0;
    
    /* ContentType: Handshake (22) */
    if (buf[0] != VEX_TLS_TYPE_HANDSHAKE) return 0;
    
    /* Version: 3.x */
    if (buf[1] != 3) return 0;
    
    return 1;
}

int vex_tls_parse_client_hello(const uint8_t *buf, size_t len, 
                               vex_tls_client_hello_t *hello)
{
    if (!vex_tls_is_handshake(buf, len)) {
        return VEX_TLS_ERR_NOT_TLS;
    }

    size_t pos = 5; /* Skip Record Header */
    
    /* Handshake Header */
    if (pos + 4 > len) return VEX_TLS_ERR_TRUNCATED;
    
    uint8_t msg_type = buf[pos];
    if (msg_type != VEX_TLS_HANDSHAKE_CLIENT_HELLO) {
        return VEX_TLS_ERR_INVALID;
    }
    
    /* Skip Handshake Length (3 bytes) */
    pos += 4;
    
    /* Client Version */
    if (pos + 2 > len) return VEX_TLS_ERR_TRUNCATED;
    hello->version = read_be16(buf + pos);
    pos += 2;
    
    /* Random */
    if (pos + 32 > len) return VEX_TLS_ERR_TRUNCATED;
    memcpy(hello->random, buf + pos, 32);
    pos += 32;
    
    /* Session ID */
    if (pos + 1 > len) return VEX_TLS_ERR_TRUNCATED;
    hello->session_id_len = buf[pos];
    pos += 1;
    
    if (pos + hello->session_id_len > len) return VEX_TLS_ERR_TRUNCATED;
    if (hello->session_id_len > 32) {
        /* Cap at 32 for storage, but skip full length */
        memcpy(hello->session_id, buf + pos, 32);
    } else {
        memcpy(hello->session_id, buf + pos, hello->session_id_len);
    }
    pos += hello->session_id_len;
    
    /* Cipher Suites */
    if (pos + 2 > len) return VEX_TLS_ERR_TRUNCATED;
    uint16_t cipher_len = read_be16(buf + pos);
    pos += 2;
    
    if (pos + cipher_len > len) return VEX_TLS_ERR_TRUNCATED;
    pos += cipher_len;
    
    /* Compression Methods */
    if (pos + 1 > len) return VEX_TLS_ERR_TRUNCATED;
    uint8_t comp_len = buf[pos];
    pos += 1;
    
    if (pos + comp_len > len) return VEX_TLS_ERR_TRUNCATED;
    pos += comp_len;
    
    /* Extensions */
    if (pos + 2 > len) {
        /* No extensions present */
        hello->has_sni = 0;
        hello->has_alpn = 0;
        return VEX_TLS_OK;
    }
    
    uint16_t ext_len = read_be16(buf + pos);
    pos += 2;
    
    if (pos + ext_len > len) return VEX_TLS_ERR_TRUNCATED;
    size_t limit = pos + ext_len;
    
    hello->has_sni = 0;
    hello->has_alpn = 0;
    hello->sni[0] = '\0';
    hello->alpn[0] = '\0';
    
    while (pos + 4 <= limit) {
        uint16_t type = read_be16(buf + pos);
        uint16_t len = read_be16(buf + pos + 2);
        pos += 4;
        
        if (pos + len > limit) break;
        
        /* SNI Extension (0) */
        if (type == 0) {
            if (len > 2) {
                uint16_t list_len = read_be16(buf + pos);
                if (pos + 2 + list_len <= limit && list_len > 3) {
                    uint8_t name_type = buf[pos + 2];
                    uint16_t name_len = read_be16(buf + pos + 3);
                    
                    if (name_type == 0 && name_len < sizeof(hello->sni)) {
                        memcpy(hello->sni, buf + pos + 5, name_len);
                        hello->sni[name_len] = '\0';
                        hello->has_sni = 1;
                    }
                }
            }
        }
        /* ALPN Extension (16) */
        else if (type == 16) {
            if (len > 2) {
                uint16_t list_len = read_be16(buf + pos);
                if (pos + 2 + list_len <= limit && list_len > 0) {
                    /* Just take the first protocol for simplicity */
                    uint8_t proto_len = buf[pos + 2];
                    if (proto_len < sizeof(hello->alpn)) {
                        memcpy(hello->alpn, buf + pos + 3, proto_len);
                        hello->alpn[proto_len] = '\0';
                        hello->has_alpn = 1;
                    }
                }
            }
        }
        
        pos += len;
    }
    
    return VEX_TLS_OK;
}
