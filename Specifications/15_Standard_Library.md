# Standard Library

**Version:** 0.9.0  
**Last Updated:** November 3, 2025

This document provides an overview of the Vex standard library organization and API reference.

---

## Table of Contents

1. [Library Architecture](#library-architecture)
2. [Layer 1: I/O Core](#layer-1-io-core)
3. [Layer 2: Protocol Layer](#layer-2-protocol-layer)
4. [Layer 3: Application Layer](#layer-3-application-layer)
5. [Module Reference](#module-reference)

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
│  Layer 0: Vex Runtime (Rust)                   │
│  io_uring, async scheduler, allocator          │
└─────────────────────────────────────────────────┘
```

### Design Principles

1. **Safety by default**: Layers 2 and 3 are 100% safe Vex code
2. **Unsafe isolation**: All `unsafe` code contained in Layer 1
3. **Zero-cost abstractions**: No runtime overhead
4. **Composable**: Layers build on each other

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

**Status**: 🚧 Basic functions implemented

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

**Status**: 🚧 Planned

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

**Status**: ❌ Not implemented

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

**Status**: 🚧 Basic bindings available

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
├── io/              ✅ Basic (Layer 1)
│   ├── mod.vx       - print, println, readln
│   ├── file.vx      - File I/O (planned)
│   └── stream.vx    - Stream operations (planned)
├── ffi/             🚧 Planned (Layer 1)
│   └── mod.vx       - FFI declarations
├── unsafe/          ❌ Not implemented (Layer 1)
│   └── mod.vx       - Unsafe operations
├── hpc/             🚧 Planned (Layer 1)
│   ├── simd.vx      - SIMD operations
│   └── gpu.vx       - GPU primitives
├── libc/            🚧 Basic (Layer 1)
│   └── mod.vx       - libc bindings
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
├── testing/         🚧 Planned (Layer 2)
│   └── mod.vx       - Test framework
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
└── collections/     ❌ Not implemented
    ├── vec.vx       - Dynamic array
    ├── hashmap.vx   - Hash map
    └── ...
```

### Implementation Status

| Layer   | Modules                      | Status         | Completion |
| ------- | ---------------------------- | -------------- | ---------- |
| Layer 3 | http, json, xml, yaml        | ❌ Not started | 0%         |
| Layer 2 | net, sync, testing, datetime | 🚧 Planned     | 0%         |
| Layer 1 | io, ffi, unsafe, hpc, libc   | 🚧 Partial     | 20%        |
| Layer 0 | Vex Runtime (Rust)           | 🚧 Basic       | 30%        |

**Overall**: ~15% complete

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
