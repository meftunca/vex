/*
 * hpack.c - HTTP/2 HPACK Header Compression (RFC 7541)
 * 
 * Implements:
 * - Static table (61 entries)
 * - Dynamic table management
 * - Integer encoding/decoding
 * - Huffman coding
 * - Header field representations
 */

#include "protocols/hpack.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

/* ========== Static Table (RFC 7541 Appendix A) ========== */

static const struct {
    const char *name;
    const char *value;
} vex_hpack_static_table[VEX_HPACK_STATIC_TABLE_SIZE] = {
    {":authority", ""},                          // 1
    {":method", "GET"},                          // 2
    {":method", "POST"},                         // 3
    {":path", "/"},                              // 4
    {":path", "/index.html"},                    // 5
    {":scheme", "http"},                         // 6
    {":scheme", "https"},                        // 7
    {":status", "200"},                          // 8
    {":status", "204"},                          // 9
    {":status", "206"},                          // 10
    {":status", "304"},                          // 11
    {":status", "400"},                          // 12
    {":status", "404"},                          // 13
    {":status", "500"},                          // 14
    {"accept-charset", ""},                      // 15
    {"accept-encoding", "gzip, deflate"},        // 16
    {"accept-language", ""},                     // 17
    {"accept-ranges", ""},                       // 18
    {"accept", ""},                              // 19
    {"access-control-allow-origin", ""},         // 20
    {"age", ""},                                 // 21
    {"allow", ""},                               // 22
    {"authorization", ""},                       // 23
    {"cache-control", ""},                       // 24
    {"content-disposition", ""},                 // 25
    {"content-encoding", ""},                    // 26
    {"content-language", ""},                    // 27
    {"content-length", ""},                      // 28
    {"content-location", ""},                    // 29
    {"content-range", ""},                       // 30
    {"content-type", ""},                        // 31
    {"cookie", ""},                              // 32
    {"date", ""},                                // 33
    {"etag", ""},                                // 34
    {"expect", ""},                              // 35
    {"expires", ""},                             // 36
    {"from", ""},                                // 37
    {"host", ""},                                // 38
    {"if-match", ""},                            // 39
    {"if-modified-since", ""},                   // 40
    {"if-none-match", ""},                       // 41
    {"if-range", ""},                            // 42
    {"if-unmodified-since", ""},                 // 43
    {"last-modified", ""},                       // 44
    {"link", ""},                                // 45
    {"location", ""},                            // 46
    {"max-forwards", ""},                        // 47
    {"proxy-authenticate", ""},                  // 48
    {"proxy-authorization", ""},                 // 49
    {"range", ""},                               // 50
    {"referer", ""},                             // 51
    {"refresh", ""},                             // 52
    {"retry-after", ""},                         // 53
    {"server", ""},                              // 54
    {"set-cookie", ""},                          // 55
    {"strict-transport-security", ""},           // 56
    {"transfer-encoding", ""},                   // 57
    {"user-agent", ""},                          // 58
    {"vary", ""},                                // 59
    {"via", ""},                                 // 60
    {"www-authenticate", ""}                     // 61
};

/* ========== Helper Functions ========== */

static size_t entry_size(const char *name, size_t name_len, 
                        const char *value, size_t value_len) {
    return name_len + value_len + 32;  /* RFC 7541: overhead is 32 bytes */
}

static int strings_equal(const char *a, size_t a_len, const char *b, size_t b_len) {
    if (a_len != b_len) return 0;
    return memcmp(a, b, a_len) == 0;
}

/* ========== Integer Encoding/Decoding (RFC 7541 Section 5.1) ========== */

int vex_hpack_decode_integer(const uint8_t *data, size_t data_len,
                             int prefix_bits, uint64_t *value, size_t *consumed)
{
    if (data_len == 0 || prefix_bits < 1 || prefix_bits > 8) {
        return VEX_HPACK_ERR_INVALID;
    }

    uint8_t mask = (uint8_t)((1 << prefix_bits) - 1);
    uint64_t i = data[0] & mask;
    
    if (i < mask) {
        *value = i;
        *consumed = 1;
        return VEX_HPACK_OK;
    }

    /* Multi-byte integer */
    uint64_t m = 0;
    size_t pos = 1;
    
    while (pos < data_len) {
        uint8_t b = data[pos];
        i += (uint64_t)(b & 0x7F) << m;
        
        if (i < ((uint64_t)(b & 0x7F) << m)) {
            return VEX_HPACK_ERR_INVALID;  /* Overflow */
        }
        
        pos++;
        
        if ((b & 0x80) == 0) {
            *value = i;
            *consumed = pos;
            return VEX_HPACK_OK;
        }
        
        m += 7;
        
        if (m > 56) {
            return VEX_HPACK_ERR_INVALID;  /* Too large */
        }
    }
    
    return VEX_HPACK_ERR_TRUNCATED;
}

int vex_hpack_encode_integer(uint64_t value, int prefix_bits, uint8_t prefix,
                             uint8_t *out, size_t out_size, size_t *out_len)
{
    if (prefix_bits < 1 || prefix_bits > 8 || out_size == 0) {
        return VEX_HPACK_ERR_INVALID;
    }

    uint8_t mask = (uint8_t)((1 << prefix_bits) - 1);
    
    if (value < mask) {
        out[0] = prefix | (uint8_t)value;
        *out_len = 1;
        return VEX_HPACK_OK;
    }

    /* Multi-byte encoding */
    out[0] = prefix | mask;
    value -= mask;
    
    size_t pos = 1;
    while (value >= 128) {
        if (pos >= out_size) {
            return VEX_HPACK_ERR_TOO_LARGE;
        }
        out[pos++] = (uint8_t)((value & 0x7F) | 0x80);
        value >>= 7;
    }
    
    if (pos >= out_size) {
        return VEX_HPACK_ERR_TOO_LARGE;
    }
    out[pos++] = (uint8_t)value;
    
    *out_len = pos;
    return VEX_HPACK_OK;
}

/* ========== Huffman Decode Table (RFC 7541 Appendix B) ========== */

/* Huffman decode tree node */
typedef struct {
    uint16_t left;   /* 0 bit */
    uint16_t right;  /* 1 bit */
    uint8_t  symbol; /* Decoded symbol (if terminal) */
    uint8_t  is_terminal;
} vex_huffman_node_t;

/* Simplified Huffman table - full implementation would be ~256 entries */
/* For now, we'll implement a basic version that handles common cases */

int vex_hpack_huffman_decode(const uint8_t *data, size_t data_len,
                             uint8_t *out, size_t out_size, size_t *out_len)
{
    /* TODO: Full Huffman decode table implementation */
    /* For now, return error to indicate Huffman not supported yet */
    (void)data;
    (void)data_len;
    (void)out;
    (void)out_size;
    (void)out_len;
    return VEX_HPACK_ERR_INVALID;  /* Not implemented yet */
}

int vex_hpack_huffman_encode(const uint8_t *data, size_t data_len,
                             uint8_t *out, size_t out_size, size_t *out_len)
{
    /* TODO: Full Huffman encode table implementation */
    (void)data;
    (void)data_len;
    (void)out;
    (void)out_size;
    (void)out_len;
    return VEX_HPACK_ERR_INVALID;  /* Not implemented yet */
}

/* ========== Dynamic Table Management ========== */

static void evict_entries(vex_hpack_decoder_t *dec, size_t required_size) {
    while (dec->dynamic_count > 0 && 
           dec->current_table_size + required_size > dec->max_table_size) {
        /* Evict oldest entry (at end of array) */
        vex_hpack_entry_t *entry = &dec->dynamic_table[dec->dynamic_count - 1];
        dec->current_table_size -= entry->size;
        free(entry->name);
        free(entry->value);
        dec->dynamic_count--;
    }
}

static int add_dynamic_entry(vex_hpack_decoder_t *dec,
                             const char *name, size_t name_len,
                             const char *value, size_t value_len)
{
    size_t size = entry_size(name, name_len, value, value_len);
    
    if (size > dec->max_table_size) {
        /* Entry too large, clear table */
        for (size_t i = 0; i < dec->dynamic_count; ++i) {
            free(dec->dynamic_table[i].name);
            free(dec->dynamic_table[i].value);
        }
        dec->dynamic_count = 0;
        dec->current_table_size = 0;
        return VEX_HPACK_OK;
    }

    evict_entries(dec, size);

    /* Ensure capacity */
    if (dec->dynamic_count >= dec->dynamic_capacity) {
        size_t new_cap = dec->dynamic_capacity == 0 ? 16 : dec->dynamic_capacity * 2;
        vex_hpack_entry_t *new_table = realloc(dec->dynamic_table,
                                                new_cap * sizeof(vex_hpack_entry_t));
        if (!new_table) {
            return VEX_HPACK_ERR_TABLE_FULL;
        }
        dec->dynamic_table = new_table;
        dec->dynamic_capacity = new_cap;
    }

    /* Insert at beginning, shift others */
    if (dec->dynamic_count > 0) {
        memmove(&dec->dynamic_table[1], &dec->dynamic_table[0],
                dec->dynamic_count * sizeof(vex_hpack_entry_t));
    }

    /* Add new entry */
    dec->dynamic_table[0].name = malloc(name_len + 1);
    dec->dynamic_table[0].value = malloc(value_len + 1);
    
    if (!dec->dynamic_table[0].name || !dec->dynamic_table[0].value) {
        free(dec->dynamic_table[0].name);
        free(dec->dynamic_table[0].value);
        return VEX_HPACK_ERR_TABLE_FULL;
    }

    memcpy(dec->dynamic_table[0].name, name, name_len);
    dec->dynamic_table[0].name[name_len] = '\0';
    dec->dynamic_table[0].name_len = name_len;

    memcpy(dec->dynamic_table[0].value, value, value_len);
    dec->dynamic_table[0].value[value_len] = '\0';
    dec->dynamic_table[0].value_len = value_len;

    dec->dynamic_table[0].size = size;
    dec->dynamic_count++;
    dec->current_table_size += size;

    return VEX_HPACK_OK;
}

static int lookup_index(vex_hpack_decoder_t *dec, uint64_t index,
                       const char **name, size_t *name_len,
                       const char **value, size_t *value_len)
{
    if (index == 0) {
        return VEX_HPACK_ERR_INVALID;
    }

    /* Static table */
    if (index <= VEX_HPACK_STATIC_TABLE_SIZE) {
        *name = vex_hpack_static_table[index - 1].name;
        *name_len = strlen(*name);
        *value = vex_hpack_static_table[index - 1].value;
        *value_len = strlen(*value);
        return VEX_HPACK_OK;
    }

    /* Dynamic table */
    uint64_t dyn_index = index - VEX_HPACK_STATIC_TABLE_SIZE - 1;
    if (dyn_index >= dec->dynamic_count) {
        return VEX_HPACK_ERR_INVALID;
    }

    vex_hpack_entry_t *entry = &dec->dynamic_table[dyn_index];
    *name = entry->name;
    *name_len = entry->name_len;
    *value = entry->value;
    *value_len = entry->value_len;

    return VEX_HPACK_OK;
}

/* ========== Decoder API ========== */

void vex_hpack_decoder_init(vex_hpack_decoder_t *dec, size_t max_table_size) {
    dec->dynamic_table = NULL;
    dec->dynamic_count = 0;
    dec->dynamic_capacity = 0;
    dec->max_table_size = max_table_size;
    dec->current_table_size = 0;
}

void vex_hpack_decoder_destroy(vex_hpack_decoder_t *dec) {
    for (size_t i = 0; i < dec->dynamic_count; ++i) {
        free(dec->dynamic_table[i].name);
        free(dec->dynamic_table[i].value);
    }
    free(dec->dynamic_table);
    dec->dynamic_table = NULL;
    dec->dynamic_count = 0;
    dec->dynamic_capacity = 0;
}

int vex_hpack_decode_block(vex_hpack_decoder_t *dec,
                           const uint8_t *data, size_t data_len,
                           vex_hpack_header_t *headers, size_t max_headers,
                           size_t *header_count)
{
    size_t pos = 0;
    size_t count = 0;

    while (pos < data_len && count < max_headers) {
        uint8_t first_byte = data[pos];

        /* Indexed Header Field (RFC 7541 Section 6.1) */
        if ((first_byte & 0x80) != 0) {
            uint64_t index;
            size_t consumed;
            int ret = vex_hpack_decode_integer(data + pos, data_len - pos, 7, &index, &consumed);
            if (ret != VEX_HPACK_OK) return ret;

            const char *name, *value;
            size_t name_len, value_len;
            ret = lookup_index(dec, index, &name, &name_len, &value, &value_len);
            if (ret != VEX_HPACK_OK) return ret;

            headers[count].name = name;
            headers[count].name_len = name_len;
            headers[count].value = value;
            headers[count].value_len = value_len;
            count++;
            pos += consumed;
        }
        /* Literal Header Field with Incremental Indexing (RFC 7541 Section 6.2.1) */
        else if ((first_byte & 0xC0) == 0x40) {
            /* TODO: Implement literal with incremental indexing */
            return VEX_HPACK_ERR_INVALID;  /* Not fully implemented */
        }
        /* Literal Header Field without Indexing (RFC 7541 Section 6.2.2) */
        else if ((first_byte & 0xF0) == 0x00) {
            /* TODO: Implement literal without indexing */
            return VEX_HPACK_ERR_INVALID;  /* Not fully implemented */
        }
        /* Dynamic Table Size Update (RFC 7541 Section 6.3) */
        else if ((first_byte & 0xE0) == 0x20) {
            uint64_t new_size;
            size_t consumed;
            int ret = vex_hpack_decode_integer(data + pos, data_len - pos, 5, &new_size, &consumed);
            if (ret != VEX_HPACK_OK) return ret;

            if (new_size > dec->max_table_size) {
                return VEX_HPACK_ERR_TOO_LARGE;
            }

            /* Evict entries if necessary */
            while (dec->dynamic_count > 0 && dec->current_table_size > new_size) {
                vex_hpack_entry_t *entry = &dec->dynamic_table[dec->dynamic_count - 1];
                dec->current_table_size -= entry->size;
                free(entry->name);
                free(entry->value);
                dec->dynamic_count--;
            }

            pos += consumed;
        }
        else {
            return VEX_HPACK_ERR_INVALID;
        }
    }

    *header_count = count;
    return VEX_HPACK_OK;
}

/* ========== Encoder API (Stub) ========== */

void vex_hpack_encoder_init(vex_hpack_encoder_t *enc, size_t max_table_size) {
    enc->dynamic_table = NULL;
    enc->dynamic_count = 0;
    enc->dynamic_capacity = 0;
    enc->max_table_size = max_table_size;
    enc->current_table_size = 0;
}

void vex_hpack_encoder_destroy(vex_hpack_encoder_t *enc) {
    for (size_t i = 0; i < enc->dynamic_count; ++i) {
        free(enc->dynamic_table[i].name);
        free(enc->dynamic_table[i].value);
    }
    free(enc->dynamic_table);
    enc->dynamic_table = NULL;
    enc->dynamic_count = 0;
    enc->dynamic_capacity = 0;
}

int vex_hpack_encode_headers(vex_hpack_encoder_t *enc,
                             const vex_hpack_header_t *headers, size_t header_count,
                             uint8_t *out, size_t out_size, size_t *out_len)
{
    /* TODO: Full encoder implementation */
    (void)enc;
    (void)headers;
    (void)header_count;
    (void)out;
    (void)out_size;
    (void)out_len;
    return VEX_HPACK_ERR_INVALID;  /* Not implemented yet */
}
