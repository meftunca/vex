#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <ctype.h>

#include "protocols/http_parser.h"
#include "protocols/http2_parser.h"
#include "protocols/websocket_parser.h"
#include "protocols/dns_parser.h"
#include "protocols/tls_detector.h"
#include "protocols/icmp_parser.h"
#include "protocols/simd_utils.h"

#define MAX_BUFFER 65536

/* --- Utils --- */

/* Convert hex string to binary */
size_t hex_to_bin(const char *hex, uint8_t *bin, size_t max_len) {
    size_t len = 0;
    while (*hex && len < max_len) {
        if (isspace(*hex)) { hex++; continue; }
        char high = *hex++;
        char low = *hex++;
        if (!low) break;
        
        uint8_t val = 0;
        if (high >= '0' && high <= '9') val = (high - '0') << 4;
        else if (high >= 'a' && high <= 'f') val = (high - 'a' + 10) << 4;
        else if (high >= 'A' && high <= 'F') val = (high - 'A' + 10) << 4;
        
        if (low >= '0' && low <= '9') val |= (low - '0');
        else if (low >= 'a' && low <= 'f') val |= (low - 'a' + 10);
        else if (low >= 'A' && low <= 'F') val |= (low - 'A' + 10);
        
        bin[len++] = val;
    }
    return len;
}

/* Read file line by line and process chunks */
void run_test(const char *filename, const char *name, int is_hex,
              int (*test_func)(const uint8_t *, size_t)) 
{
    FILE *f = fopen(filename, "r");
    if (!f) {
        printf("Skipping %s (not found)\n", name);
        return;
    }

    char line[MAX_BUFFER];
    uint8_t buffer[MAX_BUFFER];
    size_t buf_len = 0;
    int in_block = 0;
    
    int total = 0;
    int passed = 0;
    clock_t start = clock();

    while (fgets(line, sizeof(line), f)) {
        if (strncmp(line, "---BEGIN---", 11) == 0) {
            in_block = 1;
            buf_len = 0;
            continue;
        }
        if (strncmp(line, "---END---", 9) == 0) {
            if (in_block) {
                total++;
                if (test_func(buffer, buf_len) == 0) {
                    passed++;
                }
            }
            in_block = 0;
            continue;
        }
        
        if (in_block) {
            if (is_hex) {
                buf_len = hex_to_bin(line, buffer, sizeof(buffer));
            } else {
                size_t l = strlen(line);
                if (buf_len + l < sizeof(buffer)) {
                    memcpy(buffer + buf_len, line, l);
                    buf_len += l;
                }
            }
        }
    }
    
    fclose(f);
    clock_t end = clock();
    double time_ms = (double)(end - start) * 1000.0 / CLOCKS_PER_SEC;
    
    printf("[%s] %d/%d passed (%.2f ms, %.2f req/ms)\n", 
           name, passed, total, time_ms, total / (time_ms > 0 ? time_ms : 1));
}

/* --- Test Functions --- */

int test_http1(const uint8_t *buf, size_t len) {
    vex_http_request_t req;
    /* Cast to char* as HTTP parser expects char* */
    return vex_http_parse((const char*)buf, len, &req) == VEX_HTTP_OK ? 0 : -1;
}

int test_http2(const uint8_t *buf, size_t len) {
    /* Check for preface or frame header */
    if (vex_http2_is_preface(buf, len)) return 0;
    
    vex_http2_frame_header_t frame;
    /* Skip preface if present for frame test */
    if (len > 24 && vex_http2_is_preface(buf, 24)) {
        return vex_http2_parse_frame_header(buf + 24, len - 24, &frame);
    }
    return vex_http2_parse_frame_header(buf, len, &frame);
}

int test_websocket(const uint8_t *buf, size_t len) {
    vex_ws_frame_t frame;
    size_t consumed;
    int ret = vex_ws_parse_frame(buf, len, &frame, &consumed);
    if (ret == VEX_WS_OK && frame.masked) {
        /* Test unmasking too */
        uint8_t payload_copy[4096];
        size_t copy_len = frame.payload_len > sizeof(payload_copy) ? sizeof(payload_copy) : frame.payload_len;
        memcpy(payload_copy, frame.payload, copy_len);
        vex_ws_unmask_payload(payload_copy, copy_len, frame.mask_key);
    }
    return ret;
}

int test_dns(const uint8_t *buf, size_t len) {
    vex_dns_header_t hdr;
    if (vex_dns_parse_header(buf, len, &hdr) != VEX_DNS_OK) return -1;
    
    /* Try to parse first question */
    if (hdr.qdcount > 0) {
        size_t offset = 12;
        vex_dns_question_t q;
        return vex_dns_parse_question(buf, len, &offset, &q);
    }
    return 0;
}

int test_tls(const uint8_t *buf, size_t len) {
    if (!vex_tls_is_handshake(buf, len)) return -1;
    
    vex_tls_client_hello_t hello;
    /* We expect success or truncated (since mock might be partial) */
    int ret = vex_tls_parse_client_hello(buf, len, &hello);
    return (ret == VEX_TLS_OK || ret == VEX_TLS_ERR_TRUNCATED) ? 0 : -1;
}

int test_icmp(const uint8_t *buf, size_t len) {
    vex_icmp_packet_t pkt;
    return vex_icmp_parse(buf, len, &pkt);
}

/* --- Main --- */

int main(void) {
    printf("=== VEX_NET Protocol Test Runner ===\n");
    printf("SIMD Backend: %s\n\n", vex_simd_backend());
    
    run_test("tests/protocols/http1.mock.txt", "HTTP/1.1", 0, test_http1);
    run_test("tests/protocols/http2.mock.txt", "HTTP/2", 1, test_http2);
    run_test("tests/protocols/websocket.mock.txt", "WebSocket", 1, test_websocket);
    run_test("tests/protocols/dns.mock.txt", "DNS", 1, test_dns);
    run_test("tests/protocols/tls.mock.txt", "TLS", 1, test_tls);
    run_test("tests/protocols/icmp.mock.txt", "ICMP", 1, test_icmp);
    
    return 0;
}
