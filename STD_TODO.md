# Vex Standard Library Implementation Roadmap

**Target:** Rust/Go-level standard library with comprehensive ecosystem  
**Timeline:** 7 phases, P0-P3 priorities  
**Version:** 0.1.0 → 1.0.0  
**Runtime:** C-based (vex-runtime/c/) for zero-cost overhead

---

## 🎯 Core Principles

1. **Zero-Cost Abstractions** - Thin Vex wrappers → inline to C runtime calls
2. **C Runtime Integration** - Use existing vex-runtime/c/ implementations
3. **Vex Syntax Only** - No Rust features (no `mut`, `->`, `::`, etc.)
4. **Comprehensive Testing** - 100% code coverage, edge cases
5. **Real-World Ready** - Production-grade APIs
6. **Performance First** - Benchmarks against Rust/Go/C++

---

## 📦 Standard Directory Structure

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

---

## 📦 Phase 1: Core Foundations (1-2 weeks) 🔴 P0

### ✅ 1.1 std.collections - HashMap (2-3 days)

**Native Code Available:**

- ✅ `vex-runtime/c/swisstable/vex_swisstable_v2.c` - **WORLD-CLASS** HashMap
  - Insert: 30.47M ops/s (32.8 ns/op) - **2-3x faster than Rust!**
  - Lookup: 53.86M ops/s (18.6 ns/op)
  - SIMD-optimized (NEON/AVX2)
  - WyHash for fast hashing

**Vex API (vex-libs/std/collections/src/hashmap.vx):**

```vex
// HashMap with generic types
struct HashMap<K, V> {
    inner: *void,  // Opaque C SwissMap
}

impl HashMap<K, V> {
    // Constructor (no Self, use struct name)
    fn new(): HashMap<K, V> {
        let map = vex_map_new_v2(16);
        return HashMap { inner: map };
    }

    // Methods with &self and &self! (NOT &mut self!)
    fn insert(&self!, key: K, value: V): Option<V> {
        // Note: &self! for mutable methods
        let key_str = to_string(key);  // Helper function
        let old = vex_map_insert_v2(self.inner, key_str, value);
        if old == null {
            return Option.None;
        } else {
            return Option.Some(old);
        }
    }

    fn get(&self, key: K): Option<V> {
        let key_str = to_string(key);
        let ptr = vex_map_get_v2(self.inner, key_str);
        if ptr == null {
            return Option.None;
        } else {
            return Option.Some(ptr as V);
        }
    }

    fn remove(&self!, key: K): bool {
        let key_str = to_string(key);
        return vex_map_remove_v2(self.inner, key_str);
    }

    fn len(&self): i64 {
        return vex_map_len_v2(self.inner);
    }
}
```

**HashSet (wraps HashMap):**

```vex
struct HashSet<T> {
    map: HashMap<T, unit>,  // unit type () for values
}

impl HashSet<T> {
    fn new(): HashSet<T> {
        return HashSet { map: HashMap.new() };
    }

    fn insert(&self!, value: T): bool {
        let prev = self.map.insert(value, ());
        return match prev {
            Option.Some(_) => false,  // Already existed
            Option.None => true,      // Newly inserted
        };
    }

    fn contains(&self, value: T): bool {
        return match self.map.get(value) {
            Option.Some(_) => true,
            Option.None => false,
        };
    }
}
```

**Native Setup:**

```bash
cp vex-runtime/c/swisstable/vex_swisstable_v2.c vex-libs/std/collections/native/
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

**Tests (vex-libs/std/collections/tests/hashmap_test.vx):**

```vex
fn test_insert_get() {
    let! map = HashMap.new();
    map.insert("key1", 42);

    let val = map.get("key1");
    match val {
        Option.Some(v) => assert(v == 42),
        Option.None => panic("key not found"),
    }
}

fn test_collision() {
    let! map = HashMap.new();
    // Insert 1000 items
    for i in 0..1000 {
        map.insert(i, i * 2);
    }

    for i in 0..1000 {
        let val = map.get(i);
        match val {
            Option.Some(v) => assert(v == i * 2),
            Option.None => panic("missing key"),
        }
    }
}
```

---

### ✅ 1.2 std.io - Input/Output (2-3 days)

**Native Code Available:**

- ✅ `vex-runtime/c/vex_io.c` - Core I/O operations
- ✅ `vex-runtime/c/vex_file.c` - File operations

**Vex API (NO TRAITS - Vex has traits but different syntax):**

```vex
// BufReader struct (no generic Reader trait)
struct BufReader {
    fd: i32,           // File descriptor
    buffer: Vec<u8>,   // Internal buffer
    pos: i64,
    cap: i64,
}

impl BufReader {
    fn new(fd: i32): BufReader {
        let buf = Vec.with_capacity(8192);  // 8KB
        return BufReader {
            fd: fd,
            buffer: buf,
            pos: 0,
            cap: 0,
        };
    }

    fn read_line(&self!): Result<str, Error> {
        // Read until \n
        let! line = Vec.new();

        loop {
            if self.pos >= self.cap {
                // Refill buffer
                let n = vex_read(self.fd, self.buffer.data, 8192);
                if n <= 0 {
                    break;
                }
                self.cap = n;
                self.pos = 0;
            }

            let ch = self.buffer.get(self.pos);
            self.pos = self.pos + 1;

            if ch == '\n' {
                break;
            }
            line.push(ch);
        }

        return Result.Ok(string_from_vec(line));
    }
}

// Standard streams (global functions, not methods)
fn stdin(): BufReader {
    return BufReader.new(0);
}

fn stdout(): BufWriter {
    return BufWriter.new(1);
}

fn stderr(): BufWriter {
    return BufWriter.new(2);
}
```

**Usage:**

```vex
let! reader = stdin();
let line = reader.read_line();

match line {
    Result.Ok(s) => println(s),
    Result.Err(e) => eprintln("Error: {}", e),
}
```

---

### ✅ 1.3 std.string - String Operations (1-2 days)

**Native Code Available:**

- ✅ `vex-runtime/c/vex_string.c`
- ✅ `vex-runtime/c/vex_strconv.c`

**Vex API (extend built-in `str` type):**

```vex
// Note: Vex has `str` primitive, NOT `string` or `String`
// We add utility functions, not methods on str itself

// String operations (free functions)
fn str_len(s: str): i64 {
    return vex_strlen(s);
}

fn str_is_empty(s: str): bool {
    return str_len(s) == 0;
}

fn str_contains(s: str, substr: str): bool {
    return vex_strstr(s, substr) != null;
}

fn str_split(s: str, delimiter: str): Vec<str> {
    let! parts = Vec.new();
    // ... implementation
    return parts;
}

fn str_join(parts: Vec<str>, separator: str): str {
    // ... implementation
}

fn str_trim(s: str): str {
    // ... implementation
}

// StringBuilder struct
struct StringBuilder {
    buffer: Vec<u8>,
}

impl StringBuilder {
    fn new(): StringBuilder {
        return StringBuilder { buffer: Vec.new() };
    }

    fn append(&self!, s: str): unit {
        // Copy bytes from s to buffer
        for i in 0..str_len(s) {
            self.buffer.push(s[i]);
        }
    }

    fn build(&self): str {
        // Convert Vec<u8> to str
        return str_from_bytes(self.buffer);
    }
}
```

---

### ✅ 1.4 std.fmt - Formatting (1 day)

**Vex has Display trait! Use it:**

```vex
// Display trait (already in Vex)
trait Display {
    fn to_string(&self): str;
}

// Implement for custom types
struct Point {
    x: i32,
    y: i32,
}

impl Point impl Display {
    fn to_string(&self): str {
        // Use format function (if exists) or build manually
        let! sb = StringBuilder.new();
        sb.append("(");
        sb.append(int_to_string(self.x));
        sb.append(", ");
        sb.append(int_to_string(self.y));
        sb.append(")");
        return sb.build();
    }
}

// Format function (variadic args not in Vex yet?)
// For now: manual string building
fn format_int(template: str, value: i32): str {
    // Replace {} with value
    // ... implementation
}
```

---

## 📦 Phase 2: File System & I/O (1 week) 🟡 P1

### ✅ 2.1 std.fs - File System (2-3 days)

**Native:** `vex-runtime/c/vex_file.c`

```vex
struct File {
    fd: i32,
}

impl File {
    fn open(path: str): Result<File, Error> {
        let fd = vex_file_open(path, 0);  // O_RDONLY
        if fd < 0 {
            return Result.Err(Error.from_errno());
        }
        return Result.Ok(File { fd: fd });
    }

    fn create(path: str): Result<File, Error> {
        let fd = vex_file_create(path, 0644);
        if fd < 0 {
            return Result.Err(Error.from_errno());
        }
        return Result.Ok(File { fd: fd });
    }

    fn read(&self!, buf: &[u8]!): Result<i64, Error> {
        let n = vex_read(self.fd, buf, buf.len);
        if n < 0 {
            return Result.Err(Error.from_errno());
        }
        return Result.Ok(n);
    }

    fn close(&self!) {
        vex_close(self.fd);
    }
}

// Utility functions
fn read_file(path: str): Result<str, Error> {
    let file = File.open(path);
    match file {
        Result.Ok(f) => {
            // Read all
            let! buf = Vec.with_capacity(1024);
            loop {
                let chunk = [u8; 1024];
                let n = f.read(chunk);
                match n {
                    Result.Ok(size) => {
                        if size == 0 { break; }
                        for i in 0..size {
                            buf.push(chunk[i]);
                        }
                    },
                    Result.Err(e) => return Result.Err(e),
                }
            }
            f.close();
            return Result.Ok(str_from_bytes(buf));
        },
        Result.Err(e) => return Result.Err(e),
    }
}
```

---

## 📊 Implementation Status

| Module          | Priority | Status | Native Code Source                |
| --------------- | -------- | ------ | --------------------------------- |
| std.collections | 🔴 P0    | 0%     | ✅ swisstable/vex_swisstable_v2.c |
| std.io          | 🔴 P0    | 0%     | ✅ vex_io.c, vex_file.c           |
| std.string      | 🔴 P0    | 0%     | ✅ vex_string.c, vex_strconv.c    |
| std.fmt         | 🔴 P0    | 0%     | ⏳ String buffer ops              |
| std.fs          | 🟡 P1    | 0%     | ✅ vex_file.c                     |
| std.time        | 🟡 P1    | 0%     | ✅ vex_time/                      |
| std.net         | 🟡 P1    | 0%     | ✅ vex_net/                       |
| std.crypto      | 🟢 P2    | 0%     | ✅ vex_openssl/                   |
| std.sync        | 🟢 P2    | 0%     | ✅ vex_sync.c, vex_channel.c      |

**Total Progress:** 0% (clean slate with correct Vex syntax!)  
**Native Runtime:** ~70% of needed C code already exists

---

## 🎯 Next Sprint (Week 1)

**Goal:** Phase 1 - Core Foundations

**Tasks:**

1. **std.collections.HashMap** (2-3 days)

   - HashMap<K, V> wrapper over vex_swisstable_v2.c
   - HashSet<T> using HashMap internally
   - Tests: insert, get, remove, collisions, SIMD verification

2. **std.io** (2 days)

   - BufReader, BufWriter structs
   - stdin(), stdout(), stderr() functions
   - Tests: buffered vs unbuffered performance

3. **std.string** (1 day)

   - str utility functions (str_len, str_contains, etc.)
   - StringBuilder struct
   - Tests: UTF-8, performance

4. **std.fmt** (1 day)
   - Display trait implementations
   - Format helpers
   - Tests: custom Display, nested structs

**Deliverables:**

- vex.json for each module
- Native C code in native/ directories
- Comprehensive test suites
- Performance benchmarks
- Documentation + examples

---

## 📝 Vex Syntax Reminders

**What Vex HAS:**

- ✅ `let` and `let!` (not `mut`)
- ✅ `&self` and `&self!` (not `&mut self`)
- ✅ `fn name(): Type` (not `->`)
- ✅ `.` for all access (not `::`)
- ✅ `Option.Some(x)` (not `Option::Some`)
- ✅ `impl Struct { }` and `impl Struct impl Trait { }`
- ✅ `struct Struct with Policy impl Trait { }`
- ✅ Traits with methods
- ✅ Generics `<T>` with bounds `T: Display`
- ✅ `match`, `if`, `for`, `while`, `loop`
- ✅ Enums with tuple variants
- ✅ Pattern matching with data extraction

**What Vex DOES NOT HAVE:**

- ❌ `mut` keyword (use `let!` instead)
- ❌ `->` syntax (use `: Type`)
- ❌ `::` operator (use `.`)
- ❌ `Self` keyword (use struct name)
- ❌ `&mut T` (use `&T!`)
- ❌ Associated types (use generics)
- ❌ Variadic functions (yet?)
- ❌ `impl Trait for Struct` (use `impl Struct impl Trait`)

---

## 🚀 Getting Started

**Create first std module:**

```bash
# 1. Create directory structure
mkdir -p vex-libs/std/collections/{src,tests,native,examples}

# 2. Copy native code
cp vex-runtime/c/swisstable/vex_swisstable_v2.c vex-libs/std/collections/native/

# 3. Create vex.json
cat > vex-libs/std/collections/vex.json << 'EOF'
{
  "name": "std.collections",
  "version": "0.1.0",
  "native": {
    "sources": ["native/vex_swisstable_v2.c"],
    "include_dirs": ["native/", "../../vex-runtime/c/include/"],
    "cflags": ["-O3", "-march=native", "-DUSE_SIMD"]
  }
}
EOF

# 4. Write HashMap wrapper (see examples above)
vim vex-libs/std/collections/src/hashmap.vx

# 5. Write tests
vim vex-libs/std/collections/tests/hashmap_test.vx

# 6. Run tests
vex test vex-libs/std/collections/
```

---

**END OF CORRECTED STD_TODO.md**
