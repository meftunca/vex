/*
 * icmp_parser.c - ICMP Protocol Parser
 */

#include "protocols/icmp_parser.h"

/* Helper: Read 16-bit big-endian */
static inline uint16_t read_be16(const uint8_t *p) {
    return (uint16_t)((uint16_t)p[0] << 8 | (uint16_t)p[1]);
}

int vex_icmp_parse(const uint8_t *buf, size_t len, vex_icmp_packet_t *pkt) {
    if (len < 8) {
        return VEX_ICMP_ERR_TRUNCATED;
    }

    pkt->type     = buf[0];
    pkt->code     = buf[1];
    pkt->checksum = read_be16(buf + 2);
    
    /* Verify checksum */
    if (vex_icmp_checksum(buf, len) != 0) {
        return VEX_ICMP_ERR_CHECKSUM;
    }

    if (pkt->type == VEX_ICMP_TYPE_ECHO_REQUEST || 
        pkt->type == VEX_ICMP_TYPE_ECHO_REPLY) {
        pkt->id       = read_be16(buf + 4);
        pkt->sequence = read_be16(buf + 6);
    } else {
        pkt->id       = 0;
        pkt->sequence = 0;
    }

    pkt->data     = buf + 8;
    pkt->data_len = len - 8;

    return VEX_ICMP_OK;
}

uint16_t vex_icmp_checksum(const uint8_t *buf, size_t len) {
    uint32_t sum = 0;
    const uint16_t *p = (const uint16_t *)buf;

    while (len > 1) {
        sum += *p++;
        len -= 2;
    }

    if (len > 0) {
        sum += *(const uint8_t *)p;
    }

    while (sum >> 16) {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }

    return (uint16_t)~sum;
}
