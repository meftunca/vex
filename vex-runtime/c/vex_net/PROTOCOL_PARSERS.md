# VEX_NET Protocol Parsers

High-performance, modular, zero-copy protocol parsers for `vex_net`.

## Features

- **Zero-Copy**: All parsers return pointers into the original buffer.
- **Modular**: Use only what you need.
- **SIMD-Accelerated**: AVX-512, AVX2, SSE2, NEON support.
- **No Dependencies**: Only standard C library.

## Modules

### 1. HTTP/1.x (`http_parser.h`)
- Streaming and One-shot parsing
- Chunked Transfer Encoding support
- Pipelining support
- Header limit protection

### 2. HTTP/2 (`http2_parser.h`, `hpack.h`)
- Client Preface detection
- Frame Header parsing
- **HPACK Compression** (RFC 7541)
  - Static Table (61 entries)
  - Dynamic Table management
  - Integer encoding/decoding
  - *Note: Huffman coding in progress*

### 3. WebSocket (`websocket_parser.h`)
- Frame parsing (RFC 6455)
- Masking/Unmasking (SIMD-ready structure)
- Control frames (Ping, Pong, Close)
- Payload length handling (7-bit, 16-bit, 64-bit)

### 4. DNS (`dns_parser.h`)
- Query and Response parsing (RFC 1035)
- **Name Decompression** (pointer `0xC0` handling)
- Resource Record parsing (A, AAAA, MX, etc.)
- Loop detection for compressed names

### 5. TLS Detection (`tls_detector.h`)
- TLS Handshake detection (ClientHello)
- **SNI Extraction** (Server Name Indication)
- **ALPN Extraction** (Application-Layer Protocol Negotiation)
- Version detection (TLS 1.0 - 1.3)

### 6. ICMP (`icmp_parser.h`)
- Echo Request/Reply (Ping) parsing
- Checksum calculation/verification
- Destination Unreachable handling

### 7. UDP (`udp_parser.h`)
- IPv4 and IPv6 header parsing
- IP address string conversion

---

## Usage Examples

### HTTP/1.1
```c
vex_http_request_t req;
if (vex_http_parse(buf, len, &req) == VEX_HTTP_OK) {
    // Handle request
}
```

### WebSocket
```c
vex_ws_frame_t frame;
size_t consumed;
if (vex_ws_parse_frame(buf, len, &frame, &consumed) == VEX_WS_OK) {
    if (frame.masked) {
        vex_ws_unmask_payload(frame.payload, frame.payload_len, frame.mask_key);
    }
}
```

### DNS
```c
vex_dns_header_t hdr;
vex_dns_parse_header(buf, len, &hdr);

size_t offset = 12;
vex_dns_question_t q;
vex_dns_parse_question(buf, len, &offset, &q);
printf("Query: %s\n", q.name);
```

### TLS SNI
```c
if (vex_tls_is_handshake(buf, len)) {
    vex_tls_client_hello_t hello;
    vex_tls_parse_client_hello(buf, len, &hello);
    if (hello.has_sni) {
        printf("SNI: %s\n", hello.sni);
    }
}
```

## Build

These files are integrated into `libvexnet.a`.

```makefile
# Makefile
SRCS_PROTOCOLS := src/protocols/simd_utils.c \
                   src/protocols/http_parser.c \
                   src/protocols/http2_parser.c \
                   src/protocols/hpack.c \
                   src/protocols/websocket_parser.c \
                   src/protocols/dns_parser.c \
                   src/protocols/tls_detector.c \
                   src/protocols/icmp_parser.c \
                   src/protocols/udp_parser.c
```

## Performance

- **Zero Allocations**: Parsers (except HPACK dynamic table) do not allocate memory.
- **SIMD**: `simd_utils` accelerates character searches (used in HTTP).
- **In-Place**: WebSocket unmasking and Chunked decoding can be done in-place.

## Limitations

- **HTTP/2**: Full stream state machine and flow control not yet implemented.
- **HPACK**: Huffman coding pending.
- **TLS**: Detection only, no decryption (use OpenSSL/BoringSSL).
