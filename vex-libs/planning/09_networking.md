# Vex Stdlib Planning - 09: Networking

**Priority:** 9
**Status:** Partial (net exists, others missing)
**Dependencies:** builtin, io, sync, time

## 📦 Packages in This Category

### 9.1 net (extend existing)
**Status:** ✅ Exists (comprehensive extension needed)
**Description:** Network interface

#### Current Implementation
- Basic network functions exist

#### Required Extensions
```vex
// Address types
trait Addr {
    fn network(self: &Self): str
    fn string(self: &Self): str
}

struct TCPAddr {
    ip: IP,
    port: int,
    zone: str,
}

struct UDPAddr {
    ip: IP,
    port: int,
    zone: str,
}

struct UnixAddr {
    name: str,
    net: str,
}

// Connection types
trait Conn {
    fn read(self: &mut Self, b: []u8): Result<usize, Error>
    fn write(self: &mut Self, b: []u8): Result<usize, Error>
    fn close(self: Self) -> Result<(), Error>
    fn local_addr(self: &Self): Result<Addr, Error>
    fn remote_addr(self: &Self): Result<Addr, Error>
    fn set_deadline(self: &Self, t: time.Time) -> Result<(), Error>
    fn set_read_deadline(self: &Self, t: time.Time) -> Result<(), Error>
    fn set_write_deadline(self: &Self, t: time.Time) -> Result<(), Error>
}

struct TCPConn {
    // implements Conn
}

struct UDPConn {
    // implements Conn
}

struct UnixConn {
    // implements Conn
}

// Listener types
trait Listener {
    fn accept(self: &Self): Result<Conn, Error>
    fn close(self: Self) -> Result<(), Error>
    fn addr(self: &Self): Result<Addr, Error>
}

struct TCPListener {
    // implements Listener
}

struct UDPConn {
    // implements Listener for UDP
}

struct UnixListener {
    // implements Listener
}

// Dialer and resolver
struct Dialer {
    timeout: time.Duration,
    deadline: time.Time,
    local_addr: Addr,
    dual_stack: bool,
    fallback_delay: time.Duration,
    keep_alive: time.Duration,
    resolver: *Resolver,
}

struct Resolver {
    prefer_go: bool,
    strict_errors: bool,
    dial: fn(ctx: context.Context, network: str, address: str): Result<Conn, Error>,
}

fn dial(network: str, address: str): Result<Conn, Error>
fn dial_timeout(network: str, address: str, timeout: time.Duration): Result<Conn, Error>
fn listen(network: str, address: str): Result<Listener, Error>
fn resolve_tcp_addr(network: str, address: str): Result<*TCPAddr, Error>
fn resolve_udp_addr(network: str, address: str): Result<*UDPAddr, Error>
fn resolve_unix_addr(network: str, address: str): Result<*UnixAddr, Error>
```

#### Required Types
```vex
type IP = []u8

struct IPNet {
    ip: IP,
    mask: IPMask,
}

type IPMask = []u8

struct Interface {
    index: int,
    mtu: int,
    name: str,
    hardware_addr: HardwareAddr,
    flags: InterfaceFlags,
    addrs: []Addr,
}

type HardwareAddr = []u8

struct InterfaceFlags {
    // bit flags
}
```

#### Dependencies
- builtin
- io
- context
- time

### 9.2 net/http
**Status:** ❌ Missing (critical for web applications)
**Description:** HTTP client and server

#### Required Types
```vex
struct Request {
    method: str,
    url: *url.URL,
    proto: str,
    proto_major: int,
    proto_minor: int,
    header: Header,
    body: io.ReadCloser,
    get_body: fn(): Result<io.ReadCloser, Error>,
    content_length: i64,
    transfer_encoding: []str,
    close: bool,
    host: str,
    form: url.Values,
    post_form: url.Values,
    multipart_form: *multipart.Form,
    trailer: Header,
    remote_addr: str,
    request_uri: str,
    tls: *tls.ConnectionState,
    cancel: <-chan struct{},
    response: *Response,
}

struct Response {
    status: str,
    status_code: int,
    proto: str,
    proto_major: int,
    proto_minor: int,
    header: Header,
    body: io.ReadCloser,
    content_length: i64,
    transfer_encoding: []str,
    close: bool,
    trailer: Header,
    request: *Request,
    tls: *tls.ConnectionState,
}

type Header = Map<str, []str>

struct Client {
    transport: RoundTripper,
    check_redirect: fn(req: *Request, via: []*Request): Result<Error, Error>,
    jar: CookieJar,
    timeout: time.Duration,
}

trait RoundTripper {
    fn round_trip(self: &Self, req: *Request): Result<*Response, Error>
}

struct Server {
    addr: str,
    handler: Handler,
    tls_config: *tls.Config,
    read_timeout: time.Duration,
    read_header_timeout: time.Duration,
    write_timeout: time.Duration,
    idle_timeout: time.Duration,
    max_header_bytes: int,
    tls_next_proto: Map<str, fn(*Server, *tls.Conn, Handler)>,
    conn_state: fn(net.Conn, ConnState),
    error_log: *log.Logger,
}

trait Handler {
    fn serve_http(self: &Self, w: ResponseWriter, r: *Request)
}

trait ResponseWriter {
    fn header(self: &Self): Header
    fn write(self: &Self, data: []u8): Result<usize, Error>
    fn write_header(self: &Self, status_code: int)
}
```

#### Required Functions
```vex
// HTTP methods
fn get(url: str): Result<*Response, Error>
fn post(url: str, content_type: str, body: io.Reader): Result<*Response, Error>
fn post_form(url: str, data: url.Values): Result<*Response, Error>
fn head(url: str): Result<*Response, Error>

// Client operations
fn new_request(method: str, url: str, body: io.Reader): Result<*Request, Error>
fn new_client(): *Client

// Server operations
fn listen_and_serve(addr: str, handler: Handler): Result<(), Error>
fn listen_and_serve_tls(addr: str, cert_file: str, key_file: str, handler: Handler): Result<(), Error>

// Utility functions
fn status_text(code: int): str
fn redirect(w: ResponseWriter, r: *Request, url: str, code: int)
fn not_found(w: ResponseWriter, r: *Request)
fn error(w: ResponseWriter, error: str, code: int)
```

#### Dependencies
- net
- url
- tls
- context
- io
- log

### 9.3 net/url
**Status:** ❌ Missing (important for URL handling)
**Description:** URL parsing and manipulation

#### Required Types
```vex
struct URL {
    scheme: str,
    opaque: str,
    user: *Userinfo,
    host: str,
    path: str,
    raw_path: str,
    force_query: bool,
    raw_query: str,
    fragment: str,
}

struct Userinfo {
    username: str,
    password: str,
    password_set: bool,
}

type Values = Map<str, []str>
```

#### Required Functions
```vex
// URL parsing
fn parse(rawurl: str): Result<URL, Error>
fn parse_request_uri(rawurl: str): Result<URL, Error>

// URL construction
fn new_url(): URL

// URL operations
fn string(u: &URL): str
fn resolve_reference(base: &URL, ref: &URL): URL
fn query_unescape(s: str): Result<str, Error>
fn query_escape(s: str): str
fn path_unescape(s: str): Result<str, Error>
fn path_escape(s: str): str

// Values operations
fn new_values(): Values
fn get(v: &Values, key: str): str
fn set(v: &Values, key: str, value: str)
fn add(v: &Values, key: str, value: str)
fn del(v: &Values, key: str)
fn encode(v: &Values): str
fn parse_query(query: str): Result<Values, Error>
```

#### Dependencies
- builtin
- strings
- strconv

### 9.4 net/rpc
**Status:** ❌ Missing (useful for distributed systems)
**Description:** Remote procedure calls

#### Required Types
```vex
struct Client {
    // internal
}

struct Server {
    // internal
}

trait Service {
    // service methods
}
```

#### Required Functions
```vex
fn new_client(conn: io.ReadWriteCloser): *Client
fn new_server(): *Server
fn register(rcvr: any): Result<(), Error>
fn accept(lis: net.Listener)
fn serve_conn(conn: io.ReadWriteCloser)
fn serve_request(codec: ServerCodec): Result<(), Error>
```

#### Dependencies
- net
- reflect
- io

## 🎯 Implementation Priority

1. **net extensions** - Complete network interface
2. **net/url** - URL parsing and manipulation
3. **net/http** - HTTP client and server
4. **net/rpc** - Remote procedure calls

## ⚠️ Language Feature Issues

- **Reflection:** RPC needs runtime type information
- **Interfaces:** Complex interface hierarchies
- **Goroutines:** HTTP server needs concurrent handling

## 📋 Missing Critical Dependencies

- **TLS Support:** For HTTPS
- **Reflection:** For RPC
- **Context Integration:** For request cancellation

## 🚀 Next Steps

1. Extend net package with full TCP/UDP support
2. Implement URL parsing
3. Add HTTP client and server
4. Create RPC framework