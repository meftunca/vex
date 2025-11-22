#ifndef VEX_DNS_PARSER_H
#define VEX_DNS_PARSER_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* DNS Record Types */
typedef enum {
    VEX_DNS_TYPE_A     = 1,
    VEX_DNS_TYPE_NS    = 2,
    VEX_DNS_TYPE_CNAME = 5,
    VEX_DNS_TYPE_SOA   = 6,
    VEX_DNS_TYPE_PTR   = 12,
    VEX_DNS_TYPE_MX    = 15,
    VEX_DNS_TYPE_TXT   = 16,
    VEX_DNS_TYPE_AAAA  = 28,
    VEX_DNS_TYPE_SRV   = 33,
    VEX_DNS_TYPE_ANY   = 255
} vex_dns_type_t;

/* DNS Classes */
typedef enum {
    VEX_DNS_CLASS_IN   = 1,
    VEX_DNS_CLASS_ANY  = 255
} vex_dns_class_t;

/* DNS Header Flags */
#define VEX_DNS_FLAG_QR     0x8000  /* Query (0) / Response (1) */
#define VEX_DNS_FLAG_OPCODE 0x7800  /* Opcode mask */
#define VEX_DNS_FLAG_AA     0x0400  /* Authoritative Answer */
#define VEX_DNS_FLAG_TC     0x0200  /* Truncated */
#define VEX_DNS_FLAG_RD     0x0100  /* Recursion Desired */
#define VEX_DNS_FLAG_RA     0x0080  /* Recursion Available */
#define VEX_DNS_FLAG_RCODE  0x000F  /* Response Code mask */

/* DNS Header */
typedef struct {
    uint16_t id;
    uint16_t flags;
    uint16_t qdcount;   /* Question count */
    uint16_t ancount;   /* Answer count */
    uint16_t nscount;   /* Authority count */
    uint16_t arcount;   /* Additional count */
} vex_dns_header_t;

/* DNS Question */
typedef struct {
    char name[256];
    uint16_t type;
    uint16_t class_;
} vex_dns_question_t;

/* DNS Resource Record */
typedef struct {
    char name[256];
    uint16_t type;
    uint16_t class_;
    uint32_t ttl;
    uint16_t rdlen;
    const uint8_t *rdata; /* Points into original buffer */
} vex_dns_record_t;

/* Error Codes */
#define VEX_DNS_OK              0
#define VEX_DNS_ERR_TRUNCATED  -1
#define VEX_DNS_ERR_INVALID    -2
#define VEX_DNS_ERR_NAME_TOO_LONG -3
#define VEX_DNS_ERR_LOOP       -4

/**
 * Parse DNS Header
 * @param buf Packet buffer
 * @param len Packet length
 * @param header Output header structure
 * @return VEX_DNS_OK on success
 */
int vex_dns_parse_header(const uint8_t *buf, size_t len, vex_dns_header_t *header);

/**
 * Parse DNS Question Section
 * @param buf Packet buffer (start of packet)
 * @param len Packet length
 * @param offset Current offset in buffer (updated on success)
 * @param question Output question structure
 * @return VEX_DNS_OK on success
 */
int vex_dns_parse_question(const uint8_t *buf, size_t len, size_t *offset,
                           vex_dns_question_t *question);

/**
 * Parse DNS Resource Record
 * @param buf Packet buffer (start of packet)
 * @param len Packet length
 * @param offset Current offset in buffer (updated on success)
 * @param record Output record structure
 * @return VEX_DNS_OK on success
 */
int vex_dns_parse_record(const uint8_t *buf, size_t len, size_t *offset,
                         vex_dns_record_t *record);

/**
 * Decompress DNS Name (RFC 1035)
 * @param buf Packet buffer (start of packet)
 * @param len Packet length
 * @param offset Current offset (updated to point after the name)
 * @param name Output buffer
 * @param name_size Size of output buffer
 * @return VEX_DNS_OK on success
 */
int vex_dns_parse_name(const uint8_t *buf, size_t len, size_t *offset,
                       char *name, size_t name_size);

#ifdef __cplusplus
}
#endif

#endif /* VEX_DNS_PARSER_H */
