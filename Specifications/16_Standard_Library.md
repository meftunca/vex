# Standard Library

**Version:** 0.1.2  
**Last Updated:** November 2025

This document provides an overview of the Vex standard library organization and API reference.

---

## Table of Contents

1. [Library Architecture](#library-architecture)
2. [Builtin Functions](#builtin-functions)
3. [Layer 1: I/O Core](#layer-1-io-core)
4. [Layer 2: Protocol Layer](#layer-2-protocol-layer)
5. [Layer 3: Application Layer](#layer-3-application-layer)
6. [Module Reference](#module-reference)

---

## Library Architecture

### Four-Layer Design

```
┌─────────────────────────────────────────────────┐
│  Layer 3: Application (100% Safe Vex)          │
│  http, json, xml, yaml, toml                   │
├─────────────────────────────────────────────────┤
│  Layer 2: Protocol (100% Safe Vex)             │
│  net, sync, testing, datetime                   │
├─────────────────────────────────────────────────┤
│  Layer 1: I/O Core (Unsafe Bridge)             │
│  io, ffi, unsafe, hpc, libc                     │
├─────────────────────────────────────────────────┤
│  Layer 0: Vex Runtime                   │
│  io_uring, async scheduler, allocator          │
└─────────────────────────────────────────────────┘
```

### Design Principles

1. **Safety by default**: Layers 2 and 3 are 100% safe Vex code
2. **Unsafe isolation**: All `unsafe` code contained in Layer 1
3. **Zero-cost abstractions**: No runtime overhead
4. **Composable**: Layers build on each other

---

## Builtin Functions

Vex provides a comprehensive set of builtin functions that are always available without imports. These functions are implemented directly in the compiler and provide low-level access to memory, type information, LLVM intrinsics, and compiler optimizations.

### Memory Operations

Low-level memory management functions:

```vex
fn main(): i32 {
    // Allocate memory
    let ptr = alloc(1024);  // Allocate 1024 bytes

    // Get type information
    let size = sizeof(i64);     // Returns 8
    let align = alignof(i64);   // Returns 8

    // Memory operations
    memset(ptr, 0, 1024);       // Zero out memory

    // Copy memory
    let src = alloc(100);
    let dst = alloc(100);
    memcpy(dst, src, 100);      // Copy 100 bytes

    // Compare memory
    let result = memcmp(ptr1, ptr2, 100);  // Returns 0 if equal

    // Move overlapping memory
    memmove(dst, src, 100);     // Safe for overlapping regions

    // Resize allocation
    let new_ptr = realloc(ptr, 2048);

    // Free memory
    free(new_ptr);
    return 0;
}
```

**Available Functions**:

- `alloc(size: u64): &u8!` - Allocate memory
- `free(ptr: &u8!)` - Free memory
- `realloc(ptr: &u8!, size: u64): &u8!` - Resize allocation
- `sizeof<T>(): u64` - Get type size
- `alignof<T>(): u64` - Get type alignment
- `memcpy(dst: &u8!, src: &u8, size: u64)` - Copy memory
- `memset(ptr: &u8!, value: i32, size: u64)` - Set memory
- `memcmp(ptr1: &u8, ptr2: &u8, size: u64): i32` - Compare memory
- `memmove(dst: &u8!, src: &u8, size: u64)` - Move memory (overlapping safe)

**Status**: ✅ Fully implemented

### String Operations

C-style string manipulation:

```vex
fn main(): i32 {
    let str1 = "Hello";
    let str2 = "World";

    // Get string length
    let len = strlen(str1);  // Returns 5

    // Compare strings
    let cmp = strcmp(str1, str2);  // Returns <0, 0, or >0

    // Copy string
    let dest = alloc(100);
    strcpy(dest, str1);

    // Concatenate strings
    strcat(dest, str2);

    // Duplicate string
    let copy = strdup(str1);

    return 0;
}
```

**Available Functions**:

- `strlen(s: string): u64` - Get string length
- `strcmp(s1: string, s2: string): i32` - Compare strings
- `strcpy(dst: &u8!, src: string)` - Copy string
- `strcat(dst: &u8!, src: string)` - Concatenate strings
- `strdup(s: string): string` - Duplicate string

**Status**: ✅ Fully implemented

### UTF-8 Support

Unicode string validation and manipulation:

```vex
fn main(): i32 {
    let text = "Hello 🌍";

    // Validate UTF-8
    if utf8_valid(text) {
        // Count Unicode characters (not bytes)
        let char_count = utf8_char_count(text);  // Returns 7 (not 10)

        // Get character at index
        let ch = utf8_char_at(text, 6);  // Returns '🌍'
    }

    return 0;
}
```

**Available Functions**:

- `utf8_valid(s: string): bool` - Check if string is valid UTF-8
- `utf8_char_count(s: string): u64` - Count Unicode characters
- `utf8_char_at(s: string, index: u64): u32` - Get character at index

**Status**: ✅ Fully implemented

### Type Reflection

Runtime type information and checking:

```vex
fn main(): i32 {
    let x: i32 = 42;
    let y: f64 = 3.14;

    // Get type name as string
    let type_name = typeof(x);  // Returns "i32"

    // Get numeric type ID
    let id = type_id(x);  // Returns unique ID for i32

    // Get type size and alignment
    let size = type_size(x);   // Returns 4
    let align = type_align(x); // Returns 4

    // Check type categories
    if is_int_type(x) {
        println("x is an integer");
    }

    if is_float_type(y) {
        println("y is a float");
    }

    if is_pointer_type(&x) {
        println("This is a pointer");
    }

    return 0;
}
```

**Available Functions**:

- `typeof<T>(value: T): string` - Get type name
- `type_id<T>(value: T): u64` - Get unique type identifier
- `type_size<T>(value: T): u64` - Get type size
- `type_align<T>(value: T): u64` - Get type alignment
- `is_int_type<T>(value: T): bool` - Check if integer type
- `is_float_type<T>(value: T): bool` - Check if floating-point type
- `is_pointer_type<T>(value: T): bool` - Check if pointer type

**Status**: ✅ Fully implemented

### LLVM Intrinsics

Direct access to LLVM's optimized intrinsic functions:

#### Bit Manipulation

```vex
fn main(): i32 {
    let x: u32 = 0b00001000;

    // Count leading zeros
    let lz = ctlz(x);  // Returns 28

    // Count trailing zeros
    let tz = cttz(x);  // Returns 3

    // Count population (number of 1 bits)
    let pop = ctpop(x);  // Returns 1

    // Byte swap (reverse byte order)
    let swapped = bswap(0x12345678);  // Returns 0x78563412

    // Reverse all bits
    let reversed = bitreverse(0b00001111);  // Returns 0b11110000...

    return 0;
}
```

#### Overflow Checking

```vex
fn main(): i32 {
    let a: i32 = 2147483647;  // Max i32
    let b: i32 = 1;

    // Signed addition with overflow detection
    let result = sadd_overflow(a, b);
    // Returns: {sum: -2147483648, overflow: true}

    // Signed subtraction with overflow
    let result2 = ssub_overflow(a, b);

    // Signed multiplication with overflow
    let result3 = smul_overflow(a, 2);

    return 0;
}
```

**Available Intrinsics**:

**Bit Manipulation**:

- `ctlz(x: int): int` - Count leading zeros
- `cttz(x: int): int` - Count trailing zeros
- `ctpop(x: int): int` - Count population (1 bits)
- `bswap(x: int): int` - Byte swap
- `bitreverse(x: int): int` - Reverse all bits

**Overflow Checking**:

- `sadd_overflow(a: int, b: int): {int, bool}` - Signed add with overflow flag
- `ssub_overflow(a: int, b: int): {int, bool}` - Signed subtract with overflow flag
- `smul_overflow(a: int, b: int): {int, bool}` - Signed multiply with overflow flag

**Status**: ✅ Fully implemented

### Compiler Hints

Optimization hints for the compiler:

```vex
fn main(): i32 {
    let x = 10;

    // Tell compiler to assume condition is true
    assume(x > 0);  // Enables optimizations

    // Branch prediction hints
    if likely(x > 0) {
        // This branch is expected to execute
        println("Positive");
    }

    if unlikely(x == 0) {
        // This branch is rarely executed
        println("Zero");
    }

    // Memory prefetch hint
    let data: [i32; 1000] = [...];
    prefetch(&data[500], 0, 3, 1);  // Prefetch for reading
    // Parameters: addr, rw (0=read, 1=write), locality (0-3), cache_type

    return 0;
}
```

**Available Hints**:

- `assume(condition: bool)` - Assert condition is true (undefined if false)
- `likely(x: bool): bool` - Hint that condition is likely true
- `unlikely(x: bool): bool` - Hint that condition is likely false
- `prefetch(addr: &T, rw: i32, locality: i32, cache_type: i32)` - Prefetch memory

**Status**: ✅ Fully implemented

### Standard Library Modules

These modules are implemented as builtin functions and available without imports:

#### Logger Module

```vex
import * as logger from "logger";

fn main(): i32 {
    logger.debug("Debug message");
    logger.info("Information message");
    logger.warn("Warning message");
    logger.error("Error message");
    return 0;
}
```

**Available Functions**:

- `logger.debug(msg: string)` - Log debug message
- `logger.info(msg: string)` - Log info message
- `logger.warn(msg: string)` - Log warning message
- `logger.error(msg: string)` - Log error message

**Status**: ✅ Fully implemented

#### Time Module

```vex
import * as time from "time";

fn main(): i32 {
    // Get current time (seconds since epoch)
    let now = time.now();

    // Get high-resolution time (nanoseconds)
    let precise = time.high_res();

    // Sleep for milliseconds
    time.sleep_ms(1000);  // Sleep 1 second

    return 0;
}
```

**Available Functions**:

- `time.now(): i64` - Get current Unix timestamp (seconds)
- `time.high_res(): i64` - Get high-resolution time (nanoseconds)
- `time.sleep_ms(ms: i64)` - Sleep for milliseconds

**Status**: ✅ Fully implemented

#### Testing Module

```vex
import * as testing from "testing";

fn main(): i32 {
    let result = 2 + 2;

    // Basic assertion
    testing.assert(result == 4);

    // Equality assertion
    testing.assert_eq(result, 4);

    // Inequality assertion
    testing.assert_ne(result, 5);

    return 0;
}
```

**Available Functions**:

- `testing.assert(condition: bool)` - Assert condition is true
- `testing.assert_eq<T>(a: T, b: T)` - Assert values are equal
- `testing.assert_ne<T>(a: T, b: T)` - Assert values are not equal

**Status**: ✅ Fully implemented

---

## Layer 1: I/O Core

### io

Basic input/output operations:

```vex
import { println, print, readln } from "io";

fn main(): i32 {
    println("Hello, World!");
    print("Enter name: ");
    let name = readln();
    return 0;
}
```

**Functions**:

- `print(s: string)` - Print without newline
- `println(s: string)` - Print with newline
- `readln(): string` - Read line from stdin
- `eprint(s: string)` - Print to stderr
- `eprintln(s: string)` - Print to stderr with newline

**Status**: ✅ Basic I/O functions implemented and working

### ffi

Foreign Function Interface:

```vex
import * as ffi from "ffi";

extern "C" fn printf(format: string, ...): i32;
extern "C" fn malloc(size: u64): &u8!;
extern "C" fn free(ptr: &u8!);

fn main(): i32 {
    let ptr = malloc(1024);
    free(ptr);
    return 0;
}
```

**Status**: ✅ Memory operations (alloc, free, realloc) implemented as builtins

### unsafe

Unsafe operations:

```vex
import * as unsafe_ops from "unsafe";

fn raw_pointer_operations() {
    unsafe {
        let ptr: *const i32 = 0x1000 as *const i32;
        let value = *ptr;  // Dereference raw pointer
    }
}
```

**Status**: ✅ Unsafe blocks and raw pointers implemented

### hpc

High-Performance Computing primitives:

```vex
import * as hpc from "hpc";

fn main(): i32 {
    let vec = hpc.simd.Vector.new([1, 2, 3, 4]);
    let doubled = vec.mul(2);  // SIMD multiplication
    return 0;
}
```

**Status**: ❌ Planned

### libc

libc function bindings:

```vex
import { printf } from "libc";

fn main(): i32 {
    @printf("Hello from C!\n");
    return 0;
}
```

**Status**: ✅ FFI bindings working (extern declarations, raw pointers)

---

## Layer 2: Protocol Layer

### net

Networking primitives:

```vex
import { TcpStream } from "net/tcp";

fn main(): i32 {
    let stream = TcpStream.connect("127.0.0.1:8080");
    stream.write("GET / HTTP/1.1\r\n\r\n");
    let response = stream.read();
    return 0;
}
```

**Modules**:

- `"net/tcp"` - TCP sockets
- `"net/udp"` - UDP sockets
- `"net/ip"` - IP address handling

**Status**: 🚧 Planned (Layer 2)

### sync

Synchronization primitives:

```vex
import { Mutex } from "sync";

fn main(): i32 {
    let mutex = Mutex.new(0);

    {
        let! guard = mutex.lock();
        *guard = *guard + 1;
    }  // Automatically unlocked

    return 0;
}
```

**Primitives**:

- `Mutex<T>` - Mutual exclusion
- `RwLock<T>` - Read-write lock
- `Semaphore` - Counting semaphore
- `Barrier` - Thread barrier
- `WaitGroup` - Go-style wait group

**Status**: 🚧 Planned (Layer 2)

### testing

Testing framework:

```vex
import { assert_eq } from "testing";

test "addition works" {
    let result = add(2, 2);
    assert_eq(result, 4);
}

test "subtraction works" {
    let result = subtract(5, 3);
    assert_eq(result, 2);
}
```

**Assertions**:

- `assert(condition)` - Basic assertion
- `assert_eq(a, b)` - Equality assertion
- `assert_ne(a, b)` - Inequality assertion
- `assert_lt(a, b)` - Less than
- `assert_gt(a, b)` - Greater than

**Status**: 🚧 Planned (Layer 2)

### datetime

Date and time operations:

```vex
import * as datetime from "datetime";

fn main(): i32 {
    let now = datetime.now();
    let unix_time = now.unix_timestamp();
    let formatted = now.format("%Y-%m-%d %H:%M:%S");
    return 0;
}
```

**Status**: 🚧 Planned (Layer 2)

---

## Layer 3: Application Layer

### net/http

HTTP client and server:

```vex
import { get } from "net/http";
import { println } from "io";

fn main(): i32 {
    let response = get("https://api.example.com/data");
    match response {
        Response(body) => {
            println(body);
        }
        Error(msg) => {
            println(msg);
        }
    }
    return 0;
}
```

**Client API**:

- `get(url: string): (Response | Error)`
- `post(url: string, body: string): (Response | Error)`
- `put(url: string, body: string): (Response | Error)`
- `delete(url: string): (Response | Error)`

**Server API** (Future):

```vex
let server = http::Server::new();
server.route("/", handle_root);
server.listen(8080);
```

**Status**: 🚧 Planned (Layer 3)

### json

JSON parsing and serialization:

```vex
import { parse } from "json";

fn main(): i32 {
    let json_str = "{\"name\": \"Alice\", \"age\": 30}";
    let parsed = parse(json_str);

    match parsed {
        Object(obj) => {
            let name = obj.get("name");
        }
        Error(msg) => {
            println(msg);
        }
    }
    return 0;
}
```

**API**:

- `parse(s: string): (Value | Error)`
- `stringify(v: Value): string`
- `Value` enum: Object, Array, String, Number, Bool, Null

**Status**: 🚧 Planned (Layer 3)

### xml

XML parsing:

```vex
import { parse } from "xml";

fn main(): i32 {
    let xml_str = "<root><item>value</item></root>";
    let doc = parse(xml_str);
    return 0;
}
```

**Status**: 🚧 Planned (Layer 3)

### yaml

YAML parsing:

```vex
import { parse } from "yaml";

fn main(): i32 {
    let yaml_str = "name: Alice\nage: 30";
    let parsed = parse(yaml_str);
    return 0;
}
```

**Status**: 🚧 Planned (Layer 3)

### collections

Data structures:

```vex
import { HashMap, Vec } from "collections";

fn main(): i32 {
    let map = HashMap.new();
    map.insert("key", "value");

    let vec = Vec.new();
    vec.push(42);

    return 0;
}
```

**Types**:

- `Vec<T>` - Dynamic array
- `HashMap<K, V>` - Hash map
- `HashSet<T>` - Hash set
- `LinkedList<T>` - Linked list
- `BTreeMap<K, V>` - Ordered map
- `BTreeSet<T>` - Ordered set

**Status**: ❌ Not implemented

---

## Module Reference

### Complete Module Tree

```
std/
├── io/              ✅ Basic I/O working (Layer 1)
│   ├── mod.vx       - print, println, readln
│   ├── file.vx      - File I/O (planned)
│   └── stream.vx    - Stream operations (planned)
├── ffi/             ✅ FFI working (Layer 1)
│   └── mod.vx       - extern declarations, raw pointers
├── unsafe/          ✅ Implemented (Layer 1)
│   └── mod.vx       - Unsafe blocks, raw pointers
├── hpc/             🚧 Planned (Layer 1)
│   ├── simd.vx      - SIMD operations
│   └── gpu.vx       - GPU primitives
├── libc/            ✅ Basic bindings (Layer 1)
│   └── mod.vx       - libc bindings via @intrinsic
├── net/             🚧 Planned (Layer 2)
│   ├── mod.vx       - Common types
│   ├── tcp.vx       - TCP operations
│   ├── udp.vx       - UDP operations
│   └── ip.vx        - IP addressing
├── sync/            🚧 Planned (Layer 2)
│   ├── mod.vx       - Synchronization
│   ├── mutex.vx     - Mutex
│   ├── rwlock.vx    - RwLock
│   └── atomic.vx    - Atomic operations
├── testing/         ✅ Basic framework (Layer 2)
│   └── mod.vx       - assert functions, testing module
├── datetime/        🚧 Planned (Layer 2)
│   └── mod.vx       - Date/time operations
├── http/            🚧 Planned (Layer 3)
│   ├── mod.vx       - HTTP client/server
│   ├── client.vx    - Client API
│   └── server.vx    - Server API
├── json/            🚧 Planned (Layer 3)
│   └── mod.vx       - JSON parser
├── xml/             🚧 Planned (Layer 3)
│   └── mod.vx       - XML parser
├── yaml/            🚧 Planned (Layer 3)
│   └── mod.vx       - YAML parser
└── collections/     ✅ Builtins implemented
    ├── Vec<T>       - Dynamic array (builtin)
    ├── Map<K,V>     - Hash map (builtin)
    ├── Set<T>       - Hash set (builtin)
    ├── Box<T>       - Heap allocation (builtin)
    └── Channel<T>   - MPSC channel (builtin)
```

### Implementation Status

| Layer   | Modules                      | Status         | Completion |
| ------- | ---------------------------- | -------------- | ---------- |
| Layer 3 | http, json, xml, yaml        | 🚧 Planned     | 0%         |
| Layer 2 | net, sync, testing, datetime | 🚧 Planned     | 5%         |
| Layer 1 | io, ffi, unsafe, hpc, libc   | ✅ Partial     | 60%        |
| Layer 0 | Vex Runtime                  | ✅ Implemented | 80%        |

**Overall**: ~45% complete (builtins + I/O + FFI + unsafe working)

---

## Usage Examples

### Hello World

```vex
import { println } from "io";

fn main(): i32 {
    println("Hello, World!");
    return 0;
}
```

### Reading Input

```vex
import { println, readln } from "io";

fn main(): i32 {
    println("Enter your name:");
    let name = readln();
    println("Hello, " + name + "!");
    return 0;
}
```

### HTTP Request (Future)

```vex
import { get } from "net/http";
import { println } from "io";

fn main(): i32 {
    let response = get("https://api.example.com/data");
    match response {
        Response(body) => {
            println(body);
            return 0;
        }
        Error(msg) => {
            println("Error: " + msg);
            return 1;
        }
    }
}
```

### JSON Parsing (Future)

```vex
import { parse } from "json";
import { println } from "io";

fn main(): i32 {
    let json_str = "{\"name\": \"Alice\", \"age\": 30}";
    let parsed = parse(json_str);

    match parsed {
        Object(obj) => {
            println("Name: " + obj.get("name"));
            return 0;
        }
        Error(msg) => {
            println("Parse error: " + msg);
            return 1;
        }
    }
}
```

### Concurrency (Future)

```vex
import { WaitGroup } from "sync";
import { println } from "io";

fn worker(id: i32, wg: &WaitGroup!) {
    defer wg.done();
    println("Worker " + id + " starting");
    // Do work
    println("Worker " + id + " done");
}

fn main(): i32 {
    let wg = WaitGroup.new();

    for i in 0..5 {
        wg.add(1);
        go worker(i, &wg);
    }

    wg.wait();
    return 0;
}
```

---

## Development Roadmap

### Phase 1: Layer 1 Completion (High Priority 🔴)

**Duration**: 2-3 months

**Tasks**:

1. Complete `"io"` module
   - File I/O operations
   - Buffered I/O
   - Stream abstraction
2. Implement `"ffi"` module
   - FFI declarations
   - C interop
   - Type conversions
3. Basic `"libc"` bindings
   - Core functions
   - String operations
   - Memory operations

### Phase 2: Layer 2 Protocols (High Priority 🔴)

**Duration**: 3-4 months

**Tasks**:

1. `"net"` module family
   - `"net/tcp"` - TCP sockets
   - `"net/udp"` - UDP sockets
   - `"net/ip"` - IP addressing
2. `"sync"` primitives
   - Mutex, RwLock
   - Atomic operations
   - WaitGroup
3. `"testing"` framework
   - Test runner
   - Assertions
   - Benchmarks

### Phase 3: Layer 3 Applications (Medium Priority 🟡)

**Duration**: 4-6 months

**Tasks**:

1. `"net/http"` module
   - HTTP client
   - HTTP server
   - WebSocket support
2. Data formats
   - `"json"` parser
   - `"xml"` parser
   - `"yaml"` parser
3. `"collections"` module
   - Vec, HashMap, HashSet
   - Iterators
   - Algorithms

### Phase 4: Advanced Features (Low Priority 🟢)

**Duration**: Ongoing

**Tasks**:

1. `"hpc"` for SIMD/GPU
2. `"crypto"` for cryptography
3. `"database"` for SQL
4. Third-party ecosystem

---

## Contributing

Standard library is open for contributions. See:

- `vex-libs/std/README.md` for architecture details
- `STD_INTEGRATION_STATUS.md` for current status
- `STD_PACKAGE_REORGANIZATION.md` for reorganization plan

---

**Previous**: [14_Modules_and_Imports.md](./14_Modules_and_Imports.md)  
**Back to**: [01_Introduction_and_Overview.md](./01_Introduction_and_Overview.md)

**Maintained by**: Vex Language Team  
**Location**: `vex-libs/std/`
