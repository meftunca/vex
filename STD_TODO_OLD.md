# Vex Standard Library Implementation Roadmap

**Target:** Rust/Go-level standard library with comprehensive ecosystem  
**Timeline:** 7 phases, P0-P3 priorities  
**Version:** 0.1.0 → 1.0.0  
**Runtime:** C-based (vex-runtime/c/) for zero-cost overhead

---

## 🎯 Core Principles

1. **Zero-Cost Abstractions** - Thin Vex wrappers → inline to C runtime calls
2. **C Runtime Integration** - Use existing vex-runtime/c/ implementations (swisstable, vex_io.c, vex_string.c, etc.)
3. **Comprehensive Testing** - 100% code coverage, edge cases
4. **Real-World Ready** - Production-grade APIs
5. **Developer Experience** - Rust-quality documentation + examples
6. **Performance First** - Benchmarks against Rust/Go/C++

---

## � Standard Directory Structure

Every `std` module follows this layout:

```
vex-libs/std/<module>/
├── vex.json             # Package manifest (REQUIRED)
├── src/
│   ├── lib.vx           # Public API (thin wrappers)
│   ├── internal.vx      # Private helpers
│   └── platform.vx      # OS-specific code
├── tests/
│   ├── unit_test.vx     # Unit tests
│   └── integration_test.vx
├── native/
│   ├── *.c              # C implementations (copied from vex-runtime/c/)
│   └── *.h              # Or symlinks to vex-runtime/c/
├── examples/
│   └── basic_usage.vx
└── README.md
```

### vex.json Template

```json
{
  "name": "std.<module>",
  "version": "0.1.0",
  "description": "Standard library - <module>",
  "authors": ["Vex Team"],
  "license": "MIT",
  "dependencies": {
    // Std packages CAN import each other (no circular deps!)
    // Example: std.io can depend on std.fmt
  },
  "native": {
    "sources": ["native/*.c"],
    "include_dirs": ["native/", "../../vex-runtime/c/include/"],
    "cflags": ["-O3", "-Wall"]
  }
}
```

### Native Code Strategy

**Most C runtime code already exists in `vex-runtime/c/`:**

- Collections: `swisstable/vex_swisstable_v2.c` (SwissTable HashMap)
- I/O: `vex_io.c`, `vex_file.c`
- String: `vex_string.c`, `vex_string_type.c`, `vex_strconv.c`
- Memory: `vex_alloc.c`, `vex_memory.c`, `vex_vec.c`
- Network: `vex_net/`
- Crypto: `vex_openssl/`
- Time: `vex_time/`
- Sync: `vex_sync.c`, `vex_channel.c`

**Options:**

1. **Copy** needed `.c/.h` files to `std/<module>/native/`
2. **Symlink** to vex-runtime/c/ (faster, but Git may not like it)
3. **Reference** via `include_dirs` in vex.json

**Recommendation:** Copy files to `native/` for self-contained packages.

**Rules:**

- All public APIs documented with examples
- 100% test coverage required
- Benchmarks for performance-critical code
- Native code only when necessary (performance/system APIs)
- Cross-platform: macOS, Linux, Windows

---

## 🎯 Priority Levels

- 🔴 **P0 - Critical:** Required for basic programs (io, collections, strings)
- 🟡 **P1 - High:** Common use cases (fs, json, time, http)
- 🟢 **P2 - Medium:** Advanced features (crypto, db, regex)
- 🔵 **P3 - Low:** Nice to have (compress, encoding)

---

## 📦 Phase 1: Core Foundations (1-2 weeks) 🔴 P0

### ✅ 1.1 std.io - Input/Output (2-3 days)

**Status:** Basic implementation exists, needs completion

**Current:**

- ✅ `print()`, `println()` via runtime
- ✅ Basic file operations
- ⏳ Missing: buffered I/O, readers/writers

**Native Code Available:**

- ✅ `vex-runtime/c/vex_io.c` - Core I/O operations (read, write, flush)
- ✅ `vex-runtime/c/vex_file.c` - File operations (open, close, seek)

**TODO:**

- [ ] **Reader/Writer Traits** (1 day)

  ```vex
  trait Reader {
      fn read(buf: &[u8]!): Result<usize, Error>;
  }

  trait Writer {
      fn write(buf: &[u8]): Result<usize, Error>;
  }
  ```

- [ ] **Buffered I/O** (1 day)
      **Strategy:** Wrap `vex_io.c` with Vex buffering layer

  ```vex
  struct BufReader<R: Reader> {
      inner: R,
      buffer: Vec<u8>,
      pos: usize,
  }

  impl<R: Reader> BufReader<R> {
      @inline
      fn new(reader: R): BufReader<R> {
          BufReader {
              inner: reader,
              buffer: Vec.with_capacity(8192),  // 8KB default
              pos: 0,
          }
      }

      fn read_line(&self): Result<string, Error>;
  }
  ```

- [ ] **Standard Streams** (0.5 days)

  ```vex
  @inline fn stdin(): BufReader<Stdin> {
      BufReader.new(Stdin { fd: 0 })
  }

  @inline fn stdout(): BufWriter<Stdout> {
      BufWriter.new(Stdout { fd: 1 })
  }

  @inline fn stderr(): BufWriter<Stderr> {
      BufWriter.new(Stderr { fd: 2 })
  }
  ```

**Native Setup:**

```bash
# Copy I/O runtime to std/io/native/
cp vex-runtime/c/vex_io.c vex-libs/std/io/native/
cp vex-runtime/c/vex_file.c vex-libs/std/io/native/
```

**vex.json:**

```json
{
  "name": "std.io",
  "dependencies": {
    "std.string": "0.1.0" // ← Std packages can import each other!
  },
  "native": {
    "sources": ["native/vex_io.c", "native/vex_file.c"],
    "include_dirs": ["native/", "../../vex-runtime/c/include/"]
  }
}
```

**Tests:**

- Basic read/write
- Buffered vs unbuffered performance
- Error handling (EOF, permission denied)
- Binary vs text mode

**Native:**

- `native/src/io.c` - System calls (read, write, open, close)
- Platform-specific: Windows HANDLE vs Unix fd

---

### ✅ 1.2 std.collections - Data Structures (3-4 days)

**Status:** Partial (Vec exists)

**Current:**

- ✅ `Vec<T>` - Dynamic array
- ⏳ HashMap, HashSet, LinkedList, BTreeMap missing

**Native Code Available:**

- ✅ `vex-runtime/c/swisstable/vex_swisstable_v2.c` - **WORLD-CLASS** HashMap (2-3x faster than Rust!)
  - Insert: 30.47M ops/s (32.8 ns/op)
  - Lookup: 53.86M ops/s (18.6 ns/op)
  - SIMD-optimized (NEON/AVX2)
  - WyHash for fast hashing

**TODO:**

#### HashMap<K, V> (1 day)

**Strategy:** Thin Vex wrapper → inline to `vex_swisstable_v2.c`

```vex
// vex-libs/std/collections/src/hashmap.vx
struct HashMap<K, V> {
    inner: VexMap,  // Opaque C type from vex.h
}

impl<K: Hash + Eq, V> HashMap<K, V> {
    @inline
    fn new(): HashMap<K, V> {
        let map: VexMap;
        vex_map_new_v2(&map, 16);  // C runtime call
        HashMap { inner: map }
    }

    @inline
    fn insert(key: K, value: V): Option<V> {
        // Marshal K to string (if not string)
        let key_str = key.to_string();
        vex_map_insert_v2(&self.inner, key_str.as_ptr(), value as *void);
    }

    @inline
    fn get(&self, key: &K): Option<&V> {
        let key_str = key.to_string();
        let ptr = vex_map_get_v2(&self.inner, key_str.as_ptr());
        if ptr.is_null() { None } else { Some(ptr as &V) }
    }

    @inline
    fn remove(key: &K): bool {
        let key_str = key.to_string();
        vex_map_remove_v2(&self.inner, key_str.as_ptr())
    }

    @inline
    fn len(&self): usize {
        vex_map_len_v2(&self.inner)
    }
}
```

**Native Setup:**

```bash
# Copy SwissTable to std/collections/native/
cp vex-runtime/c/swisstable/vex_swisstable_v2.c vex-libs/std/collections/native/
cp vex-runtime/c/swisstable/vex_swisstable_optimized.h vex-libs/std/collections/native/
```

**vex.json:**

```json
{
  "name": "std.collections",
  "native": {
    "sources": ["native/vex_swisstable_v2.c"],
    "include_dirs": ["native/", "../../vex-runtime/c/include/"],
    "cflags": ["-O3", "-march=native", "-DUSE_SIMD"]
  }
}
```

#### HashSet<T> (0.5 days)

```vex
struct HashSet<T> {
    map: HashMap<T, unit>,
}

impl<T: Hash + Eq> HashSet<T> {
    fn insert(value: T): bool;
    fn contains(&self, value: &T): bool;
    fn remove(value: &T): bool;
}
```

#### LinkedList<T> (1 day)

```vex
struct LinkedList<T> {
    head: Option<Box<Node<T>>>,
    tail: *mut Node<T>,
    len: usize,
}

impl<T> LinkedList<T> {
    fn push_front(value: T);
    fn push_back(value: T);
    fn pop_front(&self): Option<T>;
    fn pop_back(&self): Option<T>;
}
```

#### BTreeMap<K, V> (1 day)

```vex
struct BTreeMap<K, V> {
    root: Option<Box<Node<K, V>>>,
    len: usize,
}

impl<K: Ord, V> BTreeMap<K, V> {
    fn insert(key: K, value: V): Option<V>;
    fn get(&self, key: &K): Option<&V>;
    fn range(&self, start: &K, end: &K): Iterator<(&K, &V)>;
}
```

**Tests:**

- Insert/get/remove operations
- Collision handling (SwissTable H2 fingerprints)
- Iterator correctness
- Performance benchmarks (should beat Rust std::HashMap!)
- Load factor stress tests
- SIMD path verification (NEON on ARM, AVX2 on x86)

**Native:**

- ✅ `vex-runtime/c/swisstable/vex_swisstable_v2.c` (READY - copy to native/)
- ✅ WyHash already integrated (no SipHash needed)
- ✅ SIMD optimizations included

---

### ✅ 1.3 std.string - String Operations (2-3 days)

**Status:** Basic type exists, missing utilities

**Native Code Available:**

- ✅ `vex-runtime/c/vex_string.c` - Core string operations
- ✅ `vex-runtime/c/vex_string_type.c` - String type implementation
- ✅ `vex-runtime/c/vex_strconv.c` - String conversions

**TODO:**

#### String Utilities (1 day)

**Strategy:** Wrap existing `vex_string.c` functions

```vex
impl string {
    @inline fn len(&self): usize {
        vex_strlen(self.as_ptr())
    }

    @inline fn is_empty(&self): bool {
        self.len() == 0
    }

    fn split(&self, delimiter: string): Vec<string>;
    fn join(parts: &[string], separator: string): string;
    fn trim(&self): string;
    fn to_uppercase(&self): string;
    fn to_lowercase(&self): string;

    @inline fn contains(&self, substring: string): bool {
        vex_strstr(self.as_ptr(), substring.as_ptr()) != null
    }

    fn starts_with(&self, prefix: string): bool;
    fn ends_with(&self, suffix: string): bool;
}
```

#### String Builder (0.5 days)

```vex
struct StringBuilder {
    buffer: Vec<u8>,
}

impl StringBuilder {
    fn new(): StringBuilder;
    fn append(s: string): &StringBuilder;
    fn append_char(c: char): &StringBuilder;
    fn build(&self): string;
}
```

#### Format (1 day)

```vex
fn format(template: string, args: ...): string;

// Usage:
let msg = format("Hello, {}! You have {} messages.", name, count);
```

**Native Setup:**

```bash
# Copy string runtime to std/string/native/
cp vex-runtime/c/vex_string.c vex-libs/std/string/native/
cp vex-runtime/c/vex_string_type.c vex-libs/std/string/native/
cp vex-runtime/c/vex_strconv.c vex-libs/std/string/native/
```

**Tests:**

- UTF-8 edge cases (emoji, CJK characters)
- Performance: StringBuilder vs concatenation
- Format string security (no injection)

---

### ✅ 1.4 std.fmt - Formatting & Display (1 day)

**Status:** Not started

**Native Code Available:**

- ✅ Can reuse `vex_string.c` for buffer operations
- ⏳ Format parsing needed (pure Vex likely)

**TODO:**

#### Display Trait (0.5 days)

```vex
trait Display {
    fn fmt(&self, f: &Formatter): Result<unit, Error>;
}

struct Formatter {
    buffer: StringBuilder,
}
```

#### Debug Trait (0.5 days)

```vex
trait Debug {
    fn fmt(&self, f: &Formatter): Result<unit, Error>;
}

// Auto-derive for structs/enums
#[derive(Debug)]
struct Point { x: i32, y: i32 }
```

**vex.json:**

```json
{
  "name": "std.fmt",
  "dependencies": {
    "std.string": "0.1.0" // ← Import StringBuilder
  }
}
```

**Tests:**

- Custom Display implementations
- Nested struct formatting
- Circular reference detection

---

## 📦 Phase 2: File System & I/O (1 week) 🟡 P1

### ✅ 2.1 std.fs - File System Operations (2-3 days)

**TODO:**

#### File Operations (1 day)

```vex
struct File {
    fd: i32,  // Platform-specific
}

impl File {
    fn open(path: string): Result<File, Error>;
    fn create(path: string): Result<File, Error>;
    fn read(buf: &[u8]): Result<usize, Error>;
    fn write(buf: &[u8]): Result<usize, Error>;
    fn close(self);
}

fn read_file(path: string): Result<string, Error>;
fn write_file(path: string, content: string): Result<unit, Error>;
```

#### Directory Operations (1 day)

```vex
fn create_dir(path: string): Result<unit, Error>;
fn remove_dir(path: string): Result<unit, Error>;
fn read_dir(path: string): Result<Vec<DirEntry>, Error>;

struct DirEntry {
    name: string,
    path: string,
    is_file: bool,
    is_dir: bool,
}
```

#### Metadata (0.5 days)

```vex
struct Metadata {
    size: u64,
    is_file: bool,
    is_dir: bool,
    created: Time,
    modified: Time,
    permissions: Permissions,
}

fn metadata(path: string): Result<Metadata, Error>;
```

**Native:**

- `native/src/fs.c` - POSIX file operations
- `native/src/fs_windows.c` - Windows equivalents
- Error mapping: errno → Vex Error

**Tests:**

- Create/read/write/delete files
- Permissions (read-only, execute)
- Symlinks, hard links
- Large files (>2GB)

---

### ✅ 2.2 std.path - Path Manipulation (1 day)

```vex
struct Path {
    inner: string,
}

impl Path {
    fn new(path: string): Path;
    fn join(&self, other: string): Path;
    fn parent(&self): Option<Path>;
    fn file_name(&self): Option<string>;
    fn extension(&self): Option<string>;
    fn is_absolute(&self): bool;
    fn exists(&self): bool;
}
```

**Platform Handling:**

- Unix: `/home/user/file.txt`
- Windows: `C:\Users\User\file.txt`
- Normalize: `/path//to/../file` → `/path/file`

---

### ✅ 2.3 std.time - Date & Time (1-2 days)

```vex
struct Time {
    seconds: i64,
    nanos: i32,
}

impl Time {
    fn now(): Time;
    fn unix_timestamp(): i64;
    fn format(&self, layout: string): string;
    fn parse(s: string, layout: string): Result<Time, Error>;
}

struct Duration {
    seconds: i64,
    nanos: i32,
}

impl Duration {
    fn from_secs(s: i64): Duration;
    fn from_millis(ms: i64): Duration;
    fn as_secs(&self): i64;
}
```

**Native:**

- `native/src/time.c` - `clock_gettime()`, `gettimeofday()`
- Timezone support via system libs

---

## 📦 Phase 3: Networking (1-2 weeks) 🟡 P1

### ✅ 3.1 std.net - TCP/UDP Sockets (2-3 days)

```vex
struct TcpListener {
    fd: i32,
}

impl TcpListener {
    fn bind(addr: string): Result<TcpListener, Error>;
    fn accept(&self): Result<TcpStream, Error>;
}

struct TcpStream {
    fd: i32,
}

impl TcpStream {
    fn connect(addr: string): Result<TcpStream, Error>;
    fn read(buf: &[u8]): Result<usize, Error>;
    fn write(buf: &[u8]): Result<usize, Error>;
}

struct UdpSocket {
    fn bind(addr: string): Result<UdpSocket, Error>;
    fn send_to(&self, buf: &[u8], addr: string): Result<usize, Error>;
    fn recv_from(buf: &[u8]): Result<(usize, string), Error>;
}
```

**Native:**

- `native/src/net.c` - BSD sockets API
- IPv4/IPv6 support
- Non-blocking I/O with `epoll`/`kqueue`

---

### ✅ 3.2 std.http - HTTP Client/Server (3-5 days)

```vex
// HTTP Client
struct Client {
    fn get(url: string): Result<Response, Error>;
    fn post(url: string, body: string): Result<Response, Error>;
}

struct Response {
    status: i32,
    headers: HashMap<string, string>,
    body: string,
}

// HTTP Server
struct Server {
    fn new(addr: string): Server;
    fn route(path: string, handler: fn(Request): Response);
    fn listen(&self);
}

struct Request {
    method: string,
    path: string,
    headers: HashMap<string, string>,
    body: string,
}
```

**Implementation:**

- Built on top of `std.net.TcpListener`
- HTTP/1.1 parser (native C for speed)
- Chunked transfer encoding
- Connection pooling

---

## 📦 Phase 4: Serialization & Encoding (1 week) 🟡 P1

### ✅ 4.1 std.json - JSON Parser/Serializer (2-3 days)

```vex
enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(string),
    Array(Vec<JsonValue>),
    Object(HashMap<string, JsonValue>),
}

fn parse(json: string): Result<JsonValue, Error>;
fn stringify(value: JsonValue): string;

// Derive macro
#[derive(Serialize, Deserialize)]
struct User {
    name: string,
    age: i32,
}

let user = User { name: "Alice", age: 30 };
let json = json.stringify(user);
```

**Native:**

- `native/src/json.c` - Fast parser using SIMD
- Optional: Use existing C library (cJSON, simdjson)

---

### ✅ 4.2 std.encoding - Base64, Hex, URL Encoding (1 day)

```vex
mod base64 {
    fn encode(data: &[u8]): string;
    fn decode(s: string): Result<Vec<u8>, Error>;
}

mod hex {
    fn encode(data: &[u8]): string;
    fn decode(s: string): Result<Vec<u8>, Error>;
}

mod url {
    fn encode(s: string): string;
    fn decode(s: string): Result<string, Error>;
}
```

---

## 📦 Phase 5: Cryptography (1 week) 🟢 P2

### ✅ 5.1 std.crypto - Hashing & Encryption (3-5 days)

```vex
mod crypto {
    mod hash {
        fn sha256(data: &[u8]): [u8; 32];
        fn sha512(data: &[u8]): [u8; 64];
        fn blake3(data: &[u8]): [u8; 32];
    }

    mod aes {
        fn encrypt_gcm(key: &[u8], nonce: &[u8], plaintext: &[u8]): Vec<u8>;
        fn decrypt_gcm(key: &[u8], nonce: &[u8], ciphertext: &[u8]): Result<Vec<u8>, Error>;
    }

    mod rsa {
        struct PublicKey { ... }
        struct PrivateKey { ... }

        fn generate_keypair(): (PublicKey, PrivateKey);
        fn encrypt(key: &PublicKey, data: &[u8]): Vec<u8>;
        fn decrypt(key: &PrivateKey, data: &[u8]): Result<Vec<u8>, Error>;
    }
}
```

**Native:**

- Use OpenSSL/BoringSSL for crypto primitives
- FIPS 140-2 compliance for enterprise use

---

## 📦 Phase 6: Concurrency & Parallelism (1-2 weeks) 🟢 P2

### ✅ 6.1 std.sync - Synchronization Primitives (2-3 days)

```vex
struct Mutex<T> {
    fn new(value: T): Mutex<T>;
    fn lock(&self): MutexGuard<T>;
}

struct RwLock<T> {
    fn new(value: T): RwLock<T>;
    fn read(&self): RwLockReadGuard<T>;
    fn write(&self): RwLockWriteGuard<T>;
}

struct Channel<T> {
    fn new(): (Sender<T>, Receiver<T>);
}
```

---

### ✅ 6.2 std.thread - Threading (2 days)

```vex
fn spawn<F>(f: F): JoinHandle
where F: Callable(): unit;

struct JoinHandle {
    fn join(self): Result<unit, Error>;
}
```

---

## 📦 Phase 7: Advanced Features (ongoing) 🔵 P3

### Database Drivers

- SQLite (embedded)
- PostgreSQL client
- MySQL client

### Compression

- gzip, zlib, brotli

### Regex

- PCRE-compatible regex engine

### Testing Framework

- Test runners, assertions, benchmarks

---

## 📊 Implementation Status

| Module          | Priority | Status | Lines | Tests | Native Code Source                |
| --------------- | -------- | ------ | ----- | ----- | --------------------------------- |
| std.io          | 🔴 P0    | 40%    | 200   | 5     | ✅ vex_io.c, vex_file.c           |
| std.collections | 🔴 P0    | 30%    | 500   | 10    | ✅ swisstable/vex_swisstable_v2.c |
| std.string      | 🔴 P0    | 20%    | 150   | 8     | ✅ vex_string.c, vex_strconv.c    |
| std.fmt         | 🔴 P0    | 0%     | 0     | 0     | ⏳ String buffer ops              |
| std.fs          | 🟡 P1    | 15%    | 100   | 3     | ✅ vex_file.c                     |
| std.path        | 🟡 P1    | 10%    | 50    | 2     | ⏳ Pure Vex                       |
| std.time        | 🟡 P1    | 25%    | 80    | 4     | ✅ vex_time/                      |
| std.net         | 🟡 P1    | 5%     | 50    | 1     | ✅ vex_net/                       |
| std.http        | 🟡 P1    | 0%     | 0     | 0     | ⏳ vex_net/ + HTTP parser         |
| std.json        | 🟡 P1    | 0%     | 0     | 0     | ⏳ Pure Vex parser                |
| std.crypto      | 🟢 P2    | 5%     | 30    | 1     | ✅ vex_openssl/                   |
| std.sync        | 🟢 P2    | 0%     | 0     | 0     | ✅ vex_sync.c, vex_channel.c      |
| std.thread      | 🟢 P2    | 0%     | 0     | 0     | ✅ pthread/Win32 wrappers         |

**Total Progress:** ~15% complete  
**Native Runtime:** ~70% of needed C code already exists in `vex-runtime/c/`

---

## 🎯 Next Sprint (Week 1)

**Goal:** Complete Phase 1 (Core Foundations)

**Tasks:**

1. std.io - Complete Reader/Writer traits + BufReader
2. std.collections - Implement HashMap + HashSet
3. std.string - String utilities + StringBuilder
4. std.fmt - Display + Debug traits

**Deliverables:**

- 100% test coverage for io, collections, string
- Performance benchmarks vs Rust std
- Documentation with examples
- Native C code for hash functions

**Estimated:** 8-10 days full-time work

---

## 📝 Development Guidelines

1. **API Design:**

   - Follow Rust conventions (Result, Option, iterators)
   - Zero-cost abstractions
   - Type safety over convenience

2. **Testing:**

   - Unit tests for each function
   - Integration tests for workflows
   - Property-based testing for collections
   - Benchmarks for performance-critical code

3. **Native Code:**

   - Use only when necessary
   - Wrap in safe Vex API
   - Cross-platform or provide platform-specific variants

4. **Documentation:**

   - Every public function documented
   - Examples in doc comments
   - README.md with getting started guide

5. **Performance:**
   - Benchmark against Rust/Go equivalents
   - Target: Within 10% of native performance
   - Profile and optimize hot paths

---

**Last Updated:** November 10, 2025
**Next Review:** After Phase 1 completion
