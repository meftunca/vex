/*
 * dns_parser.c - DNS Protocol Parser (RFC 1035)
 */

#include "protocols/dns_parser.h"
#include <string.h>

/* Helper: Read 16-bit big-endian */
static inline uint16_t read_be16(const uint8_t *p) {
    return (uint16_t)((uint16_t)p[0] << 8 | (uint16_t)p[1]);
}

/* Helper: Read 32-bit big-endian */
static inline uint32_t read_be32(const uint8_t *p) {
    return ((uint32_t)p[0] << 24) |
           ((uint32_t)p[1] << 16) |
           ((uint32_t)p[2] << 8)  |
           ((uint32_t)p[3]);
}

int vex_dns_parse_header(const uint8_t *buf, size_t len, vex_dns_header_t *header) {
    if (len < 12) {
        return VEX_DNS_ERR_TRUNCATED;
    }

    header->id      = read_be16(buf + 0);
    header->flags   = read_be16(buf + 2);
    header->qdcount = read_be16(buf + 4);
    header->ancount = read_be16(buf + 6);
    header->nscount = read_be16(buf + 8);
    header->arcount = read_be16(buf + 10);

    return VEX_DNS_OK;
}

int vex_dns_parse_name(const uint8_t *buf, size_t len, size_t *offset,
                       char *name, size_t name_size)
{
    size_t pos = *offset;
    size_t out_pos = 0;
    int jumped = 0;
    size_t jump_offset = 0;
    int jumps = 0;
    const int max_jumps = 5; /* Limit recursion/loops */

    if (pos >= len) return VEX_DNS_ERR_TRUNCATED;

    while (1) {
        if (pos >= len) return VEX_DNS_ERR_TRUNCATED;
        
        uint8_t len_byte = buf[pos];

        /* End of name */
        if (len_byte == 0) {
            pos++;
            break;
        }

        /* Pointer (0xC0) */
        if ((len_byte & 0xC0) == 0xC0) {
            if (pos + 2 > len) return VEX_DNS_ERR_TRUNCATED;
            
            uint16_t ptr = read_be16(buf + pos);
            size_t target = ptr & 0x3FFF;
            
            if (target >= len) return VEX_DNS_ERR_INVALID;
            
            if (!jumped) {
                jump_offset = pos + 2; /* Save return position */
                jumped = 1;
            }
            
            pos = target;
            jumps++;
            if (jumps > max_jumps) return VEX_DNS_ERR_LOOP;
            
            continue;
        }

        /* Label */
        size_t label_len = len_byte;
        pos++;

        if (pos + label_len > len) return VEX_DNS_ERR_TRUNCATED;
        if (out_pos + label_len + 1 >= name_size) return VEX_DNS_ERR_NAME_TOO_LONG;

        if (out_pos > 0) {
            name[out_pos++] = '.';
        }
        
        memcpy(name + out_pos, buf + pos, label_len);
        out_pos += label_len;
        pos += label_len;
    }

    name[out_pos] = '\0';

    if (jumped) {
        *offset = jump_offset;
    } else {
        *offset = pos;
    }

    return VEX_DNS_OK;
}

int vex_dns_parse_question(const uint8_t *buf, size_t len, size_t *offset,
                           vex_dns_question_t *question)
{
    int ret = vex_dns_parse_name(buf, len, offset, question->name, sizeof(question->name));
    if (ret != VEX_DNS_OK) return ret;

    if (*offset + 4 > len) return VEX_DNS_ERR_TRUNCATED;

    question->type   = read_be16(buf + *offset);
    question->class_ = read_be16(buf + *offset + 2);
    *offset += 4;

    return VEX_DNS_OK;
}

int vex_dns_parse_record(const uint8_t *buf, size_t len, size_t *offset,
                         vex_dns_record_t *record)
{
    int ret = vex_dns_parse_name(buf, len, offset, record->name, sizeof(record->name));
    if (ret != VEX_DNS_OK) return ret;

    if (*offset + 10 > len) return VEX_DNS_ERR_TRUNCATED;

    record->type   = read_be16(buf + *offset);
    record->class_ = read_be16(buf + *offset + 2);
    record->ttl    = read_be32(buf + *offset + 4);
    record->rdlen  = read_be16(buf + *offset + 8);
    *offset += 10;

    if (*offset + record->rdlen > len) return VEX_DNS_ERR_TRUNCATED;

    record->rdata = buf + *offset;
    *offset += record->rdlen;

    return VEX_DNS_OK;
}
