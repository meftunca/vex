#ifndef VEX_UDP_PARSER_H
#define VEX_UDP_PARSER_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* UDP over IPv4 packet Representation */
typedef struct {
    uint8_t       version;      /* IP version (4) */
    uint8_t       ihl_bytes;    /* IP header length in bytes */
    uint8_t       protocol;     /* Protocol (17 = UDP) */
    uint32_t      src_ip;       /* Source IP (network byte order) */
    uint32_t      dst_ip;       /* Destination IP (network byte order) */
    uint16_t      src_port;     /* Source port (host byte order) */
    uint16_t      dst_port;     /* Destination port (host byte order) */
    uint16_t      length;       /* UDP length (host byte order) */
    const uint8_t *payload;     /* UDP payload pointer */
    uint16_t      payload_len;  /* Payload length */
} vex_udp4_packet_t;

/* UDP over IPv6 packet (no extension headers) */
typedef struct {
    uint8_t       version;      /* IP version (6) */
    uint8_t       protocol;     /* Next header/protocol (17 = UDP) */
    uint8_t       src_ip[16];   /* Source IPv6 address */
    uint8_t       dst_ip[16];   /* Destination IPv6 address */
    uint16_t      src_port;     /* Source port (host byte order) */
    uint16_t      dst_port;     /* Destination port (host byte order) */
    uint16_t      length;       /* UDP length (host byte order) */
    const uint8_t *payload;     /* UDP payload pointer */
    uint16_t      payload_len;  /* Payload length */
} vex_udp6_packet_t;

/**
 * Parse UDP over IPv4 packet
 * 
 * @param buf Raw packet buffer
 * @param len Buffer length
 * @param out Output packet structure
 * @return 0 on success, -1 if truncated, -2 if bad IP, -4 if not UDP
 */
int vex_udp_parse_ipv4(const uint8_t *buf, size_t len, vex_udp4_packet_t *out);

/**
 * Parse UDP over IPv6 packet (assumes no extension headers)
 * 
 * @param buf Raw packet buffer
 * @param len Buffer length
 * @param out Output packet structure
 * @return 0 on success, -1 if truncated, -2 if bad IP, -4 if not UDP
 */
int vex_udp_parse_ipv6(const uint8_t *buf, size_t len, vex_udp6_packet_t *out);

/**
 * Convert IPv4 address to string ("x.x.x.x")
 * 
 * @param ip_net_order IPv4 address in network byte order
 * @param buf16 Output buffer (must be at least 16 bytes)
 */
void vex_ipv4_to_str(uint32_t ip_net_order, char *buf16);

/**
 * Convert IPv6 address to string ("xxxx:xxxx:...:xxxx")
 * 
 * @param ip IPv6 address (16 bytes)
 * @param buf64 Output buffer (must be at least 64 bytes)
 */
void vex_ipv6_to_str(const uint8_t ip[16], char *buf64);

#ifdef __cplusplus
}
#endif

#endif /* VEX_UDP_PARSER_H */
