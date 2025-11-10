# Vex Standard Library

Vex dilinin standart kütüphanesi. Her modül ayrı bir paket olarak organize edilmiştir.

## 📁 Dizin Yapısı

```
vex-libs/std/
├── collections/         # HashMap, Set (SwissTable)
├── core/                # Core types (Box, Vec, Option, Result)
├── crypto/              # Cryptographic functions + TLS (OpenSSL)
├── db/                  # Database drivers (PostgreSQL, MySQL, SQLite, MongoDB, Redis)
├── encoding/            # Base64, Base32, Hex, UUID (vex_fastenc)
├── fmt/                 # Formatting utilities
├── http/                # HTTP client/server
├── io/                  # I/O operations (print, println, file)
├── json/                # JSON parsing
├── memory/              # Memory operations (alloc, free, memcpy)
├── net/                 # Networking (TCP, UDP, event loop)
├── path/                # Path operations
├── strconv/             # Fast string conversion (int/float parsing)
├── string/              # String operations, UTF-8/16/32
├── sync/                # Concurrency (Channel, Mutex)
├── testing/             # Testing framework
└── time/                # Time operations (duration, timezone, RFC3339)

Note: async/await, spawn, goroutines dile gömülüdür (built-in keywords).
```

## 🎯 Usage Examples

Standard library modülleri doğrudan import edilir (NOT: `std/` prefix yok!):

```vex
// ===== I/O operations =====
import { println, print, readLine } from "io";

println("Hello, Vex!");
let input = readLine();

// ===== Core types =====
import { Box, Vec, Option, Result } from "core";

let boxed = Box.new(42);
let vec = Vec.new();
let maybe: Option<i32> = Some(10);

// ===== Collections (SwissTable) =====
import { HashMap, Set } from "collections";

let map = HashMap.new();
map.insert("key", "value");
let set = Set.new();

// ===== String operations =====
import { len, compare, is_valid_utf8, char_count } from "string";

let s = "Hello 世界";
let byte_len = len(s);
let char_count = char_count(s);  // UTF-8 aware

// ===== Memory operations =====
import { alloc, free, memcpy, memset } from "memory";

let ptr = alloc(1024);
memset(ptr, 0, 1024);
free(ptr);

// ===== Async/Await (Built-in to language) =====
// No import needed! async/await are keywords
async fn fetch_data() : string {
    let response = await http_get("https://api.example.com");
    return response;
}

// Spawn goroutine (built-in)
spawn {
    println("Running in goroutine!");
};

// ===== Concurrency (Channel - MPSC) =====
import { Channel } from "sync";

let chan = Channel.new(100);
chan.send(&data);
let received = chan.recv();

// ===== Time operations =====
import { now, sleep, parse_duration, parse_rfc3339, utc, local } from "time";

let t = now();
sleep(parse_duration("1s500ms"));  // 1.5 seconds
let instant = parse_rfc3339("2025-01-15T10:30:00Z");
let tz = local();

// ===== Networking =====
import { socket_tcp, bind, listen, accept, connect, read, write } from "net";

let fd = socket_tcp(false);  // IPv4
bind(fd, "127.0.0.1", 8080);
listen(fd, 128);
let client_fd = accept(fd);

// ===== Encoding (Base64, Hex, UUID) =====
import { base64_encode, hex_decode, uuid_v4, uuid_v7 } from "encoding";

let encoded = base64_encode(&data);
let decoded = base64_decode(s);
let id = uuid_v7();  // Time-based, sortable

// ===== Crypto + TLS =====
import { hash, random_bytes, aead_seal, tls_ctx_create, tls_wrap_fd } from "crypto";

let digest = hash("sha256", &data);
let random = random_bytes(32);
let tls_ctx = tls_ctx_create(false, true, "example.com");
let tls_conn = tls_wrap_fd(&tls_ctx, fd);

// ===== Database (Universal driver API) =====
import { connect, execute_query, fetch_next, DRIVER_POSTGRES } from "db";

let conn = connect(DRIVER_POSTGRES, "host=localhost user=vex dbname=mydb");
let res = execute_query(&conn, "SELECT * FROM users");
while let Some(row) = fetch_next(&mut res) {
    // Process row
}

// ===== String conversion (Fast parsing) =====
import { parse_i64, parse_f64, i64_to_str, f64_to_str } from "strconv";

match parse_i64("12345", 10) {
    ParseResult.Ok(value, consumed) => println("Parsed: {}", value),
    ParseResult.Err(code) => println("Error: {}", code),
}

let s = i64_to_str(42);  // "42"

// ===== Path operations =====
import { join, exists, normalize, dirname, basename } from "path";

let full_path = join("/home/user", "documents/file.txt");
if exists(full_path) {
    let dir = dirname(full_path);
    let file = basename(full_path);
}

// ===== Testing framework =====
import { assert, assert_eq, log, skip, subtest, set_bytes, reset_timer } from "testing";

fn test_addition() {
    assert(1 + 1 == 2, "Math broken!");
    assert_eq(2 + 2, 4, "Addition failed");
}

fn test_with_subtests() {
    subtest("case1", fn() {
        assert(true, "Should pass");
    });

    subtest("case2", fn() {
        log("Running case 2");
        assert_eq(10, 10, "Equal check");
    });
}

// Benchmark example
fn bench_string_ops() {
    let data = "x".repeat(1024);
    set_bytes(1024);  // For throughput calculation

    reset_timer();  // Exclude setup time

    for i in 0..1000 {
        let _ = data.len();
    }
}
import { create_client, get, post, create_server, serve } from "http";

let client = create_client();
let response = get(&client, "https://api.example.com/data");

fn handler(req: Request) : Response {
    return Response {
        status_code: 200,
        status_text: "OK",
        headers: HashMap.new(),
        body: "Hello, World!".as_bytes(),
    };
}
let server = create_server("0.0.0.0", 8080, handler);
serve(&server);  // Blocking
```

## 🔗 C Runtime Integration

Standard library, `vex-runtime/c/` altındaki C runtime fonksiyonlarını `extern "C"` ile çağırır:

| Vex Module    | C Runtime Files                                                        |
| ------------- | ---------------------------------------------------------------------- |
| `io`          | `vex_io.c`, `vex_file.c`                                               |
| `core`        | `vex_box.c`, `vex_vec.c`, `vex_array.c`                                |
| `collections` | `swisstable/vex_swisstable_v2.c`, `vex_set.c`                          |
| `string`      | `vex_string.c` (UTF-8/16/32 validation, SIMD)                          |
| `memory`      | `vex_alloc.c` (arena + free list), `vex_memory.c` (SIMD memcpy/memset) |
| `sync`        | `vex_channel.c` (lock-free MPSC)                                       |
| **Built-in**  | `async_runtime/` (M:N scheduler - language feature, not stdlib)        |
| `time`        | `vex_time/` (duration, timezone, RFC3339, Go layout, io_uring timers)  |
| `net`         | `vex_net/` (epoll/kqueue/IOCP/io_uring, Happy Eyeballs v2)             |
| `encoding`    | `vex_fastenc/` (Hex/Base64/Base32 SIMD, UUID v1-v8)                    |
| `crypto`      | `vex_openssl/` (AEAD, hash, HKDF, X25519, Ed25519, RSA, ECDSA, TLS)    |
| `db`          | `vex_db/` (PostgreSQL, MySQL, SQLite, MongoDB, Redis)                  |
| `strconv`     | `vex_strconv.c` (SIMD number parsing, 3-30x faster than Go)            |
| `path`        | `vex_path.c` (cross-platform path ops)                                 |
| `testing`     | `vex_testing.c` (TAP/JUnit reporters, fixtures, auto-calibration)      |

### Zero-Cost Abstractions

```vex

export fn print(s: string) {
    unsafe {
        vex_print(s.as_ptr(), s.len());
    }
}
```

Compiler, bu fonksiyonları **çağrı noktasına inline eder**, böylece:

- ✅ Zero call overhead
- ✅ Direct C function call
- ✅ No vtable/dispatch
- ✅ Full LLVM optimization

### Performance Highlights

| Component            | Performance           | vs Go        | vs Rust           |
| -------------------- | --------------------- | ------------ | ----------------- |
| **SwissTable**       | 34M lookup/s          | 4-6x faster  | Matches hashbrown |
| **strconv**          | 3-30x faster parsing  | ✅           | -                 |
| **UUID v7**          | 11M/s (91 ns)         | 110x faster  | -                 |
| **Base64 decode**    | 2913 MB/s             | 19.3x faster | -                 |
| **Hex encode**       | 4599 MB/s             | 6.4x faster  | -                 |
| **Memory ops**       | SIMD-accelerated      | -            | -                 |
| **UTF-8 validation** | SIMD (AVX2/NEON)      | -            | -                 |
| **Channel**          | Lock-free MPSC        | -            | -                 |
| **Allocator**        | TLS arena + free list | 5-15x faster | -                 |

## 🌍 Platform-Specific Files

Platform-specific implementasyonlar dosya adı ile belirlenir:

**Priority Order:**

1. `{file}.testing.vx` (test mode)
2. `{file}.{os}.{arch}.vx` (e.g., `lib.linux.x64.vx`)
3. `{file}.{arch}.vx` (e.g., `lib.x64.vx`)
4. `{file}.{os}.vx` (e.g., `lib.linux.vx`)
5. `{file}.vx` (fallback)

**Supported Platforms:**

- **OS**: `linux`, `macos`, `windows`, `freebsd`, `openbsd`
- **Arch**: `x64`, `arm64`, `wasm`, `wasi`, `riscv64`

## 📦 Version

Standard library versiyonu Vex compiler versiyonu ile senkronize edilir:

- `vex v0.1.1` → `std v0.1.1`
- No dependency declaration needed in `vex.json`
- Built-in, always available

## 🚀 Development

### Adding a New Module

1. Create directory structure:

```bash
mkdir -p vex-libs/std/mymodule/{src,tests}
```

2. Create `vex.json`:

```json
{
  "name": "mymodule",
  "version": "0.1.1",
  "description": "My module description",
  "authors": ["Vex Language Team"],
  "license": "MIT",
  "main": "src/lib.vx"
}
```

3. Create `src/lib.vx` with `extern "C"` declarations and `export` functions

4. Add tests in `tests/` directory

### Running Tests

```bash
vex test vex-libs/std/io
vex test vex-libs/std/string
```

## 📝 License

MIT License - See LICENSE file for details
