# Vex Standard Library (std)

**Version:** 0.2.0  
**Status:** Early Development  
**Last Updated:** November 3, 2025

The Vex standard library follows a layered architecture for maximum safety and performance.

## 🏗️ Architecture Overview

```
Layer 3: Application (100% Safe Vex)
├── std::http        - HTTP client/server (TODO)
├── std::json        - JSON parsing (TODO)
└── std::xml         - XML parsing (TODO)

Layer 2: Protocol & Collections (Safe Vex)
├── std::collections - ✅ HashMap, Set, BTree
├── std::array       - ✅ Array utilities
├── std::string      - ✅ String manipulation
├── std::net         - TCP/UDP networking (TODO)
├── std::sync        - Concurrency primitives (TODO)
└── std::testing     - Test framework (TODO)

Layer 1: Core Utilities (Safe Vex wrapping builtins)
├── std::mem         - ✅ Memory operations
├── std::intrinsics  - ✅ LLVM intrinsics
├── std::reflect     - ✅ Type reflection
├── std::io          - File I/O (TODO)
├── std::fs          - Filesystem (TODO)
└── std::time        - Time/Date (TODO)

Layer 0: Builtins (Compiler-level, LLVM IR)
└── 53 builtin functions (see builtins/)
```

---

## 📦 Implemented Modules

### ✅ std::collections - Data Structures

**Status:** HashMap implemented

```vex
import { HashMap } from "std::collections";

let! map = HashMap::new();
map.insert("name", "Alice");

if map.contains_key("name") {
    println("Found!");
}

map.free();
```

### ✅ std::array - Array Operations

**Status:** 14 core functions

```vex
import { array } from "std";

let! numbers = [1, 2, 3, 4, 5];
println(array::sum_i32(numbers));
array::reverse(numbers);
```

### ✅ std::string - String Utilities

**Status:** Core + UTF-8 support

```vex
import { string } from "std";

let text = "Hello 👋 World";
println(string::char_count(text)); // Unicode-aware
```

### ✅ std::mem - Memory Management

**Status:** Safe wrappers complete

```vex
import { mem } from "std";

let ptr = mem::allocate(1024);
mem::zero(ptr, 1024);
mem::deallocate(ptr);
```

### ✅ std::intrinsics - LLVM Intrinsics

**Status:** Bit ops + overflow checking

```vex
import { intrinsics } from "std";

let (result, overflow) = intrinsics::add_with_overflow_i32(x, y);
```

### ✅ std::reflect - Type Reflection

**Status:** RTTI complete

```vex
import { reflect } from "std";

println(reflect::type_name(x));    // "i32"
println(reflect::size_of(x));      // 4
```

---

## 📊 Implementation Status

| Module          | Status | Functions | Description          |
| --------------- | ------ | --------- | -------------------- |
| **collections** | ✅     | 10        | HashMap (SwissTable) |
| **array**       | ✅     | 14        | Array utilities      |
| **string**      | ✅     | 12        | String + UTF-8       |
| **mem**         | ✅     | 9         | Memory ops           |
| **intrinsics**  | ✅     | 20        | LLVM intrinsics      |
| **reflect**     | ✅     | 9         | Type reflection      |
| **io**          | ❌     | -         | File I/O (planned)   |
| **fs**          | ❌     | -         | Filesystem (planned) |
| **time**        | ❌     | -         | Time/Date (planned)  |

**Total:** 74 functions across 6 modules  
**Coverage:** ~30% of planned stdlib

---

## Philosophy

**99% of std is 100% safe native Vex code.**

All complexity is isolated in small "core" packages (std::io, std::hpc) that use `unsafe` code and runtime intrinsics. Everything above these layers is pure, safe Vex.

## Module Documentation

### std::io - File and I/O Operations

```vex
import { io } from "std";

// Read entire file
let content = await io.read_to_string("data.txt");

// Write to file
await io.write_string("output.txt", "Hello, World!");

// Manual file operations
let file = await io.open("data.bin");
let! buffer = make([byte], 1024);
let n = await file.read(&buffer!);
await file.close();
```

**Key Types:**

- `File`: File descriptor wrapper
- `Reader`: Trait for anything that can read bytes
- `Writer`: Trait for anything that can write bytes

### std::net - Network Programming

```vex
import { net } from "std";

// TCP client
let! conn = await net.connect("example.com", 80);
await conn.write("GET / HTTP/1.1\r\n\r\n".as_bytes());
let! buf = make([byte], 4096);
let n = await conn.read(&buf!);
await conn.close();

// TCP server
let listener = await net.listen("0.0.0.0", 8080);
loop {
    let! client = await listener.accept();
    // Handle client...
}

// UDP
let socket = await net.bind_udp("0.0.0.0", 9000);
await socket.send_to(&data, "192.168.1.100:9000");
```

### std::http - HTTP Client and Server

```vex
import { http } from "std";

// Simple GET request
let resp = await http.get("http://api.example.com/data");
print(f"Status: {resp.status_code}");
print(resp.body);

// POST request
let body = '{"name": "John", "age": 30}';
let resp = await http.post(
    "http://api.example.com/users",
    body,
    "application/json"
);

// HTTP Server
let server = await http.serve("0.0.0.0", 8080);
await server.handle_request(fn(req) {
    return http.Response {
        status_code: 200,
        headers: [],
        body: "Hello from Vex!",
    };
});
```

### std::sync - Concurrency

```vex
import { sync } from "std";

// Mutex for shared state
let! counter = sync.new_mutex(0);
go {
    let val = counter.lock();
    *val = *val + 1;
    counter.unlock();
};

// Channel for message passing
let! ch = sync.new_channel<i32>(10);
go {
    await ch.send(42);
};
let value = await ch.recv();

// WaitGroup for coordination
let! wg = sync.new_waitgroup();
wg.add(3);
for i := 0; i < 3; i++ {
    go {
        // Do work...
        wg.done();
    };
}
await wg.wait();
```

### std::testing - Test Framework

```vex
import { testing } from "std";

fn test_addition(t: &testing.TestContext) {
    t.assert_eq(2 + 2, 4, "addition works");
    t.assert(true, "always passes");
}

fn main() -> i32 {
    let! suite = testing.new_suite("My Tests");
    suite.add_test("addition", test_addition);
    let (passed, failed) = suite.run();
    return if failed > 0 { 1 } else { 0 };
}
```

### std::hpc - GPU and SIMD

```vex
import { hpc } from "std";

// GPU kernel
fn add_kernel(a: &[f32], b: &[f32], c: &[f32]!, n: i32) {
    let idx = hpc.thread_idx().x;
    if idx < n {
        c[idx] = a[idx] + b[idx];
    }
}

// Launch on GPU
launch add_kernel(&a, &b, &c, n) {
    grid: (256, 1, 1),
    block: (256, 1, 1),
};
await hpc.gpu_sync();

// Parallel CPU loop
await hpc.parallel_for(0, 1000, fn(i) {
    // This runs in parallel on multiple CPU cores
    process(i);
});
```

### std::unsafe - Low-Level Operations

```vex
import { unsafe } from "std";

// Raw pointers (use with caution!)
let x = 42;
let ptr = unsafe.ptr(&x);
let value = unsafe.deref(ptr);

// Memory operations
let src = [1, 2, 3, 4];
let! dest = make([i32], 4);
unsafe.copy(&dest[0] as *i32!, &src[0] as *i32, 4);

// Atomic operations
let! counter = 0;
unsafe.atomic_add(&counter!, 1);
```

### std::ffi - Foreign Function Interface

```vex
import { ffi } from "std";

// Call C function
extern "C" {
    fn strlen(s: *byte) -> u32;
}

let msg = ffi.to_c_string("Hello!");
let len = unsafe { strlen(msg.ptr) };
ffi.free_c_string(msg);

// Load dynamic library
let lib = ffi.load_library("libmath.so");
let func = ffi.get_symbol(lib, "calculate");
```

## Examples

See `examples/` directory for complete programs:

- `http_client.vx` - HTTP GET request
- `concurrent_channels.vx` - Producer/consumer with channels
- `gpu_vector_add.vx` - GPU-accelerated computation
- `test_suite.vx` - Unit testing example

## Implementation Status

✅ **Complete (as library code):**

- Module structure and organization
- API design following specification
- Example programs

🚧 **TODO (requires compiler support):**

- String type implementation in codegen
- `import` statement parsing
- `trait` keyword and implementation
- `async`/`await` codegen
- `go` keyword for task spawning
- `launch` keyword for GPU kernels
- Runtime intrinsic linking
- Error type and `match` expressions
- Generic type parameters `<T>`

## Design Principles

1. **Layered abstraction**: Upper layers never know about lower layer internals
2. **Safe by default**: 99% of code is safe, unsafe isolated in core modules
3. **Zero-cost abstractions**: High-level APIs compile to optimal code
4. **Explicit async**: All I/O is async, no hidden blocking
5. **Batteries included**: Common tasks have stdlib support

## Contributing

When adding new std modules:

1. Identify correct layer (1, 2, or 3)
2. Use only safe Vex if Layer 2 or 3
3. Document all public APIs
4. Add examples to demonstrate usage
5. Write tests using std::testing
