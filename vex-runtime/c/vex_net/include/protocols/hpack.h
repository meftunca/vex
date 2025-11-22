#ifndef VEX_HPACK_H
#define VEX_HPACK_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* HPACK error codes */
#define VEX_HPACK_OK                0
#define VEX_HPACK_ERR_TRUNCATED    -1
#define VEX_HPACK_ERR_INVALID      -2
#define VEX_HPACK_ERR_TABLE_FULL   -3
#define VEX_HPACK_ERR_TOO_LARGE    -4

/* Maximum dynamic table size (configurable) */
#ifndef VEX_HPACK_MAX_TABLE_SIZE
#define VEX_HPACK_MAX_TABLE_SIZE 4096
#endif

/* Static table size (RFC 7541 Appendix A) */
#define VEX_HPACK_STATIC_TABLE_SIZE 61

/* Header field representation */
typedef struct {
    const char *name;
    size_t      name_len;
    const char *value;
    size_t      value_len;
} vex_hpack_header_t;

/* Dynamic table entry */
typedef struct {
    char   *name;
    size_t  name_len;
    char   *value;
    size_t  value_len;
    size_t  size;  /* name_len + value_len + 32 (RFC overhead) */
} vex_hpack_entry_t;

/* HPACK decoder context */
typedef struct {
    vex_hpack_entry_t *dynamic_table;
    size_t             dynamic_count;
    size_t             dynamic_capacity;
    size_t             max_table_size;
    size_t             current_table_size;
} vex_hpack_decoder_t;

/* HPACK encoder context */
typedef struct {
    vex_hpack_entry_t *dynamic_table;
    size_t             dynamic_count;
    size_t             dynamic_capacity;
    size_t             max_table_size;
    size_t             current_table_size;
} vex_hpack_encoder_t;

/* ========== Decoder API ========== */

/**
 * Initialize HPACK decoder
 * @param dec Decoder context
 * @param max_table_size Maximum dynamic table size in bytes
 */
void vex_hpack_decoder_init(vex_hpack_decoder_t *dec, size_t max_table_size);

/**
 * Destroy HPACK decoder (free dynamic table)
 * @param dec Decoder context
 */
void vex_hpack_decoder_destroy(vex_hpack_decoder_t *dec);

/**
 * Decode HPACK header block
 * @param dec Decoder context
 * @param data Encoded header block
 * @param data_len Length of encoded data
 * @param headers Output headers array
 * @param max_headers Maximum headers to decode
 * @param header_count Output: number of headers decoded
 * @return VEX_HPACK_OK on success, negative on error
 */
int vex_hpack_decode_block(vex_hpack_decoder_t *dec,
                           const uint8_t *data, size_t data_len,
                           vex_hpack_header_t *headers, size_t max_headers,
                           size_t *header_count);

/* ========== Encoder API ========== */

/**
 * Initialize HPACK encoder
 * @param enc Encoder context
 * @param max_table_size Maximum dynamic table size in bytes
 */
void vex_hpack_encoder_init(vex_hpack_encoder_t *enc, size_t max_table_size);

/**
 * Destroy HPACK encoder (free dynamic table)
 * @param enc Encoder context
 */
void vex_hpack_encoder_destroy(vex_hpack_encoder_t *enc);

/**
 * Encode headers to HPACK format
 * @param enc Encoder context
 * @param headers Headers to encode
 * @param header_count Number of headers
 * @param out Output buffer
 * @param out_size Size of output buffer
 * @param out_len Output: bytes written
 * @return VEX_HPACK_OK on success, negative on error
 */
int vex_hpack_encode_headers(vex_hpack_encoder_t *enc,
                             const vex_hpack_header_t *headers, size_t header_count,
                             uint8_t *out, size_t out_size, size_t *out_len);

/* ========== Utility Functions ========== */

/**
 * Decode HPACK integer (RFC 7541 Section 5.1)
 * @param data Input data
 * @param data_len Length of input
 * @param prefix_bits Number of prefix bits (1-8)
 * @param value Output: decoded integer
 * @param consumed Output: bytes consumed
 * @return VEX_HPACK_OK on success, negative on error
 */
int vex_hpack_decode_integer(const uint8_t *data, size_t data_len,
                             int prefix_bits, uint64_t *value, size_t *consumed);

/**
 * Encode HPACK integer (RFC 7541 Section 5.1)
 * @param value Integer to encode
 * @param prefix_bits Number of prefix bits (1-8)
 * @param prefix Prefix value to OR with first byte
 * @param out Output buffer
 * @param out_size Size of output buffer
 * @param out_len Output: bytes written
 * @return VEX_HPACK_OK on success, negative on error
 */
int vex_hpack_encode_integer(uint64_t value, int prefix_bits, uint8_t prefix,
                             uint8_t *out, size_t out_size, size_t *out_len);

/**
 * Decode Huffman-encoded string (RFC 7541 Section 5.2)
 * @param data Huffman-encoded data
 * @param data_len Length of encoded data
 * @param out Output buffer
 * @param out_size Size of output buffer
 * @param out_len Output: bytes written
 * @return VEX_HPACK_OK on success, negative on error
 */
int vex_hpack_huffman_decode(const uint8_t *data, size_t data_len,
                             uint8_t *out, size_t out_size, size_t *out_len);

/**
 * Encode string with Huffman coding (RFC 7541 Section 5.2)
 * @param data String to encode
 * @param data_len Length of string
 * @param out Output buffer
 * @param out_size Size of output buffer
 * @param out_len Output: bytes written
 * @return VEX_HPACK_OK on success, negative on error
 */
int vex_hpack_huffman_encode(const uint8_t *data, size_t data_len,
                             uint8_t *out, size_t out_size, size_t *out_len);

#ifdef __cplusplus
}
#endif

#endif /* VEX_HPACK_H */
