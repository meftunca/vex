/*
 * udp_parser.c - UDP over IPv4/IPv6 packet parser
 */

#include "protocols/udp_parser.h"
#include <string.h>
#include <stdio.h>

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

int vex_udp_parse_ipv4(const uint8_t *buf, size_t len, vex_udp4_packet_t *out) {
    /* Minimum: 20 byte IP header + 8 byte UDP header */
    if (len < 28) {
        return -1; /* Truncated */
    }

    /* Parse IP header */
    uint8_t vihl    = buf[0];
    uint8_t version = (uint8_t)(vihl >> 4);
    uint8_t ihl     = (uint8_t)(vihl & 0x0F);

    if (version != 4) {
        return -2; /* Bad IP version */
    }
    if (ihl < 5) {
        return -2; /* Invalid IHL */
    }

    uint8_t ip_header_len = (uint8_t)(ihl * 4);
    if (len < (size_t)ip_header_len + 8u) {
        return -1; /* Truncated */
    }

    uint16_t total_length = read_be16(buf + 2);
    if (total_length < (uint16_t)(ip_header_len + 8)) {
        return -2; /* Bad total length */
    }
    if (len < (size_t)total_length) {
        return -1; /* Truncated */
    }

    uint8_t protocol = buf[9];
    if (protocol != 17) {
        return -4; /* Not UDP */
    }

    out->version   = version;
    out->ihl_bytes = ip_header_len;
    out->protocol  = protocol;
    out->src_ip    = read_be32(buf + 12);
    out->dst_ip    = read_be32(buf + 16);

    /* Parse UDP header */
    const uint8_t *udp = buf + ip_header_len;
    uint16_t sport  = read_be16(udp + 0);
    uint16_t dport  = read_be16(udp + 2);
    uint16_t udplen = read_be16(udp + 4);

    if (udplen < 8) {
        return -2; /* Invalid UDP length */
    }
    if ((size_t)ip_header_len + udplen > len) {
        return -1; /* Truncated */
    }

    out->src_port    = sport;
    out->dst_port    = dport;
    out->length      = udplen;
    out->payload     = udp + 8;
    out->payload_len = (uint16_t)(udplen - 8);

    return 0; /* Success */
}

int vex_udp_parse_ipv6(const uint8_t *buf, size_t len, vex_udp6_packet_t *out) {
    /* Minimum: 40 byte IPv6 header + 8 byte UDP header */
    if (len < 48) {
        return -1; /* Truncated */
    }

    uint8_t version = (uint8_t)(buf[0] >> 4);
    if (version != 6) {
        return -2; /* Bad IP version */
    }

    uint16_t payload_len = read_be16(buf + 4);
    uint8_t  next_header = buf[6];

    if (next_header != 17) {
        return -4; /* Not UDP (or has extension headers) */
    }

    size_t total_len = 40u + (size_t)payload_len;
    if (len < total_len) {
        return -1; /* Truncated */
    }

    memcpy(out->src_ip, buf + 8,  16);
    memcpy(out->dst_ip, buf + 24, 16);
    out->version  = 6;
    out->protocol = next_header;

    /* Parse UDP header */
    const uint8_t *udp = buf + 40;
    uint16_t sport  = read_be16(udp + 0);
    uint16_t dport  = read_be16(udp + 2);
    uint16_t udplen = read_be16(udp + 4);

    if (udplen < 8) {
        return -2; /* Invalid UDP length */
    }
    if (40u + udplen > len) {
        return -1; /* Truncated */
    }

    out->src_port    = sport;
    out->dst_port    = dport;
    out->length      = udplen;
    out->payload     = udp + 8;
    out->payload_len = (uint16_t)(udplen - 8);

    return 0; /* Success */
}

void vex_ipv4_to_str(uint32_t ip_net_order, char *buf16) {
    unsigned char b0 = (unsigned char)((ip_net_order >> 24) & 0xFF);
    unsigned char b1 = (unsigned char)((ip_net_order >> 16) & 0xFF);
    unsigned char b2 = (unsigned char)((ip_net_order >> 8)  & 0xFF);
    unsigned char b3 = (unsigned char)(ip_net_order & 0xFF);
    
    sprintf(buf16, "%u.%u.%u.%u",
            (unsigned)b0, (unsigned)b1, (unsigned)b2, (unsigned)b3);
}

void vex_ipv6_to_str(const uint8_t ip[16], char *buf64) {
    sprintf(buf64,
            "%02x%02x:%02x%02x:%02x%02x:%02x%02x:"
            "%02x%02x:%02x%02x:%02x%02x:%02x%02x",
            ip[0], ip[1], ip[2], ip[3],
            ip[4], ip[5], ip[6], ip[7],
            ip[8], ip[9], ip[10], ip[11],
            ip[12], ip[13], ip[14], ip[15]);
}
