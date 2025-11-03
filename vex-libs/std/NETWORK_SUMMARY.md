# Vex Standard Library - Network Packages

## Overview
Complete implementation of networking capabilities for the Vex standard library, including both TCP/UDP socket operations and HTTP client/server functionality.

## Package: std::net

**Location**: `vex-libs/std/net/mod.vx` (340 lines)

### TCP Support
- **TcpStream**: Client-side TCP connection
  - `connect(host: string, port: u16)`: Connect to remote host
  - `read(buffer: *byte, size: usize)`: Read data from socket
  - `write(data: *byte, size: usize)`: Write data to socket
  - `close()`: Close the connection

- **TcpListener**: Server-side TCP listener
  - `listen(addr: string, port: u16)`: Bind and listen on address
  - `accept()`: Accept incoming connections
  - `close()`: Close the listener

### UDP Support
- **UdpSocket**: UDP socket operations
  - `bind_udp(addr: string, port: u16)`: Bind UDP socket
  - `recv_from(buffer: *byte, size: usize)`: Receive packet with sender info
  - `send_to(data: *byte, size: usize, addr: string, port: u16)`: Send packet to address
  - `close()`: Close the socket

### Implementation Details
- Uses libc socket syscalls (socket, connect, bind, listen, accept)
- Supports IPv4 (AF_INET) addressing
- Proper error handling with Vex error types
- Network byte order conversion (htons)
- SO_REUSEADDR for server sockets

## Package: std::http

**Location**: `vex-libs/std/http/` (5 modules)

### Module: request.vx (120 lines)
- **HttpMethod** enum: GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS
- **HttpRequest** struct: Method, path, version, headers, body
- `new_request(method, path)`: Create new request
- `parse_request(raw)`: Parse HTTP request from string
- Header management methods

### Module: response.vx (125 lines)
- **HttpResponse** struct: Status, reason, version, headers, body
- `new_response(status)`: Create new response
- `parse_response(raw)`: Parse HTTP response from string
- Standard HTTP status codes (200, 404, 500, etc.)

### Module: client.vx (180 lines)
- **HttpClient**: HTTP client implementation
- `new_client()`: Create HTTP client with defaults
- `request(req)`: Execute generic HTTP request
- `get(url)`: Perform GET request
- `post(url, body)`: Perform POST request
- `put(url, body)`: Perform PUT request
- `delete(url)`: Perform DELETE request

**Features:**
- URL parsing (http://host:port/path)
- Automatic Content-Length handling
- Connection management over TcpStream
- Response buffering with proper Content-Length detection

### Module: server.vx (110 lines)
- **HttpServer**: HTTP server implementation
- **Handler** type: Request handler function type
- `new_server(addr, port, handler)`: Create HTTP server
- `start()`: Start listening and handling requests
- `route(req, routes)`: Route requests to handlers
- `handle_connection()`: Process individual connections

**Features:**
- Accept loop for incoming connections
- Request parsing and validation
- Response generation and sending
- Error handling with 400/404 responses

### Module: mod.vx
- Exports all HTTP types and functions
- Module organization

## FFI Extensions: libc.vx

Added socket-related FFI bindings:

### Socket Functions
- `socket(domain, type, protocol)`: Create socket
- `connect(sockfd, addr, addrlen)`: Connect to address
- `bind(sockfd, addr, addrlen)`: Bind to address
- `listen(sockfd, backlog)`: Listen for connections
- `accept(sockfd, addr, addrlen)`: Accept connection
- `send/recv`: Send/receive data
- `sendto/recvfrom`: UDP operations
- `setsockopt/getsockopt`: Socket options
- `inet_pton/inet_ntop`: IP address conversion

### Socket Constants
- Address families: AF_INET, AF_INET6, AF_UNIX
- Socket types: SOCK_STREAM, SOCK_DGRAM, SOCK_RAW
- Socket options: SOL_SOCKET, SO_REUSEADDR, SO_KEEPALIVE
- Shutdown modes: SHUT_RD, SHUT_WR, SHUT_RDWR

## Usage Examples

### TCP Client
```vex
import { net } from "std";

stream := net::connect("127.0.0.1", 8080)?;
defer stream.close();

msg := "Hello, Server!";
stream.write(msg.as_ptr(), msg.len())?;

buffer: [byte; 1024] = [0; 1024];
bytes_read := stream.read(&buffer[0], 1024)?;
```

### TCP Server
```vex
import { net } from "std";

listener := net::listen("0.0.0.0", 8080)?;
println("Server listening on port 8080");

loop {
    client := listener.accept()?;
    // Handle client connection
    client.close()?;
}
```

### HTTP Client
```vex
import { http } from "std";

client := http::new_client();
response := client.get("http://example.com")?;

println("Status: {}", response.status);
println("Body: {}", response.body);
```

### HTTP Server
```vex
import { http, HttpRequest, HttpResponse, new_response } from "std";

fn handler(req: HttpRequest): HttpResponse {
    mut res := new_response(200);
    res.set_body("Hello, World!");
    return res;
}

server := http::new_server("0.0.0.0", 8080, handler);
server.start()?;
```

## Technical Details

### Architecture
- **Layer 1 (FFI)**: libc socket syscalls
- **Layer 2 (net)**: Socket abstractions (TCP/UDP)
- **Layer 3 (http)**: HTTP protocol implementation

### Error Handling
- All operations return `(Result | error)` types
- Proper error propagation with `?` operator
- Descriptive error messages

### Memory Management
- Stack-allocated buffers for efficiency
- Automatic cleanup with defer
- Safe pointer operations

### Standards Compliance
- HTTP/1.1 protocol
- Standard socket API (POSIX)
- Proper CRLF line endings

## Status

✅ Complete implementation
✅ TCP client and server support
✅ UDP socket support
✅ HTTP client with URL parsing
✅ HTTP server with routing
✅ Proper error handling
✅ FFI bindings for all socket operations

## Next Steps

1. Testing with actual network operations
2. TLS/HTTPS support (via OpenSSL FFI)
3. IPv6 address support
4. WebSocket protocol
5. HTTP/2 implementation
