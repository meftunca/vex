#ifndef VEX_TLS_DETECTOR_H
#define VEX_TLS_DETECTOR_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* TLS Content Types */
#define VEX_TLS_TYPE_CHANGE_CIPHER_SPEC 20
#define VEX_TLS_TYPE_ALERT              21
#define VEX_TLS_TYPE_HANDSHAKE          22
#define VEX_TLS_TYPE_APPLICATION_DATA   23

/* TLS Handshake Types */
#define VEX_TLS_HANDSHAKE_CLIENT_HELLO  1
#define VEX_TLS_HANDSHAKE_SERVER_HELLO  2

/* TLS Versions */
#define VEX_TLS_VERSION_1_0 0x0301
#define VEX_TLS_VERSION_1_1 0x0302
#define VEX_TLS_VERSION_1_2 0x0303
#define VEX_TLS_VERSION_1_3 0x0304

/* TLS ClientHello Info */
typedef struct {
    uint16_t version;
    uint8_t  random[32];
    uint8_t  session_id_len;
    uint8_t  session_id[32];
    char     sni[256];      /* Server Name Indication */
    char     alpn[128];     /* Application-Layer Protocol Negotiation */
    int      has_sni;
    int      has_alpn;
} vex_tls_client_hello_t;

/* Error Codes */
#define VEX_TLS_OK              0
#define VEX_TLS_ERR_TRUNCATED  -1
#define VEX_TLS_ERR_NOT_TLS    -2
#define VEX_TLS_ERR_INVALID    -3

/**
 * Check if buffer looks like a TLS handshake
 * @param buf Buffer
 * @param len Length
 * @return 1 if TLS handshake, 0 otherwise
 */
int vex_tls_is_handshake(const uint8_t *buf, size_t len);

/**
 * Parse TLS ClientHello
 * @param buf Buffer (start of record)
 * @param len Length
 * @param hello Output structure
 * @return VEX_TLS_OK on success
 */
int vex_tls_parse_client_hello(const uint8_t *buf, size_t len, 
                               vex_tls_client_hello_t *hello);

#ifdef __cplusplus
}
#endif

#endif /* VEX_TLS_DETECTOR_H */
