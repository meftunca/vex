# UTF-8/UTF-16 & Networking (Socket/HTTP/WebSocket) Integration Plan

## 📅 Plan Tarihi: 2 Kasım 2025

## 🎯 Hedef

Vex diline UTF-8/UTF-16 encoding desteği ve tam kapsamlı networking stack (TCP/UDP sockets, HTTP, WebSocket) entegre etmek.

---

## 📦 Paket 1: std::encoding - Unicode & Text Encoding

### Sorumluluklar

- UTF-8 ↔ UTF-16 ↔ UTF-32 dönüşümleri
- ASCII, Latin-1, ISO-8859-x desteği
- Base64, Hex encoding/decoding
- String validation ve normalization

### Dosya Yapısı

```
vex-libs/std/encoding/
├── mod.vx              # Main module, re-exports
├── utf8.vx             # UTF-8 utilities
├── utf16.vx            # UTF-16 utilities
├── utf32.vx            # UTF-32 utilities
├── base64.vx           # Base64 encoding/decoding
├── hex.vx              # Hex encoding/decoding
└── ascii.vx            # ASCII utilities
```

### API Tasarımı

#### utf8.vx

```vex
// UTF-8 validation
export fn is_valid_utf8(bytes: &[byte]) -> bool;

// UTF-8 string operations
export fn char_count(s: string) -> usize;  // Count Unicode chars (not bytes)
export fn char_at(s: string, index: usize) -> (char | error);
export fn chars(s: string) -> CharIterator;  // Iterator over Unicode chars

// UTF-8 ↔ UTF-16 conversion
export fn to_utf16(s: string) -> [u16];
export fn from_utf16(utf16: &[u16]) -> (string | error);

// UTF-8 ↔ UTF-32 conversion
export fn to_utf32(s: string) -> [u32];
export fn from_utf32(utf32: &[u32]) -> (string | error);

// Character encoding
export fn encode_char(c: char) -> [byte];  // char -> UTF-8 bytes
export fn decode_char(bytes: &[byte]) -> ((char, usize) | error);  // UTF-8 bytes -> char + bytes consumed
```

#### utf16.vx

```vex
// UTF-16 representation
export struct Utf16String {
    data: [u16],
}

// UTF-16 validation
export fn is_valid_utf16(data: &[u16]) -> bool;

// UTF-16 operations
export fn from_utf8(s: string) -> Utf16String;
export fn to_utf8(utf16: &Utf16String) -> (string | error);

// Windows interop (UTF-16 LE is Windows native)
export fn to_wide_string(s: string) -> [u16];  // For Windows API
export fn from_wide_string(wide: &[u16]) -> (string | error);
```

#### base64.vx

```vex
// Base64 encoding/decoding
export fn encode(data: &[byte]) -> string;
export fn decode(encoded: string) -> ([byte] | error);

// URL-safe Base64
export fn encode_url_safe(data: &[byte]) -> string;
export fn decode_url_safe(encoded: string) -> ([byte] | error);
```

#### hex.vx

```vex
// Hex encoding/decoding
export fn encode(data: &[byte]) -> string;
export fn decode(hex: string) -> ([byte] | error);

// Lowercase/uppercase options
export fn encode_lower(data: &[byte]) -> string;
export fn encode_upper(data: &[byte]) -> string;
```

---

## 📦 Paket 2: std::net - Low-level Networking (Sockets)

### Sorumluluklar

- TCP/UDP socket operations
- Socket address handling (IPv4/IPv6)
- Low-level socket options
- Non-blocking I/O support

### Dosya Yapısı

```
vex-libs/std/net/
├── mod.vx              # Main module, re-exports
├── tcp.vx              # TCP operations
├── udp.vx              # UDP operations
├── addr.vx             # Address types (SocketAddr, IpAddr)
└── socket.vx           # Low-level socket operations
```

### FFI Eklentileri (ffi/libc.vx)

```vex
// Socket syscalls
extern "C" fn socket(domain: i32, type: i32, protocol: i32) -> i32;
extern "C" fn bind(sockfd: i32, addr: *sockaddr, addrlen: u32) -> i32;
extern "C" fn listen(sockfd: i32, backlog: i32) -> i32;
extern "C" fn accept(sockfd: i32, addr: *mut sockaddr, addrlen: *mut u32) -> i32;
extern "C" fn connect(sockfd: i32, addr: *sockaddr, addrlen: u32) -> i32;
extern "C" fn send(sockfd: i32, buf: *byte, len: usize, flags: i32) -> i64;
extern "C" fn recv(sockfd: i32, buf: *mut byte, len: usize, flags: i32) -> i64;
extern "C" fn sendto(sockfd: i32, buf: *byte, len: usize, flags: i32, dest_addr: *sockaddr, addrlen: u32) -> i64;
extern "C" fn recvfrom(sockfd: i32, buf: *mut byte, len: usize, flags: i32, src_addr: *mut sockaddr, addrlen: *mut u32) -> i64;
extern "C" fn shutdown(sockfd: i32, how: i32) -> i32;
extern "C" fn setsockopt(sockfd: i32, level: i32, optname: i32, optval: *byte, optlen: u32) -> i32;
extern "C" fn getsockopt(sockfd: i32, level: i32, optname: i32, optval: *mut byte, optlen: *mut u32) -> i32;
extern "C" fn getsockname(sockfd: i32, addr: *mut sockaddr, addrlen: *mut u32) -> i32;
extern "C" fn getpeername(sockfd: i32, addr: *mut sockaddr, addrlen: *mut u32) -> i32;

// Address resolution
extern "C" fn getaddrinfo(node: *byte, service: *byte, hints: *addrinfo, res: **mut addrinfo) -> i32;
extern "C" fn freeaddrinfo(res: *mut addrinfo);
extern "C" fn getnameinfo(addr: *sockaddr, addrlen: u32, host: *mut byte, hostlen: u32, serv: *mut byte, servlen: u32, flags: i32) -> i32;

// Network byte order
extern "C" fn htons(hostshort: u16) -> u16;
extern "C" fn htonl(hostlong: u32) -> u32;
extern "C" fn ntohs(netshort: u16) -> u16;
extern "C" fn ntohl(netlong: u32) -> u32;

// inet_pton/inet_ntop for IPv4/IPv6 conversion
extern "C" fn inet_pton(af: i32, src: *byte, dst: *mut byte) -> i32;
extern "C" fn inet_ntop(af: i32, src: *byte, dst: *mut byte, size: u32) -> *byte;

// Socket constants
export const AF_INET: i32 = 2;        // IPv4
export const AF_INET6: i32 = 10;      // IPv6
export const SOCK_STREAM: i32 = 1;    // TCP
export const SOCK_DGRAM: i32 = 2;     // UDP
export const SOCK_NONBLOCK: i32 = 2048; // Non-blocking
export const IPPROTO_TCP: i32 = 6;
export const IPPROTO_UDP: i32 = 17;
export const SOL_SOCKET: i32 = 1;
export const SO_REUSEADDR: i32 = 2;
export const SO_REUSEPORT: i32 = 15;
export const SO_KEEPALIVE: i32 = 9;
export const SO_RCVTIMEO: i32 = 20;
export const SO_SNDTIMEO: i32 = 21;
export const SHUT_RD: i32 = 0;
export const SHUT_WR: i32 = 1;
export const SHUT_RDWR: i32 = 2;
```

### API Tasarımı

#### addr.vx

```vex
// IP address (IPv4 or IPv6)
export enum IpAddr {
    V4(Ipv4Addr),
    V6(Ipv6Addr),
}

export struct Ipv4Addr {
    octets: [byte; 4],
}

export struct Ipv6Addr {
    segments: [u16; 8],
}

// Socket address (IP + Port)
export enum SocketAddr {
    V4(SocketAddrV4),
    V6(SocketAddrV6),
}

export struct SocketAddrV4 {
    ip: Ipv4Addr,
    port: u16,
}

export struct SocketAddrV6 {
    ip: Ipv6Addr,
    port: u16,
    flowinfo: u32,
    scope_id: u32,
}

// Constructors
export fn parse_ipv4(s: string) -> (Ipv4Addr | error);  // "127.0.0.1"
export fn parse_ipv6(s: string) -> (Ipv6Addr | error);  // "::1"
export fn parse_socket_addr(s: string) -> (SocketAddr | error);  // "127.0.0.1:8080"
```

#### tcp.vx

```vex
// TCP stream (connected socket)
export struct TcpStream {
    fd: i32,
    peer_addr: SocketAddr,
    local_addr: SocketAddr,
}

// Connect to TCP server
export fn connect(addr: SocketAddr) -> (TcpStream | error);
export fn connect_timeout(addr: SocketAddr, timeout: Duration) -> (TcpStream | error);

// TcpStream operations
export fn (s: &TcpStream) read(buf: &mut [byte]) -> (usize | error);
export fn (s: &TcpStream) write(buf: &[byte]) -> (usize | error);
export fn (s: &TcpStream) shutdown(how: Shutdown) -> (nil | error);  // Shutdown::Read, Write, Both
export fn (s: &mut TcpStream) close() -> (nil | error);

// Socket options
export fn (s: &TcpStream) set_nodelay(nodelay: bool) -> (nil | error);  // TCP_NODELAY (Nagle's algorithm)
export fn (s: &TcpStream) set_keepalive(keepalive: bool) -> (nil | error);
export fn (s: &TcpStream) set_read_timeout(timeout: Duration) -> (nil | error);
export fn (s: &TcpStream) set_write_timeout(timeout: Duration) -> (nil | error);

// TCP listener (server-side)
export struct TcpListener {
    fd: i32,
    local_addr: SocketAddr,
}

// Bind and listen
export fn bind(addr: SocketAddr) -> (TcpListener | error);

// Accept incoming connections
export fn (l: &TcpListener) accept() -> (TcpStream | error);
export fn (l: &TcpListener) incoming() -> IncomingIterator;  // Iterator over connections
```

#### udp.vx

```vex
// UDP socket
export struct UdpSocket {
    fd: i32,
    local_addr: SocketAddr,
}

// Bind UDP socket
export fn bind(addr: SocketAddr) -> (UdpSocket | error);

// Send/receive
export fn (s: &UdpSocket) send_to(buf: &[byte], addr: SocketAddr) -> (usize | error);
export fn (s: &UdpSocket) recv_from(buf: &mut [byte]) -> ((usize, SocketAddr) | error);

// Connect UDP socket (filters packets from specific peer)
export fn (s: &UdpSocket) connect(addr: SocketAddr) -> (nil | error);
export fn (s: &UdpSocket) send(buf: &[byte]) -> (usize | error);  // After connect()
export fn (s: &UdpSocket) recv(buf: &mut [byte]) -> (usize | error);  // After connect()
```

---

## 📦 Paket 3: std::http - HTTP Client & Server

### Sorumluluklar

- HTTP/1.1 client (GET, POST, PUT, DELETE, etc.)
- HTTP server with routing
- Header parsing and generation
- Content-Type handling
- Chunked transfer encoding
- Keep-alive connections

### Dosya Yapısı

```
vex-libs/std/http/
├── mod.vx              # Main module, re-exports
├── client.vx           # HTTP client
├── server.vx           # HTTP server
├── request.vx          # Request type
├── response.vx         # Response type
├── headers.vx          # Header utilities
├── method.vx           # HTTP methods enum
├── status.vx           # Status codes
└── router.vx           # URL routing for server
```

### API Tasarımı

#### client.vx

```vex
// HTTP client
export struct Client {
    timeout: Duration,
    user_agent: string,
    default_headers: HeaderMap,
}

// Create client
export fn new_client() -> Client;

// HTTP methods
export fn (c: &Client) get(url: string) -> (Response | error);
export fn (c: &Client) post(url: string, body: string, content_type: string) -> (Response | error);
export fn (c: &Client) put(url: string, body: string, content_type: string) -> (Response | error);
export fn (c: &Client) delete(url: string) -> (Response | error);
export fn (c: &Client) head(url: string) -> (Response | error);
export fn (c: &Client) patch(url: string, body: string, content_type: string) -> (Response | error);

// Custom request
export fn (c: &Client) request(req: Request) -> (Response | error);

// Builder pattern
export fn (c: &mut Client) set_timeout(timeout: Duration) -> &mut Client;
export fn (c: &mut Client) set_user_agent(ua: string) -> &mut Client;
export fn (c: &mut Client) add_default_header(key: string, value: string) -> &mut Client;
```

#### request.vx

```vex
// HTTP request
export struct Request {
    method: Method,
    url: string,
    headers: HeaderMap,
    body: string,
}

// HTTP methods
export enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    HEAD,
    OPTIONS,
    PATCH,
    TRACE,
    CONNECT,
}

// Builder pattern
export fn new_request(method: Method, url: string) -> Request;
export fn (r: &mut Request) header(key: string, value: string) -> &mut Request;
export fn (r: &mut Request) body(body: string) -> &mut Request;
```

#### response.vx

```vex
// HTTP response
export struct Response {
    status: StatusCode,
    headers: HeaderMap,
    body: string,
}

export struct StatusCode {
    code: u16,
    reason: string,
}

// Status code helpers
export fn (s: StatusCode) is_success() -> bool;  // 2xx
export fn (s: StatusCode) is_redirect() -> bool;  // 3xx
export fn (s: StatusCode) is_client_error() -> bool;  // 4xx
export fn (s: StatusCode) is_server_error() -> bool;  // 5xx

// Common status codes
export const OK: StatusCode = StatusCode { code: 200, reason: "OK" };
export const CREATED: StatusCode = StatusCode { code: 201, reason: "Created" };
export const NOT_FOUND: StatusCode = StatusCode { code: 404, reason: "Not Found" };
export const INTERNAL_SERVER_ERROR: StatusCode = StatusCode { code: 500, reason: "Internal Server Error" };
```

#### server.vx

```vex
// HTTP server
export struct Server {
    addr: SocketAddr,
    router: Router,
}

// Create server
export fn new_server(addr: SocketAddr) -> Server;

// Register routes
export fn (s: &mut Server) get(path: string, handler: Handler) -> &mut Server;
export fn (s: &mut Server) post(path: string, handler: Handler) -> &mut Server;
export fn (s: &mut Server) put(path: string, handler: Handler) -> &mut Server;
export fn (s: &mut Server) delete(path: string, handler: Handler) -> &mut Server;

// Handler type
export type Handler = fn(req: Request) -> Response;

// Start server (blocking)
export fn (s: &Server) listen() -> error;

// Example usage:
// let mut server = http::new_server("127.0.0.1:8080".parse()?);
// server.get("/", |req| {
//     Response { status: OK, headers: HeaderMap::new(), body: "Hello, World!" }
// });
// server.listen()?;
```

---

## 📦 Paket 4: std::websocket - WebSocket Support

### Sorumluluklar

- WebSocket handshake (RFC 6455)
- Frame encoding/decoding
- Message fragmentation
- Ping/Pong heartbeat
- Binary & text messages

### Dosya Yapısı

```
vex-libs/std/websocket/
├── mod.vx              # Main module
├── client.vx           # WebSocket client
├── server.vx           # WebSocket server
├── frame.vx            # Frame encoding/decoding
└── message.vx          # Message types
```

### API Tasarımı

#### client.vx

```vex
// WebSocket client
export struct WebSocket {
    stream: TcpStream,
    is_client: bool,
}

// Connect to WebSocket server
export fn connect(url: string) -> (WebSocket | error);

// Send message
export fn (ws: &WebSocket) send_text(message: string) -> (nil | error);
export fn (ws: &WebSocket) send_binary(data: &[byte]) -> (nil | error);

// Receive message
export fn (ws: &WebSocket) recv() -> (Message | error);

// Close connection
export fn (ws: &mut WebSocket) close(code: u16, reason: string) -> (nil | error);

// Ping/Pong
export fn (ws: &WebSocket) ping(data: &[byte]) -> (nil | error);
```

#### message.vx

```vex
// WebSocket message
export enum Message {
    Text(string),
    Binary([byte]),
    Ping([byte]),
    Pong([byte]),
    Close(u16, string),  // code, reason
}

// Opcode constants
export const OPCODE_CONTINUATION: byte = 0x0;
export const OPCODE_TEXT: byte = 0x1;
export const OPCODE_BINARY: byte = 0x2;
export const OPCODE_CLOSE: byte = 0x8;
export const OPCODE_PING: byte = 0x9;
export const OPCODE_PONG: byte = 0xA;
```

---

## 🔧 Implementation Order & Timeline

### Phase 1: UTF-8/UTF-16 Support (Week 1)

**Priority:** HIGH - Required for string handling in network protocols

1. ✅ Create `std::encoding` directory structure
2. ✅ Implement `utf8.vx`:
   - `is_valid_utf8()`, `char_count()`, `char_at()`
   - `to_utf16()`, `from_utf16()`
   - `encode_char()`, `decode_char()`
3. ✅ Implement `utf16.vx`:
   - `Utf16String` struct
   - `from_utf8()`, `to_utf8()`
   - Windows interop (`to_wide_string`, `from_wide_string`)
4. ✅ Implement `base64.vx`:
   - `encode()`, `decode()`
   - URL-safe variants
5. ✅ Implement `hex.vx`:
   - `encode()`, `decode()`
6. ✅ Add tests in `examples/encoding_test.vx`

### Phase 2: Low-level Sockets (Week 2)

**Priority:** HIGH - Foundation for all networking

1. ✅ Extend `ffi/libc.vx` with socket syscalls:
   - `socket()`, `bind()`, `listen()`, `accept()`, `connect()`
   - `send()`, `recv()`, `sendto()`, `recvfrom()`
   - `setsockopt()`, `getsockopt()`
   - `getaddrinfo()`, `freeaddrinfo()`
   - `inet_pton()`, `inet_ntop()`
   - All socket constants (AF_INET, SOCK_STREAM, etc.)
2. ✅ Create `std::net` directory structure
3. ✅ Implement `addr.vx`:
   - `IpAddr`, `Ipv4Addr`, `Ipv6Addr`
   - `SocketAddr`, `SocketAddrV4`, `SocketAddrV6`
   - Parsing functions
4. ✅ Implement `tcp.vx`:
   - `TcpStream` with connect/read/write
   - `TcpListener` with bind/accept
   - Socket options (nodelay, keepalive, timeouts)
5. ✅ Implement `udp.vx`:
   - `UdpSocket` with bind/send_to/recv_from
6. ✅ Add tests in `examples/tcp_echo_server.vx`, `examples/udp_test.vx`

### Phase 3: HTTP Support (Week 3)

**Priority:** MEDIUM - Common use case

1. ✅ Create `std::http` directory structure
2. ✅ Implement `request.vx` and `response.vx`:
   - `Request`, `Response` structs
   - `Method` enum
   - `StatusCode` with helpers
3. ✅ Implement `headers.vx`:
   - `HeaderMap` for storing headers
   - Common header constants
4. ✅ Implement `client.vx`:
   - `Client` struct with timeout/user-agent
   - GET, POST, PUT, DELETE methods
   - Builder pattern for custom requests
5. ✅ Implement `server.vx`:
   - `Server` struct with routing
   - Request parsing
   - Response generation
6. ✅ Implement `router.vx`:
   - URL pattern matching
   - Path parameters
7. ✅ Add tests in `examples/http_server.vx`, `examples/http_client_test.vx`

### Phase 4: WebSocket Support (Week 4)

**Priority:** LOW - Advanced feature

1. ✅ Create `std::websocket` directory structure
2. ✅ Implement `frame.vx`:
   - WebSocket frame encoding/decoding
   - Masking for client frames
3. ✅ Implement `message.vx`:
   - `Message` enum (Text, Binary, Ping, Pong, Close)
4. ✅ Implement `client.vx`:
   - WebSocket handshake
   - `connect()`, `send_text()`, `send_binary()`, `recv()`
   - Ping/Pong heartbeat
5. ✅ Implement `server.vx`:
   - Accept WebSocket upgrades
   - Handle multiple clients
6. ✅ Add tests in `examples/websocket_echo_server.vx`, `examples/websocket_client_test.vx`

---

## ✅ Success Criteria

### UTF-8/UTF-16

- ✅ Full Unicode support (all valid UTF-8/UTF-16 strings)
- ✅ Zero-cost abstractions (compile to efficient LLVM)
- ✅ Windows interop (UTF-16 for Windows API)
- ✅ Base64/Hex encoding for network protocols

### Sockets

- ✅ TCP client & server working
- ✅ UDP send/receive working
- ✅ IPv4 & IPv6 support
- ✅ Non-blocking I/O support
- ✅ Socket options (keepalive, timeout, etc.)

### HTTP

- ✅ HTTP client can fetch web pages
- ✅ HTTP server can serve requests
- ✅ Routing with path parameters
- ✅ Header parsing/generation
- ✅ Keep-alive connections

### WebSocket

- ✅ WebSocket client can connect and send/receive
- ✅ WebSocket server can handle multiple clients
- ✅ Ping/Pong heartbeat working
- ✅ Binary & text messages supported
- ✅ RFC 6455 compliant

---

## 📝 Notes

### Platform Considerations

- **Unix/Linux:** Use POSIX sockets (socket, bind, connect, etc.)
- **Windows:** Use Winsock2 (WSAStartup required, different error codes)
- **macOS:** Same as Unix but check kqueue for async I/O
- Use `#[cfg(unix)]` and `#[cfg(windows)]` for platform-specific code

### Dependencies

- **FFI:** All socket operations go through `ffi/libc.vx`
- **std::io:** File operations share `read()`/`write()` interface with sockets
- **std::time:** Timeouts use `Duration` from `std::time`
- **std::sync:** Thread-safe operations use `Mutex` from `std::sync`

### Security Considerations

- **TLS/SSL:** Future work - requires OpenSSL/BoringSSL bindings
- **Certificate validation:** Future work
- **WebSocket masking:** Required for client frames (RFC 6455)
- **Input validation:** All user input must be validated

### Performance Considerations

- **Zero-copy:** Use slices and references instead of copying
- **Buffer pooling:** Reuse buffers for network I/O
- **Async I/O:** Future work - requires tokio-like runtime
- **Connection pooling:** For HTTP client (keep-alive)

---

## 🚀 Let's Start!

**Current Status:** Planning complete
**Next Step:** Phase 1 - Implement UTF-8/UTF-16 support

Start with:

```bash
mkdir -p vex-libs/std/encoding
touch vex-libs/std/encoding/{mod.vx,utf8.vx,utf16.vx,base64.vx,hex.vx,ascii.vx}
```
