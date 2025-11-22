#include <stdio.h>
#include <string.h>
#include <stdlib.h>

#include "protocols/simd_utils.h"
#include "protocols/http_parser.h"
#include "protocols/http2_parser.h"
#include "protocols/udp_parser.h"
#include "protocols/websocket_parser.h"
#include "protocols/dns_parser.h"
#include "protocols/tls_detector.h"
#include "protocols/icmp_parser.h"

void print_hex(const uint8_t *buf, size_t len) {
    for (size_t i = 0; i < len; ++i) {
        printf("%02x ", buf[i]);
    }
    printf("\n");
}

int main(void) {
    printf("=== VEX_NET Protocol Parsers Demo ===\n");
    printf("SIMD Backend: %s\n\n", vex_simd_backend());

    /* --- SIMD Tests --- */
    printf("[SIMD Utils]\n");
    const char *simd_test = "Hello, World! This is a test.";
    size_t idx;
    
    /* Find char */
    idx = vex_simd_find_char(simd_test, strlen(simd_test), 'W');
    printf("✓ Find 'W': %zu (Expected: 7)\n", idx);
    
    /* Find Set 2 */
    idx = vex_simd_find_set2(simd_test, strlen(simd_test), '!', '.');
    printf("✓ Find '!' or '.': %zu (Expected: 12)\n", idx);
    
    /* Find Set 4 */
    idx = vex_simd_find_set4(simd_test, strlen(simd_test), 'z', 'x', 'y', 'T');
    printf("✓ Find 'z','x','y','T': %zu (Expected: 14)\n", idx);
    printf("\n");

    /* --- HTTP/1.1 --- */
    printf("[HTTP/1.1]\n");
    const char *http_req = "GET /api/v1/users?id=123 HTTP/1.1\r\nHost: example.com\r\n\r\n";
    vex_http_request_t req;
    if (vex_http_parse(http_req, strlen(http_req), &req) == VEX_HTTP_OK) {
        printf("✓ Parsed: %.*s %.*s\n", 
               (int)req.request_line.method_len, req.request_line.method,
               (int)req.request_line.uri_len, req.request_line.uri);
    } else {
        printf("✗ Failed to parse HTTP/1.1\n");
    }
    printf("\n");

    /* --- HTTP/2 --- */
    printf("[HTTP/2]\n");
    const char *h2_preface = "PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";
    if (vex_http2_is_preface((const uint8_t*)h2_preface, 24)) {
        printf("✓ Client preface detected\n");
    }
    printf("\n");

    /* --- WebSocket --- */
    printf("[WebSocket]\n");
    /* Fin, Text, Masked, Len=5, MaskKey, "Hello" */
    uint8_t ws_frame[] = {
        0x81, 0x85, 0x37, 0xfa, 0x21, 0x3d, 0x7f, 0x9f, 0x4d, 0x51, 0x58
    };
    vex_ws_frame_t ws;
    size_t consumed;
    if (vex_ws_parse_frame(ws_frame, sizeof(ws_frame), &ws, &consumed) == VEX_WS_OK) {
        printf("✓ Frame parsed: Opcode=%d, Len=%llu, Masked=%d\n", 
               ws.opcode, (unsigned long long)ws.payload_len, ws.masked);
        
        if (ws.masked) {
            /* Copy payload to unmask safely */
            uint8_t payload[32];
            memcpy(payload, ws.payload, ws.payload_len);
            vex_ws_unmask_payload(payload, ws.payload_len, ws.mask_key);
            printf("  Payload: %.*s\n", (int)ws.payload_len, payload);
        }
    }
    printf("\n");

    /* --- DNS --- */
    printf("[DNS]\n");
    /* Standard query for example.com (A record) */
    uint8_t dns_query[] = {
        0x12, 0x34, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x07, 'e', 'x', 'a', 'm', 'p', 'l', 'e', 0x03, 'c', 'o', 'm', 0x00,
        0x00, 0x01, 0x00, 0x01
    };
    vex_dns_header_t dns_hdr;
    if (vex_dns_parse_header(dns_query, sizeof(dns_query), &dns_hdr) == VEX_DNS_OK) {
        printf("✓ Header parsed: ID=0x%04x, Q=%d\n", dns_hdr.id, dns_hdr.qdcount);
        
        size_t offset = 12;
        vex_dns_question_t q;
        if (vex_dns_parse_question(dns_query, sizeof(dns_query), &offset, &q) == VEX_DNS_OK) {
            printf("  Question: %s (Type=%d)\n", q.name, q.type);
        }
    }
    printf("\n");

    /* --- TLS Detection --- */
    printf("[TLS]\n");
    /* ClientHello (TLS 1.2) with SNI extension for "example.com" */
    uint8_t tls_hello[] = {
        0x16, 0x03, 0x01, 0x00, 0x36, /* Record Header */
        0x01, 0x00, 0x00, 0x32,       /* Handshake Header */
        0x03, 0x03,                   /* Version 1.2 */
        0x00,0x01,0x02,0x03,0x04,0x05,0x06,0x07,0x08,0x09,0x0a,0x0b,0x0c,0x0d,0x0e,0x0f,
        0x10,0x11,0x12,0x13,0x14,0x15,0x16,0x17,0x18,0x19,0x1a,0x1b,0x1c,0x1d,0x1e,0x1f, /* Random */
        0x00,                         /* Session ID Len */
        0x00, 0x02, 0x00, 0x2f,       /* Cipher Suites */
        0x01, 0x00,                   /* Compression */
        0x00, 0x0b,                   /* Extensions Len */
        0x00, 0x00, 0x00, 0x07,       /* SNI Extension */
        0x00, 0x05, 0x00, 0x00, 0x02, 'h', '2' /* SNI Data (fake for brevity) */
    };
    
    if (vex_tls_is_handshake(tls_hello, sizeof(tls_hello))) {
        printf("✓ TLS Handshake detected\n");
        vex_tls_client_hello_t hello;
        /* Note: The fake packet above might fail full parsing due to length checks, 
           but let's try basic detection */
        if (vex_tls_parse_client_hello(tls_hello, sizeof(tls_hello), &hello) == VEX_TLS_OK) {
             printf("  SNI: %s\n", hello.has_sni ? hello.sni : "(none)");
        } else {
             printf("  (Partial parse - simplified packet)\n");
        }
    }
    printf("\n");

    /* --- ICMP --- */
    printf("[ICMP]\n");
    /* Echo Request */
    uint8_t icmp_pkt[] = {
        0x08, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x05,
        'P', 'i', 'n', 'g'
    };
    /* Calculate correct checksum */
    uint16_t cksum = vex_icmp_checksum(icmp_pkt, sizeof(icmp_pkt));
    *(uint16_t*)(icmp_pkt + 2) = cksum;

    vex_icmp_packet_t icmp;
    if (vex_icmp_parse(icmp_pkt, sizeof(icmp_pkt), &icmp) == VEX_ICMP_OK) {
        printf("✓ Echo Request parsed: ID=%d, Seq=%d\n", icmp.id, icmp.sequence);
        printf("  Data: %.*s\n", (int)icmp.data_len, icmp.data);
    } else {
        printf("✗ Failed to parse ICMP (Checksum error?)\n");
    }
    printf("\n");

    printf("All parsers ready!\n");
    return 0;
}
