# Vex Standard Library Package Guide

**Version:** 0.1.2  
**Last Updated:** November 11, 2025  
**Target Audience:** Vex Stdlib Developers & AI Agents

---

## 📋 Table of Contents

1. [Overview](#overview)
2. [Package Architecture](#package-architecture)
3. [File Structure Standards](#file-structure-standards)
4. [Code Organization](#code-organization)
5. [Contract & Struct Best Practices](#contract--struct-best-practices)
6. [Multi-File Module Patterns](#multi-file-module-patterns)
7. [Common Issues & Solutions](#common-issues--solutions)
8. [Quality Checklist](#quality-checklist)
9. [Examples from Production](#examples-from-production)

---

## Overview

This guide defines the **canonical structure** for Vex standard library packages. It ensures:

- ✅ Clean, maintainable multi-file organization
- ✅ Correct trait/struct/function usage
- ✅ Consistent API design across stdlib
- ✅ Predictable build integration
- ✅ Easy debugging and testing

### Design Principles

1. **Separation of Concerns**: Vex API vs C implementation
2. **Single Responsibility**: One module, one purpose
3. **Explicit Exports**: Public API clearly defined
4. **Type Safety**: Traits enforce contracts
5. **Zero Overhead**: Direct FFI, no unnecessary abstractions

---

## Package Architecture

### Standard Package Layout

```
std/
└── module_name/
    ├── vex.json              # Package manifest (REQUIRED)
    ├── README.md             # Module documentation
    ├── src/
    │   ├── lib.vx            # Main entrypoint (REQUIRED)
    │   ├── types.vx          # Type definitions (if complex)
    │   ├── contracts.vx     # Contract definitions (if reusable)
    │   └── utils.vx          # Internal helpers (if needed)
    ├── native/               # C implementation (optional)
    │   └── src/
    │       ├── vex_module.c  # Native implementation
    │       └── vex_module.h  # Header
    ├── tests/
    │   ├── basic_test.vx     # Unit tests
    │   └── integration_test.vx
    └── examples/
        └── demo.vx           # Usage examples
```

### Key Files

#### 1. `vex.json` - Package Manifest (REQUIRED)

**Purpose**: Define package metadata, dependencies, and build configuration.

**Template**:

```json
{
  "name": "module_name",
  "version": "0.2.0",
  "description": "Brief description of the module",
  "authors": ["Vex Team"],
  "license": "MIT",
  "repository": "https://github.com/meftunca/vex",

  "dependencies": {},

  "main": "src/lib.vx",

  "testing": {
    "dir": "tests",
    "pattern": "**/*.test.vx"
  },

  "targets": {
    "default": "x64",
    "supported": ["x64", "arm64", "wasm", "wasi"]
  },

  "native": {
    "sources": ["native/src/vex_module.c"],
    "libraries": [],
    "search_paths": [],
    "static_libs": [],
    "cflags": ["-O3", "-Wall", "-fPIC", "-std=c11"],
    "include_dirs": ["native/src", "../../../vex-runtime/c"]
  }
}
```

**Fields**:

- `name`: Module name (matches directory name)
- `version`: Semantic version (e.g., "0.2.0")
- `main`: Entry point (default: `src/lib.vx`)
- `testing`: Test configuration
  - `dir`: Test directory (default: `"tests"`)
  - `pattern`: Test file pattern (default: `"*.test.vx"`)
  - `timeout`: Test timeout in seconds (optional)
  - `parallel`: Run tests in parallel (default: `true`)
- `targets`: Platform support configuration
  - `default`: Default target platform
  - `supported`: List of supported platforms
- `native.sources`: C source files (if using FFI)
- `native.libraries`: System libraries (e.g., `["m"]` for math)
- `native.search_paths`: Library search directories
- `native.static_libs`: Static library files (.a)
- `native.cflags`: C compiler flags
- `native.include_dirs`: Header include directories

**Common Native Configurations**:

```json
// Pure Vex module (no FFI)
{
  "native": {}
}

// System library linking only
{
  "native": {
    "libraries": ["m"],
    "search_paths": ["/usr/local/lib"]
  }
}

// Custom C implementation
{
  "native": {
    "sources": ["native/src/vex_module.c"],
    "cflags": ["-O3", "-Wall", "-fPIC"],
    "include_dirs": ["native/src", "../../../vex-runtime/c"]
  }
}

// Static library linking
{
  "native": {
    "static_libs": ["vendor/libcustom.a"],
    "include_dirs": ["vendor/include"]
  }
}
```

#### 2. `src/lib.vx` - Main Entrypoint (REQUIRED)

**Purpose**: Define public API and orchestrate module functionality.

**Structure**:

```vex
// Module: module_name
// Description: Brief description
//
// Usage:
//   import { function_name, TypeName } from "module_name";

// ============================================================================
// EXTERNAL C FUNCTIONS (if using FFI)
// ============================================================================

extern "C" {
    fn vex_module_function(arg: i32): i32;
}

// ============================================================================
// CONSTANTS
// ============================================================================

export const MODULE_VERSION: str = "0.2.0";
export const MAX_SIZE: i32 = 1024;

// ============================================================================
// TYPE DEFINITIONS
// ============================================================================

export struct DataType {
    field1: i32,
    field2: str,
}

// ============================================================================
// TRAITS
// ============================================================================

export contract Processable {
    fn process(): i32;
}

// ============================================================================
// PUBLIC FUNCTIONS
// ============================================================================

export fn public_api_function(x: i32): i32 {
    return vex_module_function(x);
}

// ============================================================================
// INTERNAL HELPERS (no export)
// ============================================================================

fn internal_helper(): i32 {
    return 42;
}
```

---

## File Structure Standards

### Rule 1: File Size Limit

**CRITICAL**: All Rust files MUST NOT exceed **400 lines**.

**Vex files**: Aim for **< 500 lines** per file. If larger, split logically.

### Rule 2: File Naming Conventions

| File Type              | Naming Pattern             | Example            |
| ---------------------- | -------------------------- | ------------------ |
| **Main entrypoint**    | `lib.vx`                   | `src/lib.vx`       |
| **Platform-specific**  | `lib.{os}.vx`              | `src/lib.macos.vx` |
| **Arch-specific**      | `lib.{arch}.vx`            | `src/lib.x64.vx`   |
| **Type definitions**   | `types.vx`                 | `src/types.vx`     |
| **Trait definitions**  | `traits.vx`                | `src/traits.vx`    |
| **Internal utilities** | `utils.vx` or `helpers.vx` | `src/utils.vx`     |
| **Submodules**         | `{feature}.vx`             | `src/iterator.vx`  |

### Rule 3: Platform-Specific Files (Only When Needed)

**⚠️ IMPORTANT**: Platform-specific files (`lib.{os}.vx`) should **ONLY** be created when the module has **actual platform differences**. Most modules can use a single `lib.vx` file.

**When to use platform-specific files:**

- ✅ Different syscalls (e.g., `kqueue` vs `epoll`)
- ✅ OS-specific APIs (e.g., Windows registry)
- ✅ Platform-specific optimizations

**When NOT to use:**

- ❌ Cross-platform FFI (use conditional compilation in C instead)
- ❌ Simple modules without OS dependencies
- ❌ "Just in case" - only when actually needed

**Priority Chain** (when platform files exist):

1. `lib.{os}.{arch}.vx` (e.g., `lib.macos.arm64.vx`)
2. `lib.{arch}.vx` (e.g., `lib.arm64.vx`)
3. `lib.{os}.vx` (e.g., `lib.macos.vx`)
4. `lib.vx` (generic fallback)

**Example** (only when truly needed):

```
fs/src/
├── lib.vx           # Generic implementation (ALWAYS REQUIRED)
├── lib.macos.vx     # macOS-specific (ONLY if using kqueue)
└── lib.linux.vx     # Linux-specific (ONLY if using epoll)
```

---

## Code Organization

### Module Sections (Recommended Order)

1. **Header Comment**: Module purpose, usage examples
2. **Extern C Declarations**: FFI bindings (if any)
3. **Constants**: Module-level constants
4. **Type Definitions**: Structs, enums
5. **Traits**: Behavior contracts
6. **Trait Implementations**: Struct + trait combinations
7. **Public Functions**: Exported API
8. **Internal Functions**: Helpers (no `export`)

### Example: Well-Organized Module

```vex
// ============================================================================
// Module: math
// Mathematical operations with hardware acceleration support
//
// Usage:
//   import { sin, cos, sqrt, PI } from "math";
//   let y = sin(PI / 2.0); // 1.0
// ============================================================================

// ============================================================================
// EXTERNAL C FUNCTIONS
// ============================================================================

extern "C" {
    fn sin(x: f64): f64;
    fn cos(x: f64): f64;
    fn sqrt(x: f64): f64;
}

// ============================================================================
// CONSTANTS
// ============================================================================

export const PI: f64 = 3.14159265358979323846;
export const E: f64 = 2.71828182845904523536;

// ============================================================================
// PUBLIC FUNCTIONS
// ============================================================================

export fn sin_f64(x: f64): f64 {
    return sin(x);
}

export fn cos_f64(x: f64): f64 {
    return cos(x);
}

export fn sqrt_f64(x: f64): f64 {
    return sqrt(x);
}
```

---

## Contract & Struct Best Practices

### Trait Definition Rules

**✅ DO:**

```vex
// Correct: Trait defines contract
contract Logger {
    fn log(msg: str);         // Immutable method
    fn clear()!;              // Mutable method

    fn info(msg: str) {       // Default method
        self.log("[INFO] " + msg);
    }
}
```

**❌ DON'T:**

```vex
// WRONG: Contract methods cannot have bodies (except defaults)
contract Logger {
    fn log(msg: str) {        // ERROR: Non-default with body
        print(msg);
    }
}
```

### Struct + Contract Implementation

**✅ CORRECT (Inline Implementation):**

```vex
struct ConsoleLogger impl Logger {
    prefix: str,

    // ALL contract methods MUST be implemented here
    fn log(msg: str) {
        print(self.prefix, ": ", msg);
    }

    fn clear()! {
        // Mutable method implementation
    }

    // info() inherited from contract default
}
```

**❌ WRONG (External Implementation):**

```vex
// ERROR: Contract methods cannot be external
fn (logger: &ConsoleLogger) log(msg: str) {
    print(msg);
}
```

### Method Mutability - Hybrid Model

Vex uses **two syntaxes** depending on context:

#### Syntax 1: Inline Methods (in `struct` or `contract`)

**Immutable:**

```vex
struct Point {
    x: i32,
    y: i32,

    fn distance(): f64 {
        return sqrt(self.x * self.x + self.y * self.y);
    }
}
```

**Mutable:**

```vex
struct Counter {
    value: i32,

    fn increment()! {
        self.value = self.value + 1;
    }
}
```

**Call:**

```vex
let! counter = Counter { value: 0 };
counter.increment();  // No '!' at call site
```

#### Syntax 2: External Methods (Golang-Style)

**Immutable:**

```vex
fn (p: &Point) area(): i32 {
    return p.x * p.y;
}
```

**Mutable:**

```vex
fn (c: &Counter!) reset() {
    c.value = 0;
}
```

**Call:**

```vex
let! counter = Counter { value: 10 };
counter.reset();  // No '!' at call site
```

### Contract Methods vs Extra Methods

| Method Type          | Location               | Syntax          |
| -------------------- | ---------------------- | --------------- |
| **Contract Methods** | MUST be in struct body | Inline only     |
| **Extra Methods**    | Can be external        | Golang-style OK |

**Example**:

```vex
contract Shape {
    fn area(): f64;
}

struct Rectangle impl Shape {
    width: f64,
    height: f64,

    // Trait method - MUST be inline
    fn area(): f64 {
        return self.width * self.height;
    }
}

// Extra method - CAN be external
fn (rect: &Rectangle) diagonal(): f64 {
    return sqrt(rect.width * rect.width + rect.height * rect.height);
}
```

---

## Multi-File Module Patterns

### Pattern 1: Simple Module (Single File)

**Use Case**: < 500 lines, single responsibility

**Structure**:

```
io/
├── vex.json
└── src/
    └── lib.vx
```

**Example**: `io` module (basic print/println)

### Pattern 2: Modular Split (Multiple Files)

**Use Case**: > 500 lines, logical separation

**Structure**:

```
collections/
├── vex.json
└── src/
    ├── lib.vx          # Re-exports
    ├── hashmap.vx      # HashMap implementation
    └── hashset.vx      # HashSet implementation
```

**`lib.vx` (Hub Pattern)**:

```vex
// Re-export pattern
import { HashMap, new_hashmap } from "hashmap";
import { HashSet, new_hashset } from "hashset";

export { HashMap, new_hashmap, HashSet, new_hashset };
```

### Pattern 3: Type-Contract Split

**Use Case**: Complex traits, reusable across types

**Structure**:

```
testing/
├── vex.json
└── src/
    ├── lib.vx        # Main API
    ├── traits.vx     # Testable, Benchmarkable traits
    └── types.vx      # T, B structs
```

**`contracts.vx`**:

```vex
export contract Testable {
    fn error(msg: str)!;
    fn skip(msg: str)!;
    fn log(msg: str);
    fn failed(): bool;
}

export contract Benchmarkable {
    fn reset_timer()!;
    fn start_timer()!;
    fn stop_timer()!;
}
```

**`types.vx`**:

```vex
import { Testable, Benchmarkable } from "contracts";

export struct T impl Testable {
    name: str,
    failed: bool,

    fn error(msg: str)! {
        self.failed = true;
    }

    // ... other methods
}
```

**`lib.vx`**:

```vex
import { Testable, Benchmarkable } from "traits";
import { T, B } from "types";

export { Testable, Benchmarkable, T, B };
```

### Pattern 4: Platform-Specific Impl (Rare)

**Use Case**: OS-specific syscalls (ONLY when truly needed)

**⚠️ Note**: Most modules do NOT need this pattern. Only use when:

- Different system calls required per OS
- Platform-specific features (Windows registry, macOS keychain, etc.)

**Structure**:

```
process/
├── vex.json
└── src/
    ├── lib.vx         # Generic fallback (REQUIRED)
    ├── lib.macos.vx   # macOS implementation (optional)
    └── lib.linux.vx   # Linux implementation (optional)
```

**`lib.macos.vx`**:

```vex
extern "C" {
    fn getpid(): i32;
    fn getppid(): i32;
}

export fn pid(): i32 {
    return getpid();
}

export fn ppid(): i32 {
    return getppid();
}
```

---

## Common Issues & Solutions

### Issue 1: String Move Semantics

**Problem**: Strings are moved, not copied.

**Symptoms**:

```vex
let path = "/tmp/file.txt";
write_string(path, "data");  // OK
write_string(path, "more");  // ERROR: path already moved
```

**Solution 1: Use string literals**

```vex
write_string("/tmp/file.txt", "data");
write_string("/tmp/file.txt", "more");  // OK
```

**Solution 2: Clone strings (when available)**

```vex
let path = "/tmp/file.txt";
write_string(path.clone(), "data");
write_string(path.clone(), "more");
```

**Solution 3: Pass by reference (FFI workaround)**

```vex
fn write_string_ref(path: &str, content: &str): bool {
    // FFI with pointers, no move
}
```

**Status**: Borrow checker improvements planned for v0.2.0.

### Issue 2: Trait Methods Must Be Inline

**Problem**: Attempting to implement trait methods externally.

**Symptoms**:

```vex
trait Logger {
    fn log(msg: str);
}

struct ConsoleLogger impl Logger {
    prefix: str,
}

// ERROR: Trait method cannot be external
fn (logger: &ConsoleLogger) log(msg: str) {
    print(msg);
}
```

**Solution**: Implement ALL trait methods in struct body.

```vex
struct ConsoleLogger impl Logger {
    prefix: str,

    fn log(msg: str) {
        print(self.prefix, ": ", msg);
    }
}
```

### Issue 3: Missing Trait Methods

**Problem**: Implementing only some trait methods.

**Symptoms**:

```vex
trait Shape {
    fn area(): f64;
    fn perimeter(): f64;
}

// ERROR: Missing perimeter()
struct Circle impl Shape {
    radius: f64,

    fn area(): f64 {
        return 3.14 * self.radius * self.radius;
    }
}
```

**Solution**: Implement ALL required trait methods.

```vex
struct Circle impl Shape {
    radius: f64,

    fn area(): f64 {
        return 3.14 * self.radius * self.radius;
    }

    fn perimeter(): f64 {
        return 2.0 * 3.14 * self.radius;
    }
}
```

### Issue 4: Export Enum Not Parsed

**Problem**: Parser didn't recognize `export enum`.

**Status**: ✅ FIXED (November 8, 2025)

**Before**:

```vex
// ERROR: Unexpected token
export enum Color { Red, Green, Blue }
```

**After**:

```vex
// ✅ Now works
export enum Color { Red, Green, Blue }
```

**Fix**: Added `Token::Enum` to `parse_export()` in `vex-parser/src/parser/items/exports.rs`.

### Issue 5: FFI Type Mismatches

**Problem**: Vex type doesn't match C type.

**Symptoms**:

```vex
extern "C" {
    fn strlen(s: *u8): i32;  // WRONG: strlen returns size_t
}
```

**Solution**: Use correct Vex-C type mapping.

**Type Mapping**:

| C Type   | Vex Type         |
| -------- | ---------------- |
| `int`    | `i32`            |
| `long`   | `i64`            |
| `size_t` | `u64`            |
| `char*`  | `*u8`            |
| `void*`  | `*u8` or `*void` |
| `bool`   | `bool`           |
| `float`  | `f32`            |
| `double` | `f64`            |

**Corrected**:

```vex
extern "C" {
    fn strlen(s: *u8): u64;  // ✅ Correct
}
```

### Issue 6: Circular Dependencies

**Problem**: Module A imports B, B imports A.

**Symptoms**:

```
Error: Circular dependency detected: A -> B -> A
```

**Solution 1: Extract Common Interface**

```vex
// common.vx
export contract Common {
    fn shared_method(): i32;
}

// a.vx
import { Common } from "common";

// b.vx
import { Common } from "common";
```

**Solution 2: Forward Declarations (Future)**

Not yet supported in Vex v0.1.2.

### Issue 7: File Size > 400 Lines (Rust Compiler)

**Problem**: Rust compiler files exceed 400-line limit.

**Symptoms**:

```
vex-compiler/src/codegen_ast/mod.rs: 1247 lines (VIOLATION)
```

**Solution**: Split into logical submodules.

**Before**:

```
vex-compiler/src/
└── codegen_ast/
    └── mod.rs (1247 lines)
```

**After**:

```
vex-compiler/src/
└── codegen_ast/
    ├── mod.rs (exports)
    ├── types.rs (< 400 lines)
    ├── functions.rs (< 400 lines)
    └── statements.rs (< 400 lines)
```

---

## Quality Checklist

### Before Committing New Module

- [ ] `vex.json` exists and is valid
- [ ] `src/lib.vx` is the main entrypoint
- [ ] All public API uses `export` keyword
- [ ] Trait methods are inline in struct body
- [ ] All trait methods implemented (no missing methods)
- [ ] File size < 500 lines (or logically split)
- [ ] No circular dependencies
- [ ] FFI types match C types
- [ ] Constants use `SCREAMING_SNAKE_CASE`
- [ ] Functions use `snake_case`
- [ ] Types use `PascalCase`
- [ ] Header comment explains module purpose
- [ ] Usage examples in header or README.md
- [ ] Tests exist in `tests/` directory
- [ ] Test files follow `*.test.vx` naming pattern
- [ ] Examples exist in `examples/` directory
- [ ] No string move issues (use literals or references)
- [ ] Platform-specific code uses `lib.{os}.vx` pattern
- [ ] Native code (if any) listed in `vex.json`

### Documentation Requirements

- [ ] README.md with:
  - Module overview
  - API reference table
  - Usage examples
  - Known limitations
- [ ] Inline comments for complex logic
- [ ] Extern C functions documented with purpose
- [ ] Trait contracts clearly defined

### Testing Requirements

- [ ] Unit tests for all public functions
- [ ] Integration tests for common workflows
- [ ] Platform-specific tests (if applicable)
- [ ] Edge case coverage (nil, empty, overflow)

---

## Examples from Production

### Example 1: `io` Module (Simple)

**Structure**:

```
io/
├── vex.json
└── src/
    └── lib.vx (45 lines)
```

**Key Features**:

- Single file
- Extern C FFI
- Simple API (print, println, eprint, eprintln)
- No traits, no complex types

**`lib.vx`**:

```vex
extern "C" {
    fn vex_print(ptr: *u8, len: u64);
    fn vex_println(ptr: *u8, len: u64);
}

export fn print(s: String) {
    let ptr: *String = &s;
    let data: *u8 = vex_string_as_cstr(ptr);
    let length: u64 = vex_string_len(ptr);
    vex_print(data, length);
}

export fn println(s: String) {
    let ptr: *String = &s;
    let data: *u8 = vex_string_as_cstr(ptr);
    let length: u64 = vex_string_len(ptr);
    vex_println(data, length);
}
```

**Lessons**:

- ✅ Clean FFI wrapper
- ✅ Minimal abstraction
- ✅ Well-documented

### Example 2: `math` Module (Moderate)

**Structure**:

```
math/
├── vex.json
└── src/
    └── lib.vx (219 lines)
```

**Key Features**:

- Extern C to libm
- Constants (PI, E)
- Type-specific wrappers (f32 vs f64)
- Pure Vex helpers (min, max, clamp)

**Sections**:

1. Extern C declarations (trigonometry, exponentials)
2. Constants (PI, E, PHI)
3. Exported wrappers (sin_f64, cos_f64)
4. Pure Vex utilities (abs_i32, min_f64, clamp_f64)

**Lessons**:

- ✅ Clear section separation
- ✅ Type-safe wrappers
- ✅ Mix of FFI + pure Vex

### Example 3: `collections` Module (Complex)

**Structure**:

```
collections/
├── vex.json
└── src/
    ├── lib.vx (re-exports)
    ├── hashmap.vx
    └── hashset.vx
```

**Key Features**:

- Hub pattern (lib.vx re-exports)
- Generic types (HashMap<K,V>, HashSet<T>)
- Multiple constructors (new, with_capacity)

**`lib.vx` (Hub)**:

```vex
import { HashMap, new_hashmap, with_capacity_hashmap } from "hashmap";
import { HashSet, new_hashset, with_capacity_hashset } from "hashset";

export {
    HashMap, new_hashmap, with_capacity_hashmap,
    HashSet, new_hashset, with_capacity_hashset
};
```

**Lessons**:

- ✅ Clean separation (HashMap separate from HashSet)
- ✅ Re-export pattern for unified API
- ✅ Generic type support

### Example 4: `testing` Module (Trait-Heavy)

**Structure**:

```
testing/
├── vex.json
└── src/
    └── testing.vx (168 lines)
```

**Key Features**:

- Trait definitions (Testable, Benchmarkable)
- Struct implementations (T impl Testable, B impl Benchmarkable)
- External methods (Golang-style)
- High-level API (run_test, run_benchmark)

**Trait Definition**:

```vex
trait Testable {
    fn error(msg: String)!;
    fn skip(msg: String)!;
    fn log(msg: String);
    fn failed(): bool;
}
```

**Struct Implementation**:

```vex
export struct T impl Testable {
    name: String,
    failed: bool,
    skipped: bool,
    logs: Vec<String>,

    fn error(msg: String)! {
        self.failed = true;
    }

    fn skip(msg: String)! {
        self.skipped = true;
    }

    fn log(msg: String) {
        // Log implementation
    }

    fn failed(): bool {
        return self.failed;
    }
}
```

**External Methods** (extra, non-trait):

```vex
export fn (self:&T!) error(msg: String)! {
    self.failed = true;
}
```

**Lessons**:

- ✅ Traits define contracts
- ✅ Inline trait implementations
- ✅ External methods for extras
- ✅ High-level API wraps low-level

### Example 5: `fs` Module (Comprehensive)

**Structure**:

```
fs/
├── vex.json
├── README_COMPREHENSIVE.md
├── src/
│   ├── lib.vx (basic, 112 lines)
│   └── lib_comprehensive.vx (460 lines, 50+ functions)
├── native/
│   └── src/
│       ├── vex_fs.c
│       └── vex_fs.h
└── examples/
    └── comprehensive_demo.vx
```

**Key Features**:

- FFI to vex-runtime C functions
- 50+ exported functions
- Go-inspired API (path_join, create_dir_all)
- Memory-mapped file support
- Platform abstractions (path separators)

**Sections**:

1. Extern C declarations (90 lines)
2. File operations (8 functions)
3. Directory operations (5 functions)
4. Path manipulation (9 functions)
5. Path queries (6 functions)
6. Permissions (5 functions)
7. Symlinks (2 functions)
8. Temp files (2 functions)
9. Pattern matching (1 function)
10. Memory-mapped files (6 functions)

**Lessons**:

- ✅ Comprehensive API design
- ✅ Logical grouping (comments as section headers)
- ✅ Cross-platform abstractions
- ✅ Zero-copy support (mmap)
- ⚠️ Watch file size (460 lines, approaching limit)

---

## Summary

### Golden Rules

1. **One module, one purpose** - Don't mix concerns
2. **Traits in struct bodies** - No external trait methods
3. **Export everything public** - Clear API surface
4. **< 500 lines per file** - Split if larger
5. **vex.json is mandatory** - Build system requires it
6. **Test everything** - Unit + integration tests
7. **Document usage** - README + header comments
8. **Platform-specific code** - Use `lib.{os}.vx` pattern

### Anti-Patterns to Avoid

- ❌ Monolithic files (> 500 lines)
- ❌ External trait method implementations
- ❌ Missing trait method implementations
- ❌ Circular dependencies
- ❌ FFI type mismatches
- ❌ String moves without cloning
- ❌ Missing exports on public API
- ❌ Undocumented extern C functions

### Best Practices

- ✅ Use hub pattern for multi-file modules
- ✅ Separate types, traits, and implementations
- ✅ Inline trait methods, external extra methods
- ✅ Group related functions with comment headers
- ✅ Use string literals to avoid move issues
- ✅ Match FFI types precisely
- ✅ Write usage examples in headers
- ✅ Test on multiple platforms

---

**Maintained by**: Vex Language Team  
**Reference Modules**: `io`, `math`, `fs`, `collections`, `testing`  
**Compiler Version**: Vex v0.1.2
