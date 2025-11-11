# Standard Library

**Version:** 0.1.2 
**Last Updated:** November 2025

This document provides an overview of the Vex standard library organization and API reference.

---

## Table of Contents

1. \1
2. \1
3. \1
4. \1
5. \1
6. \1

---

## Library Architecture

### Four-Layer Design

[13 lines code: (unknown)]

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

[29 lines code: ```vex]

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

[22 lines code: ```vex]

**Available Functions**:

- `strlen(s: string): u64` - Get string length
- `strcmp(s1: string, s2: string): i32` - Compare strings
- `strcpy(dst: &u8!, src: string)` - Copy string
- `strcat(dst: &u8!, src: string)` - Concatenate strings
- `strdup(s: string): string` - Duplicate string

**Status**: ✅ Fully implemented

### UTF-8 Support

Unicode string validation and manipulation:

[14 lines code: ```vex]

**Available Functions**:

- `utf8_valid(s: string): bool` - Check if string is valid UTF-8
- `utf8_char_count(s: string): u64` - Count Unicode characters
- `utf8_char_at(s: string, index: u64): u32` - Get character at index

**Status**: ✅ Fully implemented

### Type Reflection

Runtime type information and checking:

[29 lines code: ```vex]

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

### Bit Manipulation

[20 lines code: ```vex]

### Overflow Checking

[16 lines code: ```vex]

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

[24 lines code: ```vex]

**Available Hints**:

- `assume(condition: bool)` - Assert condition is true (undefined if false)
- `likely(x: bool): bool` - Hint that condition is likely true
- `unlikely(x: bool): bool` - Hint that condition is likely false
- `prefetch(addr: &T, rw: i32, locality: i32, cache_type: i32)` - Prefetch memory

**Status**: ✅ Fully implemented

### Standard Library Modules

These modules are implemented as builtin functions and available without imports:

### Logger Module

[9 lines code: ```vex]

**Available Functions**:

- `logger.debug(msg: string)` - Log debug message
- `logger.info(msg: string)` - Log info message
- `logger.warn(msg: string)` - Log warning message
- `logger.error(msg: string)` - Log error message

**Status**: ✅ Fully implemented

### Time Module

[14 lines code: ```vex]

**Available Functions**:

- `time.now(): i64` - Get current Unix timestamp (seconds)
- `time.high_res(): i64` - Get high-resolution time (nanoseconds)
- `time.sleep_ms(ms: i64)` - Sleep for milliseconds

**Status**: ✅ Fully implemented

### Testing Module

[16 lines code: ```vex]

**Available Functions**:

- `testing.assert(condition: bool)` - Assert condition is true
- `testing.assert_eq<T>(a: T, b: T)` - Assert values are equal
- `testing.assert_ne<T>(a: T, b: T)` - Assert values are not equal

**Status**: ✅ Fully implemented

---

## Layer 1: I/O Core

### io

Basic input/output operations:

``````vex
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

[11 lines code: ```vex]

**Status**: ✅ Memory operations (alloc, free, realloc) implemented as builtins

### unsafe

Unsafe operations:

``````vex
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

``````vex
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

``````vex
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

``````vex
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

[12 lines code: ```vex]

**Primitives**:

- `Mutex<T>` - Mutual exclusion
- `RwLock<T>` - Read-write lock
- `Semaphore` - Counting semaphore
- `Barrier` - Thread barrier
- `WaitGroup` - Go-style wait group

**Status**: 🚧 Planned (Layer 2)

### testing

Testing framework:

[11 lines code: ```vex]

**Assertions**:

- `assert(condition)` - Basic assertion
- `assert_eq(a, b)` - Equality assertion
- `assert_ne(a, b)` - Inequality assertion
- `assert_lt(a, b)` - Less than
- `assert_gt(a, b)` - Greater than

**Status**: 🚧 Planned (Layer 2)

### datetime

Date and time operations:

``````vex
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

[15 lines code: ```vex]

**Client API**:

- `get(url: string): (Response | Error)`
- `post(url: string, body: string): (Response | Error)`
- `put(url: string, body: string): (Response | Error)`
- `delete(url: string): (Response | Error)`

**Server API** (Future):

``````vex
let server = http::Server::new();
server.route("/", handle_root);
server.listen(8080);
```

**Status**: 🚧 Planned (Layer 3)

### json

JSON parsing and serialization:

[16 lines code: ```vex]

**API**:

- `parse(s: string): (Value | Error)`
- `stringify(v: Value): string`
- `Value` enum: Object, Array, String, Number, Bool, Null

**Status**: 🚧 Planned (Layer 3)

### xml

XML parsing:

``````vex
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

``````vex
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

[11 lines code: ```vex]

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

[44 lines code: (unknown)]

### Implementation Status

• Layer — Modules — Status — Completion
• ------- — ---------------------------- — -------------- — ----------
• Layer 3 — http, json, xml, yaml — 🚧 Planned — 0%
• Layer 2 — net, sync, testing, datetime — 🚧 Planned — 5%
• Layer 1 — io, ffi, unsafe, hpc, libc — ✅ Partial — 60%
• Layer 0 — Vex Runtime (Rust) — ✅ Implemented — 80%

**Overall**: ~45% complete (builtins + I/O + FFI + unsafe working)

---

## Usage Examples

### Hello World

``````vex
import { println } from "io";

fn main(): i32 {
    println("Hello, World!");
    return 0;
}
```

### Reading Input

``````vex
import { println, readln } from "io";

fn main(): i32 {
    println("Enter your name:");
    let name = readln();
    println("Hello, " + name + "!");
    return 0;
}
```

### HTTP Request (Future)

[16 lines code: ```vex]

### JSON Parsing (Future)

[18 lines code: ```vex]

### Concurrency (Future)

[21 lines code: ```vex]

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

**Previous**: \1 
**Back to**: \1

**Maintained by**: Vex Language Team 
**Location**: `vex-libs/std/`
