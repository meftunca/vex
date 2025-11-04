# Modules and Imports

**Version:** 0.9.0  
**Last Updated:** November 3, 2025

This document defines the module system and import/export mechanism in Vex.

---

## Table of Contents

1. [Module System](#module-system)
2. [Import Statements](#import-statements)
3. [Export Declarations](#export-declarations)
4. [Module Resolution](#module-resolution)
5. [Standard Library Modules](#standard-library-modules)

---

## Module System

### File-Based Modules

Each `.vx` file is a module:

```vex
// file: math.vx
fn add(x: i32, y: i32): i32 {
    return x + y;
}

export fn multiply(x: i32, y: i32): i32 {
    return x * y;
}
```

**Properties**:

- One module per file
- File name becomes module name
- Private by default (use `export` keyword)

### Module Paths

Standard library modules:

```
vex-libs/std/
├── io/
│   ├── mod.vx          # Main module
│   ├── file.vx         # Submodule
│   └── stream.vx       # Submodule
├── net/
│   ├── mod.vx
│   ├── http.vx
│   └── tcp.vx
└── ...
```

**Import Path**: `"io"` → `vex-libs/std/io/mod.vx`

---

## Import Statements

### Basic Import with Alias

**Syntax**: `import * as alias from "module";`

```vex
import * as io from "io";

fn main(): i32 {
    io.println("Hello");
    return 0;
}
```

### Named Imports

**Syntax**: `import { name1, name2 } from "module";`

```vex
import { println, readln } from "io";

fn main(): i32 {
    println("Hello");
    let input = readln();
    return 0;
}
```

### Import Nested Modules

```vex
import * as http from "net/http";
import { TcpStream } from "net/tcp";
import { parse } from "json";

fn main(): i32 {
    let response = http.get("https://example.com");
    return 0;
}
```

### Multiple Named Imports

```vex
import { println } from "io";
import { get, post } from "net/http";
import { parse, stringify } from "json";
```

### Wildcard Import (Discouraged)

```vex
import * from "io";
// Imports all exported names directly (not recommended)
```

---

## Export Declarations

### Export Keyword (v0.9)

Make declarations public:

```vex
// Private function (not exported)
fn internal_helper(): i32 {
    return 42;
}

// Public function (exported)
export fn public_api(): i32 {
    return internal_helper();
}
```

### Export Structs

```vex
export struct Point {
    pub x: i32,  // Public field
    pub y: i32,
    private_data: i32,  // Private field
}
```

### Export Enums

```vex
export enum Status {
    Active,
    Inactive,
    Pending,
}
```

### Export Traits

```vex
export trait Display {
    fn (self: &Self!) show();
}
```

### Export Constants

```vex
export const MAX_SIZE: i32 = 1024;
export const VERSION: string = "0.9.0";
```

### Re-exports

```vex
// Re-export from another module
import { helper } from "internal";
export { helper };

// Or directly
export { helper } from "internal";
```

---

## Module Resolution

### Resolution Process

1. **Parse import path**: `"io"` → `["io"]`
2. **Locate module**: `vex-libs/std/io/mod.vx`
3. **Load and parse**: Parse `.vx` file
4. **Merge AST**: Combine with main program
5. **Resolve symbols**: Link function calls

### Standard Library Path

**Base Path**: `vex-libs/std/`

**Examples**:

- `"io"` → `vex-libs/std/io/mod.vx`
- `"net/http"` → `vex-libs/std/net/http.vx`
- `"collections"` → `vex-libs/std/collections/mod.vx`
- `"net/tcp"` → `vex-libs/std/net/tcp.vx`

### Module Loader

**Implementation**: `ModuleResolver` in `vex-compiler/src/module_resolver.rs`

**Process**:

```rust
fn resolve_import(path: &str) -> Result<Program, String> {
    let file_path = convert_to_path(path);
    let source = read_file(file_path)?;
    let ast = parse(source)?;
    Ok(ast)
}
```

---

## Standard Library Modules

### Layer 0: Vex Runtime (Rust)

Core runtime written in Rust:

- `io_uring` integration
- Async scheduler
- Memory allocator
- System calls

### Layer 1: I/O Core (Unsafe Bridge)

Low-level operations (100% Vex with `unsafe`):

```vex
// vex-libs/std/io/mod.vx
export fn print(s: string) {
    @libc::printf(s);
}

export fn read_file(path: string): string {
    // FFI to libc
}
```

**Modules**:

- `"io"` - Basic I/O
- `"ffi"` - Foreign function interface
- `"unsafe"` - Unsafe operations
- `"hpc"` - High-performance computing

### Layer 2: Protocol Layer (100% Safe Vex)

Safe abstractions:

```vex
// vex-libs/std/net/mod.vx
export struct TcpStream {
    handle: i32,
}

export fn connect(addr: string): TcpStream {
    // Safe wrapper around unsafe operations
}
```

**Modules**:

- `"net"` - Networking base
- `"net/tcp"` - TCP operations
- `"net/udp"` - UDP operations
- `"sync"` - Synchronization
- `"testing"` - Test framework

### Layer 3: Application Layer (100% Safe Vex)

High-level APIs:

```vex
// vex-libs/std/net/http.vx
export fn get(url: string): (Response | Error) {
    // HTTP client implementation
}
```

**Modules**:

- `"net/http"` - HTTP client/server
- `"json"` - JSON parsing
- `"xml"` - XML parsing

---

## Examples

### Basic Import

```vex
import * as io from "io";

fn main(): i32 {
    io.println("Hello, World!");
    return 0;
}
```

### Named Imports

```vex
import { println, readln } from "io";

fn main(): i32 {
    println("Enter your name:");
    let name = readln();
    println("Hello, " + name);
    return 0;
}
```

### Multiple Modules

```vex
import { println } from "io";
import * as http from "net/http";
import { parse } from "json";

fn main(): i32 {
    println("Starting server");
    let server = http.Server.new();
    server.listen(8080);
    return 0;
}
```

### Creating a Module

```vex
// file: utils.vx
export fn add(x: i32, y: i32): i32 {
    return x + y;
}

export fn multiply(x: i32, y: i32): i32 {
    return x * y;
}

fn internal_helper(): i32 {
    // Not exported, private
    return 42;
}
```

```vex
// file: main.vx
import { add, multiply } from "utils";

fn main(): i32 {
    let sum = add(10, 20);
    let product = multiply(5, 6);
    return sum + product;
}
```

---

## Best Practices

### 1. Explicit Imports

```vex
// Good: Explicit named imports
import { println, readln } from "io";

// Bad: Wildcard import without alias
import * from "io";

// Good alternative: Import with alias
import * as io from "io";
```

### 2. Module Organization

```vex
// Good: Hierarchical structure
import { TcpStream } from "net/tcp";
import { get } from "net/http";
import { UdpSocket } from "net/udp";

// Bad: Trying to use :: syntax
import "std::http";  // ❌ Wrong!
```

### 3. Minimal Exports

```vex
// Good: Only export public API
export fn public_function();
fn private_helper();

// Bad: Export everything
export fn public_function();
export fn internal_implementation();
```

### 4. Clear Module Names

```vex
// Good: Clear, descriptive paths
import { HashMap } from "collections/hashmap";
import { parse } from "json";

// Bad: Using std:: prefix
import "std::json";  // ❌ Wrong!
```

---

## Module System Summary

| Feature               | Status             | Example                          |
| --------------------- | ------------------ | -------------------------------- |
| **Named Import**      | ✅ Working         | `import { println } from "io"`   |
| **Import with Alias** | ✅ Working         | `import * as io from "io"`       |
| **Export**            | ✅ v0.9            | `export fn name()`               |
| **Module Resolution** | ✅ Working         | Loads from `vex-libs/std/`       |
| **Nested Modules**    | ✅ Working         | `import { get } from "net/http"` |
| **Re-exports**        | 🚧 Partial         | `export { x } from "mod"`        |
| **Private Items**     | ✅ Working         | Default (no export)              |
| **Public Fields**     | 🚧 Future          | `pub x: i32`                     |
| **Relative Imports**  | ❌ Not implemented | `import "./local"`               |
| **Package System**    | ❌ Not implemented | No package manager               |

---

**Previous**: [13_Concurrency.md](./13_Concurrency.md)  
**Next**: [15_Standard_Library.md](./15_Standard_Library.md)

**Maintained by**: Vex Language Team
