#ifndef VEX_ICMP_PARSER_H
#define VEX_ICMP_PARSER_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ICMP Types */
#define VEX_ICMP_TYPE_ECHO_REPLY      0
#define VEX_ICMP_TYPE_DEST_UNREACH    3
#define VEX_ICMP_TYPE_ECHO_REQUEST    8
#define VEX_ICMP_TYPE_TIME_EXCEEDED   11

/* ICMP Codes (Dest Unreach) */
#define VEX_ICMP_CODE_NET_UNREACH     0
#define VEX_ICMP_CODE_HOST_UNREACH    1
#define VEX_ICMP_CODE_PROTO_UNREACH   2
#define VEX_ICMP_CODE_PORT_UNREACH    3

/* ICMP Packet */
typedef struct {
    uint8_t  type;
    uint8_t  code;
    uint16_t checksum;
    uint16_t id;        /* Only for Echo */
    uint16_t sequence;  /* Only for Echo */
    const uint8_t *data;
    size_t   data_len;
} vex_icmp_packet_t;

/* Error Codes */
#define VEX_ICMP_OK              0
#define VEX_ICMP_ERR_TRUNCATED  -1
#define VEX_ICMP_ERR_CHECKSUM   -2

/**
 * Parse ICMP Packet
 * @param buf Packet buffer
 * @param len Packet length
 * @param pkt Output structure
 * @return VEX_ICMP_OK on success
 */
int vex_icmp_parse(const uint8_t *buf, size_t len, vex_icmp_packet_t *pkt);

/**
 * Calculate Internet Checksum
 * @param buf Data buffer
 * @param len Data length
 * @return 16-bit checksum
 */
uint16_t vex_icmp_checksum(const uint8_t *buf, size_t len);

#ifdef __cplusplus
}
#endif

#endif /* VEX_ICMP_PARSER_H */
