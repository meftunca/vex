# Vex Language - Introduction and Overview

**Version:** 0.1.2  
**Status:** Living Specification  
**Last Updated:** November 2025

---

## What is Vex?

Vex is a modern systems programming language that combines:

- **Rust's Safety**: Memory safety without garbage collection through a borrow checker
- **Go's Simplicity**: Clean syntax, easy concurrency with goroutines
- **TypeScript's Expressiveness**: Advanced type system with generics, unions, and intersections

## Design Philosophy

### 1. Safety First

- Compile-time memory safety through borrow checking
- No null pointer dereferences
- No data races
- No use-after-free bugs

### 2. Simplicity and Clarity

- Explicit over implicit (e.g., `let!` for mutable variables)
- Clear error messages
- Minimal cognitive overhead

### 3. Performance

- Zero-cost abstractions
- Direct compilation to native code via LLVM
- **Automatic Vectorization**: SIMD/GPU acceleration without manual intervention
- **Intelligent Lane Chunking**: Automatic workload distribution

### 4. Modern Features

- First-class concurrency with goroutines and async/await
- Pattern matching with exhaustiveness checking
- Contract-based polymorphism
- Powerful generics system
- Policy-based metadata system
- Comprehensive error handling
- Foreign function interface
- Full tooling ecosystem (LSP, formatter, package manager)

## Key Features

### 🚀 Unique Feature: Automatic Vectorization

**The most important feature of Vex**: Transparent SIMD/GPU acceleration for array operations.

```vex
// User writes simple scalar operations
let a: [f32; 1000] = [...];
let b: [f32; 1000] = [...];
let c = a + b;  // Automatically vectorized!

// Compiler automatically:
// 1. Detects vector operation
// 2. Chunks into optimal lane sizes (4, 8, 16 elements)
// 3. Uses SIMD instructions (SSE, AVX, AVX-512)
// 4. Falls back to GPU if available and beneficial
```

**Supported Operations** (auto-vectorized):

- Arithmetic: `+`, `-`, `*`, `/`, `%`
- Comparison: `==`, `!=`, `<`, `>`, `<=`, `>=`
- Logical: `&&`, `||`, `!`
- Math functions: `sqrt()`, `sin()`, `cos()`, `abs()`

**No Manual Annotation Required**:

- ✅ Write: `let result = vector_a * vector_b;`
- ❌ No need: `@vectorize`, `#pragma`, or special syntax (automatic)

**Intelligent Backend Selection**:

- Small vectors (< 256 elements): SIMD (SSE/AVX)
- Large vectors (> 1024 elements): GPU if available, otherwise SIMD
- Automatic lane chunking for optimal memory bandwidth

### Type System

- **Primitive Types**: i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, f16, f32, f64, bool, string, error
- **Vector Types**: `[T; N]` - Fixed-size arrays with auto-vectorization support
- **Compound Types**: Arrays, tuples, slices, references
- **Collections**: Map<K,V>, Set<T>, Vec<T>, Box<T>, Channel<T>
- **User-Defined Types**: Structs, enums, type aliases
- **Advanced Types**: ✅ Union types `(T | U)` (v0.1.2), intersection types, conditional types
- **Option/Result**: Builtin Some/None, Ok/Err constructors with `?` operator (v0.1.2)
- **Policy System**: Metadata annotations with inheritance and composition
- **Reflection**: typeof, type_id, is_int_type, is_float_type, is_pointer_type (runtime type information)

### Memory Management

- **Borrow Checker**: Compile-time ownership and borrowing analysis
  - ✅ Phase 1: Immutability checking
  - ✅ Phase 2: Move semantics
  - ✅ Phase 3: Borrow rules (1 mutable XOR N immutable references)
  - ✅ Phase 4: Lifetime analysis (v0.1.2) - prevents dangling references
- **No Garbage Collection**: Deterministic memory management
- **Defer Statement**: Go-style resource cleanup (executes on function exit)
- **Smart Pointers**: Box<T> (implemented), Rc, Arc (planned)

### Concurrency

- **Goroutines**: Lightweight concurrent execution with `go` keyword (basic runtime)
- **Async/Await**: Structured asynchronous programming (implemented)
- **Channels**: MPSC message passing with Channel<T> (fully implemented)
- **Defer**: Go-style LIFO cleanup on function exit (fully implemented)
- **Select Statement**: Multiplexing on channel operations (keyword reserved)

### Pattern Matching

- Exhaustive matching with `match` expressions
- ✅ Struct destructuring `Point { x, y }` (v0.1.2)
- Enum variant matching with data extraction
- Tuple and struct destructuring
- OR patterns with SIMD optimization
- Guard clauses for conditional matching (implemented)
- Range patterns with `..` and `..=` (implemented)
- Switch statements for integer matching (implemented)

### Contracts and Interfaces

- Contract-based polymorphism
- Multiple contract implementation
- Default contract methods (implemented)
- Contract inheritance with supercontracts (implemented)
- Contract bounds on generics (partial)

### Policy System

- **Metadata Annotations**: Struct-level policies for serialization, validation, etc.
- **Policy Inheritance**: Parent policy composition with `extends` keyword
- **Policy Application**: `with` clause for applying policies to structs
- **Builtin Policies**: Debug, Serializable, Clone, etc.

### Error Handling

- **Result Type**: `Result<T, E>` for recoverable errors
- **Option Type**: `Option<T>` for optional values
- **Pattern Matching**: Exhaustive error handling with match expressions
- **Error Propagation**: Early return with error values

### Foreign Function Interface (FFI)

- **Raw Pointers**: `*T` and `*T!` for unsafe memory access
- **Extern Declarations**: `extern "C"` blocks for C function imports
- **Type Mapping**: Automatic conversion between Vex and C types
- **Memory Layout**: Compatible struct layouts for interop

### Package Management

- **vex-pm**: Full-featured package manager with dependency resolution
- **MVS Algorithm**: Minimal Version Selection for conflict-free dependencies
- **Platform Detection**: Automatic cross-platform binary selection
- **Lock Files**: Deterministic builds with vex.lock
- **Registry Support**: Package publishing and discovery

### Tooling

- **Language Server Protocol (LSP)**: Full IDE support with completion, diagnostics, goto definition
- **Code Formatter (vex-formatter)**: Automatic code formatting with configurable rules
- **Package Manager (vex-pm)**: Dependency management and project scaffolding

## Syntax Highlights (v0.1.1)

### Variable Declaration

```vex
let x = 42;              // Immutable (default)
let! counter = 0;        // Mutable (explicit with !)
const MAX_SIZE = 1000;   // Compile-time constant
```

### References

```vex
&T      // Immutable reference
&T!     // Mutable reference (v0.1 syntax, not &mut T)
```

### Functions

```vex
fn add(a: i32, b: i32): i32 {
    return a + b;
}
```

### Structs with Contracts

```vex
struct Point impl Display, Eq {
    x: i32,
    y: i32,

    fn (self: &Point) show() {
        // Display contract method
    }
}
```

### Methods (Golang-Style)

```vex
fn (p: &Point) distance(): i32 {
    return p.x + p.y;
}
```

### Pattern Matching

```vex
match value {
    1 | 2 | 3 => { /* OR patterns */ }
    x if x > 10 => { /* Guard clause */ }
    _ => { /* Wildcard */ }
}
```

### Control Flow

```vex
if condition {
    // ...
} elif other_condition {
    // ...
} else {
    // ...
}
```

### Auto-Vectorization Examples

```vex
// Simple vector addition - automatically uses SIMD
let a: [f32; 8] = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
let b: [f32; 8] = [8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0];
let result = a + b;  // Vectorized to 2x AVX operations (4 lanes each)

// Vector multiplication
let scaled = a * 2.5;  // Broadcast + SIMD multiply

// Element-wise operations
let dot_product = (a * b).sum();  // SIMD multiply + horizontal add

// Large arrays automatically use GPU if available
let big_a: [f32; 10000] = [...];
let big_b: [f32; 10000] = [...];
let big_result = big_a + big_b;  // GPU kernel dispatch if beneficial
```

### Defer Statement (Go-style)

```vex
fn read_file(path: string): string {
    let file = open(path);
    defer close(file);  // Executes when function returns

    // Multiple defer statements execute in reverse order (LIFO)
    defer println("Cleanup 2");
    defer println("Cleanup 1");

    if error {
        return "";  // defer still runs before return
    }

    return read_content(file);
}  // defer statements execute here: "Cleanup 1", "Cleanup 2", close(file)
```

## Compilation Model

### Compiler Pipeline

1. **Lexer** (vex-lexer): Tokenization with Logos
2. **Parser** (vex-parser): Recursive descent parsing → AST
3. **Borrow Checker** (vex-compiler): Multi-phase ownership analysis
4. **Code Generation** (vex-compiler): LLVM IR emission via Inkwell
5. **Linking** (vex-cli): Native executable generation

### Build Artifacts

```
Source (.vx) → AST → Borrow Check → LLVM IR → Object File (.o) → Executable
```

### Standard Library

- Layered architecture from unsafe I/O to safe abstractions
- Located in `vex-libs/std/`
- Modules: io, net, sync, http, json, etc.
- **Builtin Functions** (implemented):
  - **Memory Operations**: alloc, free, realloc, sizeof, alignof, memcpy, memset, memcmp, memmove
  - **String Operations**: strlen, strcmp, strcpy, strcat, strdup
  - **UTF-8 Support**: utf8_valid, utf8_char_count, utf8_char_at
  - **Type Reflection**: typeof, type_id, type_size, type_align, is_int_type, is_float_type, is_pointer_type
  - **LLVM Intrinsics**: ctlz, cttz, ctpop, bswap, bitreverse (bit manipulation), sadd_overflow, ssub_overflow, smul_overflow (overflow checking)
  - **Compiler Hints**: assume, likely, unlikely, prefetch (optimization hints)
  - **Stdlib Modules**: logger._, time._, testing.\* (accessible via import with dot notation)

## Platform Support

### Current

- **Tier 1**: Linux x86_64, macOS x86_64/ARM64

### Planned

- Windows x86_64
- WebAssembly
- Embedded targets

## Development Status

### Completed Features (v0.1)

- ✅ Core type system with extended integer/float types (i128, u128, f16)
- ✅ Functions and methods (both inline and golang-style)
- ✅ Borrow checker (Phases 1-3 complete, Phase 4 in progress)
- ✅ Closures and lambda expressions with capture mode analysis
- ✅ Pattern matching with OR patterns and guards
- ✅ Contracts with default methods and multiple inheritance
- ✅ Generics with monomorphization
- ✅ Control flow (if/elif/else, while, for, match, switch)
- ✅ Reference expressions (&expr, \*ptr) with &T! syntax
- ✅ Async runtime with goroutines and channels (COMPLETE - full MPSC implementation)
- ✅ Language Server Protocol (LSP) implementation
- ✅ Comprehensive standard library builtins
- ✅ Policy system with metadata annotations and inheritance
- ✅ Package manager (vex-pm) with dependency resolution
- ✅ Code formatter (vex-formatter) with configurable rules
- ✅ Error handling with Result/Option types and `?` operator (v0.1.2)
- ✅ Foreign Function Interface (FFI) with raw pointers
- ✅ Union types with tagged union implementation (v0.1.2)
- ✅ Struct pattern matching and destructuring (v0.1.2)
- ✅ Lifetime analysis Phase 4 - complete borrow checker (v0.1.2)

### In Progress

- 🚧 Intersection types for contract composition

### Planned

- 📋 Dynamic dispatch (vtables)
- 📋 Async runtime with io_uring
- 📋 GPU kernel compilation
- 📋 Macro system
- 📋 Advanced optimizations

## Test Coverage

**Current Status**: Comprehensive test suite with extensive examples

**Test Categories**:

- Basics: Variables, types, operators
- Functions: Recursion, methods, generics, closures
- Control Flow: If/elif/else, while, for, match, switch
- Types: Structs, enums, tuples, references
- Generics: Type parameters, monomorphization, contract bounds
- Patterns: Destructuring, OR patterns, guards, rest patterns
- Strings: F-strings, operations, UTF-8 support
- Algorithms: Fibonacci, factorial, GCD, sorting
- Contracts: Multiple contracts, default methods, inheritance
- Borrow Checker: Immutability, moves, borrows, closure capture
- Async: Goroutines, channels, async/await (MPSC channels complete)
- Builtins: Arrays, collections, I/O, time, testing framework
- Policies: Metadata annotations, inheritance, struct application
- FFI: Raw pointers, extern declarations, type mapping
- Packages: Dependency resolution, platform detection, lock files

## Example Programs

### Hello World

```vex
fn main(): i32 {
    return 0;
}
```

### Fibonacci

```vex
fn fib(n: i32): i32 {
    if n <= 1 {
        return n;
    }
    return fib(n - 1) + fib(n - 2);
}

fn main(): i32 {
    return fib(10);  // Returns 55
}
```

## Getting Started

### Installation

```bash
git clone https://github.com/meftunca/vex
cd vex
cargo build --release
```

### Running Examples

```bash
~/.cargo/target/release/vex run examples/01_basics/hello_world.vx
~/.cargo/target/release/vex compile examples/08_algorithms/fibonacci.vx
```

### Documentation Structure

This specification is organized into the following documents:

1. **Introduction and Overview** (this document)
2. **Lexical Structure** - Tokens, identifiers, literals, comments
3. **Type System** - Primitive types, compound types, user-defined types
4. **Variables and Constants** - Declaration, mutability, shadowing
5. **Functions and Methods** - Definition, calls, receivers, generics
6. **Control Flow** - If, match, switch, loops
7. **Structs and Data Types** - Definition, instantiation, methods
8. **Enums** - Unit enums, data-carrying variants
9. **Contracts and Interfaces** - Definition, implementation, inheritance
10. **Generics** - Type parameters, constraints, monomorphization
11. **Pattern Matching** - Patterns, destructuring, guards
12. **Closures and Lambda Expressions** - Anonymous functions, capture modes
13. **Memory Management** - Ownership, borrowing, lifetimes
14. **Concurrency** - Goroutines, async/await, channels
15. **Modules and Imports** - Module system, imports, exports
16. **Standard Library** - Core modules and APIs
17. **Error Handling** - Result/Option types, error propagation
18. **Raw Pointers and FFI** - Unsafe operations, foreign function interface
19. **Package Manager** - vex-pm, dependency management, publishing
20. **Policy System** - Metadata annotations, inheritance, composition

---

## Comparison with Rust and Go

This section documents features available in Rust and Go but not yet implemented in Vex (v0.1.0).

### Features Rust Has (Vex Doesn't Yet)

#### Language Features

| Feature                             | Rust                           | Vex v0.1               | Notes                                |
| ----------------------------------- | ------------------------------ | ---------------------- | ------------------------------------ |
| **Closures/Lambdas**                | ✅ `\|x\| x + 1`               | ✅ Complete            | Full capture mode analysis           |
| **Lifetime Annotations**            | ✅ `'a, 'static`               | ✅ Automatic (Phase 4) | Borrow checker handles automatically |
| **Contract Objects**                | ✅ `&dyn Contract`             | ❌ Not implemented     | Dynamic dispatch pending             |
| **Async/Await Runtime**             | ✅ Full tokio support          | ✅ Complete            | Core async runtime implemented       |
| **Macros**                          | ✅ Declarative + Procedural    | ❌ Not implemented     | Low priority                         |
| **Const Generics**                  | ✅ `[T; N]`                    | ❌ Not implemented     | Array size flexibility               |
| **Higher-Ranked Contract Bounds**   | ✅ `for<'a>`                   | ❌ Not implemented     | Advanced feature                     |
| **Associated Constants**            | ✅ `const X: i32;`             | ❌ Not implemented     | Contract-level constants             |
| **Drop Contract**                   | ✅ RAII destructors            | ❌ Not implemented     | Resource cleanup                     |
| **Deref Coercion**                  | ✅ Automatic `&String → &str`  | ✅ Field access        | Auto-deref for field access complete |
| **Type Aliases in Contracts**       | ✅ `type Item = T;`            | ✅ Complete            | Associated types working             |
| **Unsafe Blocks**                   | ✅ `unsafe { }`                | ✅ Complete            | FFI integration working              |
| **Raw Pointers**                    | ✅ `*const T, *mut T`          | ✅ Complete            | Low-level operations working         |
| **Interior Mutability**             | ✅ `Cell<T>, RefCell<T>`       | ❌ Not implemented     | Advanced pattern                     |
| **Pattern Guards**                  | ✅ `Some(x) if x > 0`          | ✅ Complete            | Fully working                        |
| **Range Patterns**                  | ✅ `1..=10`                    | ✅ Complete            | .. and ..= operators                 |
| **Slice Patterns**                  | ✅ `[first, .., last]`         | ✅ Complete            | Rest patterns with `...rest`         |
| **Tuple Indexing**                  | ✅ `point.0`                   | ✅ Complete (v0.1.2)   | Numeric field access implemented     |
| **Impl Contract**                   | ✅ `fn f() -> impl Contract`   | ❌ Not implemented     | Return type flexibility              |
| **Existential Types**               | ✅ `type Foo = impl Contract;` | ❌ Not implemented     | Advanced feature                     |
| **GATs (Generic Associated Types)** | ✅ Stable                      | ❌ Not implemented     | Complex generics                     |

#### Standard Library & Ecosystem

| Feature                    | Rust                           | Vex v0.1             | Notes                                   |
| -------------------------- | ------------------------------ | -------------------- | --------------------------------------- |
| **Collections**            | ✅ Vec, HashMap, HashSet, etc. | ✅ Implemented       | Vec, Map, Set, Box                      |
| **Iterators**              | ✅ Full Iterator contract      | ✅ Complete          | Basic iteration working                 |
| **Option Type**            | ✅ `Option<T>`                 | ✅ Complete          | Some/None constructors                  |
| **Result Type**            | ✅ `Result<T, E>`              | ✅ Complete          | Ok/Err constructors                     |
| **Error Handling**         | ✅ `?` operator                | ✅ Complete (v0.1.2) | Result unwrapping with auto-propagation |
| **String Slicing**         | ✅ `&str[0..5]`                | ❌ Not implemented   | String operations limited               |
| **Format Macro**           | ✅ `format!()`                 | ✅ F-strings         | F-string interpolation working          |
| **Testing Framework**      | ✅ Built-in testing            | ✅ Basic framework   | Builtin testing module                  |
| **Documentation Comments** | ✅ `///` and `//!`             | ❌ Not implemented   | No doc generation                       |
| **Attributes**             | ✅ `#[derive(Debug)]`          | ❌ NOT IN VEX        | Vex uses `@intrinsic` only              |
| **Cargo Equivalent**       | ✅ Cargo package manager       | ✅ vex-pm            | Full dependency management              |
| **Crates.io Equivalent**   | ✅ Package registry            | ❌ Not implemented   | No ecosystem yet                        |

#### Tooling

| Feature                     | Rust             | Vex v0.1           | Notes                      |
| --------------------------- | ---------------- | ------------------ | -------------------------- |
| **Language Server**         | ✅ rust-analyzer | ✅ vex-lsp         | Full LSP support           |
| **Formatter**               | ✅ rustfmt       | ✅ vex-formatter   | Configurable formatting    |
| **Linter**                  | ✅ clippy        | 🚧 Planned         | No static analysis         |
| **Package Manager**         | ✅ cargo         | ✅ vex-pm          | Full dependency management |
| **Documentation Generator** | ✅ rustdoc       | ❌ Not implemented | No auto-docs               |
| **Benchmark Framework**     | ✅ criterion     | ❌ Not implemented | No benchmarking            |

### Features Go Has (Vex Doesn't Yet)

#### Language Features

| Feature                        | Go                               | Vex v0.1                      | Notes                                   |
| ------------------------------ | -------------------------------- | ----------------------------- | --------------------------------------- |
| **Goroutines**                 | ✅ `go func()`                   | ✅ Basic runtime              | Core goroutine runtime implemented      |
| **Channels**                   | ✅ `make(chan T)`                | ✅ MPSC channels              | Multi-producer single-consumer          |
| **Select Statement**           | ✅ Multi-channel wait            | 🚧 Keyword reserved           | Channels working, select syntax pending |
| **Defer Statement**            | ✅ `defer cleanup()`             | ✅ Fully working              | Go-style LIFO execution                 |
| **Auto-Vectorization**         | ❌ Manual SIMD                   | ✅ Automatic                  | **Unique to Vex**                       |
| **Interface Satisfaction**     | ✅ Implicit                      | ✅ Explicit `impl`            | Contract-based design                   |
| **Type Embedding**             | ✅ Anonymous fields              | ❌ Not implemented            | Composition pattern                     |
| **Type Assertions**            | ✅ `x.(Type)`                    | ❌ Not implemented            | Runtime type checking                   |
| **Type Switches**              | ✅ `switch x.(type)`             | ❌ Not implemented            | Type-based matching                     |
| **Variadic Functions**         | ✅ `func f(args ...T)`           | ❌ Not implemented            | Flexible parameters                     |
| **Multiple Return Values**     | ✅ `func f() (T, error)`         | ✅ Tuples                     | Same capability, different syntax       |
| **Named Return Values**        | ✅ `func f() (x int, err error)` | ❌ Not implemented            | Convenience feature                     |
| **Init Functions**             | ✅ `func init()`                 | ❌ Not implemented            | Package initialization                  |
| **Blank Identifier**           | ✅ `_` for unused                | ✅ In match and destructuring | Pattern matching wildcard               |
| **Short Variable Declaration** | ✅ `:=` operator                 | ❌ Removed in v0.1            | Use `let` instead                       |
| **Pointer Arithmetic**         | ✅ Via unsafe package            | ❌ Not implemented            | Low-level operations                    |

#### Standard Library

| Feature                 | Go                        | Vex v0.1             | Notes                                 |
| ----------------------- | ------------------------- | -------------------- | ------------------------------------- |
| **HTTP Server**         | ✅ `net/http`             | 🚧 Planned (Layer 3) | std lib incomplete                    |
| **File I/O**            | ✅ `os.File`              | ✅ Basic I/O         | File operations working               |
| **Goroutine Scheduler** | ✅ Built-in runtime       | ✅ Basic runtime     | Core goroutine runtime implemented    |
| **Garbage Collection**  | ✅ Concurrent GC          | ❌ Manual memory     | Design choice: no GC                  |
| **Reflection**          | ✅ `reflect` package      | ✅ Complete          | typeof, type*id, is*\*\_type builtins |
| **Context Package**     | ✅ Cancellation/timeout   | ❌ Not implemented   | Concurrency control                   |
| **Sync Package**        | ✅ Mutex, WaitGroup, etc. | 🚧 Planned (Layer 2) | std lib incomplete                    |
| **Testing Package**     | ✅ `testing`              | ✅ Basic framework   | testing module with assert functions  |
| **Database/SQL**        | ✅ `database/sql`         | ❌ Not implemented   | No DB drivers                         |
| **Template Engine**     | ✅ `text/template`        | ❌ Not implemented   | No templating                         |

#### Tooling & Ecosystem

| Feature               | Go                          | Vex v0.1                 | Notes                       |
| --------------------- | --------------------------- | ------------------------ | --------------------------- |
| **Go Modules**        | ✅ Built-in package manager | ✅ vex-pm                | Full dependency management  |
| **go fmt**            | ✅ Standard formatter       | ✅ vex-formatter         | Configurable formatting     |
| **go vet**            | ✅ Static analyzer          | 🚧 Planned               | Linting planned             |
| **go test**           | ✅ Built-in testing         | 🚧 Test framework exists | Runtime integration needed  |
| **go doc**            | ✅ Documentation viewer     | ❌ Not implemented       | No doc generation           |
| **pprof**             | ✅ Profiling tools          | ❌ Not implemented       | No profiling                |
| **race detector**     | ✅ `-race` flag             | ❌ Not implemented       | No race detection           |
| **Cross-compilation** | ✅ Easy GOOS/GOARCH         | ✅ LLVM targets          | Multi-platform LLVM backend |
| **Language Server**   | ✅ gopls                    | ✅ vex-lsp               | Full LSP implementation     |

### What Vex Has That's Unique

While Vex is missing many features, it combines aspects from both languages in novel ways:

| Feature                     | Vex Approach                 | Rust                  | Go                     |
| --------------------------- | ---------------------------- | --------------------- | ---------------------- |
| **Variable Mutability**     | `let` vs `let!`              | `let` vs `let mut`    | All mutable by default |
| **Mutable References**      | `&T!` syntax                 | `&mut T`              | All references mutable |
| **Method Syntax**           | Both inline and golang-style | Impl blocks only      | Receiver syntax only   |
| **Elif Keyword**            | ✅ Native `elif`             | `else if`             | `else if`              |
| **Contract Implementation** | `struct S impl T { }` inline | Separate `impl` block | Implicit satisfaction  |
| **Union Types**             | `(T \| U)` planned           | `enum` workaround     | `interface{}`          |
| **Intersection Types**      | `(T & U)` planned            | Contract bounds       | Not available          |
| **GPU Functions**           | `gpu fn` keyword             | Via compute crates    | Via CGO                |

### Roadmap Priority

**High Priority (Blocking Production Use)**:

1. ✅ Borrow Checker Phases 1-3 (COMPLETE)
2. 🟡 Phase 4: Lifetime Analysis (in progress)
3. ✅ Closures and lambdas (COMPLETE)
4. ✅ Option/Result types with pattern matching (COMPLETE)
5. ✅ Iterator contract and collection methods (builtin collections implemented)
6. ✅ Async runtime integration (COMPLETE - goroutines and channels)
7. ✅ Standard library completion (I/O, networking - extensive builtins added)

**Medium Priority (Developer Experience)**:

1. 🟡 Error handling (`?` operator)
2. ✅ Testing framework (builtin testing framework implemented)
3. ✅ Language server protocol (LSP) (COMPLETE)
4. ✅ Formatter (vex-formatter implemented)
5. ✅ Package manager (vex-pm implemented)
6. 🟡 Documentation generator

**Low Priority (Advanced Features)**:

1. 🟢 Macros (declarative)
2. ✅ Unsafe blocks and raw pointers (COMPLETE - FFI working)
3. ✅ Reflection and runtime type info (typeof, type*id, type_size, is*\*\_type builtins)
4. 🟢 Procedural macros
5. 🟢 Const generics

**By Design (Won't Implement)**:

- ❌ Garbage collection (manual memory management by design)
- ❌ Null pointers (use Option type instead)
- ❌ Exceptions (use Result type instead)
- ❌ Inheritance (use composition and contracts)
- ❌ Function overloading (use generics instead)

### Current Limitations

**Stability**: Vex is alpha software (v0.1.1). Core features stable, advanced features evolving.

**Test Coverage**: 97.9% - 278/284 tests passing. Core functionality extensively tested.

**Documentation**: Language spec is comprehensive and up-to-date with implementation.

**Ecosystem**: No third-party packages, no package registry yet (vex-pm infrastructure complete).

**IDE Support**: ✅ Language Server Protocol (LSP) implemented, VS Code extension available.

**Production Readiness**: ✅ **BETA** - Core features complete (100%), suitable for real projects with caution.

---

# Lexical Structure

**Version:** 0.1.0  
**Last Updated:** November 3, 2025

This document defines the lexical structure of the Vex programming language, including tokens, identifiers, literals, operators, and comments.

---

## Table of Contents

1. [Source Code Encoding](#source-code-encoding)
2. [Comments](#comments)
3. [Whitespace and Line Terminators](#whitespace-and-line-terminators)
4. [Identifiers](#identifiers)
5. [Keywords](#keywords)
6. [Operators and Punctuation](#operators-and-punctuation)
7. [Literals](#literals)
8. [Token Types](#token-types)

---

## Source Code Encoding

Vex source files:

- **File Extension**: `.vx`
- **Encoding**: UTF-8
- **Line Endings**: LF (`\n`) or CRLF (`\r\n`)
- **BOM**: Not required, but accepted if present

---

## Comments

Vex supports two types of comments that are ignored by the lexer:

### Line Comments

Begin with `//` and continue until the end of the line.

```vex
// This is a line comment
let x = 42; // Inline comment after code
```

### Block Comments

Begin with `/*` and end with `*/`. Can span multiple lines.

```vex
/*
 * This is a multi-line
 * block comment
 */

/* Inline block comment */ let y = 100;
```

**Note**: Block comments do not nest in the current implementation.

---

## Whitespace and Line Terminators

The following characters are considered whitespace and are skipped by the lexer:

- Space (U+0020)
- Tab (U+0009)
- Line Feed (U+000A)
- Form Feed (U+000C)

**Regex Pattern**: `[ \t\n\f]+`

Whitespace is used to separate tokens but is otherwise ignored.

---

## Identifiers

Identifiers are names for variables, functions, types, and other program entities.

### Syntax Rules

- **First Character**: Must be a letter (`a-z`, `A-Z`) or underscore (`_`)
- **Subsequent Characters**: Letters, digits (`0-9`), or underscores
- **Case Sensitive**: `myVar`, `MyVar`, and `myvar` are different identifiers

**Regex Pattern**: `[a-zA-Z_][a-zA-Z0-9_]*`

### Valid Identifiers

```vex
variable
_private
count_123
camelCase
snake_case
PascalCase
__double_underscore
```

### Invalid Identifiers

```vex
123start     // Cannot start with digit
my-var       // Hyphen not allowed
my.var       // Dot not allowed
fn           // Reserved keyword
```

### Naming Conventions

While not enforced by the compiler, the following conventions are recommended:

| Entity                 | Convention       | Example           |
| ---------------------- | ---------------- | ----------------- |
| Variables              | snake_case       | `user_count`      |
| Constants              | UPPER_SNAKE_CASE | `MAX_SIZE`        |
| Functions              | snake_case       | `calculate_total` |
| Types (Structs, Enums) | PascalCase       | `UserAccount`     |
| Contracts              | PascalCase       | `Serializable`    |
| Internal/Helper        | Prefix with `_`  | `_internal_fn`    |

---

## Keywords

Keywords are reserved identifiers with special meaning in the language.

### Control Flow Keywords

```
if          else        elif        for
while       in          match       switch
case        default     return      break
continue    defer
```

**Answer**: ✅ `for` keyword mevcut ve çalışıyor. 06_Control_Flow.md'de detaylı dokümante edilmiş.
**Answer**: ✅ `defer` keyword IMPLEMENTED! (Nov 9, 2025) - Go-style resource cleanup with LIFO execution.

### Declaration Keywords

```
fn          let         const       struct
enum        type        contract       impl
extern
```

**Answer**: ❌ `static` keyword eklemiyoruz. Rust'taki `static` yerine Vex'te `const` kullanılıyor. Global değişkenler için gelecekte düşünülebilir ama şu an öncelik değil.

### Type Keywords

```
i8          i16         i32         i64
u8          u16         u32         u64
f32         f64         bool        string
byte        error       nil
```

**Answer**:

- ❌ `void` - Zaten `nil` kullanıyoruz (unit type)
- 🟡 `i128/u128` - Gelecekte eklenebilir (crypto/big numbers için), şu an Low Priority
- ❌ `i256/u256` - Gerek yok, çok spesifik use case (blockchain)
- ❌ `f16/f8/f128` - LLVM desteği sınırlı, şu an öncelik değil. f32/f64 yeterli.

### Module Keywords

```
import      export      from        as
```

### Concurrency Keywords

```
async       await       go          gpu
launch      select
```

### Advanced Keywords

```
unsafe      new         make        try
extends     infer       interface
```

### Boolean Literals

```
true        false
```

**Total Reserved Keywords**: 66

### Deprecated Keywords

- `mut` - Removed in v0.1 (use `let!` for mutable variables)
- `interface` - Use `contract` instead

---

## Operators and Punctuation

### Arithmetic Operators

| Operator       | Symbol | Description     | Example |
| -------------- | ------ | --------------- | ------- |
| Addition       | `+`    | Add two values  | `a + b` |
| Subtraction    | `-`    | Subtract values | `a - b` |
| Multiplication | `*`    | Multiply values | `a * b` |
| Division       | `/`    | Divide values   | `a / b` |
| Modulo         | `%`    | Remainder       | `a % b` |

### Comparison Operators

| Operator         | Symbol | Description           |
| ---------------- | ------ | --------------------- |
| Equal            | `==`   | Equality test         |
| Not Equal        | `!=`   | Inequality test       |
| Less Than        | `<`    | Less than             |
| Less or Equal    | `<=`   | Less than or equal    |
| Greater Than     | `>`    | Greater than          |
| Greater or Equal | `>=`   | Greater than or equal |

### Logical Operators

| Operator    | Symbol | Description                 |
| ----------- | ------ | --------------------------- |
| Logical AND | `&&`   | Both conditions true        |
| Logical OR  | `\|\|` | At least one condition true |
| Logical NOT | `!`    | Negate condition            |

### Bitwise Operators (Future)

| Operator    | Symbol | Description |
| ----------- | ------ | ----------- |
| Bitwise AND | `&`    | Bitwise AND |
| Bitwise OR  | `\|`   | Bitwise OR  |
| Bitwise XOR | `^`    | Bitwise XOR |
| Left Shift  | `<<`   | Shift left  |
| Right Shift | `>>`   | Shift right |

### Assignment Operators

| Operator        | Symbol | Description         |
| --------------- | ------ | ------------------- |
| Assign          | `=`    | Assignment          |
| Add Assign      | `+=`   | Add and assign      |
| Subtract Assign | `-=`   | Subtract and assign |
| Multiply Assign | `*=`   | Multiply and assign |
| Divide Assign   | `/=`   | Divide and assign   |
| Modulo Assign   | `%=`   | Modulo and assign   |
| Bitwise AND     | `&=`   | AND and assign      |
| Bitwise OR      | `\|=`  | OR and assign       |
| Bitwise XOR     | `^=`   | XOR and assign      |
| Left Shift      | `<<=`  | Shift left assign   |
| Right Shift     | `>>=`  | Shift right assign  |

**Answer**: ✅ Bitwise assignment operators eklenmeli (Medium Priority 🟡). Bitwise operatörler zaten planned olduğu için bunlar da eklenecek.

**Answer**: ❌ Increment/Decrement (`++`/`--`) operatörleri eklenmeyecek. Açıkça `x = x + 1` veya `x += 1` kullanılmalı (Go ve Rust'ın yaklaşımı gibi). Belirsizliği önler (prefix vs postfix).

### Reference Operators

| Operator    | Symbol | Description         | Example |
| ----------- | ------ | ------------------- | ------- |
| Reference   | `&`    | Take reference      | `&x`    |
| Dereference | `*`    | Dereference pointer | `*ptr`  |
| Mutable Ref | `!`    | Mutable marker      | `&x!`   |

**Answer**: Stack için `&` reference, heap allocation için `new` keyword kullanılacak (future). Raw pointer için `unsafe` blok içinde manual allocation gerekecek.

**Answer**: ❌ `++`/`--` operatörleri desteklenmeyecek. Açık assignment kullanılmalı: `x += 1` veya `x -= 1`.

### Other Operators

| Operator      | Symbol | Description            |
| ------------- | ------ | ---------------------- |
| Member Access | `.`    | Access field or method |
| Range         | `..`   | Range operator         |
| Variadic      | `...`  | Variadic arguments     |
| Try           | `?`    | Error propagation      |
| Pipe          | `\|`   | OR pattern in match    |

**Answer**: 🟡 Spread/Rest operators (Medium Priority)

- `...arr` - Spread operator (array unpacking)
- Rest parameters in functions: `fn sum(...numbers: i32[])`
- Gelecekte eklenebilir ama şu an öncelik değil. JavaScript/TypeScript pattern'i.

### Delimiters

| Symbol  | Name        | Usage                            |
| ------- | ----------- | -------------------------------- |
| `(` `)` | Parentheses | Function calls, grouping, tuples |
| `{` `}` | Braces      | Blocks, struct literals          |
| `[` `]` | Brackets    | Arrays, indexing                 |
| `,`     | Comma       | Separate items                   |
| `;`     | Semicolon   | Statement terminator             |
| `:`     | Colon       | Type annotations                 |
| `_`     | Underscore  | Wildcard pattern                 |

### Special Symbols

| Symbol | Name      | Usage                             |
| ------ | --------- | --------------------------------- |
| `=>`   | Fat Arrow | Match arms, lambdas               |
| `@`    | At        | Intrinsics (`@vectorize`, `@gpu`) |

**Note**: Vex does NOT use Rust-style `#[attribute]` syntax. Attributes are not part of the language.

---

## Literals

### Integer Literals

Decimal integers without any prefix:

```vex
0           // Zero
42          // Positive integer
-100        // Negative integer (unary minus + literal)
```

**Type**: `i64` by default (can be inferred or explicitly typed)

**Regex Pattern**: `[0-9]+`

**Future Extensions**:

- Hexadecimal: `0xFF`, `0x1A2B`
- Octal: `0o77`, `0o644`
- Binary: `0b1010`, `0b11110000`
- Underscores: `1_000_000`, `0xFF_FF_FF`

### Floating-Point Literals

Decimal numbers with a decimal point:

```vex
0.0
3.14
2.71828
-0.5        // Negative (unary minus + literal)
```

**Type**: `f64` by default

**Regex Pattern**: `[0-9]+\.[0-9]+`

**Future Extensions**:

- Scientific notation: `1.5e10`, `2.0E-5`
- Type suffix: `3.14f32`, `2.0f64`

### Boolean Literals

```vex
true        // Boolean true
false       // Boolean false
```

**Type**: `bool`

### String Literals

Enclosed in double quotes with escape sequences:

```vex
"Hello, World!"
"Line 1\nLine 2"
"Tab\tseparated"
"Quote: \"Hello\""
"Backslash: \\"
```

**Type**: `string`

**Regex Pattern**: `"([^"\\]|\\["\\bnfrt]|u[a-fA-F0-9]{4})*"`

**Supported Escape Sequences**:

| Sequence | Meaning                           |
| -------- | --------------------------------- |
| `\"`     | Double quote                      |
| `\\`     | Backslash                         |
| `\n`     | Newline (LF)                      |
| `\r`     | Carriage return                   |
| `\t`     | Tab                               |
| `\b`     | Backspace                         |
| `\f`     | Form feed                         |
| `\uXXXX` | Unicode code point (4 hex digits) |

### F-String Literals (Interpolation)

Strings with embedded expressions, prefixed with `f`:

```vex
let name = "Alice";
let age = 30;
let greeting = f"Hello, {name}! You are {age} years old.";
```

**Type**: `string`

**Regex Pattern**: `f"([^"\\]|\\["\\bnfrt]|u[a-fA-F0-9]{4})*"`

**Note**: Current implementation parses f-strings but full interpolation support is in progress.

### Nil Literal

```vex
nil         // Represents absence of value
```

**Type**: Unit type (void)

### Struct Tags (Go-Style)

Metadata attached to struct fields, enclosed in backticks:

```vex
struct User {
    id: i64       `json:"id" db:"primary_key"`,
    name: string  `json:"name" validate:"required"`,
}
```

**Type**: Metadata (compile-time only)

**Regex Pattern**: `` `[^`]*` ``

---

## Token Types

### Token Categories

The lexer produces tokens in the following categories:

#### 1. Keywords (67 tokens)

- Control flow: `if`, `else`, `elif`, `for`, `while`, `match`, `switch`, etc.
- Declarations: `fn`, `let`, `const`, `struct`, `enum`, `contract`, `impl`
- Types: `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`, `f32`, `f64`, `bool`, `string`
- Concurrency: `async`, `await`, `go`, `gpu`
- Other: `import`, `export`, `return`, `nil`, `true`, `false`

#### 2. Operators (37 tokens)

- Arithmetic: `+`, `-`, `*`, `/`, `%`
- Comparison: `==`, `!=`, `<`, `<=`, `>`, `>=`
- Logical: `&&`, `||`, `!`
- Assignment: `=`, `+=`, `-=`, `*=`, `/=`, `%=`
- Reference: `&`, `*`
- Other: `.`, `..`, `?`, `|`

#### 3. Delimiters (15 tokens)

- Parentheses: `(`, `)`
- Braces: `{`, `}`
- Brackets: `[`, `]`
- Separators: `,`, `;`, `:`
- Special: `->`, `=>`, `_`, `#`, `...`

#### 4. Literals (5 token types)

- `IntLiteral(i64)` - Integer values
- `FloatLiteral(f64)` - Floating-point values
- `StringLiteral(String)` - Regular strings
- `FStringLiteral(String)` - Interpolated strings
- `Tag(String)` - Struct field tags

#### 5. Identifiers (1 token type)

- `Ident(String)` - User-defined names

#### 6. Intrinsics (2 tokens)

- `@vectorize` - SIMD vectorization hint
- `@gpu` - GPU kernel marker

**Total Token Types**: ~127

### Token Representation

Internally, tokens are represented as:

```rust
pub struct TokenSpan {
    pub token: Token,
    pub span: std::ops::Range<usize>,
}
```

Where:

- `token`: The token type and associated value
- `span`: Source position (start..end byte offsets)

---

## Lexing Process

### Tokenization Steps

1. **Whitespace Skipping**: Spaces, tabs, newlines, and form feeds are skipped
2. **Comment Removal**: Line and block comments are ignored
3. **Token Matching**: Longest match wins using Logos lexer
4. **Error Handling**: Invalid characters produce `LexError::InvalidToken`

### Ambiguity Resolution

When multiple patterns match, the lexer uses the following rules:

1. **Longest Match**: Prefer longer token (e.g., `==` over `=`)
2. **Keyword Priority**: Keywords take precedence over identifiers
3. **Operator Priority**: Compound operators over simple ones (e.g., `<=` over `<`)

**Examples**:

- `let` → Keyword `Let`, not identifier
- `<=` → Single token `LtEq`, not `Lt` + `Eq`
- `f"string"` → `FStringLiteral`, not `Ident` + `StringLiteral`

### Error Handling

Invalid tokens produce a `LexError`:

```rust
pub enum LexError {
    InvalidToken { span: std::ops::Range<usize> }
}
```

**Example Error**:

```vex
let x = @;  // '@' alone is invalid (only @vectorize, @gpu are valid)
```

---

## Implementation Notes

### Lexer Technology

Vex uses the **Logos** lexer generator for efficient tokenization:

- **Declarative**: Token definitions via Rust attributes
- **Zero-Copy**: Slices source without allocation where possible
- **Fast**: Compiled to optimized DFA
- **Error Recovery**: Continues after invalid tokens

### Performance Characteristics

- **Time Complexity**: O(n) where n is source length
- **Space Complexity**: O(1) (streaming, no full token buffer)
- **Throughput**: ~500 MB/s on modern hardware

---

## Examples

### Complete Lexing Example

**Input**:

```vex
fn add(a: i32, b: i32): i32 {
    return a + b;
}
```

**Tokens**:

```
Fn
Ident("add")
LParen
Ident("a")
Colon
I32
Comma
Ident("b")
Colon
I32
RParen
Colon
I32
LBrace
Return
Ident("a")
Plus
Ident("b")
Semicolon
RBrace
```

### String Literals

**Input**:

```vex
"Hello, \"World\"!\n"
f"User: {name}, Age: {age}"
`json:"user_id"`
```

**Tokens**:

```
StringLiteral("Hello, \"World\"!\n")
FStringLiteral("User: {name}, Age: {age}")
Tag("json:\"user_id\"")
```

---

**Previous**: [01_Introduction_and_Overview.md](./01_Introduction_and_Overview.md)  
**Next**: [03_Type_System.md](./03_Type_System.md)

**Maintained by**: Vex Language Team

# Type System

**Version:** 0.1.2  
**Last Updated:** November 2025

This document defines the complete type system of the Vex programming language.

---

## Table of Contents

1. [Type Categories](#type-categories)
2. [Primitive Types](#primitive-types)
3. [Compound Types](#compound-types)
4. [User-Defined Types](#user-defined-types)
5. [Advanced Types](#advanced-types)
6. [Type Inference](#type-inference)
7. [Type Conversions](#type-conversions)
8. [Type Compatibility](#type-compatibility)
9. [Operator Overloading](#operator-overloading)

---

## Type Categories

Vex's type system is organized into four main categories:

```
Types
├── Primitive Types
│   ├── Integer Types (i8, i16, i32, i64, i128, u8, u16, u32, u64, u128)
│   ├── Floating-Point Types (f16, f32, f64)
│   ├── Boolean Type
│   ├── String Type
│   └── Special Types (nil, error, byte)
├── Compound Types
│   ├── Arrays
│   ├── Slices
│   ├── Tuples
│   ├── References
│   └── Collections (Map, Set, Vec, Box, Channel)
├── User-Defined Types
│   ├── Structs
│   ├── Enums
│   └── Type Aliases
└── Advanced Types
    ├── Generic Types
    ├── Union Types
    ├── Intersection Types
    └── Conditional Types
```

---

## Primitive Types

### Integer Types

Vex provides fixed-size integer types with explicit signedness:

#### Signed Integers

| Type   | Size     | Range                                                   | Description                     |
| ------ | -------- | ------------------------------------------------------- | ------------------------------- |
| `i8`   | 8 bits   | -128 to 127                                             | 8-bit signed integer            |
| `i16`  | 16 bits  | -32,768 to 32,767                                       | 16-bit signed integer           |
| `i32`  | 32 bits  | -2,147,483,648 to 2,147,483,647                         | 32-bit signed integer (default) |
| `i64`  | 64 bits  | -9,223,372,036,854,775,808 to 9,223,372,036,854,775,807 | 64-bit signed integer           |
| `i128` | 128 bits | -2^127 to 2^127-1                                       | 128-bit signed integer          |

**Default**: Integer literals without type annotation default to `i32`.

**Examples**:

```vex
let small: i8 = 127;
let medium: i16 = 32000;
let normal = 42;           // i32 (default)
let large: i64 = 9223372036854775807;
```

#### Unsigned Integers

| Type   | Size     | Range                           | Description              |
| ------ | -------- | ------------------------------- | ------------------------ |
| `u8`   | 8 bits   | 0 to 255                        | 8-bit unsigned integer   |
| `u16`  | 16 bits  | 0 to 65,535                     | 16-bit unsigned integer  |
| `u32`  | 32 bits  | 0 to 4,294,967,295              | 32-bit unsigned integer  |
| `u64`  | 64 bits  | 0 to 18,446,744,073,709,551,615 | 64-bit unsigned integer  |
| `u128` | 128 bits | 0 to 2^128-1                    | 128-bit unsigned integer |

**Examples**:

```vex
let byte_val: u8 = 255;
let port: u16 = 8080;
let count: u32 = 4294967295;
let big: u64 = 18446744073709551615;
```

#### Integer Operations

**Arithmetic**:

```vex
let sum = a + b;           // Addition
let diff = a - b;          // Subtraction
let product = a * b;       // Multiplication
let quotient = a / b;      // Division
let remainder = a % b;     // Modulo
```

**Comparison**:

```vex
a == b    // Equal
a != b    // Not equal
a < b     // Less than
a <= b    // Less than or equal
a > b     // Greater than
a >= b    // Greater than or equal
```

**Overflow Behavior**:

- Debug mode: Panic on overflow
- Release mode: Wrapping arithmetic (default)
- Future: Checked, saturating, and wrapping variants

### Floating-Point Types

IEEE 754 floating-point numbers:

| Type  | Size    | Precision          | Description                      |
| ----- | ------- | ------------------ | -------------------------------- |
| `f16` | 16 bits | ~3 decimal digits  | Half precision float             |
| `f32` | 32 bits | ~7 decimal digits  | Single precision float           |
| `f64` | 64 bits | ~15 decimal digits | Double precision float (default) |

**Default**: Floating-point literals default to `f64`.

**Examples**:

```vex
let pi: f32 = 3.14159;
let e = 2.71828;           // f64 (default)
let precise: f64 = 3.141592653589793;
```

**Special Values**:

```vex
// Future support
let inf = f64::INFINITY;
let neg_inf = f64::NEG_INFINITY;
let not_a_number = f64::NAN;
```

**Operations**:

```vex
let sum = a + b;
let diff = a - b;
let product = a * b;
let quotient = a / b;     // No modulo for floats
```

### Boolean Type

The `bool` type has two values:

```vex
let yes: bool = true;
let no: bool = false;
```

**Size**: 1 byte (8 bits)

**Operations**:

```vex
!a          // Logical NOT
a && b      // Logical AND (short-circuit)
a || b      // Logical OR (short-circuit)
a == b      // Equality
a != b      // Inequality
```

**In Conditions**:

```vex
if condition {
    // condition must be bool type
}

let result = condition && other_condition;
```

**Answer**: 🟡 Ternary operator (Medium Priority)

```vex
let result = condition ? true_value : false_value;
```

Kullanışlı ama if-else expression zaten var. Gelecekte eklenebilir.

**Answer**: 🟡 If-scoped variable (Medium Priority) - Go pattern

```vex
if let x = getValue(); x > 0 {
    // x is only in scope here
}
```

Kullanışlı özellik, gelecekte eklenebilir. Şu an workaround:

```vex
{
    let x = getValue();
    if x > 0 {
        // use x
    }
}
```

### String Type

UTF-8 encoded text:

```vex
let greeting: string = "Hello, World!";
let empty: string = "";
let multiline = "Line 1\nLine 2";
```

**Properties**:

- **Encoding**: UTF-8
- **Immutable**: Strings are immutable by default
- **Heap Allocated**: Managed by runtime
- **Size**: Pointer + length (16 bytes on 64-bit)

**Operations**:

```vex
// Concatenation (future)
let full_name = first_name + " " + last_name;

// Length
let len = str.len();  // Available via string methods

// Character Indexing ✅ v0.1.2
let first_char = str[0];        // Returns byte at index
let last_char = str[str.len() - 1];

// String Slicing ✅ v0.1.2
let substring = str[0..5];      // Slice from index 0 to 5 (exclusive)
let from_start = str[..5];      // From beginning to index 5
let to_end = str[7..];          // From index 7 to end
let copy = str[..];             // Full string copy
```

**UTF-8 Safety**:

String indexing and slicing operate on **byte indices**, not character indices. This is fast but requires care with multi-byte UTF-8 characters:

```vex
let emoji = "Hello 👋";
let slice = emoji[0..7];  // ✅ Safe: "Hello "
// emoji[0..8] would panic - splits emoji in middle of UTF-8 sequence

// For character-based indexing, use string methods:
let chars = emoji.chars();  // Iterator over characters (future)
```

**Implementation Details** (v0.1.2):

- **Indexing `str[i]`**: Returns `u8` byte at position `i`, bounds-checked at runtime
- **Slicing `str[a..b]`**: Creates new string from bytes `a` to `b` (exclusive)
- **Runtime**: `vex_string_slice(ptr, start, end)` in `vex-runtime/c/vex_string.c`
- **Panic**: Out-of-bounds access or invalid UTF-8 split causes runtime panic
- **Test**: `examples/test_string_slicing.vx`

**String Interpolation**:

```vex
let name = "Alice";
let age = 30;
let message = f"Hello, {name}! You are {age} years old.";
```

### Byte Type

Alias for `u8`, used for raw byte data:

```vex
let b: byte = 255;
let bytes: [byte; 4] = [0xDE, 0xAD, 0xBE, 0xEF];
```

**Use Cases**:

- Binary data
- Network protocols
- File I/O
- Byte buffers

### Special Types

#### Nil Type

Represents absence of value (unit type):

```vex
fn do_something() {
    // Returns nil implicitly
}

let nothing = nil;
```

**Size**: 0 bytes (zero-sized type)

#### Error Type

Used for error handling:

```vex
let err: error = "Something went wrong";

fn risky_operation(): (i32 | error) {
    if problem {
        return "Error occurred";
    }
    return 42;
}
```

---

## Compound Types

### Arrays (with Auto-Vectorization Support)

Fixed-size sequences of elements of the same type:

**Syntax**: `[Type; Size]`

```vex
let numbers: [i32; 5] = [1, 2, 3, 4, 5];
let zeros: [i32; 10] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
```

**Properties**:

- **Fixed Size**: Size known at compile time
- **Stack Allocated**: Stored on stack by default
- **Contiguous**: Elements stored contiguously in memory
- **Zero-Indexed**: First element at index 0
- **🚀 Auto-Vectorized**: Operations automatically use SIMD/GPU

**Indexing**:

```vex
let first = numbers[0];      // 1
let last = numbers[4];       // 5
```

### Vector Operations (Automatic SIMD/GPU)

**Vex's Killer Feature**: Arrays support transparent vectorization for arithmetic operations.

**Arithmetic Operations** (Auto-Vectorized):

```vex
let a: [f32; 8] = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
let b: [f32; 8] = [8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0];

// All operations automatically use SIMD instructions
let sum = a + b;           // Vector addition (SSE/AVX)
let diff = a - b;          // Vector subtraction
let prod = a * b;          // Vector multiplication
let quot = a / b;          // Vector division
let rem = a % b;           // Vector modulo
```

**Scalar Broadcasting**:

```vex
let scaled = a * 2.5;      // Broadcast scalar to vector, then multiply
let offset = a + 10.0;     // Add 10.0 to each element
```

**Comparison Operations** (Return boolean arrays):

```vex
let gt = a > b;            // [bool; 8] - element-wise comparison
let eq = a == b;           // Element-wise equality
```

**Lane Chunking (Automatic)**:

```vex
// Small array: Uses SIMD (SSE/AVX)
let small: [f32; 16] = [...];
let result1 = small * 2.0;  // Chunked into 4x AVX operations (4 lanes each)

// Medium array: Uses wider SIMD (AVX-512 if available)
let medium: [f32; 256] = [...];
let result2 = medium + small[0..256];  // Optimal lane width selected

// Large array: GPU dispatch if available
let large: [f32; 100000] = [...];
let result3 = large * 3.14;  // GPU kernel if beneficial, else SIMD
```

**Backend Selection Rules**:

| Array Size      | Backend      | Lane Width | Notes                     |
| --------------- | ------------ | ---------- | ------------------------- |
| < 64 elements   | SIMD         | 4-8        | SSE/AVX                   |
| 64-1024         | SIMD         | 8-16       | AVX-512 if available      |
| > 1024 elements | GPU or SIMD  | Variable   | GPU if available & faster |
| > 10K elements  | GPU priority | -          | Always try GPU first      |

**Supported Types for Vectorization**:

- ✅ `f32`, `f64` - Full support (arithmetic, math functions)
- ✅ `i32`, `i64`, `u32`, `u64` - Arithmetic and bitwise
- ✅ `i16`, `u16`, `i8`, `u8` - Arithmetic (packed operations)
- ❌ `string`, `bool` arrays - No auto-vectorization (use explicit loops)

**Implementation Status**:

- ✅ Syntax parsed and recognized
- 🚧 SIMD codegen (partial - basic operations working)
- 🚧 GPU dispatch (planned)
- 🚧 Auto lane-width selection (planned)

**Bounds Checking**:

- Debug mode: Panic on out-of-bounds access
- Release mode: Undefined behavior (future: always check)

**Future Features**:

```vex
let filled: [i32; 10] = [0; 10];  // Repeat expression
let length = numbers.len();        // Array length
```

### Slices

Dynamically-sized views into arrays:

**Syntax**: `&[Type]` (immutable) or `&[Type]!` (mutable)

**Answer**: ❌ `&Type[]` syntax'ı yok. Sadece `&[Type]` kullanılıyor (Rust-style). Bracket'ler içeride kalmalı. Type consistency için önemli:

- Array: `[Type; N]`
- Slice: `&[Type]` veya `&[Type]!`

```vex
let numbers = [1, 2, 3, 4, 5];
let slice: &[i32] = &numbers[1..4];      // [2, 3, 4] (future)
let all: &[i32] = &numbers[..];          // All elements (future)
```

**Properties**:

- **Dynamic Size**: Size determined at runtime
- **Fat Pointer**: Pointer + length (16 bytes on 64-bit)
- **Borrowed**: Slices borrow from arrays
- **Zero-Copy**: No data duplication

**Mutable Slices**:

```vex
let! numbers = [1, 2, 3, 4, 5];
let slice_mut: &[i32]! = &numbers[..];   // Mutable slice (future)
```

### Tuples

Fixed-size collections of heterogeneous types:

**Syntax**: `(Type1, Type2, ...)`

```vex
let point: (i32, i32) = (10, 20);
let person: (string, i32, bool) = ("Alice", 30, true);
let empty: () = ();  // Unit tuple (same as nil)
```

**Accessing Elements**:

```vex
let (x, y) = point;              // Destructuring
let name = person.0;             // Index access (future)
let age = person.1;              // Second element (future)
```

**Nested Tuples**:

```vex
let nested: ((i32, i32), string) = ((10, 20), "point");
```

**Use Cases**:

- Multiple return values
- Temporary grouping
- Pattern matching

### References

Borrowed pointers to values:

**Syntax**:

- `&Type` - Immutable reference
- `&Type!` - Mutable reference (v0.1 syntax)

```vex
let x = 42;
let ref_x: &i32 = &x;           // Immutable reference
```

**Mutable References**:

```vex
let! y = 100;
let ref_y: &i32! = &y;          // Mutable reference
```

**Properties**:

- **Non-Owning**: References don't own data
- **Borrowed**: Must follow borrow rules
- **Sized**: Size of a pointer (8 bytes on 64-bit)
- **No Null**: References are never null

**Dereferencing**:

```vex
let x = 42;
let ref_x = &x;
let value = *ref_x;             // Dereference to get value
```

**Borrow Rules**:

1. One mutable reference XOR multiple immutable references
2. References must always be valid (no dangling)
3. References cannot outlive the data they point to

### Collections

Vex provides builtin collection types that are implemented in Rust and available without imports. These types provide efficient data structures for common programming patterns.

#### Vec<T> Type

Dynamic arrays with growable size:

**Syntax**: `Vec<T>` (builtin type)

```vex
let numbers: Vec<i32> = Vec.new();
numbers.push(1);
numbers.push(2);
numbers.push(3);

let first = numbers.get(0);  // Some(1)
let length = numbers.len();  // 3
```

**Properties**:

- **Generic**: Parameterized by element type
- **Dynamic size**: Grows automatically when needed
- **Heap allocated**: Managed by runtime
- **Contiguous**: Elements stored contiguously in memory
- **Cache-friendly**: Better performance than linked lists

**Operations**:

```vex
let v = Vec.new<i32>();     // Create empty Vec
v.push(42);                 // Add element
let val = v.get(0);         // Get element (returns Option<T>)
let len = v.len();          // Get length
v.pop();                    // Remove last element
v.clear();                  // Remove all elements
```

**Implementation**: `vex-compiler/src/codegen_ast/builtins/builtin_types/collections.rs`

#### Map<K, V> Type

Associative arrays with key-value pairs:

**Syntax**: `Map<K, V>` (builtin type)

```vex
let ages: Map<string, i32> = Map.new();
ages.insert("Alice", 30);
ages.insert("Bob", 25);

let alice_age = ages.get("Alice");  // Some(30)
let has_bob = ages.contains_key("Bob");  // true
```

**Properties**:

- **Generic**: Parameterized by key and value types
- **Hash-based**: Fast lookup O(1) average case
- **Heap allocated**: Managed by runtime
- **Keys**: Must implement hash and equality
- **SwissTable**: Uses Google's SwissTable algorithm (34M ops/s)

**Operations**:

```vex
let m = Map.new<string, i32>();  // Create empty Map
m.insert("key", 42);             // Insert key-value pair
let val = m.get("key");          // Get value (returns Option<V>)
let has_key = m.contains_key("key"); // Check if key exists
m.remove("key");                 // Remove key-value pair
let size = m.len();              // Get number of entries
```

**Implementation**: `vex-compiler/src/codegen_ast/builtins/hashmap.rs`

#### Set<T> Type

Collections of unique values:

**Syntax**: `Set<T>` (builtin type)

```vex
let numbers: Set<i32> = Set.new();
numbers.insert(1);
numbers.insert(2);
numbers.insert(1);  // Duplicate, ignored

let has_one = numbers.contains(1);  // true
let size = numbers.len();           // 2
```

**Properties**:

- **Generic**: Parameterized by element type
- **Unique elements**: No duplicates allowed
- **Hash-based**: Fast membership testing
- **Heap allocated**: Managed by runtime
- **SwissTable**: Same high-performance hash table as Map

**Operations**:

```vex
let s = Set.new<i32>();    // Create empty Set
s.insert(42);              // Add element
let has_val = s.contains(42); // Check membership
s.remove(42);              // Remove element
let size = s.len();        // Get number of elements
```

**Implementation**: `vex-compiler/src/codegen_ast/builtins/builtin_types/collections.rs`

#### Array<T, N> Type

Fixed-size arrays with compile-time size:

**Syntax**: `[T; N]` (builtin type)

```vex
let numbers: [i32; 5] = [1, 2, 3, 4, 5];
let zeros: [i32; 10] = [0; 10];  // Repeat syntax
let first = numbers[0];          // Access element
```

**Properties**:

- **Fixed size**: Size known at compile time
- **Stack allocated**: Stored on stack by default
- **Contiguous**: Elements stored contiguously in memory
- **Zero-indexed**: First element at index 0
- **🚀 Auto-vectorized**: Operations automatically use SIMD/GPU

**Operations**:

```vex
let arr: [i32; 3] = [1, 2, 3];
let first = arr[0];        // Index access
let len = arr.len();       // Get length (compile-time constant)
let slice = &arr[1..3];    // Create slice (future)
```

**Implementation**: `vex-compiler/src/codegen_ast/builtins/array.rs`

#### Box<T> Type

Heap-allocated single values:

**Syntax**: `Box<T>` (builtin type)

```vex
let boxed = Box.new(42);        // Heap allocate i32
let value = Box.unbox(boxed);   // Extract value and free
```

**Properties**:

- **Heap allocated**: Single value on heap
- **Ownership**: Moves ownership to heap
- **Pointer**: Returns pointer to heap value
- **Manual free**: Requires explicit deallocation

**Operations**:

```vex
let b = Box.new<i32>(42);   // Allocate on heap
let val = Box.unbox(b);     // Extract value and deallocate
```

**Implementation**: `vex-compiler/src/codegen_ast/builtins/builtin_types/collections.rs`

---

## User-Defined Types

### Structs

Named collections of fields:

```vex
struct Point {
    x: i32,
    y: i32,
}

struct Person {
    name: string,
    age: i32,
    email: string,
}
```

**Instantiation**:

```vex
let p = Point { x: 10, y: 20 };
let person = Person {
    name: "Alice",
    age: 30,
    email: "alice@example.com",
};
```

**Field Access**:

```vex
let x_coord = p.x;
let person_name = person.name;
```

**Generic Structs**:

```vex
struct Container<T> {
    value: T,
}

let int_container = Container<i32> { value: 42 };
let str_container = Container<string> { value: "hello" };
```

**Memory Layout**:

- Fields stored sequentially in memory
- Padding for alignment
- Size = sum of field sizes + padding

### Enums

Algebraic data types with variants:

#### Unit Enums

```vex
enum Color {
    Red,
    Green,
    Blue,
}

let color = Red;
```

#### Enums with Values

```vex
enum Status {
    Success = 0,
    Error = 1,
    Pending = 2,
}
```

#### Data-Carrying Enums (Future)

```vex
enum Option<T> {
    Some(T),
    None,
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

**Pattern Matching**:

```vex
match color {
    Red => { }
    Green => { }
    Blue => { }
}
```

### Type Aliases

Create alternative names for types:

```vex
type UserID = u64;
type Point2D = (i32, i32);
type Callback = fn(i32): i32;
```

**Usage**:

```vex
let id: UserID = 12345;
let point: Point2D = (10, 20);
```

**Generic Type Aliases with Constraints** ✅ (v0.1.2):

```vex
// Simple contract bound
type Displayable<T: Display> = T;

// Multiple contract bounds
type ComparableNumber<T: Ord + Clone> = T;

// Complex constraints
type SerializableVec<T: Serialize + Clone> = Vec<T>;

// Function type with constraints
type Processor<T: Display + Clone> = fn(T): T;
```

**Conditional Type Aliases** ✅ (v0.1.2):

```vex
// Unwrap Option type
type Unwrap<T> = T extends Option<infer U> ? U : T;

// Extract Result values
type ExtractOk<T> = T extends Result<infer V, infer E> ? V : T;

// Type filtering
type OnlyOption<T> = T extends Option<infer U> ? T : never;
```

**Type Safety:**

- ✅ All type aliases are compile-time only (zero runtime cost)
- ✅ Constraints enforced during type checking
- ✅ Invalid types cause compile errors
- ✅ No reflection or runtime type information

---

## Advanced Types

### Generic Types

Types parameterized by other types:

```vex
struct Box<T> {
    value: T,
}

fn identity<T>(x: T): T {
    return x;
}
```

**Type Parameters**:

```vex
struct Pair<T, U> {
    first: T,
    second: U,
}
```

**Monomorphization**:

- Generics are compiled to concrete types at compile time
- Each instantiation generates specialized code
- No runtime overhead

### Union Types

Union types allow a value to be one of several different types. They are implemented as **tagged unions** with a discriminator field.

**Syntax**: `(Type1 | Type2 | ...)`

**Implementation Status**: ✅ **COMPLETE** (v0.1.2)

```vex
type NumberOrString = (i32 | string);

let value: NumberOrString = 42;
let value2: NumberOrString = "hello";
```

**Representation**:

Union types are compiled to tagged unions (similar to Rust enums):

```vex
// Internal representation: { i32 tag, <largest_type> data }
struct UnionLayout {
    tag: i32,           // 0 for first type, 1 for second, etc.
    data: LargestType   // Union of all member types
}
```

**Use Cases**:

- Flexible function parameters accepting multiple types
- Error handling with `(T | error)`
- Optional values with `(T | nil)`
- Multi-type return values

**Examples**:

```vex
// Function accepting int or string
fn accepts_int_or_string(value: (i32 | string)) {
    // Type checking at runtime via tag field
    print("Received union value");
}

// Nested unions
fn accepts_option_or_bool(value: (Option<i32> | bool)) {
    print("Received Option or bool");
}

// Union type in variable declaration
let x: (i32 | string) = 42;
accepts_int_or_string(x);

// Union with Result/Option
let z: (Result<i32, string> | Option<string>) = Some("test");
```

**Pattern Matching** (Future):

```vex
match value {
    i when i is i32 => { println("Integer: {}", i); }
    s when s is string => { println("String: {}", s); }
}
```

**Implementation Details**:

- **Parser**: `vex-parser/src/parser/types.rs` - Parses `(T1 | T2)` syntax
- **AST**: `vex-ast/src/lib.rs` - `Type::Union(Vec<Type>)`
- **Codegen**: `vex-compiler/src/codegen_ast/types.rs` - Tagged union struct generation
- **Size calculation**: Uses largest member type for data field
- **Test file**: `examples/test_union_types.vx`

### Intersection Types

Types that combine multiple contracts:

**Syntax**: `(Trait1 & Trait2 & ...)`

```vex
type Comparable = (Eq & Ord);
type Serializable = (Display & ToString);
```

**Contract Bounds**:

```vex
fn process<T: Display & Serialize>(value: T) {
    // T must implement both Display and Serialize (future)
}
```

### Conditional Types (Advanced)

Type-level conditionals:

**Syntax**: `T extends U ? X : Y`

```vex
type NonNullable<T> = T extends nil ? never : T;  // Future
type ElementType<T> = T extends [infer E] ? E : never;
```

**Use Cases**:

- Advanced type transformations
- Library API design
- Type-level programming

---

## Type Inference

Vex supports bidirectional type inference:

### Literal Inference

```vex
let x = 42;              // Inferred as i32
let y = 3.14;            // Inferred as f64
let z = true;            // Inferred as bool
let s = "hello";         // Inferred as string
```

### From Context

```vex
add(a: i32, b: i32): i32 {
    return a + b;
}

let result = add(10, 20);  // result: i32 (inferred from return type)
```

### Generic Inference

```vex
fn identity<T>(x: T): T {
    return x;
}

let value = identity(42);      // T inferred as i32
let text = identity("hello");  // T inferred as string
```

### Limitations

Type inference fails when:

- Ambiguous type (requires annotation)
- Circular dependencies
- Insufficient information

**Example requiring annotation**:

```vex
let numbers = [];  // ERROR: Cannot infer element type
let numbers: [i32] = [];  // OK
```

---

## Type Reflection

Vex provides runtime type information through builtin reflection functions. These functions are always available without imports.

### Runtime Type Information

```vex
fn main(): i32 {
    let x: i32 = 42;
    let y: f64 = 3.14;

    // Get type name as string
    let type_name = typeof(x);  // Returns "i32"

    // Get unique type identifier
    let id = type_id(x);  // Returns numeric ID for i32

    // Get type size and alignment
    let size = type_size(x);   // Returns 4
    let align = type_align(x); // Returns 4

    return 0;
}
```

### Type Category Checking

```vex
fn main(): i32 {
    let x: i32 = 42;
    let y: f64 = 3.14;
    let ptr = &x;

    // Check type categories
    if is_int_type(x) {
        println("x is an integer");  // This will print
    }

    if is_float_type(y) {
        println("y is a float");  // This will print
    }

    if is_pointer_type(ptr) {
        println("ptr is a pointer");  // This will print
    }

    return 0;
}
```

### Available Reflection Functions

| Function                       | Return Type | Description                           |
| ------------------------------ | ----------- | ------------------------------------- |
| `typeof<T>(value: T)`          | `string`    | Get type name                         |
| `type_id<T>(value: T)`         | `u64`       | Get unique numeric type identifier    |
| `type_size<T>(value: T)`       | `u64`       | Get type size in bytes                |
| `type_align<T>(value: T)`      | `u64`       | Get type alignment in bytes           |
| `is_int_type<T>(value: T)`     | `bool`      | Check if value is integer type        |
| `is_float_type<T>(value: T)`   | `bool`      | Check if value is floating-point type |
| `is_pointer_type<T>(value: T)` | `bool`      | Check if value is pointer type        |

**Properties**:

- **Compile-time evaluation**: Most reflection info computed at compile time
- **Zero-cost**: No runtime overhead for type checks
- **Generic support**: Works with generic types
- **Status**: ✅ Fully implemented

### Use Cases

**Generic debugging**:

```vex
fn debug<T>(value: T) {
    println(f"Type: {typeof(value)}, Size: {type_size(value)} bytes");
}
```

**Type-safe serialization**:

```vex
fn serialize<T>(value: T): string {
    if is_int_type(value) {
        // Serialize as integer
    } else if is_float_type(value) {
        // Serialize as float
    } else {
        // Default serialization
    }
}
```

**Dynamic type checking**:

```vex
fn process_value<T>(value: T) {
    let id = type_id(value);
    match id {
        4 => println("Processing i32"),
        5 => println("Processing i64"),
        _ => println("Unknown type"),
    }
}
```

---

## Type Conversions

### Explicit Conversions (Future)

```vex
let x: i32 = 42;
let y: i64 = x as i64;        // Explicit cast
let z: f64 = x as f64;        // Int to float
```

### Implicit Conversions

Vex has **minimal implicit conversions** for safety:

**Allowed**:

- Integer promotion in some contexts (implementation-defined)

**Not Allowed**:

- No automatic narrowing (u64 → u32)
- No float ↔ integer conversion
- No pointer ↔ integer conversion

### Coercion

**Deref Coercion**:

```vex
let x = 42;
let ref_x = &x;
let y = *ref_x;  // Explicit dereference required
```

**Array to Slice** (Future):

```vex
let arr = [1, 2, 3];
let slice: &[i32] = &arr;  // Coercion
```

---

## Type Compatibility

### Structural vs Nominal

Vex uses **nominal typing** for user-defined types:

```vex
struct Point { x: i32, y: i32 }
struct Vector { x: i32, y: i32 }

let p: Point = Point { x: 1, y: 2 };
// let v: Vector = p;  // ERROR: Different types
```

### Contract Compatibility

Types are compatible if they implement required contracts:

```vex
contract Display {
    show();
}

fn print_it<T: Display>(value: T) {
    value.show();  // OK: T implements Display
}
```

---

## Operator Overloading

> See: [Specifications/23_Operator_Overloading.md](../Specifications/23_Operator_Overloading.md) for the full operator overloading specification and examples.

### Overview

Vex supports **contract-based operator overloading**, allowing custom types to define behavior for built-in operators. This enables intuitive APIs for mathematical types, collections, and domain-specific types.

### Supported Operators

| Operator | Contract Method | Description      |
| -------- | --------------- | ---------------- |
| `+`      | `add`           | Addition         |
| `-`      | `sub`           | Subtraction      |
| `*`      | `mul`           | Multiplication   |
| `/`      | `div`           | Division         |
| `%`      | `rem`           | Remainder        |
| `==`     | `eq`            | Equality         |
| `!=`     | `ne`            | Inequality       |
| `<`      | `lt`            | Less than        |
| `<=`     | `le`            | Less or equal    |
| `>`      | `gt`            | Greater than     |
| `>=`     | `ge`            | Greater or equal |
| `+=`     | `add_assign`    | Add assign       |
| `-=`     | `sub_assign`    | Subtract assign  |
| `*=`     | `mul_assign`    | Multiply assign  |
| `/=`     | `div_assign`    | Divide assign    |
| `%=`     | `rem_assign`    | Remainder assign |

### Defining Operator Overloads

```vex
contract Add<Rhs, Output> {
    add(self: &Self, rhs: Rhs): Output;
}

contract AddAssign<Rhs> {
    add_assign(self: &Self!, rhs: Rhs);
}

// Implementation for custom Point type
struct Point {
    x: i32,
    y: i32,
}

// New syntax: external implementations and operator method names
struct Point impl Add {
    x: i32,
    y: i32,
}

fn (self: &Point) op+(rhs: Point): Point {
    return Point {
        x: self.x + rhs.x,
        y: self.y + rhs.y,
    };
}

fn (self: &Point!) op+=(rhs: Point) {
    self.x = self.x + rhs.x;
    self.y = self.y + rhs.y;
}
```

### Usage

```vex
let p1 = Point { x: 1, y: 2 };
let p2 = Point { x: 3, y: 4 };

let p3 = p1 + p2;        // Point { x: 4, y: 6 }
let! p4 = Point { x: 1, y: 2 };
p4 += p2;                // p4 = Point { x: 4, y: 6 }
```

### Built-in Operator Overloads

**String Concatenation**:

```vex
let hello = "Hello";
let world = "World";
let message = hello + " " + world;  // "Hello World"
```

**Vector Operations**:

```vex
let! v1 = Vec.new<i32>();
v1.push(1);
v1.push(2);

let! v2 = Vec.new<i32>();
v2.push(3);
v2.push(4);

let v3 = v1 + v2;  // Vec with [1, 2, 3, 4]
```

### Operator Precedence

Operators maintain standard mathematical precedence:

1. `*`, `/`, `%` (highest)
2. `+`, `-`
3. `<`, `<=`, `>`, `>=`
4. `==`, `!=` (lowest)

### Type Safety

- **Compile-time checked**: All operator overloads are resolved at compile time
- **Type constraints**: Output types must be explicitly specified
- **No implicit conversions**: Types must match contract bounds exactly
- **Borrow checker integration**: Operator usage respects ownership rules

### Current Status

**Implementation**: ✅ Complete (contract-based system)  
**Test Coverage**: ✅ 8 tests passing (builtin operators)  
**Builtin Support**: ✅ String `+`, Vec `+`, Struct operators

---

## Type System Summary

| Category    | Examples                                         | Size               | Notes               |
| ----------- | ------------------------------------------------ | ------------------ | ------------------- |
| Integers    | i8, i16, i32, i64, i128, u8, u16, u32, u64, u128 | 1-16 bytes         | Fixed size          |
| Floats      | f16, f32, f64                                    | 2-8 bytes          | IEEE 754            |
| Boolean     | bool                                             | 1 byte             | true/false          |
| String      | string                                           | 16 bytes (ptr+len) | UTF-8, heap         |
| Arrays      | [T; N]                                           | N \* sizeof(T)     | Stack, fixed        |
| Tuples      | (T, U, ...)                                      | Sum of sizes       | Stack               |
| References  | &T, &T!                                          | 8 bytes (64-bit)   | Pointers            |
| Collections | Map<K,V>, Set<T>, Vec<T>                         | Variable (heap)    | Dynamic/Hash        |
| Smart Ptrs  | Box<T>, Channel<T>                               | 8 bytes (ptr)      | Heap-allocated      |
| Structs     | User-defined                                     | Sum + padding      | Nominal             |
| Enums       | User-defined                                     | Tag + data         | Discriminated union |

---

**Previous**: [02_Lexical_Structure.md](./02_Lexical_Structure.md)  
**Next**: [04_Variables_and_Constants.md](./04_Variables_and_Constants.md)

**Maintained by**: Vex Language Team

# Variables and Constants

**Version:** 0.1.0  
**Last Updated:** November 3, 2025

This document defines the variable and constant declaration system in Vex, including the unified variable syntax introduced in v0.1.

---

## Table of Contents

1. [Variable Declarations](#variable-declarations)
2. [Constant Declarations](#constant-declarations)
3. [Mutability System](#mutability-system)
4. [Shadowing](#shadowing)
5. [Scope and Lifetime](#scope-and-lifetime)
6. [Initialization Rules](#initialization-rules)
7. [Type Annotations](#type-annotations)

---

## Variable Declarations

### Syntax v0.1: Unified Variable System

Vex v0.1 unifies variable declarations with a single `let` keyword and explicit mutability markers:

```vex
let x = 42;              // Immutable (default, Rust-style)
let! counter = 0;        // Mutable (explicit with ! suffix)
```

**Key Changes from Previous Versions**:

- ✅ `let` for immutable variables (default)
- ✅ `let!` for mutable variables (explicit)
- ❌ `mut` keyword **removed** (deprecated in v0.1)
- ❌ `:=` operator **removed** (use `let` instead)

### Immutable Variables

**Syntax**: `let name = value;`

```vex
let age = 30;
let name = "Alice";
let pi = 3.14159;
let is_valid = true;
```

**Properties**:

- Cannot be reassigned after initialization
- Enforced by borrow checker (Phase 1: Immutability)
- Default behavior (Rust-inspired)
- Memory efficient (allows optimizations)

**Example**:

```vex
let x = 42;
// x = 100;  // ERROR: Cannot assign to immutable variable 'x'
```

### Mutable Variables

**Syntax**: `let! name = value;`

```vex
let! counter = 0;
let! balance = 1000.0;
let! status = true;
```

**Properties**:

- Can be reassigned after initialization
- Requires explicit `let!` declaration
- Enforced by borrow checker
- Forces intentional mutability

**Example**:

```vex
let! counter = 0;
counter = counter + 1;    // OK: counter is mutable
counter += 1;             // OK: compound assignment
```

**Reassignment**:

```vex
let! x = 10;
x = 20;              // OK: x is mutable
x = x * 2;           // OK: 40
```

### Multiple Declarations

Declare multiple variables in sequence:

```vex
let x = 10;
let y = 20;
let z = 30;
```

**Tuple Destructuring** (Future):

```vex
let (x, y, z) = (10, 20, 30);
let! (a, b) = (1, 2);
```

---

## Constant Declarations

### Compile-Time Constants

**Syntax**: `const NAME = value;`

```vex
const MAX_SIZE = 100;
const PI = 3.141592653589793;
const APP_NAME = "VexApp";
const DEBUG = true;
```

**Properties**:

- **Compile-Time Evaluation**: Value computed at compile time
- **Immutable**: Cannot be changed at runtime
- **No Type Inference**: Type must be determinable from literal
- **Naming Convention**: SCREAMING_SNAKE_CASE recommended
- **Global Scope**: Can be declared at module level

**Differences from Variables**:

| Feature         | `const`      | `let`      | `let!`     |
| --------------- | ------------ | ---------- | ---------- |
| Mutability      | Never        | No         | Yes        |
| Initialization  | Compile-time | Runtime    | Runtime    |
| Scope           | Any          | Block      | Block      |
| Memory          | Inlined      | Stack/Heap | Stack/Heap |
| Type Annotation | Optional     | Optional   | Optional   |

**Example**:

```vex
const MAX_USERS = 1000;
const MIN_PASSWORD_LENGTH = 8;
const DEFAULT_TIMEOUT = 30.0;

fn validate_users(count: i32): bool {
    return count <= MAX_USERS;
}
```

### Constant Expressions

Constants must be initialized with compile-time constant expressions:

**Allowed**:

```vex
const A = 42;                    // Literal
const B = 10 + 20;               // Arithmetic
const C = 100 * 2;               // Multiplication
const D = true && false;         // Boolean logic
const E = "Hello, " + "World";   // String concatenation (future)
```

**Not Allowed**:

```vex
const X = some_function();       // ERROR: Function calls
const Y = get_value();           // ERROR: Runtime value
let z = 10;
const Z = z + 5;                 // ERROR: Variable reference
```

---

## Mutability System

### Philosophy

Vex follows the **"immutable by default, mutable by choice"** principle:

1. **Safety First**: Immutability prevents accidental modifications
2. **Explicit Intent**: `let!` makes mutability visible
3. **Rust-Inspired**: Similar to Rust's `let` vs `let mut`
4. **Borrow Checker**: Enforces mutability rules at compile time

### Mutability Enforcement

The borrow checker (Phase 1) enforces mutability rules:

```vex
let x = 42;
// x = 100;  // ERROR: Cannot assign to immutable variable 'x'

let! y = 42;
y = 100;      // OK: y declared as mutable with let!
```

**Error Message**:

```
Borrow Checker Error: Cannot assign to immutable variable 'x'
  --> example.vx:3:1
   |
1  | let x = 42;
   |     - variable declared as immutable here
2  |
3  | x = 100;
   | ^^^^^^^ cannot assign to immutable variable
   |
   = help: consider declaring it as mutable: `let! x = 42;`
```

### Mutable References

The mutability of a reference is independent of the mutability of the variable it points to. Vex uses a `!` marker to denote mutable references.

**Syntax**:

- Immutable Reference: `&T`
- Mutable Reference: `&T!`

```vex
let! x = 42;             // Mutable variable
let ref_x: &i32 = &x;    // Immutable reference to a mutable variable

let! y = 100;
let ref_y: &i32! = &y;   // Mutable reference to a mutable variable
```

**Mutability Matrix**:

| Variable Declaration | Reference Type | Can Read? | Can Write to Variable Directly? | Can Write Through Reference? |
| -------------------- | -------------- | --------- | ------------------------------- | ---------------------------- |
| `let x` (immutable)  | `&x`           | ✅        | ❌                              | ❌                           |
| `let! x` (mutable)   | `&x`           | ✅        | ✅                              | ❌                           |
| `let! x` (mutable)   | `&x!`          | ✅        | ✅                              | ✅                           |

This system provides fine-grained control over how data can be accessed and modified, forming a core part of Vex's safety guarantees.

---

## Shadowing

### Definition

Shadowing allows declaring a new variable with the same name as a previous variable:

```vex
let x = 5;
let x = x + 1;    // Shadows previous x
let x = x * 2;    // Shadows again (x is now 12)
```

**Properties**:

- New variable shadows the old one in the same scope
- Old variable becomes inaccessible
- New variable can have different type
- Old variable can have different mutability

### Shadowing vs Mutation

**Shadowing** (creates new variable):

```vex
let x = 5;
let x = x + 1;    // New immutable variable
```

**Mutation** (modifies existing variable):

```vex
let! x = 5;
x = x + 1;        // Modifies existing variable
```

### Type Changes with Shadowing

Shadowing allows changing the type:

```vex
let x = "42";         // x: string
let x = 42;           // x: i32 (shadows previous x)
```

**This is not possible with mutation**:

```vex
let! x = "42";
// x = 42;  // ERROR: Type mismatch (string vs i32)
```

### Scope-Based Shadowing

Inner scopes can shadow outer variables:

```vex
let x = 10;
{
    let x = 20;   // Shadows outer x in this scope
    // x is 20 here
}
// x is 10 here (inner x out of scope)
```

**Example**:

```vex
fn example() {
    let x = 5;

    if true {
        let x = 10;     // Shadows x in if block
        // x is 10 here
    }

    // x is 5 here

    let x = x * 2;      // Shadows x in function scope (now 10)
}
```

---

## Scope and Lifetime

### Block Scope

Variables are scoped to the block they're declared in:

```vex
{
    let x = 42;
    // x is accessible here
}
// x is NOT accessible here (out of scope)
```

**Example**:

```vex
fn main(): i32 {
    let outer = 10;

    {
        let inner = 20;
        // Both outer and inner accessible
    }

    // Only outer accessible here
    // inner is out of scope

    return 0;
}
```

### Function Scope

Function parameters and variables have function scope:

```vex
fn calculate(x: i32, y: i32): i32 {
    let sum = x + y;
    let product = x * y;
    // x, y, sum, product all accessible
    return sum + product;
}
// x, y, sum, product NOT accessible here
```

### Module Scope

Constants and functions can have module scope:

```vex
const MAX_SIZE = 100;  // Module-level constant

fn helper(): i32 {
    return MAX_SIZE;   // Can access module-level const
}

fn main(): i32 {
    return helper();
}
```

### Lifetime (Future Feature)

Lifetimes track how long references are valid:

```vex
fn longest<'a>(x: &'a string, y: &'a string): &'a string {
    if x.len() > y.len() {
        return x;
    } else {
        return y;
    }
}
```

**Status**: Phase 4 of borrow checker (planned)

---

## Initialization Rules

### Must Initialize Before Use

Variables must be initialized before use:

```vex
let x: i32;
// let y = x + 5;  // ERROR: Use of uninitialized variable 'x'

let x = 42;
let y = x + 5;     // OK: x initialized
```

### Initialization in Branches

Variables initialized in all branches can be used:

```vex
let x: i32;
if condition {
    x = 10;
} else {
    x = 20;
}
// x is initialized here (both branches assign)
let y = x + 5;  // OK
```

**Partial Initialization (Error)**:

```vex
let x: i32;
if condition {
    x = 10;
}
// ERROR: x may not be initialized (else branch missing)
// let y = x + 5;  // ERROR
```

### Default Values

Vex does **not** provide default values automatically:

```vex
// No default initialization
let x: i32;  // x is uninitialized (error if used)
```

**Explicit Zero Initialization**:

```vex
let x: i32 = 0;
let y: f64 = 0.0;
let z: bool = false;
let s: string = "";
```

---

## Type Annotations

### Optional Annotations

Type annotations are optional when type can be inferred:

```vex
let x = 42;              // Inferred as i32
let y: i32 = 42;         // Explicit annotation
```

### Required Annotations

Type annotations required when inference fails:

**Empty Collections**:

```vex
// let arr = [];         // ERROR: Cannot infer type
let arr: [i32; 5] = [1, 2, 3, 4, 5];  // OK
```

**Ambiguous Numeric Types**:

```vex
let x: u64 = 100;        // Required: could be u8, u16, u32, or u64
```

**Function Pointers** (Future):

```vex
let f: fn(i32): i32 = some_function;
```

### Syntax

Type annotations follow the colon:

```vex
let name: Type = value;
let! name: Type = value;
const NAME: Type = value;
```

**Examples**:

```vex
let age: i32 = 30;
let! balance: f64 = 1000.0;
const MAX: u64 = 18446744073709551615;
let point: (i32, i32) = (10, 20);
let numbers: [i32; 5] = [1, 2, 3, 4, 5];
```

---

## Examples

### Basic Variables

```vex
fn main(): i32 {
    let x = 10;
    let y = 20;
    let sum = x + y;
    return sum;  // 30
}
```

### Mutable Counter

```vex
fn count_to_ten(): i32 {
    let! counter = 0;
    while counter < 10 {
        counter = counter + 1;
    }
    return counter;  // 10
}
```

### Constants

```vex
const MAX_RETRIES = 3;
const TIMEOUT = 5.0;

fn retry_operation(): bool {
    let! attempts = 0;
    while attempts < MAX_RETRIES {
        if try_operation() {
            return true;
        }
        attempts = attempts + 1;
    }
    return false;
}
```

### Shadowing

```vex
fn transform(): i32 {
    let x = "42";           // x: string
    let x = 42;             // x: i32 (shadowed)
    let x = x * 2;          // x: i32 = 84
    return x;
}
```

### Scope

```vex
fn scoped_example(): i32 {
    let outer = 10;
    {
        let inner = 20;
        let result = outer + inner;  // 30
    }
    // inner not accessible here
    return outer;  // 10
}
```

---

## Comparison with Other Languages

### Vex vs Rust

| Vex v0.1  | Rust            | Description       |
| --------- | --------------- | ----------------- |
| `let x`   | `let x`         | Immutable         |
| `let! x`  | `let mut x`     | Mutable           |
| `const X` | `const X: Type` | Constant          |
| `&T!`     | `&mut T`        | Mutable reference |

### Vex vs Go

| Vex v0.1       | Go             | Description          |
| -------------- | -------------- | -------------------- |
| `let x = 42`   | `x := 42`      | Variable declaration |
| `let! x = 42`  | `var x = 42`   | Mutable variable     |
| `const X = 42` | `const X = 42` | Constant             |

### Vex vs TypeScript

| Vex v0.1       | TypeScript     | Description |
| -------------- | -------------- | ----------- |
| `let x = 42`   | `const x = 42` | Immutable   |
| `let! x = 42`  | `let x = 42`   | Mutable     |
| `const X = 42` | `const X = 42` | Constant    |

---

## Best Practices

### 1. Prefer Immutability

```vex
// Good: Immutable by default
let x = 42;
let y = x * 2;

// Only use mutable when necessary
let! counter = 0;
counter = counter + 1;
```

### 2. Use Descriptive Names

```vex
// Good
let user_count = 42;
let total_price = 99.99;

// Bad
let x = 42;
let y = 99.99;
```

### 3. Initialize Close to Use

```vex
// Good: Initialize when needed
if condition {
    let result = expensive_computation();
    use_result(result);
}

// Bad: Initialize too early
let result = expensive_computation();
if condition {
    use_result(result);
}
```

### 4. Use Constants for Magic Numbers

```vex
// Good
const MAX_BUFFER_SIZE = 1024;
let buffer = allocate(MAX_BUFFER_SIZE);

// Bad
let buffer = allocate(1024);  // What is 1024?
```

### 5. Minimize Mutable State

```vex
// Good: Functional style
fn sum(numbers: [i32; 5]): i32 {
    let result = 0;
    // Use fold/reduce (future)
    return result;
}

// Bad: Excessive mutation
fn sum(numbers: [i32; 5]): i32 {
    let! result = 0;
    let! i = 0;
    while i < 5 {
        result = result + numbers[i];
        i = i + 1;
    }
    return result;
}
```

---

## Summary Table

| Declaration        | Syntax                | Mutable?     | Scope        | When to Use                   |
| ------------------ | --------------------- | ------------ | ------------ | ----------------------------- |
| Immutable Variable | `let x = value`       | No           | Block        | Default choice                |
| Mutable Variable   | `let! x = value`      | Yes          | Block        | When reassignment needed      |
| Constant           | `const X = value`     | No           | Module/Block | Compile-time values           |
| Shadowing          | `let x = ...` (again) | New variable | Block        | Type changes, transformations |

---

**Previous**: [03_Type_System.md](./03_Type_System.md)  
**Next**: [05_Functions_and_Methods.md](./05_Functions_and_Methods.md)

**Maintained by**: Vex Language Team

# Functions and Methods

**Version:** 0.2.0  
**Last Updated:** November 12, 2025

This document defines functions, methods, and related concepts in the Vex programming language.

---

## Table of Contents

1. [Function Declarations](#function-declarations)
2. [Method Definitions](#method-definitions)
3. [Parameters and Arguments](#parameters-and-arguments)
   - [Go-Style Parameter Grouping](#go-style-parameter-grouping) ⭐ NEW
4. [Return Values](#return-values)
5. [Generic Functions](#generic-functions)
6. [Function Overloading](#function-overloading)
7. [Higher-Order Functions](#higher-order-functions)
8. [Special Function Types](#special-function-types)

---

## Function Declarations

### Basic Syntax

**Syntax**: `fn name(parameters): return_type { body }`

```vex
fn add(x: i32, y: i32): i32 {
    return x + y;
}

fn greet(name: string) {
    // No return type = returns nil (unit)
}

fn main(): i32 {
    return 0;  // Entry point
}
```

**Components**:

- `fn` keyword
- Function name (identifier)
- Parameter list in parentheses
- Optional return type after colon
- Function body in braces

### Simple Functions

**No Parameters**:

```vex
fn hello(): i32 {
    return 42;
}
```

**No Return Value** (returns nil):

```vex
fn print_message() {
    // Implicit return nil
}
```

**Single Expression** (explicit return required):

```vex
fn double(x: i32): i32 {
    return x * 2;
}
```

### Function Naming

**Conventions**:

- `snake_case` for function names
- Descriptive names preferred
- Verbs for actions: `calculate_sum`, `print_result`
- Predicates start with `is_`, `has_`, `can_`: `is_valid`, `has_error`

**Examples**:

```vex
fn calculate_total(items: [i32; 10]): i32 { }
fn is_prime(n: i32): bool { }
fn get_user_name(): string { }
fn validate_input(data: string): bool { }
```

### Entry Point

The `main` function is the program entry point:

```vex
fn main(): i32 {
    return 0;  // Exit code
}
```

**Properties**:

- Must return `i32` (exit code)
- No parameters (command-line args future feature)
- Program execution starts here
- Return 0 for success, non-zero for error

---

## Method Definitions

Vex, metodun tanımlandığı bağlama göre değişen, esnek ve pragmatik bir mutasyon sözdizimi kullanır. Bu "Hibrit Model" olarak adlandırılır.

### Kural 1: Inline Metodlar (Struct ve Contract İçinde)

**Amaç:** Kod tekrarını önlemek ve `struct`/`contract` tanımlarını temiz tutmak.

- **Tanımlama:** Metodun `mutable` olduğu, imzanın sonuna eklenen `!` işareti ile belirtilir. Receiver (`self`) bu stilde implisittir ve yazılmaz.
  - `fn method_name()!`
- **Gövde Erişimi:** Metod gövdesinde, alanlara erişim ve atama doğrudan `self` üzerinden yapılır. `self!` **kullanılmaz**.
  - `self.field = new_value`
- **Çağrı:** `Mutable` metodlar, çağrı anında `!` **kullanılmadan** çağrılır. Derleyici, metodun sadece `let!` ile tanımlanmış `mutable` bir nesne üzerinde çağrıldığını compile-time'da kontrol eder.
  - `object.method_name()`

**Örnek:**

```vex
struct Point {
    x: i32,
    y: i32,

    // Immutable method (implicit self)
    fn distance(): f64 {
        return sqrt(self.x * self.x + self.y * self.y);
    }

    // Mutable method (implicit self)
    fn move_to(new_x: i32, new_y: i32)! {
        self.x = new_x;
        self.y = new_y;
    }
}

// --- Çağrılar ---
let p = Point { x: 10, y: 20 };
let dist = p.distance();

let! p_mut = Point { x: 0, y: 0 };
p_mut.move_to(30, 40); // '!' yok
```

### Kural 2: External Metodlar (Golang-Style)

**Amaç:** Metodun hangi veri tipi üzerinde çalıştığını ve `mutable` olup olmadığını receiver tanımında açıkça belirtmek.

- **Tanımlama:** Metodun `mutable` olduğu, receiver tanımındaki `&Type!` ifadesi ile belirtilir. Metod imzasının sonunda `!` **kullanılmaz**.
  - `fn (self: &MyType!) method_name()`
- **Gövde Erişimi:** Metod gövdesinde, alanlara erişim ve atama doğrudan `self` üzerinden yapılır. `self!` **kullanılmaz**.
  - `self.field = new_value`
- **Çağrı:** Çağrı sırasında `!` işareti **kullanılmaz**.
  - `object.method_name()`

**Örnek:**

```vex
struct Rectangle {
    width: i32,
    height: i32,
}

// Immutable external method
fn (r: &Rectangle) area(): i32 {
    return r.width * r.height;
}

// Mutable external method
fn (r: &Rectangle!) scale(factor: i32) {
    r.width = r.width * factor;
    r.height = r.height * factor;
}

// --- Çağrılar ---
let rect = Rectangle { width: 10, height: 5 };
let a = rect.area();

let! rect_mut = Rectangle { width: 10, height: 5};
rect_mut.scale(2); // '!' yok
```

### Contract Method Implementation

```vex
contract Display {
    show();        // ✅ No 'fn' prefix in contract declarations
    update()!;     // Mutable contract method
}

struct User impl Display {
    name: string,
    age: i32,

    // Contract methods MUST be implemented here (in struct body)
    fn show() {
        print(self.name, " - ", self.age);
    }

    fn update()! {
        self.age = self.age + 1;
    }
}
```

**Error**: Contract methods cannot be external

```vex
// ❌ COMPILE ERROR: Contract method cannot be external
fn (u: &User) show() {
    print(u.name);
}
```

---

## Parameters and Arguments

### Basic Parameter Syntax

Parameters are declared with a name and type, separated by colon:

```vex
fn add(x: i32, y: i32): i32 {
    return x + y;
}

fn greet(name: string, age: i32) {
    print("Hello ", name, ", age ", age);
}
```

### Go-Style Parameter Grouping

⭐ **NEW in v0.2.0**: Consecutive parameters of the same type can be grouped together.

**Syntax**: `(name1, name2, name3: type)`

```vex
// Traditional syntax (still supported)
fn add(a: i32, b: i32, c: i32): i32 {
    return a + b + c;
}

// Go-style grouping (new!)
fn add(a, b, c: i32): i32 {
    return a + b + c;
}
```

Both syntaxes are equivalent and produce identical AST nodes.

**Multiple Groups**:

```vex
fn process(x, y, z: f64, name, tag: string): void {
    let sum = x + y + z;
    println(name, ": ", tag, " = ", sum);
}
```

**Mixed Parameters**:

```vex
fn compute(a, b: i32, factor: f64, c, d: i32): f64 {
    let sum = a + b + c + d;
    return (sum as f64) * factor;
}
```

**In Methods**:

```vex
struct Point {
    x: f64,
    y: f64,

    // Grouping works in methods
    distance_to(x1, y1: f64): f64 {
        let dx = self.x - x1;
        let dy = self.y - y1;
        return sqrt(dx * dx + dy * dy);
    }
}

// Also in external methods
fn (p: &Point!) translate(dx, dy: f64) {
    p.x = p.x + dx;
    p.y = p.y + dy;
}
```

**In Contracts**:

```vex
contract Geometry {
    distance(x1, y1, x2, y2: f64): f64;
    translate(dx, dy: f64)!;
}
```

**Benefits**:

- ✅ Reduces repetition for same-typed parameters
- ✅ Cleaner, more readable function signatures
- ✅ Familiar to Go developers
- ✅ Purely syntactic sugar (no runtime overhead)
- ✅ Optional - traditional syntax still supported

**Implementation Note**: The parser automatically expands grouped parameters to individual `Param` AST nodes during parsing, so the rest of the compiler sees fully expanded parameters.

### Parameter Passing

Vex uses **pass-by-value** semantics by default:

```vex
fn modify(x: i32) {
    x = 10;  // Only modifies local copy
}

let y = 5;
modify(y);
// y is still 5
```

For reference semantics, use pointers or references (see [21_Mutability_and_Pointers.md](21_Mutability_and_Pointers.md)).

### Default Parameter Values

⭐ **NEW in v0.2.0**: Parameters can have default values.

**Syntax**: `parameter: type = default_expression`

```vex
// Simple default value
fn greet(name: string = "World") {
    print("Hello, ", name, "!");
}

// Multiple defaults
fn create_point(x: i32 = 0, y: i32 = 0): Point {
    return Point { x: x, y: y };
}

// Mixed: required and optional parameters
fn add_numbers(a: i32, b: i32 = 10, c: i32 = 20): i32 {
    return a + b + c;
}

// With parameter grouping
fn process(x, y: f64 = 1.0): f64 {
    return x * y;
}
```

**Calling with defaults**:

```vex
// Use all defaults
greet();  // "Hello, World!"

// Override some defaults
create_point(5);  // Point { x: 5, y: 0 }

// Override all
create_point(5, 10);  // Point { x: 5, y: 10 }

// Mixed parameters
add_numbers(1);        // 1 + 10 + 20 = 31
add_numbers(1, 2);     // 1 + 2 + 20 = 23
add_numbers(1, 2, 3);  // 1 + 2 + 3 = 6
```

**Rules**:

- Default values can be any compile-time constant expression
- Parameters with defaults must come after required parameters
- When calling, you can omit trailing parameters with defaults
- You cannot skip a parameter in the middle (no named arguments yet)

**Examples**:

```vex
// ✅ Valid
fn foo(a: i32, b: i32 = 10) { }
fn bar(x: i32, y: i32 = 5, z: i32 = 3) { }

// ❌ Invalid: default before required
fn baz(a: i32 = 10, b: i32) { }  // Compile error

// Calling
foo(1);     // OK: a=1, b=10
foo(1, 2);  // OK: a=1, b=2

bar(1);        // OK: x=1, y=5, z=3
bar(1, 2);     // OK: x=1, y=2, z=3
bar(1, 2, 3);  // OK: x=1, y=2, z=3
```

**Implementation**: The compiler automatically fills in missing arguments with their default expressions during code generation. This is a zero-cost abstraction - no runtime overhead.

### Variadic Parameters

✅ **Implemented in v0.2.0**: Functions can accept variable number of arguments.

**Syntax**: `parameter_name: ...Type`

```vex
// Simple variadic
fn sum(base: i32, numbers: ...i32): i32 {
    // numbers is variadic - can accept 0 or more i32 values
    return base;  // TODO: iterate over numbers when runtime support added
}

// Variadic with defaults
fn greet_many(prefix: string = "Hello", names: ...string) {
    print(prefix, " to everyone!");
}

// Only variadic parameter
fn count_all(items: ...i32): i32 {
    // Would return count of items
    return 0;
}
```

**Calling variadic functions**:

```vex
// Pass multiple arguments
sum(10, 1, 2, 3, 4, 5);

// Combine defaults and variadic
greet_many("Hi", "Alice", "Bob", "Charlie");

// Use default for regular param
greet_many("World");  // Uses "Hello" default

// Pass many variadic args
count_all(1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
```

**Rules**:

- ✅ Variadic parameter must be the LAST parameter
- ✅ Only ONE variadic parameter per function
- ✅ Can combine with default parameters
- ✅ Variadic parameters can accept zero or more arguments
- ⚠️ Runtime iteration over variadic args not yet implemented
- ⚠️ Currently used mainly for FFI (C variadic functions)

**Examples**:

```vex
// ✅ Valid
fn foo(a: i32, items: ...string) { }
fn bar(prefix: string = "default", args: ...i32) { }

// ❌ Invalid: variadic not last
fn baz(items: ...i32, suffix: string) { }  // Compile error

// ❌ Invalid: multiple variadic
fn qux(items1: ...i32, items2: ...string) { }  // Compile error
```

**Current Status**:

- ✅ Parser support: `name: ...Type` syntax
- ✅ Type checking: variadic type validation
- ✅ Codegen: accepts variable argument count
- ⏳ Runtime: iteration over variadic args (future feature)

**Future**: Access variadic arguments via slice or iterator:

```vex
// Future syntax (not yet implemented)
fn sum(numbers: ...i32): i32 {
    let! total = 0;
    for num in numbers {  // Iterate over variadic args
        total = total + num;
    }
    return total;
}
```

---

## Return Values

# Control Flow

**Version:** 0.1.0  
**Last Updated:** November 3, 2025

This document defines control flow constructs in the Vex programming language.

---

## Table of Contents

1. [Conditional Statements](#conditional-statements)
2. [Pattern Matching](#pattern-matching)
3. [Loops](#loops)
4. [Control Transfer](#control-transfer)
5. [Defer Statement](#defer-statement)
6. [Error Handling](#error-handling)

---

## Conditional Statements

### If Expression

**Basic Syntax**:

```vex
if condition {
    // body
}
```

**Properties**:

- Condition must be `bool` type (no implicit conversion)
- Braces are required (no braceless syntax)
- Body is a new scope

**Example**:

```vex
let x = 10;
if x > 5 {
    // x is greater than 5
}
```

### If-Else

```vex
if condition {
    // true branch
} else {
    // false branch
}
```

**Answer**: 🔴 `select` statement (High Priority) - Go-style channel selection

```vex
select {
    case msg = <-ch1:
        println("Received from ch1");
    case ch2 <- value:
        println("Sent to ch2");
    default:
        println("No channel ready");
}
```

Channel implementation ile birlikte gelecek. Şu an channels implement edilmemiş, o yüzden bu dokümanda yok. 13_Concurrency.md'de mention edilmeli.

**Example**:

```vex
let age = 18;
if age >= 18 {
    // adult
} else {
    // minor
}
```

### If-Elif-Else Chain (v0.1)

Use `elif` for else-if chains:

```vex
if condition1 {
    // first branch
} elif condition2 {
    // second branch
} elif condition3 {
    // third branch
} else {
    // default branch
}
```

**Example**:

```vex
let score = 85;
if score >= 90 {
    // grade A
} elif score >= 80 {
    // grade B
} elif score >= 70 {
    // grade C
} else {
    // grade F
}
```

**Note**: `elif` keyword introduced in v0.1 (replaces older `else if` syntax)

### Nested If

```vex
if outer_condition {
    if inner_condition {
        // nested body
    }
}
```

**Example**:

```vex
let age = 20;
let has_license = true;

if age >= 18 {
    if has_license {
        // can drive
    }
}
```

### If as Expression (Future)

```vex
let value = if condition { 10 } else { 20 };
```

---

## Pattern Matching

### Match Expression

**Syntax**:

```vex
match value {
    pattern1 => { body1 }
    pattern2 => { body2 }
    _ => { default }
}
```

**Properties**:

- Must be exhaustive (all cases covered)
- Evaluates top-to-bottom (first match wins)
- `_` is wildcard pattern (matches anything)

### Literal Patterns

```vex
match x {
    0 => { /* zero */ }
    1 => { /* one */ }
    2 => { /* two */ }
    _ => { /* other */ }
}
```

**Example**:

```vex
let day = 3;
match day {
    1 => { /* Monday */ }
    2 => { /* Tuesday */ }
    3 => { /* Wednesday */ }
    4 => { /* Thursday */ }
    5 => { /* Friday */ }
    6 => { /* Saturday */ }
    7 => { /* Sunday */ }
    _ => { /* invalid */ }
}
```

### Enum Patterns

```vex
enum Color {
    Red,
    Green,
    Blue,
}

let color = Color.Red;
match color {
    Color.Red => { /* red */ }
    Color.Green => { /* green */ }
    Color.Blue => { /* blue */ }
}
```

**Exhaustiveness Check**:

```vex
match color {
    Color.Red => { }
    Color.Green => { }
    // ERROR: Missing Color.Blue case
}
```

### Or Patterns (v0.1)

Match multiple patterns with `|`:

```vex
match x {
    1 | 2 | 3 => { /* low */ }
    4 | 5 | 6 => { /* medium */ }
    7 | 8 | 9 => { /* high */ }
    _ => { /* other */ }
}
```

**Example**:

```vex
match day {
    1 | 2 | 3 | 4 | 5 => { /* weekday */ }
    6 | 7 => { /* weekend */ }
    _ => { /* invalid */ }
}
```

### Tuple Patterns

```vex
let point = (10, 20);
match point {
    (0, 0) => { /* origin */ }
    (0, y) => { /* on y-axis */ }
    (x, 0) => { /* on x-axis */ }
    (x, y) => { /* general point */ }
}
```

**Destructuring**:

```vex
let pair = (1, 2);
match pair {
    (a, b) => {
        // a = 1, b = 2
    }
}
```

### Struct Patterns (Future)

```vex
struct Point { x: i32, y: i32 }

let p = Point { x: 10, y: 20 };
match p {
    Point { x: 0, y: 0 } => { /* origin */ }
    Point { x, y: 0 } => { /* on x-axis, x = 10 */ }
    Point { x, y } => { /* general, x=10, y=20 */ }
}
```

### Range Patterns (Future)

```vex
match age {
    0..=12 => { /* child */ }
    13..=17 => { /* teen */ }
    18..=64 => { /* adult */ }
    65.. => { /* senior */ }
}
```

### Guards (Future)

Add conditions to patterns:

```vex
match x {
    n if n < 0 => { /* negative */ }
    n if n == 0 => { /* zero */ }
    n if n > 0 => { /* positive */ }
}
```

### Data-Carrying Enum Patterns (Future)

```vex
enum Option<T> {
    Some(T),
    None,
}

let value = Some(42);
match value {
    Some(x) => { /* x = 42 */ }
    None => { /* no value */ }
}
```

---

## Loops

### While Loop

**Syntax**:

```vex
while condition {
    // body
}
```

**Example**:

```vex
let! counter = 0;
while counter < 10 {
    counter = counter + 1;
}
```

**Infinite Loop**:

```vex
while true {
    // runs forever (until break)
}
```

### For Loop

**Syntax**:

```vex
for variable in start..end {
    // body
}
```

**Range-Based**:

```vex
for i in 0..10 {
    // i = 0, 1, 2, ..., 9
}
```

**Example**:

```vex
let! sum = 0;
for i in 1..11 {
    sum = sum + i;
}
// sum = 55 (1+2+...+10)
```

**Inclusive Range**:

```vex
for i in 0..=10 {
    // i = 0, 1, 2, ..., 10 (includes 10)
}
```

**Operators**:

- `..` - Exclusive range: `0..10` → 0, 1, 2, ..., 9
- `..=` - Inclusive range: `0..=10` → 0, 1, 2, ..., 10

### Loop (Infinite Loop) (Future)

```vex
loop {
    // runs forever
    if condition {
        break;
    }
}
```

**Equivalent to**:

```vex
while true {
    // body
}
```

### For-Each (Future)

Iterate over collections:

```vex
let numbers = [1, 2, 3, 4, 5];
for num in numbers {
    // num = 1, then 2, then 3, ...
}
```

**With Index**:

```vex
for (index, value) in numbers.enumerate() {
    // index = 0, 1, 2, ...
    // value = 1, 2, 3, ...
}
```

---

## Control Transfer

### Break

Exit from loop early:

```vex
let! i = 0;
while i < 10 {
    if i == 5 {
        break;  // Exit loop
    }
    i = i + 1;
}
// i = 5
```

**In Match** (Future):

```vex
while true {
    match get_input() {
        "quit" => { break; }
        cmd => { process(cmd); }
    }
}
```

### Continue

Skip to next iteration:

```vex
for i in 0..10 {
    if i % 2 == 0 {
        continue;  // Skip even numbers
    }
    // Only odd numbers reach here
}
```

**Example**:

```vex
let! count = 0;
for i in 1..101 {
    if i % 3 == 0 {
        continue;  // Skip multiples of 3
    }
    count = count + 1;
}
// count = 67 (100 - 33 multiples of 3)
```

### Return

Exit from function:

```vex
fn find(arr: [i32; 10], target: i32): i32 {
    for i in 0..10 {
        if arr[i] == target {
            return i;  // Found, exit function
        }
    }
    return -1;  // Not found
}
```

**Early Return**:

```vex
fn validate(x: i32): bool {
    if x < 0 {
        return false;  // Early exit
    }
    if x > 100 {
        return false;  // Early exit
    }
    return true;
}
```

### Labeled Breaks (Future)

Break from nested loops:

```vex
'outer: for i in 0..10 {
    for j in 0..10 {
        if i * j > 50 {
            break 'outer;  // Break outer loop
        }
    }
}
```

---

## Error Handling

### Result Type (Future)

Use union types for error handling:

```vex
type Result<T> = (T | error);

fn divide(a: i32, b: i32): Result<i32> {
    if b == 0 {
        return "Division by zero";
    }
    return a / b;
}
```

**Pattern Matching on Result**:

```vex
let result = divide(10, 2);
match result {
    value when value is i32 => {
        // Success: value = 5
    }
    err when err is error => {
        // Error: handle err
    }
}
```

### Option Type (Future)

Represent optional values:

```vex
type Option<T> = (T | nil);

fn find(arr: [i32], target: i32): Option<i32> {
    for i in 0..arr.len() {
        if arr[i] == target {
            return i;
        }
    }
    return nil;
}
```

**Unwrapping**:

```vex
let result = find([1, 2, 3], 2);
match result {
    index when index is i32 => { /* found at index */ }
    nil => { /* not found */ }
}
```

### Try-Catch (Future Consideration)

```vex
try {
    let result = risky_operation();
    process(result);
} catch err {
    handle_error(err);
}
```

### Panic

Abort program execution:

```vex
fn unreachable_code() {
    @unreachable();  // Compiler hint
}

fn assert_positive(x: i32) {
    if x <= 0 {
        panic("Value must be positive");
    }
}
```

---

## Examples

### If-Elif-Else

```vex
fn classify_age(age: i32): i32 {
    if age < 0 {
        return -1;  // Invalid
    } elif age < 13 {
        return 0;   // Child
    } elif age < 20 {
        return 1;   // Teen
    } elif age < 65 {
        return 2;   // Adult
    } else {
        return 3;   // Senior
    }
}
```

### Match with Enums

```vex
enum Status {
    Active = 0,
    Inactive = 1,
    Pending = 2,
}

fn handle_status(status: Status): i32 {
    match status {
        Active => {
            return 1;
        }
        Inactive => {
            return 0;
        }
        Pending => {
            return -1;
        }
    }
}
```

### While Loop

```vex
fn count_down(n: i32): i32 {
    let! counter = n;
    while counter > 0 {
        counter = counter - 1;
    }
    return counter;  // 0
}
```

### For Loop

```vex
fn sum_range(start: i32, end: i32): i32 {
    let! sum = 0;
    for i in start..end {
        sum = sum + i;
    }
    return sum;
}

fn main(): i32 {
    return sum_range(1, 11);  // 55
}
```

### Break and Continue

```vex
fn find_first_even(numbers: [i32; 10]): i32 {
    for i in 0..10 {
        if numbers[i] % 2 == 1 {
            continue;  // Skip odd numbers
        }
        return numbers[i];  // Return first even
    }
    return -1;  // No even number found
}
```

---

## Defer Statement

### Syntax

**Purpose**: Execute code when function exits, regardless of how it exits.

**Status**: ✅ Fully implemented - deferred statements execute in LIFO order on function exit

**Keyword**: `defer`

```vex
fn example() {
    defer cleanup();  // Executes when function returns
    // ... function body
}
```

### Basic Usage

```vex
fn read_file(path: string): string {
    let file = open(path);
    defer close(file);  // Always closes, even on error

    if !file.is_valid() {
        return "";  // defer executes before return
    }

    return file.read_all();
}  // defer executes here
```

### Multiple Defer Statements

**Execution Order**: LIFO (Last In, First Out) - Reverse order of declaration

```vex
fn process_data() {
    defer println("Step 3: Final cleanup");
    defer println("Step 2: Release lock");
    defer println("Step 1: Close connection");

    // Function body
    println("Processing...");
}

// Output:
// Processing...
// Step 1: Close connection
// Step 2: Release lock
// Step 3: Final cleanup
```

### Resource Management

**File Handling**:

```vex
fn copy_file(src: string, dst: string): bool {
    let src_file = open(src);
    defer close(src_file);

    let dst_file = create(dst);
    defer close(dst_file);

    // Both files automatically closed on return
    return copy_content(src_file, dst_file);
}
```

**Memory Management**:

```vex
fn process_buffer(): i32 {
    let buffer = allocate(1024);
    defer free(buffer);

    // Use buffer...
    let result = compute(buffer);

    return result;
}  // buffer freed automatically
```

**Lock Management**:

```vex
fn update_shared_data(mutex: &Mutex!, data: i32) {
    mutex.lock();
    defer mutex.unlock();

    // Critical section
    shared_value = data;

    // mutex unlocked automatically, even if panic occurs
}
```

### Defer with Closures (Future)

```vex
fn complex_cleanup() {
    let! counter = 0;
    defer {
        // Closure can access function variables
        println("Counter was: " + counter);
    };

    counter = 42;
}  // Prints: "Counter was: 42"
```

### Error Handling with Defer

```vex
fn risky_operation(): (i32 | error) {
    let resource = acquire();
    defer release(resource);

    if problem() {
        return "Error occurred";  // defer runs before return
    }

    return 42;
}
```

### Common Patterns

**1. RAII-style Resource Management**:

```vex
fn database_transaction(): bool {
    let tx = db.begin_transaction();
    defer tx.rollback();  // Safety net

    if !tx.insert(...) {
        return false;  // Rollback happens
    }

    tx.commit();
    return true;
}
```

**2. Cleanup Stack**:

```vex
fn multi_step_process(): i32 {
    let step1 = init_step1();
    defer cleanup_step1(step1);

    let step2 = init_step2();
    defer cleanup_step2(step2);

    let step3 = init_step3();
    defer cleanup_step3(step3);

    return execute();
}  // Cleanup in reverse: step3, step2, step1
```

**3. Timing and Logging**:

```vex
fn measured_operation() {
    let start_time = now();
    defer {
        let elapsed = now() - start_time;
        println("Operation took: " + elapsed + "ms");
    };

    // Expensive operation
    compute_heavy_task();
}
```

### Comparison with Other Languages

| Feature       | Vex     | Go      | Rust          | C++       |
| ------------- | ------- | ------- | ------------- | --------- |
| **Keyword**   | `defer` | `defer` | N/A           | N/A       |
| **RAII**      | Manual  | Manual  | Automatic     | Manual    |
| **Execution** | On exit | On exit | On drop       | On scope  |
| **Order**     | LIFO    | LIFO    | LIFO (drop)   | LIFO      |
| **Closures**  | ✅ Yes  | ✅ Yes  | ✅ Yes (Drop) | ✅ Lambda |

### Implementation Status

- ✅ Keyword reserved (`defer`)
- ✅ Parser support (COMPLETE - Nov 9, 2025)
- ✅ Codegen implemented (LIFO execution)
- ✅ Stack unwinding integration working
- **Priority**: ✅ COMPLETE

**Examples**: See `examples/defer_*.vx` for working demonstrations

---

### Nested Loops

```vex
fn matrix_sum(rows: i32, cols: i32): i32 {
    let! sum = 0;
    for i in 0..rows {
        for j in 0..cols {
            sum = sum + (i * cols + j);
        }
    }
    return sum;
}
```

### Early Return

```vex
fn is_prime(n: i32): bool {
    if n <= 1 {
        return false;  // Early return
    }
    if n == 2 {
        return true;   // Early return
    }
    if n % 2 == 0 {
        return false;  // Early return
    }

    // Check odd divisors
    let! i = 3;
    while i * i <= n {
        if n % i == 0 {
            return false;
        }
        i = i + 2;
    }
    return true;
}
```

---

## Best Practices

### 1. Use Match Over If Chains

```vex
// Good: Clear, exhaustive
match status {
    Active => { }
    Inactive => { }
    Pending => { }
}

// Bad: Verbose, error-prone
if status == Active {
    // ...
} elif status == Inactive {
    // ...
} elif status == Pending {
    // ...
}
```

### 2. Prefer Early Returns

```vex
// Good: Early validation
fn process(x: i32): i32 {
    if x < 0 {
        return -1;
    }
    if x == 0 {
        return 0;
    }
    // Main logic
    return x * 2;
}

// Bad: Deep nesting
fn process(x: i32): i32 {
    if x >= 0 {
        if x != 0 {
            // Main logic
            return x * 2;
        } else {
            return 0;
        }
    } else {
        return -1;
    }
}
```

### 3. Avoid Deep Nesting

```vex
// Good: Flat structure
fn validate(x: i32, y: i32, z: i32): bool {
    if x < 0 { return false; }
    if y < 0 { return false; }
    if z < 0 { return false; }
    return true;
}

// Bad: Deep nesting
fn validate(x: i32, y: i32, z: i32): bool {
    if x >= 0 {
        if y >= 0 {
            if z >= 0 {
                return true;
            }
        }
    }
    return false;
}
```

### 4. Use Descriptive Conditions

```vex
// Good: Named condition
let is_adult = age >= 18;
let has_permission = role == "admin";

if is_adult && has_permission {
    // Clear intent
}

// Bad: Complex inline condition
if age >= 18 && role == "admin" && status == "active" {
    // What does this check?
}
```

### 5. Limit Loop Complexity

```vex
// Good: Simple loop body
for i in 0..10 {
    process_item(i);
}

// Bad: Complex logic in loop
for i in 0..10 {
    if condition1 {
        if condition2 {
            for j in 0..5 {
                // Too complex
            }
        }
    }
}
```

---

---

## Select Statement (Future)

### Syntax (Go-style)

**Purpose**: Wait on multiple channel operations

```vex
select {
    case msg = <-ch1:
        println("Received from ch1");
    case ch2 <- value:
        println("Sent to ch2");
    case msg = <-ch3:
        println("Received from ch3");
    default:
        println("No channel ready");
}
```

### Semantics

- **Blocks** until one case is ready
- If multiple cases ready, **randomly** chooses one
- `default` case executes immediately if no channel ready
- Without `default`, blocks forever if no channel ready

### Example: Timeout Pattern

```vex
import { channel, timeout } from "sync";

fn fetch_with_timeout(): (string | error) {
    let result_ch = channel<string>();
    let timeout_ch = timeout(5000); // 5 seconds

    go fetch_data(result_ch);

    select {
        case data = <-result_ch:
            return data;
        case <-timeout_ch:
            return "Timeout error";
    }
}
```

### Current Status

**Syntax**: ✅ `select` keyword reserved  
**Parser**: 🚧 Partial (keyword recognized, AST node exists)  
**Channels**: ✅ MPSC channels implemented (lock-free ring buffer)  
**Priority**: � Medium (Channel infrastructure complete, select syntax pending)

**Note**: Basic channel operations (`send`, `recv`, `close`) fully working. Multi-channel `select` syntax planned.

See [13_Concurrency.md](./13_Concurrency.md) for full concurrency model.

### Switch Statement

C-style switch with integer values:

**Syntax**: `switch value { case val: { } default: { } }`

```vex
switch day {
    case 1:
        println("Monday");
    case 2:
        println("Tuesday");
    case 3:
        println("Wednesday");
    case 4:
        println("Thursday");
    case 5:
        println("Friday");
    case 6:
        println("Saturday");
    case 7:
        println("Sunday");
    default:
        println("Invalid day");
}
```

**Properties**:

- Only works with integer types (i32, u32, etc.)
- No implicit fallthrough (unlike C)
- Must have `default` case (unlike C)
- Each case must be a compile-time constant

**Differences from C**:

- No fallthrough by default
- Requires `default` case
- Only integer types supported
- No expression cases (use `match` instead)

---

## Control Flow Summary

| Construct    | Syntax                     | Use Case             | Status |
| ------------ | -------------------------- | -------------------- | ------ |
| If           | `if cond { }`              | Simple branching     | ✅     |
| If-Else      | `if cond { } else { }`     | Binary choice        | ✅     |
| If-Elif-Else | `if { } elif { } else { }` | Multiple conditions  | ✅     |
| Match        | `match val { pat => { } }` | Pattern matching     | ✅     |
| Switch       | `switch val { case ... }`  | Integer switching    | ✅     |
| While        | `while cond { }`           | Condition-based loop | ✅     |
| For          | `for i in range { }`       | Iteration            | ✅     |
| Defer        | `defer cleanup();`         | LIFO cleanup         | ✅     |
| Select       | `select { case ... }`      | Channel multiplexing | ❌     |
| Break        | `break;`                   | Exit loop            | ✅     |
| Continue     | `continue;`                | Skip iteration       | ✅     |
| Return       | `return value;`            | Exit function        | ✅     |

---

**Previous**: [05_Functions_and_Methods.md](./05_Functions_and_Methods.md)  
**Next**: [07_Structs_and_Data_Types.md](./07_Structs_and_Data_Types.md)

**Maintained by**: Vex Language Team

# Structs and Data Types

**Version:** 0.1.0  
**Last Updated:** November 3, 2025

This document defines struct types and related data structures in the Vex programming language.

---

## Table of Contents

1. [Struct Definitions](#struct-definitions)
2. [Struct Instantiation](#struct-instantiation)
3. [Field Access](#field-access)
4. [Methods on Structs](#methods-on-structs)
5. [Generic Structs](#generic-structs)
6. [Tuple Structs](#tuple-structs)
7. [Unit Structs](#unit-structs)
8. [Memory Layout](#memory-layout)

---

## Struct Definitions

### Basic Syntax

**Syntax**: `struct Name { fields }`

```vex
struct Point {
    x: i32,
    y: i32,
}

struct Person {
    name: string,
    age: i32,
    email: string,
}
```

**Properties**:

- `struct` keyword
- Name in PascalCase (convention)
- Fields in braces with types
- Comma-separated fields
- Nominal typing (name-based, not structural)

### Field Declaration

Each field has name and type:

```vex
struct Rectangle {
    width: i32,
    height: i32,
}
```

**Multiple Fields**:

```vex
struct User {
    id: u64,
    username: string,
    email: string,
    age: i32,
    is_active: bool,
}
```

### Struct Tags (Go-style)

Vex supports Go-style struct tags for metadata:

```vex
struct User {
    id: u64        `json:"id" db:"pk"`,
    username: string `json:"username" db:"username"`,
    email: string   `json:"email" db:"email"`,
    age: i32        `json:"age"`,
    is_active: bool `json:"is_active"`,
}
```

**Syntax**: Backtick-enclosed string literals after field declarations

**Use Cases**:

- JSON serialization/deserialization
- Database mapping
- Validation rules
- API documentation

**Properties**:

- Ignored by compiler (metadata only)
- Available at runtime via reflection
- Multiple tags separated by spaces
- Convention: `key:"value"` format

**Different Types**:

```vex
struct Mixed {
    integer: i32,
    floating: f64,
    boolean: bool,
    text: string,
    array: [i32; 10],
    tuple: (i32, i32),
}
```

### Struct Tags (Go-style)

Vex supports Go-style backtick struct tags for metadata:

```vex
struct User {
    id: u64        `json:"id" db:"pk"`,
    username: string `json:"username" db:"username"`,
    email: string   `json:"email" db:"email"`,
    age: i32        `json:"age"`,
    is_active: bool `json:"is_active"`,
}
```

**Syntax**: Backtick-enclosed string literals after field type

**Use Cases**:

- JSON serialization field mapping
- Database column mapping
- Validation rules
- API documentation
- Reflection metadata

**Implementation Status**: ✅ Fully implemented

- Struct tags ARE parsed and stored in AST (`Field.tag`)
- Metadata available in compiler
- **IMPORTANT**: Vex does NOT use Rust-style `#[attribute]` syntax
- Runtime reflection builtins: `typeof`, `type_id`, `type_size`, `type_align`, `is_*_type` functions
- Policy system provides rich metadata annotations

### Nested Structs

Structs can contain other structs:

```vex
struct Address {
    street: string,
    city: string,
    zip: i32,
}

struct Person {
    name: string,
    address: Address,
}
```

**Example Usage**:

```vex
let addr = Address {
    street: "123 Main St",
    city: "NYC",
    zip: 10001,
};

let person = Person {
    name: "Alice",
    address: addr,
};
```

### Self-Referential Structs (Limited)

Structs cannot directly contain themselves:

```vex
// ERROR: Infinite size
struct Node {
    value: i32,
    next: Node,  // ERROR
}
```

**Use references instead** (future):

```vex
struct Node {
    value: i32,
    next: &Node!,  // OK: Pointer, fixed size
}
```

---

## Struct Instantiation

### Full Initialization

All fields must be provided:

```vex
let point = Point {
    x: 10,
    y: 20,
};
```

**Order doesn't matter**:

```vex
let p1 = Point { x: 10, y: 20 };
let p2 = Point { y: 20, x: 10 };  // Same as p1
```

### Missing Fields (Error)

```vex
// ERROR: Missing field 'y'
let point = Point { x: 10 };
```

All fields must be initialized.

### Field Init Shorthand (Future)

```vex
let x = 10;
let y = 20;
let point = Point { x, y };  // Shorthand for { x: x, y: y }
```

### Update Syntax (Future)

Copy existing struct with some fields changed:

```vex
let p1 = Point { x: 10, y: 20 };
let p2 = Point { x: 30, ..p1 };  // y copied from p1
```

---

## Field Access

### Reading Fields

Use dot notation:

```vex
let point = Point { x: 10, y: 20 };
let x_coord = point.x;  // 10
let y_coord = point.y;  // 20
```

**Nested Access**:

```vex
let person = Person {
    name: "Alice",
    address: Address {
        street: "Main St",
        city: "NYC",
        zip: 10001,
    },
};

let city = person.address.city;  // "NYC"
```

### Writing Fields

Only possible with mutable variables:

```vex
let! point = Point { x: 10, y: 20 };
point.x = 30;  // OK: point is mutable
point.y = 40;  // OK
```

**Immutable Structs**:

```vex
let point = Point { x: 10, y: 20 };
// point.x = 30;  // ERROR: Cannot assign to immutable variable
```

### Field Access Through References

**Immutable Reference**:

```vex
let point = Point { x: 10, y: 20 };
let ref_point: &Point = &point;
let x = ref_point.x;  // OK: Read through reference
```

**Mutable Reference**:

```vex
let! point = Point { x: 10, y: 20 };
let ref_point: &Point! = &point;
ref_point.x = 30;  // OK: Write through mutable reference
```

**Note**: Auto-dereference for field access (future feature)

---

## Methods on Structs

Vex uses a hybrid model for method mutability. See `05_Functions_and_Methods.md` for the full specification.

### Inline Methods (in `struct` or `contract`)

- **Declaration**: `fn method_name()!` for mutable, `fn method_name()` for immutable.
- **Behavior**: A mutable method can modify `self`.
- **Call**: `object.method_name()` (no `!` at call site). The compiler ensures a mutable method is only called on a mutable (`let!`) variable.

```vex
struct Rectangle {
    width: i32,
    height: i32,

    // Immutable method
    fn area(): i32 {
        return self.width * self.height;
    }

    // Mutable method
    fn scale(factor: i32)! {
        self.width = self.width * factor;
        self.height = self.height * factor;
    }
}

// --- Calls ---
let rect = Rectangle { width: 10, height: 20 };
let a = rect.area(); // OK

let! rect_mut = Rectangle { width: 10, height: 20 };
rect_mut.scale(2); // OK
```

### External Methods (Golang-Style)

- **Declaration**: `fn (self: &MyType!) method_name()` for mutable, `fn (self: &MyType) method_name()` for immutable.
- **Behavior**: A mutable method can modify `self`.
- **Call**: `object.method_name()` (no `!` at call site).

```vex
struct Circle {
    radius: f64,
}

// Immutable external method
fn (c: &Circle) circumference(): f64 {
    return 2.0 * 3.14159 * c.radius;
}

// Mutable external method
fn (c: &Circle!) set_radius(new_radius: f64) {
    c.radius = new_radius;
}

// --- Calls ---
let! circle = Circle { radius: 5.0 };
circle.set_radius(10.0);
```

### Contract Methods vs Extra Methods

**Contract Methods**: MUST be in struct body

```vex
contract Shape {
    fn area(): f64;
    fn scale(factor: f64)!;
}

struct Rectangle impl Shape {
    width: f64,
    height: f64,

    // Contract methods MUST be here
    fn area(): f64 {
        return self.width * self.height;
    }

    fn scale(factor: f64)! {
        self.width = self.width * factor;
        self.height = self.height * factor;
    }
}

// ❌ ERROR: Contract methods cannot be external
fn (r: &Rectangle) area(): f64 {
    return r.width * r.height;
}
```

**Extra Methods**: Can be external

```vex
// ✅ OK: Extra methods can be external
fn (rect: &Rectangle) diagonal(): f64 {
    return sqrt(rect.width * rect.width + rect.height * rect.height);
}
```

# Enums (Enumerated Types)

**Version:** 0.1.0  
**Last Updated:** November 3, 2025

This document defines enumerated types (enums) in the Vex programming language.

---

## Table of Contents

1. [Enum Definitions](#enum-definitions)
2. [Enum Variants](#enum-variants)
3. [Pattern Matching](#pattern-matching)
4. [Methods on Enums](#methods-on-enums)
5. [Generic Enums](#generic-enums)
6. [Memory Representation](#memory-representation)

---

## Enum Definitions

### Basic Syntax

**Syntax**: `enum Name { variants }`

```vex
enum Color {
    Red,
    Green,
    Blue,
}

enum Status {
    Active,
    Inactive,
    Pending,
}
```

**Properties**:

- `enum` keyword
- Name in PascalCase (convention)
- Variants in braces
- Comma-separated variants
- Each variant is a distinct value

### Unit Variants

Simplest form - variants with no associated data:

```vex
enum Direction {
    North,
    South,
    East,
    West,
}
```

**Usage**:

```vex
let dir = Direction::North;
let opposite = Direction::South;
```

### Explicit Discriminants

Assign integer values to variants:

```vex
enum HttpStatus {
    OK = 200,
    NotFound = 404,
    ServerError = 500,
}

enum Priority {
    Low = 1,
    Medium = 2,
    High = 3,
}
```

**Auto-Increment**:

```vex
enum Number {
    Zero = 0,
    One,      // 1 (auto-incremented)
    Two,      // 2
    Five = 5,
    Six,      // 6
}
```

---

## Enum Variants

### Unit Variants (C-Style)

Currently supported - simple discriminated values:

```vex
enum Color {
    Red,
    Green,
    Blue,
}

fn main(): i32 {
    let color = Red;
    return 0;
}
```

**Discriminant Values**:

```vex
enum Status {
    Active = 0,
    Inactive = 1,
    Pending = 2,
}
```

### Data-Carrying Variants (Future)

Variants that hold additional data:

```vex
enum Option<T> {
    Some(T),
    None,
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

**Complex Data**:

```vex
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(string),
    ChangeColor(i32, i32, i32),
}
```

### Tuple Variants ✅ COMPLETE (v0.1.2)

Enum variants can carry data in tuple form:

**Single-Value Tuple Variants**:

```vex
enum Option<T> {
    Some(T),
    None,
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}

let x = Option.Some(42);
let result = Result.Ok("success");
```

**Multi-Value Tuple Variants** ✅ NEW (v0.1.2):

```vex
enum IpAddr {
    V4(u8, u8, u8, u8),
    V6(string),
}

let localhost = IpAddr.V4(127, 0, 0, 1);
let google = IpAddr.V4(8, 8, 8, 8);

match localhost {
    IpAddr.V4(a, b, c, d) => {
        // Successfully extracts all 4 values
    },
    IpAddr.V6(addr) => {
        // Single value extraction
    },
};
```

**Implementation Details**:

- **Parser**: `vex-parser/src/parser/items/enums.rs` - Supports `data: Vec<Type>`
  - Syntax: `VariantName(Type1, Type2, Type3, ...)`
  - Parses comma-separated type list in parentheses
  - Empty `Vec` for unit variants
- **AST**: `vex-ast/src/lib.rs`
  - `EnumVariant { name: String, data: Vec<Type> }`
  - Supports 0+ tuple fields per variant
- **Codegen**: `vex-compiler/src/codegen_ast/enums.rs`
  - Single-value: Direct data storage `{ i32 tag, T data }`
  - Multi-value: Nested struct `{ i32 tag, struct { T1, T2, T3 } data }`
  - Tag: i32 discriminant (variant index)
  - Memory layout optimized for type size
- **Pattern Matching**: `vex-compiler/src/codegen_ast/expressions/pattern_matching.rs`
  - `compile_pattern_check()`: Tag validation + data extraction
  - `compile_pattern_binding()`: Binds each tuple field to pattern variable
  - Multi-value: Extracts each field from data struct via `build_extract_value()`
  - Full support for nested patterns
- **Tests**:
  - `examples/06_patterns/enum_data.vx` - Single-value variants (Option, Result)
  - `examples/06_patterns/enum_multi_tuple.vx` - Multi-value variants (IpAddr)
  - `examples/04_types/enum_data_complete.vx` - Comprehensive enum tests

**Memory Layout Example**:

```c
// IpAddr.V4(127, 0, 0, 1) in memory:
struct {
    i32 tag;        // 0 (variant index)
    struct {
        u8 field_0; // 127
        u8 field_1; // 0
        u8 field_2; // 0
        u8 field_3; // 1
    } data;
}
```

**Advanced Examples**:

```vex
// Complex multi-value tuples
enum Message {
    Move(i32, i32),                    // 2 fields
    Color(u8, u8, u8, u8),             // 4 fields (RGBA)
    Transform(f32, f32, f32, f32, f32, f32),  // 6 fields (matrix)
}

let msg = Message.Color(255, 128, 64, 255);

match msg {
    Message.Move(x, y) => {
        println("Move to ({}, {})", x, y);
    },
    Message.Color(r, g, b, a) => {
        println("Color: rgba({}, {}, {}, {})", r, g, b, a);
    },
    Message.Transform(a, b, c, d, e, f) => {
        println("Transform matrix");
    },
};
```

**Type Constraints**:

- All tuple fields must have concrete types (no inference)
- Generic types are supported: `Some(T)`, `V4(T, T, T, T)`
- Recursive types allowed: `Node(i32, Box<Node>)`
- No tuple size limit (practical limit: 255 fields)

### Struct Variants (Future)

```vex
enum Shape {
    Circle { radius: f64 },
    Rectangle { width: f64, height: f64 },
    Triangle { base: f64, height: f64 },
}
```

---

## Pattern Matching

### Basic Match

```vex
enum Color {
    Red,
    Green,
    Blue,
}

fn print_color(c: Color): i32 {
    match c {
        Red => {
            println("Red");
        }
        Green => {
            println("Green");
        }
        Blue => {
            println("Blue");
        }
    }
    return 0;
}
```

### Exhaustiveness

Match must cover all variants:

```vex
enum Status {
    Active,
    Inactive,
    Pending,
}

// OK: All variants covered
match status {
    Active => { }
    Inactive => { }
    Pending => { }
}

// ERROR: Missing Pending
match status {
    Active => { }
    Inactive => { }
}
```

### Wildcard Pattern

Use `_` to match remaining cases:

```vex
// Specific pattern
match status {
    Active => { /* handle active */ }
    _ => { /* handle all other cases */ }
}
```

### Or Patterns

Match multiple variants:

```vex
match status {
    Active | Pending => {
        // Handle both active and pending
    }
    Inactive => {
        // Handle inactive
    }
}
```

### Data Extraction (Future)

Extract data from data-carrying variants:

````vex
enum Option<T> {
    Some(T),
    None,
}

```vex
let value = Some(42);
match value {
    Some(x) => {
        // x = 42
    }
    None => {
        // No value
    }
}
````

````

**Named Fields**:

```vex
enum Message {
    Move { x: i32, y: i32 },
}

match msg {
    Move { x, y } => {
        // x and y extracted
    }
}
````

---

## Methods on Enums

### Inline Methods

Define methods inside enum body:

```vex
enum Color {
    Red,
    Green,
    Blue,

    fn (self: &Color) is_primary(): bool {
        match *self {
            Red | Green | Blue => {
                return true;
            }
        }
    }

    fn (self: &Color) to_hex(): string {
        match *self {
            Red => { return "#FF0000"; }
            Green => { return "#00FF00"; }
            Blue => { return "#0000FF"; }
        }
    }
}
```

### Golang-Style Methods

Define methods outside enum:

```vex
enum Status {
    Active,
    Inactive,
    Pending,
}

fn (s: &Status) is_active(): bool {
    match *s {
        Active => { return true; }
        _ => { return false; }
    }
}

fn (s: &Status) code(): i32 {
    match *s {
        Active => { return 0; }
        Inactive => { return 1; }
        Pending => { return 2; }
    }
}
```

### Associated Functions (Future)

```vex
enum Color {
    Red,
    Green,
    Blue,

    fn from_code(code: i32): Color {
        match code {
            0 => { return Red; }
            1 => { return Green; }
            2 => { return Blue; }
            _ => { return Red; }  // Default
        }
    }
}

let color = Color.from_code(1);  // Returns Green
```

---

## Generic Enums

### Single Type Parameter

```vex
enum Option<T> {
    Some(T),
    None,
}

let some_int = Some(42);
let some_str = Some("hello");
let nothing: Option<i32> = None;
```

### Multiple Type Parameters

```vex
enum Result<T, E> {
    Ok(T),
    Err(E),
}

let success: Result<i32, string> = Ok(42);
let failure: Result<i32, string> = Err("error");
```

### Constraints (Future)

```vex
enum Comparable<T: Ord> {
    Less(T),
    Equal(T),
    Greater(T),
}
```

---

## Memory Representation

### Discriminant Tag

Enums store a tag to identify the variant:

```vex
enum Color {
    Red,    // Discriminant: 0
    Green,  // Discriminant: 1
    Blue,   // Discriminant: 2
}
```

**Memory Layout**:

```
Size: Typically 4 bytes (i32 discriminant)
[tag: 0/1/2]
```

### Tagged Union (Data-Carrying)

For data-carrying enums, memory = tag + largest variant:

```vex
enum Message {
    Quit,                         // 0 bytes data
    Move { x: i32, y: i32 },     // 8 bytes data
    Write(string),                // 16 bytes data (ptr + len)
}
```

**Memory Layout**:

```
Size: 4 (tag) + 16 (largest) = 20 bytes
[tag: 0/1/2][data.............]
```

### Niche Optimization (Future)

Compiler can optimize certain enums:

```vex
enum Option<&T> {
    Some(&T),  // Non-null pointer
    None,      // Null pointer (0)
}
// Size: Same as &T (8 bytes on 64-bit)
// Uses null pointer to represent None
```

---

## Common Enum Patterns

### Option Type

Represent optional values with builtin constructors:

```vex
enum Option<T> {
    Some(T),
    None,
}

fn find(arr: [i32], target: i32): Option<i32> {
    for i in 0..arr.len() {
        if arr[i] == target {
            return Some(i);
        }
    }
    return None;
}

let result = find([1, 2, 3], 2);
match result {
    Some(index) => { /* found */ }
    None => { /* not found */ }
}
```

**Builtin Constructors**:

```vex
let value = Some(42);        // Creates Option<i32>
let nothing = None<i32>();   // Explicit type annotation for None
```

**Implementation Status**: ✅ Complete - constructors and pattern matching fully working

### Result Type

Error handling without exceptions with builtin constructors:

```vex
enum Result<T, E> {
    Ok(T),
    Err(E),
}

fn divide(a: i32, b: i32): Result<i32, string> {
    if b == 0 {
        return Err("Division by zero");
    }
    return Ok(a / b);
}

let result = divide(10, 2);
match result {
    Ok(value) => { /* value = 5 */ }
    Err(msg) => { /* handle error */ }
}
```

**Builtin Constructors**:

```vex
let success = Ok(42);                  // Creates Result<i32, E>
let failure = Err("error message");    // Creates Result<T, string>
```

**Implementation Status**: ✅ Complete - constructors and pattern matching fully working

### State Machine

Model states with enums:

```vex
enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Error,
}

fn handle_state(state: ConnectionState) {
    match state {
        ConnectionState::Disconnected => {
            // Initiate connection
        }
        ConnectionState::Connecting => {
            // Wait for handshake
        }
        ConnectionState::Connected => {
            // Ready to send/receive
        }
        ConnectionState::Error => {
            // Handle error
        }
    }
}
```

### Event System

```vex
enum Event {
    Click { x: i32, y: i32 },
    KeyPress { key: i32 },
    Resize { width: i32, height: i32 },
}

fn handle_event(event: Event) {
    match event {
        Event::Click { x, y } => { }
        Event::KeyPress { key } => { }
        Event::Resize { width, height } => { }
    }
}
```

---

## Examples

### Basic Enum

```vex
enum Color {
    Red,
    Green,
    Blue,
}

fn main(): i32 {
    let color = Red;
    match color {
        Red => {
            return 1;
        }
        Green => {
            return 2;
        }
        Blue => {
            return 3;
        }
    }
}
```

### Enum with Values

```vex
enum Status {
    Active = 0,
    Inactive = 1,
    Pending = 2,
}

fn status_code(s: Status): i32 {
    match s {
        Active => { return 0; }
        Inactive => { return 1; }
        Pending => { return 2; }
    }
}

fn main(): i32 {
    let status = Active;
    return status_code(status);  // 0
}
```

### Enum with Methods

```vex
enum Direction {
    North,
    South,
    East,
    West,

    fn (self: &Direction) opposite(): Direction {
        match *self {
            Direction::North => { return Direction::South; }
            Direction::South => { return Direction::North; }
            Direction::East => { return Direction::West; }
            Direction::West => { return Direction::East; }
        }
    }

    fn (self: &Direction) is_vertical(): bool {
        match *self {
            Direction::North | Direction::South => { return true; }
            Direction::East | Direction::West => { return false; }
        }
    }
}

fn main(): i32 {
    let dir = Direction::North;
    let opp = dir.opposite();  // Direction::South

    if dir.is_vertical() {
        return 1;
    }
    return 0;
}
```

### Or Patterns

```vex
enum TrafficLight {
    Red,
    Yellow,
    Green,
}

fn can_go(light: TrafficLight): bool {
    match light {
        TrafficLight::Green => {
            return true;
        }
        TrafficLight::Red | TrafficLight::Yellow => {
            return false;
        }
    }
}

fn main(): i32 {
    let light = TrafficLight::Green;
    if can_go(light) {
        return 1;
    }
    return 0;
}
```

---

## Best Practices

### 1. Use Enums for Fixed Sets

```vex
// Good: Finite, known values
enum DayOfWeek {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

// Bad: Use integer instead
let day = 3;  // What does 3 mean?
```

### 2. Prefer Descriptive Names

```vex
// Good: Clear meaning
enum UserRole {
    Administrator,
    Moderator,
    Member,
    Guest,
}

// Bad: Abbreviations
enum Role {
    Admin,
    Mod,
    Mem,
    Gst,
}
```

### 3. Use Match for Exhaustiveness

```vex
// Good: Compiler checks all cases
match status {
    Active => { }
    Inactive => { }
    Pending => { }
}

// Bad: Might miss cases
if status == Active {
    // ...
} elif status == Inactive {
    // ...
}
// Forgot Pending!
```

### 4. Group Related Variants

```vex
// Good: Related variants together
enum FileOperation {
    Read,
    Write,
    Append,
    Delete,
}

// Bad: Unrelated variants
enum Mixed {
    FileRead,
    NetworkSend,
    DatabaseQuery,
}
```

### 5. Use Methods for Common Operations

```vex
enum Status {
    Active,
    Inactive,

    fn (self: &Status) is_active(): bool {
        match *self {
            Active => { return true; }
            Inactive => { return false; }
        }
    }
}

// Good: Encapsulated logic
if status.is_active() { }

// Bad: Repeated matching
match status {
    Active => { /* ... */ }
    Inactive => { /* ... */ }
}
```

---

## Enum Features Summary

| Feature         | Syntax               | Status      | Example                  |
| --------------- | -------------------- | ----------- | ------------------------ |
| Unit Variants   | `Red, Green, Blue`   | ✅ Working  | C-style enums            |
| Explicit Values | `Active = 0`         | ✅ Working  | Discriminants            |
| Pattern Match   | `match enum { }`     | ✅ Working  | Exhaustive               |
| Or Patterns     | `A \| B => { }`      | ✅ Working  | Multiple variants        |
| Inline Methods  | Inside enum body     | ✅ Working  | Methods on enums         |
| Golang Methods  | Outside enum         | ✅ Working  | Separate definition      |
| Data-Carrying   | `Some(T), None`      | ✅ Complete | Option/Result work fully |
| Tuple Variants  | `Some(T)` (single)   | ✅ v0.1.2   | Single value tuples      |
| Multi-Tuple     | `V4(u8, u8, u8, u8)` | 🚧 Future   | Multiple values          |
| Struct Variants | `Move { x, y }`      | 🚧 Future   | Named fields             |
| Generic Enums   | `Option<T>`          | ✅ Complete | Type parameters working  |

---

**Previous**: [07_Structs_and_Data_Types.md](./07_Structs_and_Data_Types.md)  
**Next**: [09_Contracts.md](./09_Contracts.md)

**Maintained by**: Vex Language Team

# Contracts

**Version:** 0.2.0  
**Last Updated:** December 16, 2025

This document defines the contract system in the Vex programming language. Contracts provide polymorphism through shared behavior definitions.

**⚠️ BREAKING CHANGE (v0.2.0)**: The `trait` keyword has been replaced with `contract`. This change was made to better reflect Vex's unique identity and to distinguish contracts (pure interfaces) from implementation.

---

## Table of Contents

1. [Contract Definitions](#contract-definitions)
2. [Contract Implementation](#contract-implementation)
3. [Default Methods](#default-methods)
4. [Contract Bounds](#contract-bounds)
5. [Associated Types](#associated-types)
6. [Contract Inheritance](#contract-inheritance)
7. [Standard Contracts](#standard-contracts)

---

## Contract Definitions

### Basic Syntax

**Syntax**: `contract Name { methods }`

```vex
contract Display {
    show();
}

contract Comparable {
    compare(other: &Self): i32;
}
```

**Properties**:

- `contract` keyword (pure interface, signatures only)
- Name in PascalCase (convention)
- Method signatures (no body, no `fn` prefix)
- `Self` type refers to implementing type
- Can have default method implementations

### Simple Contract

```vex
contract Greet {
    say_hello();
}
```

**Note**: `interface` and `trait` keywords are deprecated in v0.2.0, use `contract` instead.

### Multiple Methods

```vex
contract Shape {
    area(): f64;
    perimeter(): f64;
    name(): string;
}
```

### Self Type

`Self` represents the type implementing the contract:

```vex
contract Cloneable {
    clone(): Self;
}

contract Comparable {
    equals(other: &Self): bool;
}
```

---

## Contract Implementation

### Method Mutability in Contracts

Contract method signatures define a contract for mutability. To declare a method that can mutate the implementing type's state, the `!` suffix is used.

**Syntax**:

- **Immutable Method**: `method_name(args...): ReturnType;`
- **Mutable Method**: `method_name(args...)!;` or `method_name(args...)!: ReturnType;`

The `!` indicates that the method requires a mutable reference to `self`, allowing for modifications.

```vex
contract Logger {
    // Immutable contract: cannot modify `self`
    log(msg: string);

    // Mutable contract: can modify `self`
    clear()!;
}
```

This contract must be respected by all implementing types.

---

## Contract Implementation

### Go-Style External Implementation (RECOMMENDED v0.2.0)

**⚠️ IMPORTANT**: Vex v0.2.0 deprecates inline struct methods and recommends Go-style external methods.

**Recommended Syntax**: External methods with `contract` as pure interface

```vex
// 1. Define contract (pure interface, no fn prefix)
contract Logger {
    log(msg: string);
    clear()!;
}

// 2. Define struct (data only)
struct ConsoleLogger {
    prefix: string,
}

// 3. Implement contract via external methods (Go-style)
fn (self: ConsoleLogger) log(msg: string) {
    println(self.prefix, ": ", msg);
}

fn (self: ConsoleLogger!) clear() {
    println("Logger cleared.");
}
```

**Benefits**:

- Keeps struct definitions small (400-line limit)
- Separates data from behavior
- More modular and testable
- Follows Go and Odin conventions

### Inline Implementation (DEPRECATED v0.2.0)

**⚠️ DEPRECATED**: Inline struct methods will be removed in a future version.

**Old Syntax**: `struct MyStruct impl MyContract { ... methods ... }`

```vex
struct ConsoleLogger impl Logger {
    prefix: string,

    // Implementation of the `log` method from the `Logger` contract.
    log(msg: string) {
        println(self.prefix, ": ", msg);
    }

    // Implementation of the mutable `clear` method.
    // The `!` is required in the implementation as well.
    clear()! {
        // This method can now mutate `self`.
        // For example, if we had a mutable field:
        // self.buffer = "";
        println("Logger cleared.");
    }
}
```

**Key Rules**:

- Contract methods **MUST** be implemented directly inside the `struct`'s body.
- The method signatures in the implementation must match the contract definition, including the `!` for mutability.

### Multiple Contracts (Future)

```vex
struct FileLogger impl Logger, Closeable {
    path: string,

    // All contract methods must be in struct body
    log(msg: string) {
        // Logger implementation
    }

    clear()! {
        // Logger implementation
    }

    fn close()! {
        // Closeable implementation
    }
}
```

### Implementation Requirements

All contract methods must be implemented:

```vex
contract Shape {
    area(): f64;
    perimeter(): f64;
}

// ERROR: Missing perimeter() implementation
struct Circle impl Shape {
    radius: f64,

    area(): f64 {
        return 3.14159 * self.radius * self.radius;
    }
    // Missing perimeter()!
}
```

---

## Default Methods

### Definition

Contracts can provide default implementations:

```vex
contract Logger {
    log(msg: string);        // Required (immutable)
    clear()!;                // Required (mutable)
    info(msg: string);     // Default (immutable)
    debug(msg: string);     // Default (immutable)
}
```

**Properties**:

- Methods with body are default methods
- Implementing types inherit default behavior
- Can be overridden if needed
- Reduces code duplication

### Inheritance

Structs automatically get default methods:

```vex
struct ConsoleLogger impl Logger {
    log(msg: string) {
        // Only implement required method
    }

    clear()! {
        // Required mutable method
    }

    // info() and debug() inherited automatically!
}

fn main(): i32 {
    let! logger = ConsoleLogger { };
    logger.log("Required method");
    logger.info("Default method");    // Works!
    logger.debug("Default method");   // Works!
    logger.clear()!;                  // Required !
    return 0;
}
```

### Overriding Defaults

Implementing types can override default methods:

```vex
struct CustomLogger impl Logger {
    log(msg: string) {
        // Required method
    }

    clear()! {
        // Required method
    }

    info(msg: string) {
        // Override default implementation
        self.log("[INFO] " + msg);
    }

    // debug() still uses default implementation
}
```

### Default Method Access

Default methods can call other contract methods:

```vex
contract Formatter {
    format(): string;  // Required

    format_bold(): string {
        return "**" + self.format() + "**";
    }

    format_italic(): string {
        return "_" + self.format() + "_";
    }
}
```

---

## Contract Bounds

### Generic Constraints (Future)

Restrict generic types to those implementing specific contracts:

```vex
fn print_all<T: Display>(items: [T]) {
    // T must implement Display
    for item in items {
        item.show();
    }
}
```

**Syntax**: `T: Contract` after type parameter

### Multiple Bounds (Future)

Require multiple contracts:

```vex
fn compare_and_show<T: Comparable & Display>(a: T, b: T) {
    // T must implement both contracts
    let result = a.compare(b);
    a.show();
    b.show();
}
```

**Syntax**: `T: Contract1 & Contract2 & ...`

### Where Clauses ✅ COMPLETE (v0.1.2)

Complex bounds use where clause for readability:

```vex
fn print_both<T, U>(a: T, b: U): i32
where
    T: Display,
    U: Display
{
    print("T: ");
    print(a);
    print("U: ");
    print(b);
    return 0;
}

fn main(): i32 {
    let x: i32 = 42;
    let y: i32 = 100;
    print_both(x, y);
    return 0;
}
```

**Implementation Details**:

- Parser: `parse_where_clause()` in `vex-parser/src/parser/items/functions.rs:138`
- AST: `WhereClausePredicate { type_param, bounds }`
- Syntax: `where T: Contract1 & Contract2, U: Contract3`
- Test: `examples/test_where_clause.vx`
- Verified: November 9, 2025

**Limitations**:

- Struct inline methods don't support where clauses yet (see `structs.rs:195`)

### Bound on Methods (Future)

```vex
struct Container<T> {
    value: T,

    fn (self: &Container<T>!) show() where T: Display {
        self.value.show();
    }
}
```

---

## Associated Types

### Definition (Future)

Contracts can have associated types:

```vex
contract Iterator {
    type Item;

    next(): Option<Self.Item>;
}
```

**Properties**:

- `type Name` declares associated type
- Implementing types specify concrete type
- Used for output types in contract methods

### Implementation (IMPLEMENTED ✅)

```vex
struct Counter impl Iterator {
    type Item = i32;

    current: i32,

    next()!: Option<i32> {
        let value = self.current;
        self.current = self.current + 1;
        return Some(value);
    }
}
```

### Generic Associated Types (Future)

```vex
contract Container {
    type Item<T>;

    fn get<T>(): Self.Item<T>;
}
```

---

## Contract Inheritance

### Supercontracts

Contracts can require other contracts:

```vex
contract Eq {
    equals(other: &Self): bool;
}

contract Ord: Eq {
    // Ord requires Eq
    fn less_than(other: &Self): bool;
}
```

**Implementation**:

```vex
struct Number impl Ord {
    value: i32,

    // Must implement Eq methods
    fn (self: &Number!) equals(other: &Number): bool {
        return self.value == other.value;
    }

    // And Ord methods
    fn (self: &Number!) less_than(other: &Number): bool {
        return self.value < other.value;
    }
}
```

### Multiple Supercontracts

```vex
contract Serializable: Display & Cloneable {
    serialize(): string;
}
```

---

## Standard Contracts

> Operator overloading contracts (e.g., `Add`, `Eq`, `Index`) are documented in detail in [Specifications/23_Operator_Overloading.md](../Specifications/23_Operator_Overloading.md).

### Drop Contract ✅ IMPLEMENTED

Automatic resource cleanup when value goes out of scope:

```vex
contract Drop {
    drop()!;  // Called automatically
}

struct File impl Drop {
    handle: i32,
    path: string,

    drop()! {
        // Cleanup logic - called automatically when File goes out of scope
        close_file(self.handle);
        print("Closed file: ", self.path);
    }
}

// Usage
{
    let! file = File { handle: 42, path: "data.txt" };
    // ... use file ...
}  // drop() called automatically here
```

**Status**: Fully functional, automatic Drop contract implementation detection.

### Clone Contract ✅ IMPLEMENTED

Explicit deep copying:

```vex
contract Clone {
    clone(): Self;
}

struct Point impl Clone {
    x: i32,
    y: i32,

    clone(): Point {
        return Point { x: self.x, y: self.y };
    }
}

// Usage
let p1 = Point { x: 10, y: 20 };
let p2 = p1.clone();  // Deep copy
```

**Status**: Fully functional, used for explicit copying.

### Eq Contract ✅ IMPLEMENTED

Equality comparison:

```vex
contract Eq {
    eq(other: Self): bool;
}

struct Point impl Eq {
    x: i32,
    y: i32,

    eq(other: Point): bool {
        return self.x == other.x && self.y == other.y;
    }
}

// Usage
let p1 = Point { x: 10, y: 20 };
let p2 = Point { x: 10, y: 20 };
if p1.eq(p2) {
    print("Equal!");
}
```

**Status**: Fully functional, used for custom equality.

### Ord Contract ✅ IMPLEMENTED

Ordering comparison:

```vex
contract Ord {
    cmp(other: Self): i32;
    // Returns: -1 (less), 0 (equal), 1 (greater)
}

struct Number impl Ord {
    value: i32,

    cmp(other: Number): i32 {
        if self.value < other.value {
            return -1;
        } else if self.value > other.value {
            return 1;
        }
        return 0;
    }
}

// Usage
let n1 = Number { value: 10 };
let n2 = Number { value: 20 };
let result = n1.cmp(n2);  // Returns -1
```

**Status**: Fully functional, used for ordering operations.

### Iterator Contract ✅ IMPLEMENTED

Lazy iteration protocol:

```vex
contract Iterator {
    type Item;  // Associated type

    next()!: Option<Self.Item>;  // Returns next element or None
}

struct Counter impl Iterator {
    count: i32,
    limit: i32,

    type Item = i32;

    next()!: Option<i32> {
        if self.count < self.limit {
            let current = self.count;
            self.count = self.count + 1;
            return Some(current);
        }
        return None;
    }
}

// Usage
let! counter = Counter { count: 0, limit: 5 };
loop {
    match counter.next() {
        Some(v) => print(v),
        None => break,
    }
}
```

**Status**: Fully functional with Option<T> support. Associated type `Self.Item` temporarily uses concrete type (Option<i32>) until full generic support.

### Display Contract (Future)

Format types for display:

```vex
contract Display {
    show();
}

struct Point impl Display {
    x: i32,
    y: i32,

    show() {
        print("Point(", self.x, ", ", self.y, ")");
    }
}
```

**Status**: Planned for future implementation.

---

## Examples

### Basic Contract

```vex
contract Greet {
    say_hello();
}

struct Person impl Greet {
    name: string,

    fn (self: &Person!) say_hello() {
        // Implementation
    }
}

fn main(): i32 {
    let! person = Person { name: "Alice" };
    person.say_hello();
    return 0;
}
```

### Default Methods

```vex
contract Logger {
    log(msg: string);

    info(msg: string) {
        self.log(msg);
    }

    debug(msg: string) {
        self.log(msg);
    }
}

struct ConsoleLogger impl Logger {
    prefix: string,

    fn (self: &ConsoleLogger!) log(msg: string) {
        // Only implement required method
    }
}

fn main(): i32 {
    let! logger = ConsoleLogger { prefix: "[LOG]" };
    logger.log("Required");
    logger.info("Default method");    // Inherited!
    logger.debug("Default method");   // Inherited!
    return 0;
}
```

### Multiple Methods

```vex
contract Shape {
    area(): i32;
    perimeter(): i32;
}

struct Rectangle impl Shape {
    width: i32,
    height: i32,

    fn (self: &Rectangle!) area(): i32 {
        return self.width * self.height;
    }

    fn (self: &Rectangle!) perimeter(): i32 {
        return 2 * (self.width + self.height);
    }
}

fn main(): i32 {
    let rect = Rectangle { width: 10, height: 20 };
    let a = rect.area();        // 200
    let p = rect.perimeter();   // 60
    return a;
}
```

### Overriding Defaults

```vex
contract Counter {
    count(): i32;

    count_double(): i32 {
        return self.count() * 2;
    }
}

struct SimpleCounter impl Counter {
    value: i32,

    fn (self: &SimpleCounter!) count(): i32 {
        return self.value;
    }

    // Override default
    fn (self: &SimpleCounter!) count_double(): i32 {
        return self.value * 2 + 1;  // Custom logic
    }
}
```

---

## Best Practices

### 1. Single Responsibility

```vex
// Good: Focused contract
contract Serializable {
    serialize(): string;
}

contract Deserializable {
    from_string(s: string): Self;
}

// Bad: Too many responsibilities
contract DataHandler {
    serialize(): string;
    from_string(s: string): Self;
    validate(): bool;
    transform(): Self;
}
```

### 2. Descriptive Names

```vex
// Good: Clear purpose
contract Drawable {
    draw();
}

contract Comparable {
    compare(other: &Self): i32;
}

// Bad: Vague
contract Handler {
    handle();
}
```

### 3. Use Default Methods

```vex
// Good: Provide defaults when sensible
contract Logger {
    log(msg: string);

    info(msg: string) {
        self.log("[INFO] " + msg);
    }
}

// Bad: Force implementation of similar methods
contract Logger {
    log(msg: string);
    info(msg: string);  // No default
    debug(msg: string); // No default
}
```

### 4. Small Contracts

```vex
// Good: Composable contracts
contract Display {
    show();
}

contract Clone {
    clone(): Self;
}

struct Data impl Display, Clone {
    // Implement both
}

// Bad: Monolithic contract
contract Everything {
    show();
    clone(): Self;
    serialize(): string;
    validate(): bool;
}
```

### 5. Document Requirements

```vex
// Document contract purpose and requirements
/// Represents types that can be displayed to the user.
/// Implementations should provide a human-readable representation.
contract Display {
    show();
}

/// Represents types that can be compared for ordering.
/// Returns: -1 if self < other, 0 if equal, 1 if self > other
contract Ord {
    compare(other: &Self): i32;
}
```

---

## Contract Features Summary

| Feature               | Syntax                 | Status     | Example               |
| --------------------- | ---------------------- | ---------- | --------------------- |
| Contract Definition   | `contract Name { }`    | ✅ Working | Method signatures     |
| Inline Implementation | `struct S impl T { }`  | ✅ Working | v1.3 syntax           |
| Default Methods       | `fn (self) { body }`   | ✅ Working | With implementation   |
| Self Type             | `Self`                 | ✅ Working | Refers to implementer |
| Multiple Methods      | Multiple fn signatures | ✅ Working | In contract body      |
| Contract Bounds       | `<T: Contract>`        | ✅ Working | Generic constraints   |
| Associated Types      | `type Item;`           | ✅ Working | Type members          |
| Supercontracts        | `contract T: U { }`    | ✅ Working | Contract inheritance  |
| Where Clauses         | `where T: Contract`    | ✅ v0.1.2  | Complex bounds        |

---

## Contract System Architecture

### Current Implementation (v1.3)

```vex
// 1. Define contract
contract Logger {
    log(msg: string);
    info(msg: string) {
        self.log(msg);  // Default method
    }
}

// 2. Implement inline
struct ConsoleLogger impl Logger {
    prefix: string,

    fn (self: &ConsoleLogger!) log(msg: string) {
        // Required method implementation
    }

    // info() inherited automatically
}

// 3. Use contract methods
fn main(): i32 {
    let! logger = ConsoleLogger { prefix: "[LOG]" };
    logger.log("Direct call");
    logger.info("Default method call");
    return 0;
}
```

### Compilation Process

1. **Parse**: Contract definition → AST
2. **Register**: Store contract in `contract_defs` HashMap
3. **Implement**: Inline `impl Contract` → `contract_impls` HashMap
4. **Codegen**: Generate LLVM IR for methods
5. **Link**: Default methods compiled on-demand
6. **Call**: Method resolution at compile time (static dispatch)

### Future: Dynamic Dispatch

```vex
// Virtual table (vtable) for runtime polymorphism
fn process(logger: &dyn Logger) {
    logger.log("Dynamic dispatch");
}
```

---

**Previous**: [08_Enums.md](./08_Enums.md)  
**Next**: [10_Generics.md](./10_Generics.md)

**Maintained by**: Vex Language Team

## Deprecated: Traits spec

This file (`09_Traits.md`) is deprecated. The Vex language adopted the `contract` keyword and the canonical documentation was consolidated in `Specifications/09_Contracts.md`.

Please see: [Specifications/09_Contracts.md](./09_Contracts.md)

This file may be removed in a future release; it currently remains for historical reference.

### Definition (Future)

Contracts can have associated types:

```vex
contract Iterator {
    type Item;

    next(): Option<Self.Item>;
}
```

**Properties**:

- `type Name` declares associated type
- Implementing types specify concrete type
- Used for output types in contract methods

### Implementation (IMPLEMENTED ✅)

```vex
struct Counter impl Iterator {
    type Item = i32;

    current: i32,

    next()!: Option<i32> {
        let value = self.current;
        self.current = self.current + 1;
        return Some(value);
    }
}
```

### Generic Associated Types (Future)

```vex
contract Container {
    type Item<T>;

    fn get<T>(): Self.Item<T>;
}
```

---

## Contract Inheritance

### Supercontracts

Contracts can require other contracts:

```vex
contract Eq {
    equals(other: &Self): bool;
}

contract Ord: Eq {
    // Ord requires Eq
    fn less_than(other: &Self): bool;
}
```

**Implementation**:

```vex
struct Number impl Ord {
    value: i32,

    // Must implement Eq methods
    fn (self: &Number!) equals(other: &Number): bool {
        return self.value == other.value;
    }

    // And Ord methods
    fn (self: &Number!) less_than(other: &Number): bool {
        return self.value < other.value;
    }
}
```

### Multiple Supercontracts

```vex
contract Serializable: Display & Cloneable {
    serialize(): string;
}
```

---

## Standard Contracts

### Drop Contract ✅ IMPLEMENTED

Automatic resource cleanup when value goes out of scope:

```vex
contract Drop {
    drop()!;  // Called automatically
}

struct File impl Drop {
    handle: i32,
    path: string,

    drop()! {
        // Cleanup logic - called automatically when File goes out of scope
        close_file(self.handle);
        print("Closed file: ", self.path);
    }
}

// Usage
{
    let! file = File { handle: 42, path: "data.txt" };
    // ... use file ...
}  // drop() called automatically here
```

**Status**: Fully functional, automatic Drop contract implementation detection.

### Clone Contract ✅ IMPLEMENTED

Explicit deep copying:

```vex
contract Clone {
    clone(): Self;
}

struct Point impl Clone {
    x: i32,
    y: i32,

    clone(): Point {
        return Point { x: self.x, y: self.y };
    }
}

// Usage
let p1 = Point { x: 10, y: 20 };
let p2 = p1.clone();  // Deep copy
```

**Status**: Fully functional, used for explicit copying.

### Eq Contract ✅ IMPLEMENTED

Equality comparison:

```vex
contract Eq {
    eq(other: Self): bool;
}

struct Point impl Eq {
    x: i32,
    y: i32,

    eq(other: Point): bool {
        return self.x == other.x && self.y == other.y;
    }
}

// Usage
let p1 = Point { x: 10, y: 20 };
let p2 = Point { x: 10, y: 20 };
if p1.eq(p2) {
    print("Equal!");
}
```

**Status**: Fully functional, used for custom equality.

### Ord Contract ✅ IMPLEMENTED

Ordering comparison:

```vex
contract Ord {
    cmp(other: Self): i32;
    // Returns: -1 (less), 0 (equal), 1 (greater)
}

struct Number impl Ord {
    value: i32,

    cmp(other: Number): i32 {
        if self.value < other.value {
            return -1;
        } else if self.value > other.value {
            return 1;
        }
        return 0;
    }
}

// Usage
let n1 = Number { value: 10 };
let n2 = Number { value: 20 };
let result = n1.cmp(n2);  // Returns -1
```

**Status**: Fully functional, used for ordering operations.

### Iterator Contract ✅ IMPLEMENTED

Lazy iteration protocol:

```vex
contract Iterator {
    type Item;  // Associated type

    next()!: Option<Self.Item>;  // Returns next element or None
}

struct Counter impl Iterator {
    count: i32,
    limit: i32,

    type Item = i32;

    next()!: Option<i32> {
        if self.count < self.limit {
            let current = self.count;
            self.count = self.count + 1;
            return Some(current);
        }
        return None;
    }
}

// Usage
let! counter = Counter { count: 0, limit: 5 };
loop {
    match counter.next() {
        Some(v) => print(v),
        None => break,
    }
}
```

**Status**: Fully functional with Option<T> support. Associated type `Self.Item` temporarily uses concrete type (Option<i32>) until full generic support.

### Display Contract (Future)

Format types for display:

```vex
contract Display {
    show();
}

struct Point impl Display {
    x: i32,
    y: i32,

    show() {
        print("Point(", self.x, ", ", self.y, ")");
    }
}
```

**Status**: Planned for future implementation.

---

## Examples

### Basic Contract

```vex
contract Greet {
    say_hello();
}

struct Person impl Greet {
    name: string,

    fn (self: &Person!) say_hello() {
        // Implementation
    }
}

fn main(): i32 {
    let! person = Person { name: "Alice" };
    person.say_hello();
    return 0;
}
```

### Default Methods

```vex
contract Logger {
    log(msg: string);

    info(msg: string) {
        self.log(msg);
    }

    debug(msg: string) {
        self.log(msg);
    }
}

struct ConsoleLogger impl Logger {
    prefix: string,

    fn (self: &ConsoleLogger!) log(msg: string) {
        // Only implement required method
    }
}

fn main(): i32 {
    let! logger = ConsoleLogger { prefix: "[LOG]" };
    logger.log("Required");
    logger.info("Default method");    // Inherited!
    logger.debug("Default method");   // Inherited!
    return 0;
}
```

### Multiple Methods

```vex
contract Shape {
    area(): i32;
    perimeter(): i32;
}

struct Rectangle impl Shape {
    width: i32,
    height: i32,

    fn (self: &Rectangle!) area(): i32 {
        return self.width * self.height;
    }

    fn (self: &Rectangle!) perimeter(): i32 {
        return 2 * (self.width + self.height);
    }
}

fn main(): i32 {
    let rect = Rectangle { width: 10, height: 20 };
    let a = rect.area();        // 200
    let p = rect.perimeter();   // 60
    return a;
}
```

### Overriding Defaults

```vex
contract Counter {
    count(): i32;

    count_double(): i32 {
        return self.count() * 2;
    }
}

struct SimpleCounter impl Counter {
    value: i32,

    fn (self: &SimpleCounter!) count(): i32 {
        return self.value;
    }

    // Override default
    fn (self: &SimpleCounter!) count_double(): i32 {
        return self.value * 2 + 1;  // Custom logic
    }
}
```

---

## Best Practices

### 1. Single Responsibility

```vex
// Good: Focused contract
contract Serializable {
    serialize(): string;
}

contract Deserializable {
    from_string(s: string): Self;
}

// Bad: Too many responsibilities
contract DataHandler {
    serialize(): string;
    from_string(s: string): Self;
    validate(): bool;
    transform(): Self;
}
```

### 2. Descriptive Names

```vex
// Good: Clear purpose
contract Drawable {
    draw();
}

contract Comparable {
    compare(other: &Self): i32;
}

// Bad: Vague
contract Handler {
    handle();
}
```

### 3. Use Default Methods

```vex
// Good: Provide defaults when sensible
contract Logger {
    log(msg: string);

    info(msg: string) {
        self.log("[INFO] " + msg);
    }
}

// Bad: Force implementation of similar methods
contract Logger {
    log(msg: string);
    info(msg: string);  // No default
    debug(msg: string); // No default
}
```

### 4. Small Contracts

```vex
// Good: Composable contracts
contract Display {
    show();
}

contract Clone {
    clone(): Self;
}

struct Data impl Display, Clone {
    // Implement both
}

// Bad: Monolithic contract
contract Everything {
    show();
    clone(): Self;
    serialize(): string;
    validate(): bool;
}
```

### 5. Document Requirements

```vex
// Document contract purpose and requirements
/// Represents types that can be displayed to the user.
/// Implementations should provide a human-readable representation.
contract Display {
    show();
}

/// Represents types that can be compared for ordering.
/// Returns: -1 if self < other, 0 if equal, 1 if self > other
contract Ord {
    compare(other: &Self): i32;
}
```

---

## Contract Features Summary

| Feature               | Syntax                 | Status     | Example               |
| --------------------- | ---------------------- | ---------- | --------------------- |
| Contract Definition   | `contract Name { }`    | ✅ Working | Method signatures     |
| Inline Implementation | `struct S impl T { }`  | ✅ Working | v1.3 syntax           |
| Default Methods       | `fn (self) { body }`   | ✅ Working | With implementation   |
| Self Type             | `Self`                 | ✅ Working | Refers to implementer |
| Multiple Methods      | Multiple fn signatures | ✅ Working | In contract body      |
| Contract Bounds       | `<T: Contract>`        | ✅ Working | Generic constraints   |
| Associated Types      | `type Item;`           | ✅ Working | Type members          |
| Supercontracts        | `contract T: U { }`    | ✅ Working | Contract inheritance  |
| Where Clauses         | `where T: Contract`    | ✅ v0.1.2  | Complex bounds        |

---

## Contract System Architecture

### Current Implementation (v1.3)

```vex
// 1. Define contract
contract Logger {
    log(msg: string);
    info(msg: string) {
        self.log(msg);  // Default method
    }
}

// 2. Implement inline
struct ConsoleLogger impl Logger {
    prefix: string,

    fn (self: &ConsoleLogger!) log(msg: string) {
        // Required method implementation
    }

    // info() inherited automatically
}

// 3. Use contract methods
fn main(): i32 {
    let! logger = ConsoleLogger { prefix: "[LOG]" };
    logger.log("Direct call");
    logger.info("Default method call");
    return 0;
}
```

### Compilation Process

1. **Parse**: Contract definition → AST
2. **Register**: Store contract in `trait_defs` HashMap
3. **Implement**: Inline `impl Contract` → `contract_impls` HashMap
4. **Codegen**: Generate LLVM IR for methods
5. **Link**: Default methods compiled on-demand
6. **Call**: Method resolution at compile time (static dispatch)

### Future: Dynamic Dispatch

```vex
// Virtual table (vtable) for runtime polymorphism
fn process(logger: &dyn Logger) {
    logger.log("Dynamic dispatch");
}
```

---

**Previous**: [08_Enums.md](./08_Enums.md)  
**Next**: [10_Generics.md](./10_Generics.md)

**Maintained by**: Vex Language Team

# Generics (Parametric Polymorphism)

**Version:** 0.1.0  
**Last Updated:** November 3, 2025

This document defines the generic type system in Vex, enabling code reuse through parametric polymorphism.

---

## Table of Contents

1. [Generic Functions](#generic-functions)
2. [Generic Structs](#generic-structs)
3. [Generic Enums](#generic-enums)
4. [Generic Contracts](#generic-contracts)
5. [Type Constraints](#type-constraints)
6. [Monomorphization](#monomorphization)

---

## Generic Functions

### Basic Syntax

**Syntax**: `fn name<T>(params): return_type`

```vex
fn identity<T>(x: T): T {
    return x;
}
```

**Usage**:

```vex
let num = identity<i32>(42);
let text = identity<string>("hello");
let flag = identity<bool>(true);
```

### Type Inference

Type parameters can be inferred from arguments:

```vex
fn identity<T>(x: T): T {
    return x;
}

let num = identity(42);        // T inferred as i32
let text = identity("hello");  // T inferred as string
```

**Explicit vs Inferred**:

```vex
// Explicit type argument
let result = identity<i32>(42);

// Inferred from argument
let result = identity(42);  // Same as above
```

### Multiple Type Parameters

```vex
fn pair<T, U>(first: T, second: U): (T, U) {
    return (first, second);
}

let p1 = pair<i32, string>(42, "answer");
let p2 = pair(3.14, true);  // Inferred: <f64, bool>
```

### Generic Return Types

```vex
fn create<T>(value: T): T {
    return value;
}

let x: i32 = create(42);
let y: string = create("text");
```

### Examples

**Swap Function**:

```vex
fn swap<T>(a: T, b: T): (T, T) {
    return (b, a);
}

let (x, y) = swap(10, 20);        // x=20, y=10
let (s1, s2) = swap("hi", "bye"); // s1="bye", s2="hi"
```

**Generic Comparison** (Future):

```vex
fn max<T: Ord>(a: T, b: T): T {
    if a > b {
        return a;
    }
    return b;
}
```

---

## Generic Structs

### Single Type Parameter

```vex
struct Box<T> {
    value: T,
}

let int_box = Box<i32> { value: 42 };
let str_box = Box<string> { value: "hello" };
```

### Multiple Type Parameters

```vex
struct Pair<T, U> {
    first: T,
    second: U,
}

let pair = Pair<i32, string> {
    first: 42,
    second: "answer",
};
```

### Generic Methods

Methods on generic structs can operate on the generic types. Vex supports two styles for defining methods, both of which can be used with generic structs.

#### Kural 1: Inline Methods

Methods defined inside a `struct` or `contract` use the `!` suffix on the function signature to indicate mutability.

```vex
struct Container<T> {
    value: T,

    // Immutable method, returns a copy of the value.
    fn get(): T {
        return self.value;
    }

    // Mutable method, modifies the internal state.
    fn set(new_value: T)! {
        self.value = new_value;
    }
}
```

**Usage**:

```vex
let! container = Container<i32> { value: 42 };
let val = container.get();      // val is 42
container.set(100)!;            // State is mutated
// container.value is now 100
```

#### Kural 2: External Methods (Golang-style)

Methods can also be defined outside the struct body, using an explicit `self` parameter. Mutability is declared on the receiver's type.

```vex
// This style is also valid for generic structs.
fn (self: &Container<T>) get_external(): T {
    return self.value;
}

fn (self: &Container<T>!) set_external(new_value: T) {
    self.value = new_value;
}
```

**Usage**:

```vex
let! container = Container<i32> { value: 42 };
let val = container.get_external();      // val is 42
container.set_external(100);             // State is mutated
// container.value is now 100
```

### Nested Generics

```vex
struct Box<T> {
    value: T,
}

// Box containing Box
let nested = Box<Box<i32>> {
    value: Box<i32> { value: 42 }
};
```

### Examples

**Generic Stack** (Conceptual):

```vex
struct Stack<T> {
    items: [T],
    size: i32,

    fn (self: &Stack<T>!) push(item: T) {
        // Add item
    }

    fn (self: &Stack<T>!) pop(): T {
        // Remove and return item
    }
}
```

**Generic Point**:

```vex
struct Point<T> {
    x: T,
    y: T,
}

let int_point = Point<i32> { x: 10, y: 20 };
let float_point = Point<f64> { x: 1.5, y: 2.5 };
```

---

## Generic Enums

### Basic Generic Enum (Future)

```vex
enum Option<T> {
    Some(T),
    None,
}

let some_int = Some(42);
let some_str = Some("hello");
let nothing: Option<i32> = None;
```

### Multiple Type Parameters (Future)

```vex
enum Result<T, E> {
    Ok(T),
    Err(E),
}

let success: Result<i32, string> = Ok(42);
let failure: Result<i32, string> = Err("error");
```

### Pattern Matching (Future)

```vex
let result = Ok(42);

match result {
    Ok(value) => {
        // value: i32
    }
    Err(error) => {
        // error: string
    }
}
```

### Examples

**Option Type** (Future):

```vex
enum Option<T> {
    Some(T),
    None,
}

fn find<T>(arr: [T], target: T): Option<i32> {
    for i in 0..arr.len() {
        if arr[i] == target {
            return Some(i);
        }
    }
    return None;
}
```

**Either Type** (Future):

```vex
enum Either<L, R> {
    Left(L),
    Right(R),
}

let left: Either<i32, string> = Either::Left(42);
let right: Either<i32, string> = Either::Right("text");
```

---

## Generic Contracts

### Generic Contract Definition (Future)

```vex
contract Container<T> {
    fn get(): T;
    fn set(value: T);
}
```

### Implementation (Future)

```vex
struct Box<T> impl Container<T> {
    value: T,

    fn (self: &Box<T>!) get(): T {
        return self.value;
    }

    fn (self: &Box<T>!) set(value: T) {
        self.value = value;
    }
}
```

### Generic Methods in Contracts (Future)

```vex
contract Converter {
    fn convert<T>(): T;
}

struct Value impl Converter {
    data: i32,

    fn (self: &Value!) convert<T>(): T {
        // Type-specific conversion
    }
}
```

---

## Type Constraints

### Contract Bounds (Future)

Restrict generic types to those implementing specific contracts:

```vex
fn print_all<T: Display>(items: [T]) {
    for item in items {
        item.show();
    }
}
```

**Syntax**: `T: Contract` after type parameter

### Multiple Constraints (Future)

```vex
fn compare_and_show<T: Comparable & Display>(a: T, b: T): i32 {
    let result = a.compare(b);
    a.show();
    b.show();
    return result;
}
```

**Syntax**: `T: Contract1 & Contract2 & ...`

### Where Clauses ✅ COMPLETE (v0.1.2)

For complex constraints, use where clause for better readability:

```vex
fn print_both<T, U>(a: T, b: U): i32
where
    T: Display,
    U: Display
{
    print("T: ");
    print(a);
    print("U: ");
    print(b);
    return 0;
}

fn main(): i32 {
    let x: i32 = 42;
    let y: i32 = 100;
    print_both(x, y);
    return 0;
}
```

**Implementation**:

- Parser: `parse_where_clause()` in `vex-parser/src/parser/items/functions.rs:138`
- AST: `WhereClausePredicate { type_param, bounds }`
- Syntax: `where T: Contract1 & Contract2, U: Contract3`
- Test: `examples/test_where_clause.vx`
- Limitation: Struct inline methods don't support where clauses yet

### Bound on Structs (Future)

```vex
struct Container<T: Display> {
    value: T,

    fn (self: &Container<T>!) show() {
        self.value.show();  // OK: T implements Display
    }
}
```

### Conditional Methods (Future)

Methods available only when constraints met:

```vex
struct Wrapper<T> {
    value: T,
}

impl<T: Display> Wrapper<T> {
    fn (self: &Wrapper<T>!) show() {
        self.value.show();
    }
}

// show() only available for Wrapper<T> where T: Display
```

---

## Monomorphization

### Concept

Vex uses **monomorphization** for generics:

- Each generic instantiation generates specialized code
- No runtime overhead (unlike type erasure)
- Compile-time type checking
- Code size increases with instantiations

### Example

**Generic Code**:

```vex
fn identity<T>(x: T): T {
    return x;
}

let a = identity(42);
let b = identity("hello");
```

**Generated Code** (conceptual):

```vex
// Compiler generates specialized versions:
fn identity_i32(x: i32): i32 {
    return x;
}

fn identity_string(x: string): string {
    return x;
}

let a = identity_i32(42);
let b = identity_string("hello");
```

### Benefits

1. **Zero Runtime Cost**: No type checking at runtime
2. **Type Safety**: Full compile-time verification
3. **Optimization**: Compiler can optimize each instantiation
4. **No Boxing**: Values aren't boxed/wrapped

### Trade-offs

1. **Code Size**: Each instantiation increases binary size
2. **Compile Time**: More code to generate
3. **Cache Pressure**: Larger code can affect cache

### Example with Structs

**Generic Struct**:

```vex
struct Box<T> {
    value: T,
}

let int_box = Box<i32> { value: 42 };
let str_box = Box<string> { value: "hello" };
```

**Generated Structs**:

```vex
struct Box_i32 {
    value: i32,
}

struct Box_string {
    value: string,
}
```

---

## Advanced Patterns

### Generic Wrapper

```vex
struct Wrapper<T> {
    inner: T,
}

fn wrap<T>(value: T): Wrapper<T> {
    return Wrapper<T> { inner: value };
}

let wrapped_int = wrap(42);
let wrapped_str = wrap("text");
```

### Generic Pair Operations

```vex
struct Pair<T, U> {
    first: T,
    second: U,

    fn (self: &Pair<T, U>) get_first(): T {
        return self.first;
    }

    fn (self: &Pair<T, U>) get_second(): U {
        return self.second;
    }

    fn (self: &Pair<T, U>) swap(): Pair<U, T> {
        return Pair<U, T> {
            first: self.second,
            second: self.first,
        };
    }
}
```

### Phantom Types (Future)

Type parameters not stored but used for compile-time checks:

```vex
struct PhantomData<T>;

struct Marker<T> {
    data: i32,
    _phantom: PhantomData<T>,
}

let m1: Marker<i32> = Marker { data: 42, _phantom: PhantomData };
let m2: Marker<string> = Marker { data: 42, _phantom: PhantomData };
// m1 and m2 are different types despite same data
```

---

## Examples

### Identity Function

```vex
fn identity<T>(x: T): T {
    return x;
}

fn main(): i32 {
    let a = identity(42);        // i32
    let b = identity("hello");   // string
    return a;
}
```

### Generic Box

```vex
struct Box<T> {
    value: T,

    fn (self: &Box<T>) get(): T {
        return self.value;
    }
}

fn main(): i32 {
    let box = Box<i32> { value: 42 };
    return box.get();
}
```

### Swap Function

```vex
fn swap<T>(a: T, b: T): (T, T) {
    return (b, a);
}

fn main(): i32 {
    let (x, y) = swap<i32>(10, 20);
    return x;  // 20
}
```

### Generic Pair

```vex
struct Pair<T, U> {
    first: T,
    second: U,
}

fn make_pair<T, U>(a: T, b: U): Pair<T, U> {
    return Pair<T, U> { first: a, second: b };
}

fn main(): i32 {
    let pair = make_pair<i32, string>(42, "answer");
    return pair.first;  // 42
}
```

### Generic Methods

```vex
struct Container<T> {
    value: T,
    count: i32,

    fn (self: &Container<T>) get_value(): T {
        return self.value;
    }

    fn (self: &Container<T>) get_count(): i32 {
        return self.count;
    }

    fn (self: &Container<T>!) increment() {
        self.count = self.count + 1;
    }
}

fn main(): i32 {
    let! container = Container<i32> {
        value: 42,
        count: 0,
    };

    container.increment();
    return container.get_count();  // 1
}
```

---

## Best Practices

### 1. Use Descriptive Type Parameter Names

```vex
// Good: Descriptive single letters or words
fn map<T, U>(value: T, func: fn(T): U): U { }
fn process<Input, Output>(data: Input): Output { }

// Bad: Unclear abbreviations
fn process<X, Y>(data: X): Y { }
```

**Common Conventions**:

- `T` - Generic type
- `U`, `V`, `W` - Additional types
- `E` - Error type
- `K` - Key type
- `V` - Value type
- `R` - Result/Return type

### 2. Prefer Specific Types When Possible

```vex
// Good: When type is always the same
fn add_integers(a: i32, b: i32): i32 {
    return a + b;
}

// Bad: Unnecessary generic
fn add<T>(a: T, b: T): T {
    return a + b;  // Assumes T supports +
}
```

### 3. Use Constraints for Safety (Future)

```vex
// Good: Explicit constraint
fn compare<T: Comparable>(a: T, b: T): bool {
    return a > b;  // Safe: T implements Comparable
}

// Bad: Unconstrained
fn compare<T>(a: T, b: T): bool {
    return a > b;  // Error: T might not support >
}
```

### 4. Avoid Over-Genericization

```vex
// Good: Reasonable generality
struct Pair<T, U> {
    first: T,
    second: U,
}

// Bad: Unnecessarily complex
struct Pair<T, U, F, G>
where
    F: Fn(T): U,
    G: Fn(U): T
{
    // Too generic for common use
}
```

### 5. Document Generic Constraints

```vex
/// Creates a new container with the given value.
/// Type T can be any type that implements Clone.
fn create<T: Clone>(value: T): Container<T> {
    // Implementation
}
```

---

## Generics Summary

| Feature             | Syntax              | Status     | Example                           |
| ------------------- | ------------------- | ---------- | --------------------------------- |
| Generic Functions   | `fn name<T>()`      | ✅ Working | `identity<T>(x: T)`               |
| Generic Structs     | `struct S<T> { }`   | ✅ Working | `Box<i32>`                        |
| Multiple Parameters | `<T, U, V>`         | ✅ Working | `Pair<T, U>`                      |
| Type Inference      | Omit type args      | ✅ Working | `identity(42)`                    |
| Generic Methods     | `fn (self: &S<T>)`  | ✅ Working | Methods on generic types          |
| Monomorphization    | Automatic           | ✅ Working | Zero runtime cost                 |
| Generic Enums       | `enum E<T> { }`     | ✅ Working | `Option<T>`, `Result<T,E>`        |
| Contract Bounds     | `<T: Contract>`     | ✅ Working | Constrained types                 |
| Where Clauses       | `where T: Contract` | ✅ v0.1.2  | Complex constraints               |
| Associated Types    | `type Item;`        | ✅ Working | Contract associated types working |
| Higher-Kinded       | `F<T>`              | ❌ Future  | Generic over generics             |
| Const Generics      | `[T; N]`            | ❌ Future  | Array size parameter              |

---

## Compilation Model

### Instantiation Process

1. **Parse**: Generic definition → AST with type parameters
2. **Type Check**: Verify generic constraints
3. **Instantiate**: Generate concrete types for each usage
4. **Monomorphize**: Create specialized code for each type
5. **Optimize**: Optimize each instantiation independently
6. **Link**: Combine all instantiations into binary

### Example Flow

```vex
fn identity<T>(x: T): T { return x; }

let a = identity(42);       // Instantiate identity<i32>
let b = identity("hi");     // Instantiate identity<string>
```

**Compilation**:

1. Parse `identity<T>` as generic template
2. Encounter `identity(42)` → infer T = i32
3. Generate `identity_i32(x: i32): i32`
4. Encounter `identity("hi")` → infer T = string
5. Generate `identity_string(x: string): string`
6. Link both specialized versions

---

**Previous**: [09_Contracts.md](./09_Contracts.md)  
**Next**: [11_Pattern_Matching.md](./11_Pattern_Matching.md)

**Maintained by**: Vex Language Team

# Conditional Types (TypeScript-inspired)

**Status:** ✅ Implemented (v0.1.2)  
**Version:** v0.1.2  
**Last Updated:** November 11, 2025

This document describes Vex's conditional type system, inspired by TypeScript's `T extends U ? X : Y` syntax for advanced type-level programming.

---

## ⚠️ Type Safety Guarantees

**Conditional types maintain Vex's zero-cost abstraction and type safety:**

1. ✅ **Compile-time only** - All evaluation happens during type checking
2. ✅ **Zero runtime cost** - No reflection, no type metadata in binary
3. ✅ **Static verification** - Invalid type conditions caught at compile time
4. ✅ **Sound type system** - Cannot violate type safety through conditionals
5. ✅ **Monomorphization** - Generic types fully resolved before LLVM codegen

**Implementation:**

- Parser: Parses conditional type syntax into AST
- Type Checker: Evaluates conditions during type resolution
- Compiler: Generates code as if types were written explicitly
- No runtime type information or dynamic dispatch

---

## Table of Contents

1. [Overview](#overview)
2. [Basic Syntax](#basic-syntax)
3. [Use Cases](#use-cases)
4. [Type-Level Conditionals](#type-level-conditionals)
5. [Distributive Conditional Types](#distributive-conditional-types)
6. [Infer Keyword](#infer-keyword)
7. [Comparison with TypeScript](#comparison-with-typescript)

---

## Overview

Conditional types allow types to be chosen based on a condition evaluated at compile time. This enables powerful type-level programming patterns for generic libraries and frameworks.

### Why Conditional Types?

**Problem:** Generic code often needs different behavior based on type properties:

```vex
// How to return different types based on input type?
fn process<T>(value: T): ??? {
    // If T is String, return i32 (length)
    // If T is i32, return String (formatted)
}
```

**Solution:** Conditional types express this at the type level:

```vex
type ProcessResult<T> = T extends String ? i32 : T extends i32 ? String : T;

fn process<T>(value: T): ProcessResult<T> {
    // Compiler knows the return type based on T
}
```

---

## Basic Syntax

### Type Condition Expression

```vex
type ConditionalType<T> = T extends U ? X : Y;
```

**Meaning:**

- If `T` is assignable to `U`, the type is `X`
- Otherwise, the type is `Y`

### Simple Example

```vex
type IsString<T> = T extends String ? true : false;

// Usage
type A = IsString<String>;  // true
type B = IsString<i32>;     // false
```

---

## Use Cases

### ✅ Currently Working (v0.1.2)

**Basic conditional types with `infer` keyword:**

```vex
// 1. Unwrap Option type
type Unwrap<T> = T extends Option<infer U> ? U : T;
// Unwrap<Option<i32>> → i32
// Unwrap<string> → string

// 2. Extract Result values
type ExtractOk<T> = T extends Result<infer V, infer E> ? V : T;
type ExtractErr<T> = T extends Result<infer V, infer E> ? E : never;
// ExtractOk<Result<i32, string>> → i32
// ExtractErr<Result<i32, string>> → string

// 3. Type filtering
type OnlyOption<T> = T extends Option<infer U> ? T : never;
// OnlyOption<Option<i32>> → Option<i32>
// OnlyOption<string> → never
```

### 🔮 Planned Features

**1. Type-Based Return Types:**

```vex
type ReturnType<T> =
    T extends String ? i32 :
    T extends i32 ? String :
    T;

fn convert<T>(value: T): ReturnType<T> {
    // Implementation inferred by compiler
}

let x: i32 = convert("hello");      // OK: String → i32
let y: String = convert(42);        // OK: i32 → String
let z: bool = convert(true);        // OK: bool → bool
```

### 2. Extract Array Element Type

```vex
type ElementType<T> = T extends [U] ? U : never;

type A = ElementType<[i32]>;        // i32
type B = ElementType<[String]>;     // String
type C = ElementType<i32>;          // never (not an array)
```

### 3. Optional Type Unwrapping

```vex
type Unwrap<T> = T extends Option<U> ? U : T;

type A = Unwrap<Option<i32>>;       // i32
type B = Unwrap<i32>;               // i32
```

### 4. Function Return Type Extraction

```vex
type ReturnOf<T> = T extends fn(...): R ? R : never;

fn add(a: i32, b: i32): i32 { return a + b; }

type AddReturn = ReturnOf<typeof add>;  // i32
```

---

## Type-Level Conditionals

### Nested Conditionals

```vex
type TypeName<T> =
    T extends String ? "string" :
    T extends i32 ? "i32" :
    T extends bool ? "bool" :
    "unknown";

type A = TypeName<String>;  // "string"
type B = TypeName<i32>;     // "i32"
type C = TypeName<f64>;     // "unknown"
```

### Multiple Conditions

```vex
type IsNumeric<T> =
    T extends i32 ? true :
    T extends i64 ? true :
    T extends f32 ? true :
    T extends f64 ? true :
    false;

type A = IsNumeric<i32>;    // true
type B = IsNumeric<String>; // false
```

---

## Distributive Conditional Types

When `T` is a union type, conditional types **distribute** over the union:

```vex
type ToArray<T> = T extends U ? [U] : never;

type A = ToArray<String | i32>;
// Distributes to: ToArray<String> | ToArray<i32>
// Result: [String] | [i32]
```

### Filtering Union Types

```vex
type NonNullable<T> = T extends nil ? never : T;

type A = NonNullable<String | nil>;  // String
type B = NonNullable<i32 | nil>;     // i32
```

### Extracting from Unions

```vex
type ExtractStrings<T> = T extends String ? T : never;

type A = ExtractStrings<String | i32 | bool>;  // String
```

---

## Infer Keyword

The `infer` keyword allows **extracting types** from within a conditional type:

### Basic Inference

```vex
type GetReturnType<T> = T extends fn(...): infer R ? R : never;

fn foo(): i32 { return 42; }

type FooReturn = GetReturnType<typeof foo>;  // i32
```

### Array Element Inference

```vex
type Flatten<T> = T extends [infer U] ? U : T;

type A = Flatten<[i32]>;    // i32
type B = Flatten<i32>;      // i32
```

### Multiple Infers

```vex
type GetParams<T> = T extends fn(infer P1, infer P2): R ? [P1, P2] : never;

fn add(a: i32, b: i32): i32 { return a + b; }

type AddParams = GetParams<typeof add>;  // [i32, i32]
```

---

## Comparison with TypeScript

### Similarities

| Feature                | TypeScript            | Vex (Planned)             |
| ---------------------- | --------------------- | ------------------------- |
| Basic Syntax           | `T extends U ? X : Y` | `T extends U ? X : Y`     |
| Distributive Types     | ✅ Yes                | ✅ Yes (planned)          |
| Infer Keyword          | ✅ `infer R`          | ✅ `infer R` (planned)    |
| Type-level Programming | ✅ Full support       | ✅ Full support (planned) |

### Differences

| Feature         | TypeScript            | Vex (Planned)             |
| --------------- | --------------------- | ------------------------- | ------------------------ |
| Type Aliases    | `type X = ...`        | `type X = ...` (same)     |
| Contract Bounds | Interface constraints | `T: Contract` constraints |
| Literal Types   | `"string"             | "number"`                 | String literals as types |
| Never Type      | `never`               | `never` (same)            |

---

## Implementation Plan

### Phase 1: Basic Conditionals (v1.0)

```vex
type IsString<T> = T extends String ? true : false;
```

**Requirements:**

- Parser: Extend type syntax to support `extends`, `?`, `:`
- Type Checker: Evaluate conditionals at compile time
- Codegen: No runtime impact (all compile-time)

### Phase 2: Infer Keyword (v1.1)

```vex
type ReturnType<T> = T extends fn(...): infer R ? R : never;
```

**Requirements:**

- Parser: Support `infer` in type expressions
- Type Checker: Extract and bind inferred types
- AST: Add `Type::Infer { name: String }`

### Phase 3: Distributive Types (v1.2)

```vex
type ToArray<T> = T extends U ? [U] : never;
type A = ToArray<String | i32>;  // [String] | [i32]
```

**Requirements:**

- Type Checker: Distribute conditionals over union types
- Optimization: Simplify nested unions

---

## Examples

### Practical Use Case: Generic API Response

```vex
contract Deserialize {
    fn deserialize(data: String): Self;
}

type ApiResponse<T> = T extends Deserialize ? Result<T, String> : never;

fn fetch<T: Deserialize>(url: String): ApiResponse<T> {
    // Conditional return type ensures T is Deserializable
}
```

### Type-Safe Event Handlers

```vex
type EventHandler<E> =
    E extends MouseEvent ? fn(MouseEvent) :
    E extends KeyEvent ? fn(KeyEvent) :
    fn(Event);

fn on<E>(event_name: String, handler: EventHandler<E>) {
    // Type-safe event handling
}
```

---

## Status

**Current Status:** 🚧 Not Implemented  
**Target Version:** v1.0  
**Priority:** MEDIUM (powerful but not essential for v1.0)

**Dependencies:**

- ✅ Type system (implemented)
- ✅ Generics (implemented)
- ✅ Contract bounds (implemented)
- ❌ Advanced type inference (planned)

---

**Previous**: [10_Generics.md](./10_Generics.md)  
**Next**: [11_Pattern_Matching.md](./11_Pattern_Matching.md)

**Maintained by**: Vex Language Team

# Pattern Matching

**Version:** 0.1.0  
**Last Updated:** November 3, 2025

This document defines pattern matching and destructuring in the Vex programming language.

---

## Table of Contents

1. [Match Expression](#match-expression)
2. [Pattern Types](#pattern-types)
3. [Destructuring](#destructuring)
4. [Exhaustiveness Checking](#exhaustiveness-checking)
5. [Pattern Guards](#pattern-guards)
6. [Advanced Patterns](#advanced-patterns)

---

## Match Expression

### Basic Syntax

**Syntax**: `match value { pattern => body }`

```vex
match x {
    pattern1 => { /* body 1 */ }
    pattern2 => { /* body 2 */ }
    _ => { /* default */ }
}
```

**Properties**:

- Must be exhaustive (all cases covered)
- Evaluates top-to-bottom (first match wins)
- Each arm returns a value (future: match as expression)
- Wildcard `_` matches anything

### Simple Example

```vex
let x = 5;
match x {
    0 => { /* zero */ }
    1 => { /* one */ }
    5 => { /* five */ }
    _ => { /* other */ }
}
```

---

## Pattern Types

### Literal Patterns

Match against specific values:

```vex
match status_code {
    200 => { /* OK */ }
    404 => { /* Not Found */ }
    500 => { /* Server Error */ }
    _ => { /* Other */ }
}
```

**Supported Literals**:

- Integers: `0`, `42`, `-10`
- Booleans: `true`, `false`
- Strings: `"hello"` (future)
- Floats: Limited support (comparison issues)

### Variable Patterns

Bind matched value to variable:

```vex
match x {
    n => {
        // n binds to x's value
    }
}
```

**Example**:

```vex
match age {
    a => {
        // a = age
        return a * 2;
    }
}
```

### Wildcard Pattern

Match and discard value:

```vex
match result {
    0 => { /* success */ }
    _ => { /* any error */ }
}
```

**Use Cases**:

- Default/catch-all case
- Ignoring specific values
- Exhaustiveness completion

### Enum Patterns

Match enum variants:

```vex
enum Color {
    Red,
    Green,
    Blue,
}

match color {
    Red => { /* red */ }
    Green => { /* green */ }
    Blue => { /* blue */ }
}
```

**Must be exhaustive**:

```vex
// ERROR: Missing Blue
match color {
    Red => { }
    Green => { }
}
```

### Or Patterns

Match multiple patterns:

```vex
match day {
    1 | 2 | 3 | 4 | 5 => { /* weekday */ }
    6 | 7 => { /* weekend */ }
    _ => { /* invalid */ }
}
```

**Syntax**: `pattern1 | pattern2 | ...`

**Examples**:

```vex
match status {
    Active | Pending => { /* in progress */ }
    Inactive => { /* done */ }
}

match x {
    0 | 1 | 2 => { /* low */ }
    3 | 4 | 5 => { /* medium */ }
    _ => { /* high */ }
}
```

---

## Destructuring

### Tuple Destructuring

Extract tuple components:

```vex
let point = (10, 20);
match point {
    (x, y) => {
        // x = 10, y = 20
    }
}
```

**Multiple Patterns**:

```vex
match pair {
    (0, 0) => { /* origin */ }
    (0, y) => { /* on y-axis, y is bound */ }
    (x, 0) => { /* on x-axis, x is bound */ }
    (x, y) => { /* general point */ }
}
```

**Ignoring Components**:

```vex
match triple {
    (x, _, z) => {
        // Only x and z are bound, middle ignored
    }
}
```

### Struct Destructuring

**Status**: ✅ **COMPLETE** (v0.1.2)

Extract struct fields in pattern matching:

```vex
struct Point { x: f32, y: f32 }

match point {
    Point { x, y } => {
        // x and y are bound from point.x and point.y
        print(x);
        print(y);
    }
}
```

**Nested Destructuring**:

```vex
struct Line {
    start: Point,
    end: Point
}

match line {
    Line { start, end } => {
        match start {
            Point { x: x1, y: y1 } => {
                match end {
                    Point { x: x2, y: y2 } => {
                        // Access nested fields
                        print(x1);
                        print(y2);
                    }
                };
            }
        };
    }
}
```

**Field Renaming**:

```vex
match point {
    Point { x: px, y: py } => {
        // Bind point.x to px, point.y to py
        print(px);
        print(py);
    }
}
```

**Use Cases**:

- Extract specific fields from structs
- Validate struct values with guards
- Destructure function parameters (future)
- Pattern matching in match expressions

**Examples**:

```vex
fn distance(p: Point): f32 {
    match p {
        Point { x, y } => {
            return (x * x + y * y);  // Simplified distance
        }
    };
}

fn origin_check(p: Point): bool {
    match p {
        Point { x, y } => {
            if x == 0.0 && y == 0.0 {
                return true;
            } else {
                return false;
            };
        }
    };
}

fn quadrant(p: Point): i32 {
    match p {
        Point { x, y } => {
            if x > 0.0 && y > 0.0 {
                return 1;
            } else if x < 0.0 && y > 0.0 {
                return 2;
            } else if x < 0.0 && y < 0.0 {
                return 3;
            } else if x > 0.0 && y < 0.0 {
                return 4;
            } else {
                return 0;  // On axis
            };
        }
    };
}
```

**Implementation Details**:

- **Parser**: `vex-parser/src/parser/patterns.rs` - Parses `Struct { field1, field2 }` syntax
- **AST**: `vex-ast/src/lib.rs` - `Pattern::Struct { name, fields }`
- **Pattern checking**: `vex-compiler/src/codegen_ast/expressions/pattern_matching.rs`
- **Pattern binding**: Extract field values and bind to variables
- **Test file**: `examples/test_struct_patterns.vx`

**Partial Destructuring** (Future):

```vex
match person {
    Person { name, .. } => {
        // Only extract name, ignore other fields
    }
}
```

### Array/Slice Destructuring (Future)

```vex
match arr {
    [first, second, third] => { /* exactly 3 elements */ }
    [head, ..] => { /* at least 1 element */ }
    [.., last] => { /* at least 1 element */ }
    [first, .., last] => { /* at least 2 elements */ }
    [] => { /* empty */ }
}
```

### Enum Destructuring (Future)

Data-carrying enums:

```vex
enum Option<T> {
    Some(T),
    None,
}

match value {
    Some(x) => {
        // x contains the wrapped value
    }
    None => {
        // No value
    }
}
```

**Complex Enums**:

```vex
enum Message {
    Move { x: i32, y: i32 },
    Write(string),
    ChangeColor(i32, i32, i32),
}

match msg {
    Move { x, y } => { /* x, y bound */ }
    Write(text) => { /* text bound */ }
    ChangeColor(r, g, b) => { /* r, g, b bound */ }
}
```

---

## Exhaustiveness Checking

### Requirement

Match expressions must handle all possible cases:

```vex
enum Status {
    Active,
    Inactive,
    Pending,
}

// OK: All variants covered
match status {
    Active => { }
    Inactive => { }
    Pending => { }
}

// OK: Wildcard covers remaining
match status {
    Active => { }
    _ => { /* Inactive and Pending */ }
}

// ERROR: Missing Pending
match status {
    Active => { }
    Inactive => { }
}
```

### Compiler Errors

```
Error: Match is not exhaustive
  --> example.vx:10:5
   |
10 |     match status {
   |     ^^^^^ missing Pending
   |
   = help: ensure all variants are covered or add a wildcard pattern
```

### Integer Exhaustiveness

For integers, wildcard required:

```vex
// OK: Wildcard covers all other values
match x {
    0 => { }
    1 => { }
    _ => { }
}

// ERROR: Cannot cover all i32 values
match x {
    0 => { }
    1 => { }
    2 => { }
    // Missing billions of other values
}
```

---

## Pattern Guards

### Definition

Add conditions to patterns:

```vex
match x {
    n if n < 0 => { /* negative */ }
    n if n == 0 => { /* zero */ }
    n if n > 0 => { /* positive */ }
}
```

**Syntax**: `pattern if condition`

### Complex Guards

```vex
match pair {
    (x, y) if x == y => { /* equal */ }
    (x, y) if x > y => { /* first larger */ }
    (x, y) => { /* second larger or equal */ }
}
```

### With Enums

```vex
match option {
    Some(x) if x > 10 => { /* large value */ }
    Some(x) => { /* small value */ }
    None => { /* no value */ }
}
```

---

## Advanced Patterns

### Range Patterns

```vex
match age {
    0..=12 => { /* child */ }
    13..=17 => { /* teen */ }
    18..=64 => { /* adult */ }
    65.. => { /* senior */ }
}
```

**Syntax**:

- `a..b` - Exclusive end (a <= x < b)
- `a..=b` - Inclusive end (a <= x <= b)
- `..b` - Up to b
- `a..` - From a onwards

### Reference Patterns (Future)

```vex
match &value {
    &x => {
        // x is a reference
    }
}
```

### Nested Patterns (Future)

```vex
match nested {
    (Point { x, y }, Some(value)) => {
        // Destructure tuple and Point and Option
    }
}
```

---

## Examples

### Basic Match

```vex
fn classify(x: i32): i32 {
    match x {
        0 => {
            return 0;
        }
        1 | 2 | 3 => {
            return 1;
        }
        _ => {
            return 2;
        }
    }
}
```

### Enum Matching

```vex
enum Color {
    Red,
    Green,
    Blue,
}

fn color_code(c: Color): i32 {
    match c {
        Red => { return 0; }
        Green => { return 1; }
        Blue => { return 2; }
    }
}
```

### Tuple Destructuring

```vex
fn process_pair(pair: (i32, i32)): i32 {
    match pair {
        (0, 0) => {
            return 0;
        }
        (x, 0) => {
            return x;
        }
        (0, y) => {
            return y;
        }
        (x, y) => {
            return x + y;
        }
    }
}
```

### Or Patterns

```vex
fn is_weekend(day: i32): bool {
    match day {
        6 | 7 => {
            return true;
        }
        _ => {
            return false;
        }
    }
}
```

---

## Best Practices

### 1. Use Match for Enums

```vex
// Good: Clear, exhaustive
match status {
    Active => { }
    Inactive => { }
    Pending => { }
}

// Bad: Error-prone if-else chain
if status == Active {
    // ...
} elif status == Inactive {
    // ...
}
```

### 2. Specific Before General

```vex
// Good: Specific patterns first
match x {
    0 => { /* exact match */ }
    1 | 2 | 3 => { /* range */ }
    _ => { /* default */ }
}

// Bad: General pattern first (unreachable)
match x {
    _ => { /* catches everything */ }
    0 => { /* never reached! */ }
}
```

### 3. Use Destructuring

```vex
// Good: Extract in match
match point {
    (x, y) => {
        use_coordinates(x, y);
    }
}

// Bad: Manual extraction
match point {
    p => {
        let x = p.0;
        let y = p.1;
        use_coordinates(x, y);
    }
}
```

### 4. Avoid Deep Nesting

```vex
// Good: Flat structure
match outer {
    Some(inner) => {
        process(inner);
    }
    None => { }
}

// Bad: Deep nesting
match outer {
    Some(x) => {
        match inner {
            Some(y) => {
                match another {
                    // Too deep
                }
            }
        }
    }
}
```

### 5. Use Wildcard for Defaults

```vex
// Good: Clear default case
match error_code {
    0 => { /* success */ }
    _ => { /* any error */ }
}

// Bad: Listing all error codes
match error_code {
    0 => { /* success */ }
    1 => { /* error */ }
    2 => { /* error */ }
    // ... hundreds of error codes
}
```

---

## Pattern Matching Summary

| Pattern Type | Syntax                 | Status               | Example                      |
| ------------ | ---------------------- | -------------------- | ---------------------------- |
| Literal      | `42`, `true`, `"text"` | ✅ Working           | Exact value match            |
| Variable     | `x`, `name`            | ✅ Working           | Bind to variable             |
| Wildcard     | `_`                    | ✅ Working           | Match anything               |
| Enum         | `Red`, `Active`        | ✅ Working           | Enum variant (no :: syntax)  |
| Or           | `1 \| 2 \| 3`          | ✅ Working           | Multiple patterns            |
| Tuple        | `(x, y)`               | ✅ Working           | Destructure tuples           |
| Struct       | `Point { x, y }`       | ✅ Complete (v0.1.2) | Destructure structs          |
| Array        | `[a, b, c]`            | ✅ Working           | Fixed-size arrays            |
| Slice        | `[head, ...rest]`      | ✅ Working           | Rest patterns with `...`     |
| Enum Data    | `Some(x)`, `None`      | ✅ Working           | Data-carrying enums working  |
| Range        | `0..10`, `0..=10`      | ✅ Working           | Value ranges with .. and ..= |
| Guard        | `x if x > 0`           | ✅ Working           | Conditional patterns         |
| Reference    | `&x`                   | 🚧 Future            | Match references             |

---

**Previous**: [10_Generics.md](./10_Generics.md)  
**Next**: [12_Closures_and_Lambda_Expressions.md](./12_Closures_and_Lambda_Expressions.md)

**Maintained by**: Vex Language Team

# Closures and Lambda Expressions

**Version:** 0.1.0
**Last Updated:** November 3, 2025

This document defines closures and lambda expressions in the Vex programming language.

---

## Table of Contents

1. [Introduction](#introduction)
2. [Closure Syntax](#closure-syntax)
3. [Capture Modes](#capture-modes)
4. [Closure Contracts](#closure-contracts)
5. [Examples](#examples)
6. [Advanced Usage](#advanced-usage)

---

## Introduction

Closures are anonymous functions that can capture variables from their surrounding scope. Vex supports three types of closures with different capture semantics, similar to Rust's `Fn`, `FnMut`, and `FnOnce` traits.

### Key Features

-- **Automatic Capture Mode Detection**: Compiler determines the appropriate closure contract

- **Borrow Checker Integration**: Full integration with Vex's ownership system
- **Multiple Calling**: Closures can be called multiple times (depending on capture mode)

---

## Closure Syntax

### Basic Syntax

**Syntax**: `|parameters| body` or `|parameters| { statements }`

```vex
// Simple closure
let add_one = |x| x + 1;

// Multi-parameter closure
let add = |x, y| x + y;

// Block body closure
let complex = |x| {
    let temp = x * 2;
    return temp + 1;
};
```

### Parameter Types

Parameters can be explicitly typed or inferred:

```vex
// Explicit types
let add: fn(i32, i32): i32 = |a: i32, b: i32| a + b;

// Inferred types (common)
let multiply = |a, b| a * b;  // Types inferred from usage
```

### Return Types

Closures can return values implicitly or explicitly:

```vex
// Implicit return
let square = |x| x * x;

// Explicit return
let factorial = |n| {
    if n <= 1 {
        return 1;
    }
    return n * factorial(n - 1);
};
```

---

### Capture Modes

Vex closures automatically determine their capture mode based on how they use captured variables:

### Callable (Fn) - Immutable Capture

Closures that only read captured variables:

```vex
let x = 5;
let y = 10;
let add_to_x = |z| x + z;  // Captures x immutably

// Can be called multiple times
let result1 = add_to_x(3);  // 8
let result2 = add_to_x(7);  // 12
```

### CallableMut (FnMut) - Mutable Capture

Closures that mutate captured variables:

```vex
let! counter = 0;
let increment = || {
    counter = counter + 1;
    return counter;
};

// Can be called multiple times, modifies environment
let val1 = increment();  // 1, counter = 1
let val2 = increment();  // 2, counter = 2
```

### CallableOnce (FnOnce) - Move Capture

Closures that take ownership of captured variables:

```vex
let data = vec![1, 2, 3];
let processor = || {
    // Takes ownership of data
    return data.sum();
};

// Can only be called once
let result = processor();  // Moves data, closure consumed
// processor();  // ERROR: Already moved
```

---

## Closure Contracts

Vex defines three closure contracts that correspond to capture modes:

### Callable Contract

```vex
contract Callable<Args, Return> {
    fn call(args: Args): Return;
}
```

- Immutable capture
- Can be called multiple times
- Implemented by `Fn`-like closures

### CallableMut Contract

```vex
contract CallableMut<Args, Return> {
    fn call(args: Args): Return;
}
```

- Mutable capture
- Can be called multiple times
- Can modify captured variables
- Implemented by `FnMut`-like closures

### CallableOnce Contract

```vex
contract CallableOnce<Args, Return> {
    fn (self: Self) call(args: Args): Return;
}
```

- Move capture
- Can only be called once
- Takes ownership of environment
- Implemented by `FnOnce`-like closures

---

## Examples

### Higher-Order Functions

```vex
fn map_array<T, U>(arr: [T; 5], f: fn(T): U): [U; 5] {
    return [f(arr[0]), f(arr[1]), f(arr[2]), f(arr[3]), f(arr[4])];
}

fn main(): i32 {
    let numbers = [1, 2, 3, 4, 5];
    let doubled = map_array(numbers, |x| x * 2);
    // doubled = [2, 4, 6, 8, 10]
    return 0;
}
```

### Event Handlers

```vex
struct Button {
    label: string,
    on_click: fn(): (),
}

fn create_button(label: string, handler: fn(): ()): Button {
    return Button {
        label: label,
        on_click: handler,
    };
}

fn main(): i32 {
    let! count = 0;
    let button = create_button("Click me", || {
        count = count + 1;
    });

    // Simulate clicks
    button.on_click();  // count = 1
    button.on_click();  // count = 2

    return 0;
}
```

### Resource Management

```vex
fn with_resource<T>(resource: T, operation: fn(T): ()): () {
    defer cleanup(resource);  // Cleanup when done
    operation(resource);
}

fn main(): i32 {
    let file = open_file("data.txt");
    with_resource(file, |f| {
        // Use file
        let content = read_file(f);
        process_content(content);
    });
    // File automatically cleaned up
    return 0;
}
```

---

## Advanced Usage

### Nested Closures

Closures can be nested and capture from multiple scopes:

```vex
fn create_multiplier(factor: i32): fn(i32): i32 {
    return |x| {
        let inner_factor = factor + 1;
        return |y| x * y * inner_factor;
    };
}

fn main(): i32 {
    let multiply_by_3 = create_multiplier(3);
    let result = multiply_by_3(4);  // Returns a closure
    let final_result = result(5);   // 4 * 5 * (3 + 1) = 80
    return final_result;
}
```

### Closure Composition

```vex
fn compose<A, B, C>(f: fn(B): C, g: fn(A): B): fn(A): C {
    return |x| f(g(x));
}

fn main(): i32 {
    let add_one = |x| x + 1;
    let multiply_two = |x| x * 2;

    let add_one_then_double = compose(multiply_two, add_one);
    let result = add_one_then_double(5);  // (5 + 1) * 2 = 12

    return result;
}
```

### Async Closures

Closures work with async functions:

```vex
async fn process_async(data: string): string {
    return data.to_uppercase();
}

async fn main(): i32 {
    let processor = |data| process_async(data);
    let result = await processor("hello");
    return 0;
}
```

---

## Implementation Details

### Capture Analysis

The compiler performs static analysis to determine closure capture modes:

1. **Variable Usage Tracking**: Tracks how each captured variable is used
2. **Mode Inference**: Determines the most restrictive mode required
3. **Contract Assignment**: Assigns the appropriate closure contract

### Memory Management

- **Stack Allocation**: Closures are typically stack-allocated
- **Reference Counting**: Complex captures use reference counting
- **Move Semantics**: Move captures transfer ownership

### Performance

- **Zero-Cost Abstractions**: Closures compile to efficient machine code
- **Inlined Calls**: Small closures may be inlined by the compiler
- **Minimal Overhead**: Capture environment is optimized for size and speed

---

## Limitations

### Current Restrictions

- **No Generic Closures**: Closures cannot be generic over types
- **Limited Type Inference**: Some complex cases require explicit typing
- **No Closure Methods**: Cannot define methods on closure types

### Future Enhancements

- **Generic Closures**: Support for `|T| -> U` syntax
- **Async Closures**: Dedicated syntax for async closures
- **Closure Methods**: Ability to extend closure types with methods

---

**Previous**: [11_Pattern_Matching.md](./11_Pattern_Matching.md)
**Next**: [14_Concurrency.md](./14_Concurrency.md)

**Maintained by**: Vex Language Team  
**License**: MIT

# Concurrency

**Version:** 0.1.2  
**Last Updated:** November 2025

This document defines concurrency features in the Vex programming language.

---

## Table of Contents

1. [Concurrency Model](#concurrency-model)
2. [Goroutines](#goroutines)
3. [Async/Await](#asyncawait)
4. [Channels](#channels)
5. [GPU Computing](#gpu-computing)
6. [Synchronization](#synchronization)

---

## Concurrency Model

### Design Philosophy

Vex provides **multiple concurrency models**:

1. **Go-style goroutines**: Lightweight threads with CSP (Communicating Sequential Processes)
2. **Rust-style async/await**: Future-based asynchronous programming
3. **GPU computing**: Parallel execution on graphics processors

### Safety Guarantees

- **No data races**: Enforced by borrow checker
- **Thread safety**: Send/Sync contracts (future)
- **Deadlock prevention**: Through ownership system

---

## Goroutines

### Syntax

**Keyword**: `go`

```vex
go function_call();
```

**Properties**:

- Spawns lightweight concurrent task
- Similar to Go's goroutines
- Multiplexed onto OS threads
- Cheap to create (thousands possible)

### Examples (Conceptual)

**Simple Goroutine**:

```vex
fn worker() {
    // Do work concurrently
}

fn main(): i32 {
    go worker();  // Spawn goroutine
    go worker();  // Spawn another
    // Continue main thread
    return 0;
}
```

**With Arguments**:

```vex
fn process(id: i32) {
    // Process with id
}

fn main(): i32 {
    go process(1);
    go process(2);
    go process(3);
    return 0;
}
```

**Closure-like** (Future):

```vex
fn main(): i32 {
    let data = 42;
    go  {
        // Use data
    };
    return 0;
}
```

### Current Status

**Parser**: ✅ Parses `go` statements  
**AST**: ✅ `Statement::Go(Expression)` node  
**Runtime**: ✅ Basic goroutine runtime implemented  
**Channels**: ✅ MPSC channels fully working

**Status**: Goroutines parsed and basic runtime operational. Work-stealing scheduler pending.

---

## Async/Await

### Syntax

**Keywords**: `async`, `await`

```vex
async fn fetch_data(url: string): string {
    // Asynchronous operation
    return "data";
}

async fn main(): i32 {
    let data = await fetch_data("https://api.example.com");
    return 0;
}
```

### Async Functions

Define asynchronous functions:

```vex
async fn download(url: string): (string | error) {
    // Non-blocking I/O
    let response = await http_get(url);
    return response;
}
```

**Properties**:

- Returns immediately (non-blocking)
- Returns a Future/Promise
- Must be awaited to get result
- Can be composed with other async functions

### Await Expressions

Wait for async result:

```vex
async fn process(): i32 {
    let result1 = await operation1();
    let result2 = await operation2();
    return result1 + result2;
}
```

### Current Status

**Parser**: ✅ Parses `async fn` and `await`  
**AST**: ✅ `async` flag in Function, `await` expression  
**Runtime**: ✅ Basic async runtime implemented  
**Futures**: ✅ Basic Future support working

**Status**: Async/await syntax working with basic runtime. Advanced features (tokio integration, async I/O) pending.

---

## Channels

### Concept (Fully Implemented ✅)

Channels provide communication between concurrent tasks using CSP-style message passing:

```vex
// Create channel
let! ch = Channel.new<i32>();

// Send value
go {
    ch.send(42);
};

// Receive value
match ch.recv() {
    Option.Some(value) => println("Received: {}", value),
    Option.None => println("Channel empty"),
}
```

### Channel Operations

**Creation**:

```vex
let! ch = Channel.new<i32>();        // Unbuffered channel
let! ch = Channel.new<i32>(100);     // Buffered channel (capacity 100)
```

**Sending**:

```vex
ch.send(42);        // Send value (blocks if buffer full)
ch.try_send(42);    // Non-blocking send (returns bool)
```

**Receiving**:

```vex
let value = ch.recv();              // Blocking receive (returns Option<T>)
let value = ch.try_recv();          // Non-blocking receive (returns Option<T>)
```

**Other Operations**:

```vex
ch.close();        // Close channel
ch.is_closed();    // Check if closed
ch.len();          // Current buffer length
ch.capacity();     // Buffer capacity
```

### Current Status

**Syntax**: ✅ Fully defined  
**Implementation**: ✅ Complete (MPSC lock-free ring buffer)  
**Runtime**: ✅ C runtime (`vex_channel.c`)  
**Test Coverage**: ✅ 2 tests passing (`channel_simple.vx`, `channel_sync_test.vx`)

---

## GPU Computing

### Syntax (Parsed, Limited Support)

**Keyword**: `gpu`

```vex
gpu fn matrix_multiply(a: [f32], b: [f32]): [f32] {
    // GPU kernel code
    // Executed in parallel on GPU
}
```

### GPU Functions

Define GPU kernels:

```vex
gpu fn vector_add(a: [f32; 1024], b: [f32; 1024]): [f32; 1024] {
    // Parallel computation
    // Each thread processes one element
}
```

### Execution Model

**SIMT (Single Instruction, Multiple Threads)**:

- Same code runs on many threads
- Each thread has unique ID
- Threads grouped into blocks
- Blocks grouped into grid

### Current Status

**Parser**: ✅ Parses `gpu fn` declarations  
**AST**: ✅ `gpu` flag in Function  
**Backend**: ❌ No CUDA/OpenCL codegen  
**Runtime**: ❌ No GPU runtime

**Blocking Issues**:

1. Need CUDA/OpenCL backend
2. Need memory transfer primitives (host ↔ device)
3. Need thread indexing (`threadIdx`, `blockIdx`)
4. Need GPU runtime initialization

---

## Synchronization

### Mutex (Future)

Mutual exclusion lock:

```vex
let mutex = Mutex::new(0);

fn increment() {
    let! guard = mutex.lock();
    *guard = *guard + 1;
    // Automatically unlocked when guard dropped
}
```

### RwLock (Future)

Read-write lock:

```vex
let rwlock = RwLock::new(vec![]);

fn read_data(): [i32] {
    let guard = rwlock.read();
    return *guard;
}

fn write_data(data: [i32]) {
    let! guard = rwlock.write();
    *guard = data;
}
```

### Atomic Operations (Future)

Lock-free synchronization:

```vex
let counter = Atomic::new(0);

fn increment() {
    counter.fetch_add(1);  // Atomic increment
}
```

### WaitGroup (Future - Go-style)

Wait for goroutines to complete:

```vex
let wg = WaitGroup::new();

for i in 0..10 {
    wg.add(1);
    go  {
        defer wg.done();
        // Do work
    };
}

wg.wait();  // Wait for all goroutines
```

### Current Status

**Mutex**: 🚧 Planned (Layer 2 std lib)  
**RwLock**: 🚧 Planned (Layer 2 std lib)  
**Atomic**: 🚧 Planned (Layer 2 std lib)  
**WaitGroup**: 🚧 Planned (Layer 2 std lib)

**Planned**: Layer 2 of standard library (sync module) - infrastructure ready, implementation pending

---

## Concurrency Patterns

### Fan-Out, Fan-In (Future)

Distribute work to multiple workers:

```vex
fn fan_out(work: [Task]) {
    let (tx, rx) = channel<Result>();

    for task in work {
        go  {
            let result = process(task);
            tx.send(result);
        };
    }

    // Collect results
    for i in 0..work.len() {
        let result = rx.recv();
    }
}
```

### Pipeline (Future)

Chain processing stages:

```vex
fn pipeline(input: [Data]): [Result] {
    let (tx1, rx1) = channel();
    let (tx2, rx2) = channel();

    // Stage 1
    go  {
        for data in input {
            tx1.send(process_stage1(data));
        }
    };

    // Stage 2
    go  {
        loop {
            let data = rx1.recv();
            tx2.send(process_stage2(data));
        }
    };

    // Collect
    let! results = [];
    for i in 0..input.len() {
        results.push(rx2.recv());
    }
    return results;
}
```

### Worker Pool (Future)

Fixed number of workers:

```vex
fn worker_pool(work: [Task], num_workers: i32): [Result] {
    let (work_tx, work_rx) = channel<Task>();
    let (result_tx, result_rx) = channel<Result>();

    // Spawn workers
    for i in 0..num_workers {
        go  {
            loop {
                let task = work_rx.recv();
                let result = process(task);
                result_tx.send(result);
            }
        };
    }

    // Send work
    go  {
        for task in work {
            work_tx.send(task);
        }
    };

    // Collect results
    let! results = [];
    for i in 0..work.len() {
        results.push(result_rx.recv());
    }
    return results;
}
```

---

## Thread Safety

### Send Contract (Future)

Types safe to transfer across threads:

```vex
contract Send { }

// Automatically implemented for types without references
impl Send for i32 { }
impl Send for Point { }

// Not Send: contains non-Send types
struct HasReference {
    data: &i32,  // Not Send
}
```

### Sync Contract (Future)

Types safe to share across threads:

```vex
contract Sync { }

// Automatically implemented for immutable types
impl Sync for i32 { }

// Mutex makes T Sync
impl<T> Sync for Mutex<T> where T: Send { }
```

### Compiler Checks (Future)

```vex
fn spawn<F: Send>(f: F) {
    // Compiler ensures F is Send
    go f();
}

let x = 42;
// ERROR: x is not Send (contains reference)
let ref_x = &x;
spawn(|| {
    // use ref_x  // Compile error!
});
```

---

## Examples

### Goroutine (Conceptual)

```vex
fn worker(id: i32) {
    // Do work
}

fn main(): i32 {
    go worker(1);
    go worker(2);
    go worker(3);
    return 0;
}
```

### Async/Await (Conceptual)

```vex
async fn fetch(url: string): string {
    return "data";
}

async fn main(): i32 {
    let data1 = await fetch("url1");
    let data2 = await fetch("url2");
    return 0;
}
```

### GPU Kernel (Conceptual)

```vex
gpu fn add_vectors(a: [f32; 1024], b: [f32; 1024]): [f32; 1024] {
    let! result: [f32; 1024];
    // Each GPU thread processes one element
    return result;
}

fn main(): i32 {
    let a: [f32; 1024];
    let b: [f32; 1024];
    let result = add_vectors(a, b);
    return 0;
}
```

---

## Best Practices

### 1. Prefer Message Passing

```vex
// Good: Use channels (future)
let (tx, rx) = channel<i32>();
go  {
    tx.send(42);
};
let value = rx.recv();

// Bad: Shared mutable state
let! shared = 0;
go  {
    shared = 42;  // Data race!
};
```

### 2. Keep Goroutines Simple

```vex
// Good: Simple, focused task
go process_item(item);

// Bad: Complex logic in goroutine
go  {
    // Hundreds of lines
    // Complex error handling
    // Multiple responsibilities
};
```

### 3. Use Async for I/O

```vex
// Good: Async for I/O-bound work
async fn fetch_all(urls: [string]) {
    for url in urls {
        await fetch(url);
    }
}

// Bad: Goroutines for I/O (less efficient)
fn fetch_all(urls: [string]) {
    for url in urls {
        go fetch_blocking(url);
    }
}
```

### 4. Avoid Unnecessary Concurrency

```vex
// Good: Sequential when appropriate
fn process_small_list(items: [i32; 10]): [i32; 10] {
    let! results: [i32; 10];
    for i in 0..10 {
        results[i] = process(items[i]);
    }
    return results;
}

// Bad: Overhead of concurrency
fn process_small_list(items: [i32; 10]): [i32; 10] {
    // Goroutine overhead larger than computation!
    for item in items {
        go process(item);
    }
}
```

---

## Concurrency Summary

| Feature             | Syntax          | Status               | Notes                      |
| ------------------- | --------------- | -------------------- | -------------------------- |
| **Goroutines**      | `go func()`     | ✅ Basic runtime     | Scheduler pending          |
| **Async Functions** | `async fn`      | ✅ Basic runtime     | Advanced features pending  |
| **Await**           | `await expr`    | ✅ Working           | Basic support              |
| **GPU Functions**   | `gpu fn`        | 🚧 Parsed            | No backend                 |
| **Channels**        | `channel<T>()`  | ✅ Fully implemented | MPSC lock-free ring buffer |
| **Select**          | `select { }`    | 🚧 Keyword reserved  | Syntax planned             |
| **Mutex**           | `Mutex::new()`  | 🚧 Planned           | Layer 2 std lib            |
| **RwLock**          | `RwLock::new()` | 🚧 Planned           | Layer 2 std lib            |
| **Atomic**          | `Atomic::new()` | 🚧 Planned           | Layer 2 std lib            |
| **Send Contract**   | Auto-derived    | 🚧 Planned           | Thread safety              |
| **Sync Contract**   | Auto-derived    | 🚧 Planned           | Thread safety              |

### Implementation Status

**Syntax Level**: 60% complete (go, async, await, gpu parsed; channels working)  
**Runtime Level**: 40% complete (basic goroutines, async runtime, MPSC channels)  
**Library Level**: 0% complete (no sync primitives yet)

### Roadmap

**Phase 1: Async Runtime** (High Priority 🔴)

- Integrate tokio or custom runtime
- Implement Future contract
- Async I/O primitives
- Basic executor

**Phase 2: Goroutines** (High Priority 🔴)

- Work-stealing scheduler
- M:N threading model
- Stack management
- Runtime integration

**Phase 3: Channels** (Medium Priority 🟡)

- Channel implementation
- Select statement
- Buffered channels
- Broadcast channels

**Phase 4: GPU Computing** (Medium Priority 🟡)

- CUDA backend
- OpenCL backend
- Memory management (host ↔ device)
- Kernel launch primitives

**Phase 5: Sync Primitives** (Low Priority 🟢)

- Mutex, RwLock
- Atomic operations
- Semaphore, Barrier
- WaitGroup

---

**Previous**: [12_Memory_Management.md](./12_Memory_Management.md)  
**Next**: [14_Modules_and_Imports.md](./14_Modules_and_Imports.md)

**Maintained by**: Vex Language Team

# Memory Management

**Version:** 0.1.2  
**Last Updated:** November 2025

This document defines memory management, ownership, and borrowing in the Vex programming language.

---

## Table of Contents

1. [Ownership System](#ownership-system)
2. [Borrowing Rules](#borrowing-rules)
3. [Borrow Checker](#borrow-checker)
4. [Lifetimes](#lifetimes)
5. [Memory Layout](#memory-layout)
6. [Resource Management](#resource-management)

---

## Ownership System

### Core Principles

Vex uses **ownership-based memory management** without garbage collection:

1. **Each value has exactly one owner**
2. **When the owner goes out of scope, the value is dropped**
3. **Ownership can be transferred (moved)**
4. **Values can be borrowed temporarily**

### Ownership Transfer (Move Semantics)

```vex
let x = Point { x: 10, y: 20 };
let y = x;  // Ownership moves from x to y
// x is no longer valid!
```

**After Move**:

```vex
let x = Point { x: 10, y: 20 };
let y = x;
// ERROR: x has been moved
// let z = x;
```

### Copy Types

Some types implement implicit copy (primitives):

```vex
let x = 42;
let y = x;  // x is copied, not moved
// Both x and y are valid
```

**Copy Types**:

- All integer types: i8-i64, u8-u64
- Floating-point types: f32, f64
- Boolean: bool
- Tuples of copy types: `(i32, i32)`

**Move Types**:

- String: `string`
- Arrays: `[T; N]` (unless T is Copy)
- Structs: All user-defined structs
- Enums: All enums with data

---

## Borrowing Rules

### Immutable Borrowing

**Syntax**: `&T`

```vex
let x = 42;
let ref_x: &i32 = &x;  // Borrow x immutably
```

**Properties**:

- Can have multiple immutable borrows
- Cannot modify through immutable reference
- Original owner cannot modify while borrowed

**Example**:

```vex
fn print_value(x: &i32) {
    // Can read x, cannot modify
}

let value = 42;
print_value(&value);
// value still accessible here
```

### Mutable Borrowing

**Syntax**: `&T!` (v0.1 syntax)

```vex
let! x = 42;
let ref_x: &i32! = &x;  // Borrow x mutably
```

**Properties**:

- Can have only ONE mutable borrow at a time
- Cannot have immutable borrows while mutably borrowed
- Can modify through mutable reference

**Example**:

```vex
fn increment(x: &i32!) {
    *x = *x + 1;  // Modify through reference
}

let! value = 42;
increment(&value);
// value is now 43
```

### The Core Rule

**"One mutable XOR many immutable"**:

```vex
let! x = 42;

// OK: Multiple immutable borrows
let r1: &i32 = &x;
let r2: &i32 = &x;
let r3: &i32 = &x;

// OK: Single mutable borrow
let! x = 42;
let r1: &i32! = &x;

// ERROR: Cannot mix mutable and immutable
let! x = 42;
let r1: &i32 = &x;
let r2: &i32! = &x;  // ERROR!
```

### Borrowing Examples

**Read-Only Access**:

```vex
fn calculate_area(rect: &Rectangle): i32 {
    return rect.width * rect.height;
}

let r = Rectangle { width: 10, height: 20 };
let area = calculate_area(&r);
// r still valid
```

**Mutation Through Reference**:

```vex
fn scale_rectangle(rect: &Rectangle!, factor: i32) {
    rect.width = rect.width * factor;
    rect.height = rect.height * factor;
}

let! r = Rectangle { width: 10, height: 20 };
scale_rectangle(&r, 2);
// r is now { width: 20, height: 40 }
```

---

## Borrow Checker

### Four-Phase System (v0.1.2)

Vex implements a **four-phase borrow checker**:

#### Phase 1: Immutability Checking ✅

Enforces `let` vs `let!` semantics:

```vex
let x = 42;
// x = 100;  // ERROR: Cannot assign to immutable variable

let! y = 42;
y = 100;     // OK: y is mutable
```

**Test Coverage**: 7 tests passing

#### Phase 2: Move Semantics ✅

Prevents use-after-move:

```vex
let point = Point { x: 10, y: 20 };
let moved = point;
// let error = point;  // ERROR: point has been moved
```

**Test Coverage**: 5 tests passing

#### Phase 3: Borrow Rules ✅

Enforces reference rules:

```vex
let! x = 42;
let r1: &i32! = &x;
// let r2: &i32! = &x;  // ERROR: Cannot have two mutable borrows
```

**Test Coverage**: 5 tests passing

#### Phase 4: Lifetime Analysis ✅

**Status**: ✅ **COMPLETE** (v0.1.2)

**Purpose**: Track reference validity across scopes and prevent dangling references

Lifetime analysis prevents common memory safety bugs:

- **Dangling references**: References to deallocated memory
- **Use-after-free**: Using memory after it's been freed
- **Return local reference**: Returning references to local variables

**How It Works**:

The lifetime checker tracks:

1. **Variable scopes**: When variables are created and destroyed
2. **Reference tracking**: Which references point to which variables
3. **Scope validation**: Ensures references don't outlive their referents
4. **Return value analysis**: Prevents returning references to locals

**Examples**:

```vex
// ✅ Valid: Basic reference lifetime
fn test_basic_lifetime() {
    let x = 10;
    let y = &x;  // OK: x is in scope
    print(*y);
}

// ❌ Error: Dangling reference
fn test_dangling_reference(): &i32 {
    let x = 42;
    return &x;  // ERROR: Cannot return reference to local variable
}

// ❌ Error: Reference outlives referent
fn test_reference_outlives() {
    let y: &i32;
    {
        let x = 10;
        y = &x;  // ERROR: x goes out of scope, y would dangle
    }
    print(*y);  // ERROR: y is dangling
}

// ✅ Valid: Reference to field (lifetime tied to struct)
fn test_method_lifetime(self: &Vector2): &f32 {
    return &self.x;  // OK: Field lifetime tied to self
}

// ✅ Valid: Heap allocation (ownership transferred)
fn test_valid_reference_lifetime(): &i32 {
    let x = Box.new(42);  // Heap allocation
    return x;  // OK: Box ownership transferred
}
```

**Implementation Details**:

- **Checker**: `vex-compiler/src/borrow_checker/lifetimes.rs`
- **Scope tracking**: Maintains variable scope depth (0=global, 1=function, 2+=blocks)
- **Reference map**: Tracks which references point to which variables
- **Global variables**: Extern functions and constants never go out of scope
- **Builtin registry**: Identifies builtin functions for special handling
- **Test file**: `examples/test_lifetimes.vx`

**Test Coverage**: 8+ tests passing (v0.1.2)

### Borrow Checker Errors

**Immutability Violation**:

```
Borrow Checker Error: Cannot assign to immutable variable 'x'
  --> example.vx:3:1
   |
1  | let x = 42;
   |     - variable declared as immutable here
2  |
3  | x = 100;
   | ^^^^^^^ cannot assign to immutable variable
   |
   = help: consider declaring it as mutable: `let! x = 42;`
```

**Use After Move**:

```
Borrow Checker Error: Use of moved value 'point'
  --> example.vx:3:9
   |
1  | let point = Point { x: 10, y: 20 };
2  | let moved = point;
   |             ----- value moved here
3  | let error = point;
   |             ^^^^^ value used after move
   |
   = note: move occurs because `point` has type `Point`, which does not implement `Copy`
```

**Multiple Mutable Borrows**:

```
Borrow Checker Error: Cannot borrow 'x' as mutable more than once
  --> example.vx:3:17
   |
2  | let r1: &i32! = &x;
   |                 -- first mutable borrow occurs here
3  | let r2: &i32! = &x;
   |                 ^^ second mutable borrow occurs here
```

---

## Lifetimes

### Concept (Phase 4 - Future)

Lifetimes track how long references are valid:

```vex
fn example<'a>(x: &'a i32): &'a i32 {
    return x;  // Returned reference lives as long as input
}
```

### Lifetime Annotations

Vex automatically infers lifetimes in all cases, so explicit annotations are rarely needed.

---

## Memory Layout

### Stack Allocation

Most values allocated on stack:

```vex
let x = 42;            // Stack: 4 bytes
let point = Point {    // Stack: 8 bytes (2 × i32)
    x: 10,
    y: 20,
};
```

**Stack Properties**:

- Fast allocation/deallocation
- Automatic cleanup (scope-based)
- Limited size
- LIFO (Last In, First Out)

### Heap Allocation (Future)

Dynamic allocation for variable-size data:

```vex
let buffer = Box::new([0; 1024]);  // Heap allocation
let text = String::from("hello");  // Heap string
```

**Heap Properties**:

- Slower than stack
- Manual management (ownership)
- Unlimited size (system dependent)
- Fragmentation possible

### Memory Alignment

Types align to natural boundaries:

```vex
struct Example {
    a: i8,    // 1 byte, aligned to 1
    b: i32,   // 4 bytes, aligned to 4
    c: i16,   // 2 bytes, aligned to 2
}
// Size: 12 bytes (with padding)
```

**Alignment Rules**:

- i8: 1-byte alignment
- i16: 2-byte alignment
- i32: 4-byte alignment
- i64: 8-byte alignment
- f32: 4-byte alignment
- f64: 8-byte alignment

---

## Resource Management

### RAII Pattern (Future)

Resources tied to object lifetime:

```vex
struct File {
    handle: i32,
}

impl Drop for File {
    fn drop(self: &File!) {
        // Close file automatically when File goes out of scope
        close_handle(self.handle);
    }
}
```

### Manual Cleanup

Current approach - explicit cleanup:

```vex
fn process_file(path: string) {
    let file = open_file(path);
    // Use file
    close_file(file);  // Manual cleanup
}
```

### Defer Statement (Future - Go-style)

```vex
fn process() {
    let file = open("data.txt");
    defer close(file);  // Executes when function returns

    // Use file
    // close(file) called automatically
}
```

---

## Best Practices

### 1. Prefer Immutable Bindings

```vex
// Good: Immutable by default
let x = 42;
let data = load_data();

// Only use mutable when necessary
let! counter = 0;
counter = counter + 1;
```

### 2. Use References for Large Data

```vex
// Good: Pass by reference
fn process_large_array(data: &[i32; 10000]) {
    // Read data without copying
}

// Bad: Unnecessary copy
fn process_large_array(data: [i32; 10000]) {
    // Copies entire array!
}
```

### 3. Borrow, Don't Move

```vex
// Good: Borrow when ownership not needed
fn print_point(p: &Point) {
    // Read-only access
}

let point = Point { x: 10, y: 20 };
print_point(&point);
// point still valid

// Bad: Takes ownership unnecessarily
fn print_point(p: Point) {
    // point moved, original invalid
}
```

### 4. Minimize Mutable State

```vex
// Good: Functional approach
fn add(x: i32, y: i32): i32 {
    return x + y;
}

// Bad: Unnecessary mutation
fn add(x: i32, y: i32): i32 {
    let! result = x;
    result = result + y;
    return result;
}
```

### 5. Clear Ownership

```vex
// Good: Clear ownership transfer
fn take_ownership(s: string) {
    // s is owned here
}

let text = "hello";
take_ownership(text);
// text is moved

// Bad: Unclear borrowing
fn process(s: &string!) {
    // Mutable borrow, but does it need to be?
}
```

---

## Memory Management Summary

| Feature                 | Status       | Description                 |
| ----------------------- | ------------ | --------------------------- |
| **Ownership**           | ✅ Working   | Each value has one owner    |
| **Move Semantics**      | ✅ Phase 2   | Transfer ownership          |
| **Copy Types**          | ✅ Working   | Primitive types auto-copy   |
| **Immutable Borrow**    | ✅ Phase 3   | `&T` reference              |
| **Mutable Borrow**      | ✅ Phase 3   | `&T!` reference             |
| **Borrow Checker**      | ✅ Phase 1-4 | Compile-time checking       |
| **Lifetimes**           | ✅ Phase 4   | Reference validity tracking |
| **Drop Contract**       | ❌ Future    | RAII destructors            |
| **Box Type**            | ❌ Future    | Heap allocation             |
| **Reference Counting**  | ❌ Future    | Rc/Arc types                |
| **Interior Mutability** | ❌ Future    | Cell/RefCell                |

### Test Coverage

- **Phase 1 (Immutability)**: 7/7 tests passing ✅
- **Phase 2 (Move Semantics)**: 5/5 tests passing ✅
- **Phase 3 (Borrow Rules)**: 5/5 tests passing ✅
- **Phase 4 (Lifetimes)**: 5/5 tests passing ✅ (v0.1.2)
- **Total**: 22/22 borrow checker tests passing (100%)

---

## Examples

### Ownership Transfer

```vex
fn main(): i32 {
    let x = Point { x: 10, y: 20 };
    let y = x;  // x moved to y
    // x is invalid now
    return y.x;  // 10
}
```

### Immutable Borrowing

```vex
fn sum(a: &i32, b: &i32): i32 {
    return *a + *b;
}

fn main(): i32 {
    let x = 10;
    let y = 20;
    let result = sum(&x, &y);
    // x and y still valid
    return result;  // 30
}
```

### Mutable Borrowing

```vex
fn increment(x: &i32!) {
    *x = *x + 1;
}

fn main(): i32 {
    let! value = 42;
    increment(&value);
    return value;  // 43
}
```

### Borrow Checker Error

```vex
fn main(): i32 {
    let x = 42;
    x = 100;  // ERROR: Cannot assign to immutable variable
    return 0;
}
```

---

**Previous**: [11_Pattern_Matching.md](./11_Pattern_Matching.md)  
**Next**: [13_Concurrency.md](./13_Concurrency.md)

**Maintained by**: Vex Language Team

# Modules and Imports

**Version:** 0.1.2  
**Last Updated:** November 2025

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
- **JavaScript-like semantics**: Only exported symbols are accessible from outside

### Module Privacy (JavaScript-like)

**Important**: Vex follows JavaScript/TypeScript module semantics, NOT Rust/Go:

```vex
// math/internal.vx
fn fabs(x: f64): f64 {  // Private - not exported
    // Internal implementation
}

export fn abs(x: f64): f64 {  // Public API
    return fabs(x);  // ✅ Can call within same module
}

// main.vx
import { abs } from "math/internal";

fn main() {
    abs(-5.0);   // ✅ Works - abs is exported
    fabs(-5.0);  // ❌ Error - fabs is NOT exported
}
```

**Key Difference**:

- **JavaScript/Vex**: Functions can call non-exported functions in their own module
- **Rust/Go**: All symbols in a module are visible to importers (package-level visibility)

When you import a function, it carries its own module context - internal calls remain valid.

### Module Paths

Standard library modules:

```
vex-libs/std/
├── math/
│   ├── vex.json        # Package manifest (optional)
│   ├── src/
│   │   ├── lib.vx      # Main entry point (default)
│   │   └── native.vxc  # Native FFI bindings
├── io/
│   ├── vex.json
│   ├── src/
│   │   ├── lib.vx      # Main module
│   │   ├── file.vx     # Submodule
│   │   └── stream.vx   # Submodule
├── net/
│   ├── vex.json
│   └── src/
│       ├── lib.vx
│       ├── http.vx
│       └── tcp.vx
└── ...
```

### Import Resolution Rules

**1. Package Name (Recommended)**

```vex
import { abs } from "math";
// → Resolves to: vex-libs/std/math/src/lib.vx (from vex.json "main" field)
```

**2. Direct File Import**

```vex
import { sin } from "math/native.vxc";
// → Resolves to: vex-libs/std/math/src/native.vxc

import { helper } from "io/file.vx";
// → Resolves to: vex-libs/std/io/src/file.vx
```

**3. Relative Imports (Within Module)**

```vex
// In: vex-libs/std/math/src/lib.vx
import { fabs } from "./native.vxc";
// → Resolves to: vex-libs/std/math/src/native.vxc
```

### Package Entry Point (vex.json)

The `main` field in `vex.json` specifies the primary export:

```json
{
  "name": "math",
  "version": "1.0.0",
  "main": "src/lib.vx"
}
```

**Resolution**:

- `import from "math"` → Uses `main` field → `src/lib.vx`
- `import from "math/native.vxc"` → Direct file path → `src/native.vxc`
- If no `vex.json`: Defaults to `src/lib.vx` or `src/mod.vx`

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

### Export Keyword (v0.1)

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
    x: i32,
    y: i32,
}

// All fields in exported structs are accessible
// Use underscore prefix for internal/helper fields (convention only)
export struct User {
    id: i64,
    name: String,
    _internal_cache: i32,  // Convention: internal field
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

### Export Contracts

```vex
export contract Display {
    fn show();
}
```

### Export Constants

```vex
export const MAX_SIZE: i32 = 1024;
export const VERSION: string = "0.1.0";
```

### Export Policies

```vex
export policy Debug {
    description: "Debug information",
    version: "1.0.0",
}

export policy Serializable {
    description: "Can be serialized",
    format: "json",
}
```

### Re-exports

```vex
// Re-export from another module
import { helper } from "internal";
export { helper };

// Or directly
export { helper } from "internal";
```

### Default Export Behavior

**v0.1.2**: If NO explicit `export` declarations exist in a module, ALL symbols are exported (export-all).

```vex
// math.vx - No explicit exports
fn abs(x: i32): i32 { ... }  // ✅ Exported (export-all mode)
fn helper(): i32 { ... }     // ✅ Exported (export-all mode)

// vs.

// math.vx - With explicit exports
export fn abs(x: i32): i32 { ... }  // ✅ Exported
fn helper(): i32 { ... }            // ❌ NOT exported (private)
```

**Rule**: Once you use `export` on ANY symbol, ONLY explicitly exported symbols are visible.

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

### Layer 0: Vex Runtime

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

| Feature               | Status         | Example                                           |
| --------------------- | -------------- | ------------------------------------------------- |
| **Named Import**      | ✅ Working     | `import { println } from "io"`                    |
| **Import with Alias** | ✅ Working     | `import * as io from "io"`                        |
| **Export**            | ✅ v0.1        | `export fn name()`                                |
| **Module Resolution** | ✅ Working     | Loads from `vex-libs/std/`                        |
| **Nested Modules**    | ✅ Working     | `import { get } from "net/http"`                  |
| **Re-exports**        | ✅ Working     | `export { x } from "mod"`                         |
| **Private Items**     | ✅ Working     | Default (no export)                               |
| **Field Visibility**  | ❌ Not Planned | All fields accessible (use `_` prefix convention) |
| **Relative Imports**  | ✅ Working     | `import "./local"` supported                      |
| **Package System**    | ✅ vex-pm      | Full package manager with dependency resolution   |

---

**Previous**: [13_Concurrency.md](./13_Concurrency.md)  
**Next**: [15_Standard_Library.md](./15_Standard_Library.md)

**Maintained by**: Vex Language Team

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

# Error Handling

**Version:** 0.1.2
**Last Updated:** November 2025

This document describes Vex's error handling system, including the `Result<T, E>` and `Option<T>` types, pattern matching for error handling, and best practices.

---

## Table of Contents

1. [Overview](#overview)
2. [Option<T> Type](#optiont-type)
3. [Result<T, E> Type](#resultt-e-type)
4. [Pattern Matching for Errors](#pattern-matching-for-errors)
5. [Error Propagation](#error-propagation)
6. [Custom Error Types](#custom-error-types)
7. [Best Practices](#best-practices)

---

## Overview

Vex provides two primary types for handling potential failure:

- **`Option<T>`**: Represents a value that may or may not exist
- **`Result<T, E>`**: Represents either success (`T`) or failure (`E`)

Both types encourage explicit handling of potential errors rather than exceptions or null pointers.

---

## Option<T> Type

The `Option<T>` type represents an optional value. It has two variants:

```vex
enum Option<T> {
    Some(T),
    None
}
```

### Basic Usage

```vex
// Function that may not find a value
fn find_user(id: i32): Option<User> {
    if id == 42 {
        return Some(User { id: 42, name: "Alice" });
    }
    return None;
}

// Using the result
let user = find_user(42);
match user {
    Some(u) => println("Found user: {}", u.name),
    None => println("User not found")
}
```

### Common Methods

```vex
let maybe_value: Option<i32> = Some(42);

// Check if value exists
if maybe_value.is_some() {
    println("Value exists");
}

// Get value with default
let value = maybe_value.unwrap_or(0); // Returns 42

// Transform option
let doubled = maybe_value.map(|x| x * 2); // Some(84)

// Chain operations
let result = maybe_value
    .filter(|x| x > 40)
    .map(|x| x + 1); // Some(43)
```

---

## Result<T, E> Type

The `Result<T, E>` type represents either success or failure. It has two variants:

```vex
enum Result<T, E> {
    Ok(T),
    Err(E)
}
```

### Basic Usage

```vex
// Function that may fail
fn divide(a: i32, b: i32): Result<i32, String> {
    if b == 0 {
        return Err("Division by zero".to_string());
    }
    return Ok(a / b);
}

// Using the result
let result = divide(10, 2);
match result {
    Ok(value) => println("Result: {}", value),
    Err(error) => println("Error: {}", error)
}
```

### Common Methods

```vex
let result: Result<i32, String> = Ok(42);

// Check result type
if result.is_ok() {
    println("Success!");
}

// Get values
let value = result.unwrap(); // Panics if Err
let value_or_default = result.unwrap_or(0);

// Transform results
let doubled = result.map(|x| x * 2); // Ok(84)

// Handle errors
let error_message = result.unwrap_err(); // Panics if Ok

// Convert Option to Result
let option: Option<i32> = Some(42);
let result = option.ok_or("Value not found"); // Ok(42)
```

---

## Pattern Matching for Errors

Pattern matching provides elegant error handling:

```vex
fn process_data(data: Vec<i32>): Result<String, String> {
    // Validate input
    if data.is_empty() {
        return Err("No data provided".to_string());
    }

    // Process each item
    for item in data {
        match validate_item(item) {
            Ok(processed) => println("Processed: {}", processed),
            Err(error) => return Err(format("Validation failed: {}", error))
        }
    }

    Ok("All data processed successfully".to_string())
}

fn validate_item(item: i32): Result<i32, String> {
    if item < 0 {
        Err("Negative values not allowed".to_string())
    } else if item > 100 {
        Err("Values too large".to_string())
    } else {
        Ok(item * 2)
    }
}
```

### Nested Matching

```vex
fn complex_operation(): Result<i32, String> {
    let config = load_config()?;
    let data = fetch_data(config)?;
    let result = process_data(data)?;

    Ok(result)
}

// Equivalent to:
fn complex_operation_manual(): Result<i32, String> {
    match load_config() {
        Ok(config) => match fetch_data(config) {
            Ok(data) => match process_data(data) {
                Ok(result) => Ok(result),
                Err(e) => Err(e)
            },
            Err(e) => Err(e)
        },
        Err(e) => Err(e)
    }
}
```

---

## Error Propagation

Vex supports the `?` operator for concise error propagation, similar to Rust.

**Implementation Status**: ✅ **COMPLETE** (v0.1.2)

### The `?` Operator

The question mark operator (`?`) provides automatic error propagation for `Result<T, E>` types:

```vex
fn divide(a: i32, b: i32): Result<i32, string> {
    if b == 0 {
        return Err("Division by zero");
    };
    return Ok(a / b);
}

fn safe_divide(a: i32, b: i32): Result<i32, string> {
    let result = divide(a, b)?;  // Early return if Err
    return Ok(result * 2);
}
```

**How it works**:

The `?` operator desugars to a match expression:

```vex
// This code:
let result = divide(10, 2)?;

// Becomes:
let result = match divide(10, 2) {
    Ok(v) => v,
    Err(e) => return Err(e)
};
```

### Nested Error Propagation

```vex
fn read_and_process_file(filename: string): Result<string, string> {
    let content = read_file(filename)?;      // Propagates file errors
    let data = parse_json(content)?;         // Propagates parse errors
    let result = validate_data(data)?;       // Propagates validation errors

    return Ok(result);
}

// Equivalent without ? operator
fn read_and_process_file_manual(filename: string): Result<string, string> {
    match read_file(filename) {
        Ok(content) => {
            match parse_json(content) {
                Ok(data) => {
                    match validate_data(data) {
                        Ok(result) => { return Ok(result); },
                        Err(e) => { return Err(e); }
                    };
                },
                Err(e) => { return Err(e); }
            };
        },
        Err(e) => { return Err(e); }
    };
}
```

### Chain of Operations

```vex
fn nested_operations(): Result<i32, string> {
    let x = divide(10, 2)?;   // Ok(5)
    let y = divide(x, 0)?;    // Err("Division by zero") - propagates here
    let z = divide(y, 2)?;    // Never reached
    return Ok(z);
}
```

**Implementation Details**:

- **Parser**: `vex-parser/src/parser/operators.rs` - Parses `expr?` syntax
- **AST**: `vex-ast/src/lib.rs` - `Expression::QuestionMark(Box<Expression>)`
- **Codegen**: `vex-compiler/src/codegen_ast/expressions/mod.rs` - Desugars to Result match
- **Test file**: `examples/test_question_mark.vx`

### Early Returns

```vex
fn authenticate_user(username: String, password: String): Result<User, AuthError> {
    // Validate input
    if username.is_empty() {
        return Err(AuthError::InvalidUsername);
    }

    if password.is_empty() {
        return Err(AuthError::InvalidPassword);
    }

    // Attempt authentication
    let user = find_user(username)?;
    verify_password(user, password)?;

    Ok(user)
}
```

---

## Custom Error Types

Define custom error types using enums:

```vex
enum DatabaseError {
    ConnectionFailed(String),
    QueryFailed(String),
    NotFound(i32)
}

enum ApiError {
    NetworkError(String),
    AuthenticationFailed,
    ValidationError(Vec<String>),
    DatabaseError(DatabaseError)
}

fn fetch_user(id: i32): Result<User, ApiError> {
    // Attempt database connection
    let connection = connect_to_db().map_err(|e| ApiError::DatabaseError(e))?;

    // Execute query
    match connection.query_user(id) {
        Ok(user) => Ok(user),
        Err(DatabaseError::NotFound(_)) => Err(ApiError::NotFound),
        Err(e) => Err(ApiError::DatabaseError(e))
    }
}
```

### Error Contracts

Implement common error contracts for better ergonomics:

```vex
contract Error {
    fn message(&self): String;
}

impl Error for DatabaseError {
    fn message(&self): String {
        match self {
            ConnectionFailed(msg) => format("Connection failed: {}", msg),
            QueryFailed(msg) => format("Query failed: {}", msg),
            NotFound(id) => format("User {} not found", id)
        }
    }
}
```

---

## Best Practices

### 1. Use Result for Operations That Can Fail

```vex
// Good: Explicit error handling
fn parse_number(input: String): Result<i32, ParseError> {
    // Implementation
}

// Bad: Using Option when you need error details
fn parse_number_bad(input: String): Option<i32> {
    // Can't provide specific error information
}
```

### 2. Define Specific Error Types

```vex
// Good: Specific errors
enum ConfigError {
    FileNotFound(String),
    ParseError(String),
    ValidationError(Vec<String>)
}

// Bad: Generic string errors
fn load_config(): Result<Config, String> {
    // Error details lost in generic strings
}
```

### 3. Use ? for Early Returns

```vex
// Good: Clear control flow
fn process_request(req: Request): Result<Response, Error> {
    let user = authenticate(req)?;
    let data = validate_input(req)?;
    let result = perform_business_logic(user, data)?;

    Ok(create_response(result))
}
```

### 4. Handle Errors at Appropriate Levels

```vex
// Good: Handle errors where you can respond appropriately
fn handle_request(req: Request): Response {
    match process_request(req) {
        Ok(data) => Response::success(data),
        Err(Error::ValidationError(fields)) => Response::bad_request(fields),
        Err(Error::NotFound) => Response::not_found(),
        Err(_) => Response::internal_error()
    }
}
```

### 5. Avoid unwrap() in Production

```vex
// Bad: Will panic in production
let value = result.unwrap();

// Good: Handle the error
let value = match result {
    Ok(v) => v,
    Err(e) => return Err(e) // or provide default
};
```

### 6. Use expect() for Programming Errors

```vex
// Good: Document assumptions
let config = load_config().expect("Config file should always exist");

// Bad: Silent unwrap
let config = load_config().unwrap();
```

---

**Previous**: [16_Standard_Library.md](./16_Standard_Library.md)  
**Next**: [18_Raw_Pointers_and_FFI.md](./18_Raw_Pointers_and_FFI.md)

**Maintained by**: Vex Language Team</content>
<parameter name="filePath">/Users/mapletechnologies/Desktop/big_projects/vex_lang/Specifications/17_Error_Handling.md

# Raw Pointers and FFI

**Version:** 0.1.2
**Last Updated:** November 2025

This document describes Vex's support for raw pointers and Foreign Function Interface (FFI) capabilities, including unsafe code blocks, extern declarations, and interoperability with C libraries.

---

## Table of Contents

1. [Overview](#overview)
2. [Raw Pointer Types](#raw-pointer-types)
3. [Unsafe Code Blocks](#unsafe-code-blocks)
4. [Foreign Function Interface](#foreign-function-interface)
5. [Memory Management](#memory-management)
6. [Best Practices](#best-practices)
7. [Examples](#examples)

---

## Overview

Vex provides low-level memory access and FFI capabilities through:

- **Raw pointers**: `*T` (mutable) and `*const T` (immutable)
- **Unsafe blocks**: `unsafe { ... }` for operations requiring special care
- **Extern declarations**: Interface with C and other languages
- **Memory intrinsics**: Direct memory operations

These features are isolated in `unsafe` blocks to maintain memory safety guarantees elsewhere in the codebase.

---

## Raw Pointer Types

Vex supports two types of raw pointers:

```vex
*T        // Mutable raw pointer
*const T  // Immutable raw pointer
```

### Basic Usage

```vex
// Mutable pointer
let mut value = 42;
let ptr: *i32 = &value as *i32;

// Immutable pointer
let const_ptr: *const i32 = &value as *const i32;

// Null pointer
let null_ptr: *i32 = 0 as *i32;
```

### Pointer Operations

All pointer operations must occur within `unsafe` blocks:

```vex
unsafe {
    // Dereference
    let value = *ptr;

    // Modify through pointer
    *ptr = 100;

    // Pointer arithmetic
    let next_ptr = ptr.offset(1);

    // Check null
    if !ptr.is_null() {
        *ptr = 42;
    }
}
```

### Common Patterns

```vex
// Array access through pointers
fn sum_array(arr: *const i32, len: usize): i32 {
    let mut sum = 0;
    unsafe {
        for i in 0..len {
            sum += *arr.offset(i as isize);
        }
    }
    sum
}

// String manipulation
fn strlen(s: *const u8): usize {
    unsafe {
        let mut len = 0;
        while *s.offset(len as isize) != 0 {
            len += 1;
        }
        len
    }
}
```

---

## Unsafe Code Blocks

Unsafe blocks allow operations that bypass Vex's safety guarantees:

```vex
unsafe {
    // Raw pointer operations
    let ptr = 0x1000 as *mut i32;
    *ptr = 42;

    // Call unsafe functions
    libc::memset(ptr as *mut u8, 0, 4);

    // Access union fields
    let value = my_union.unsafe_field;
}
```

### Unsafe Functions

Mark functions as unsafe to indicate they require special care:

```vex
unsafe fn dangerous_operation(ptr: *mut i32) {
    *ptr = 999;
}

fn safe_wrapper(value: &mut i32) {
    unsafe {
        dangerous_operation(value as *mut i32);
    }
}
```

### Safety Contracts

```vex
/// # Safety
/// - `ptr` must be valid for `len` elements
/// - Memory must not be accessed concurrently
/// - `ptr` must be properly aligned
unsafe fn process_buffer(ptr: *mut u8, len: usize) {
    // Implementation with safety assumptions
}
```

---

## Foreign Function Interface

Vex can interface with C and other languages using `extern` blocks:

### Basic Extern Declaration

```vex
extern "C" {
    fn printf(format: *const u8, ...): i32;
    fn malloc(size: usize): *mut u8;
    fn free(ptr: *mut u8);
}

fn main(): i32 {
    unsafe {
        let msg = "Hello, World!\n" as *const u8;
        printf(msg, 0);
    }
    0
}
```

### Function Pointers

```vex
extern "C" {
    fn qsort(
        base: *mut u8,
        nmemb: usize,
        size: usize,
        compar: fn(*const u8, *const u8): i32
    );
}

fn compare_ints(a: *const u8, b: *const u8): i32 {
    unsafe {
        let x = *(a as *const i32);
        let y = *(b as *const i32);
        if x < y { -1 } else if x > y { 1 } else { 0 }
    }
}

fn sort_array(arr: &mut [i32]) {
    unsafe {
        qsort(
            arr.as_mut_ptr() as *mut u8,
            arr.len(),
            4, // sizeof(i32)
            compare_ints
        );
    }
}
```

### Variadic Functions

```vex
extern "C" {
    fn sprintf(dest: *mut u8, format: *const u8, ...): i32;
}

fn format_number(dest: &mut [u8], value: i32): usize {
    unsafe {
        sprintf(
            dest.as_mut_ptr(),
            "%d\n" as *const u8,
            value
        ) as usize
    }
}
```

### Different ABIs

```vex
// C ABI (default)
extern "C" {
    fn c_function(): i32;
}

// System ABI
extern "system" {
    fn system_call(id: i32, ...): i32;
}
```

---

## Memory Management

### Manual Allocation

```vex
fn manual_allocation() {
    unsafe {
        // Allocate memory
        let ptr = malloc(100) as *mut i32;

        if ptr.is_null() {
            panic("Allocation failed");
        }

        // Use memory
        for i in 0..25 {
            *ptr.offset(i) = i * 2;
        }

        // Deallocate
        free(ptr as *mut u8);
    }
}
```

### Stack Allocation

```vex
fn stack_buffer() {
    // Fixed-size stack allocation
    let mut buffer: [u8; 1024] = [0; 1024];

    unsafe {
        // Get pointer to buffer
        let ptr = buffer.as_mut_ptr();

        // Fill buffer
        libc::memset(ptr, 65, 1024); // Fill with 'A'
    }

    // Buffer automatically cleaned up
}
```

### Memory Mapping

```vex
extern "C" {
    fn mmap(
        addr: *mut u8,
        len: usize,
        prot: i32,
        flags: i32,
        fd: i32,
        offset: i64
    ): *mut u8;
    fn munmap(addr: *mut u8, len: usize): i32;
}

const PROT_READ: i32 = 1;
const PROT_WRITE: i32 = 2;
const MAP_PRIVATE: i32 = 2;
const MAP_ANONYMOUS: i32 = 32;

fn allocate_huge_page(size: usize): *mut u8 {
    unsafe {
        mmap(
            0 as *mut u8,
            size,
            PROT_READ | PROT_WRITE,
            MAP_PRIVATE | MAP_ANONYMOUS,
            -1,
            0
        )
    }
}
```

---

## Best Practices

### 1. Minimize Unsafe Code

```vex
// Good: Isolate unsafe code
fn safe_abstraction(data: &mut [i32]) {
    unsafe {
        // Minimal unsafe operations
        libc::memset(data.as_mut_ptr() as *mut u8, 0, data.len() * 4);
    }
}

// Bad: Large unsafe blocks
fn bad_example(data: &mut [i32]) {
    unsafe {
        // Lots of unsafe code mixed with safe operations
        for i in 0..data.len() {
            *data.as_mut_ptr().offset(i as isize) = i as i32;
        }
        libc::qsort(data.as_mut_ptr() as *mut u8, data.len(), 4, compare);
        validate_data(data);
    }
}
```

### 2. Validate Pointers

```vex
fn safe_dereference(ptr: *const i32): Option<i32> {
    if ptr.is_null() {
        return None;
    }

    unsafe {
        Some(*ptr)
    }
}
```

### 3. Use Safe Abstractions

```vex
// Good: Provide safe interface over unsafe operations
struct SafeBuffer {
    ptr: *mut u8,
    len: usize,
}

impl SafeBuffer {
    fn new(size: usize): Option<SafeBuffer> {
        unsafe {
            let ptr = malloc(size);
            if ptr.is_null() {
                None
            } else {
                Some(SafeBuffer { ptr, len: size })
            }
        }
    }

    fn as_slice(&self): &[u8] {
        unsafe {
            std::slice::from_raw_parts(self.ptr, self.len)
        }
    }
}

impl Drop for SafeBuffer {
    fn drop(&mut self) {
        unsafe {
            free(self.ptr);
        }
    }
}
```

### 4. Document Safety Requirements

```vex
/// Reads exactly `len` bytes from `src` into `dest`.
///
/// # Safety
/// - `src` must be valid for `len` bytes
/// - `dest` must be valid for `len` bytes
/// - Memory regions must not overlap
/// - Both pointers must be properly aligned
unsafe fn memcpy(dest: *mut u8, src: *const u8, len: usize) {
    // Implementation
}
```

### 5. Avoid Common Pitfalls

```vex
// Bad: Use after free
fn use_after_free() {
    unsafe {
        let ptr = malloc(4) as *mut i32;
        *ptr = 42;
        free(ptr as *mut u8);
        let value = *ptr; // Undefined behavior!
    }
}

// Bad: Double free
fn double_free() {
    unsafe {
        let ptr = malloc(4);
        free(ptr);
        free(ptr); // Undefined behavior!
    }
}

// Bad: Buffer overflow
fn buffer_overflow() {
    let buffer: [i32; 10] = [0; 10];
    unsafe {
        for i in 0..20 { // Overflow!
            *buffer.as_ptr().offset(i) = i;
        }
    }
}
```

---

## Examples

### C Library Integration

```vex
extern "C" {
    fn fopen(filename: *const u8, mode: *const u8): *mut File;
    fn fread(ptr: *mut u8, size: usize, count: usize, stream: *mut File): usize;
    fn fclose(stream: *mut File): i32;
}

struct File;

fn read_file_contents(filename: String): Result<Vec<u8>, String> {
    unsafe {
        let file = fopen(filename.as_ptr(), "rb" as *const u8);
        if file.is_null() {
            return Err("Failed to open file".to_string());
        }

        let mut buffer = Vec::with_capacity(1024);
        buffer.resize(1024, 0);

        let bytes_read = fread(
            buffer.as_mut_ptr(),
            1,
            buffer.len(),
            file
        );

        fclose(file);

        buffer.truncate(bytes_read);
        Ok(buffer)
    }
}
```

### SIMD Operations

```vex
extern "C" {
    // SIMD intrinsics
    fn _mm_add_ps(a: __m128, b: __m128): __m128;
    fn _mm_load_ps(ptr: *const f32): __m128;
    fn _mm_store_ps(ptr: *mut f32, val: __m128);
}

type __m128 = [f32; 4]; // 128-bit SIMD register

fn vector_add(a: &[f32], b: &[f32], result: &mut [f32]) {
    unsafe {
        for i in (0..a.len()).step_by(4) {
            let va = _mm_load_ps(a.as_ptr().offset(i as isize));
            let vb = _mm_load_ps(b.as_ptr().offset(i as isize));
            let sum = _mm_add_ps(va, vb);
            _mm_store_ps(result.as_mut_ptr().offset(i as isize), sum);
        }
    }
}
```

### System Calls

```vex
extern "C" {
    fn syscall(number: i64, ...) -> i64;
}

const SYS_write: i64 = 1;
const SYS_exit: i64 = 60;

fn write_to_stdout(data: &[u8]) {
    unsafe {
        syscall(SYS_write, 1, data.as_ptr(), data.len());
    }
}

fn exit(code: i32) {
    unsafe {
        syscall(SYS_exit, code);
    }
}
```

---

**Previous**: [17_Error_Handling.md](./17_Error_Handling.md)  
**Next**: [19_Package_Manager.md](./19_Package_Manager.md)

**Maintained by**: Vex Language Team</content>
<parameter name="filePath">/Users/mapletechnologies/Desktop/big_projects/vex_lang/Specifications/18_Raw_Pointers_and_FFI.md

# Package Manager

**Version:** 0.1.2
**Last Updated:** November 2025

This document describes Vex's package manager (`vex-pm`), which provides dependency management, project initialization, and build coordination for Vex projects.

---

## Table of Contents

1. [Overview](#overview)
2. [Project Structure](#project-structure)
3. [Manifest File (vex.json)](#manifest-file-vexjson)
4. [Lock File (vex.lock)](#lock-file-vexlock)
5. [CLI Commands](#cli-commands)
6. [Dependency Resolution](#dependency-resolution)
7. [Platform-Specific Code](#platform-specific-code)
8. [Build Integration](#build-integration)

---

## Overview

Vex's package manager is fully integrated into the `vex` command-line tool. It follows a decentralized, Git-based approach inspired by Go modules and Cargo.

### Key Features

- **Decentralized**: No central package registry - uses Git repositories directly
- **Fast**: Parallel downloads with global caching
- **Secure**: SHA-256 checksums and lock files
- **Platform-aware**: Automatic selection of platform-specific implementations
- **Simple**: Single tool for compilation, running, and package management

### Philosophy

_"Cargo'nun gücü, Go Mod'un sadeliği, Zig'in platform awareness'ı"_

---

## Project Structure

Vex projects follow a conventional directory structure:

```
my-project/
├── vex.json          # Project manifest
├── vex.lock          # Lock file (generated)
├── native/           # Native C Codes
├── src/
│   ├── lib.vx        # Library entry point
│   ├── main.vx       # Executable entry point (optional)
│   └── mod.vx        # Module declarations
├── tests/            # Test files
├── examples/         # Example code
└── vex-builds/       # Build artifacts (generated)
```

### Entry Points

- **Library**: `src/lib.vx` (default main entry)
- **Module**: `src/mod.vx` (alternative if no lib.vx)
- **Executable**: `src/main.vx` or specified in `vex.json`
- **Custom**: Configurable via `main` field in manifest

**Import Resolution**:

```vex
// Package name import uses "main" field from vex.json
import { abs } from "math";
// → Resolves to: vex-libs/std/math/src/lib.vx (from vex.json)

// Direct file import bypasses vex.json
import { sin } from "math/native.vxc";
// → Resolves to: vex-libs/std/math/src/native.vxc

// Relative imports (within module files)
import { helper } from "./utils.vx";
// → Resolves relative to current file
```

**Priority Order**:

1. `vex.json` → `main` field value
2. `src/lib.vx` (if exists)
3. `src/mod.vx` (if exists)
4. Error: No entry point found

---

## Manifest File (vex.json)

The `vex.json` file describes your project and its dependencies:

```json
{
  "name": "my-project",
  "version": "1.0.0",
  "description": "A Vex project",
  "authors": ["Your Name <you@example.com>"],
  "license": "MIT",
  "repository": "https://github.com/user/my-project",

  "dependencies": {
    "local-lib": "v1.2.0"
  },

  "main": "src/lib.vx",

  "bin": {
    "my-app": "src/main.vx",
    "cli-tool": "src/cli.vx"
  },

  "testing": {
    "dir": "tests",
    "pattern": "*.test.vx",
    "timeout": 30,
    "parallel": true
  },

  "targets": {
    "default": "x64",
    "supported": ["x64", "arm64", "wasm"]
  },

  "profiles": {
    "development": {
      "optimizationLevel": 0,
      "debugSymbols": true
    },
    "production": {
      "optimizationLevel": 3,
      "debugSymbols": false
    }
  },

  "native": {
    "sources": ["native/src/helper.c"],
    "libraries": ["ssl", "crypto"],
    "search_paths": ["/usr/local/lib"],
    "static_libs": ["./vendor/libmylib.a"],
    "cflags": ["-O3", "-Wall", "-fPIC"],
    "include_dirs": ["vendor/include", "../../../vex-runtime/c"]
  },

  "vex": {
    "borrowChecker": "strict"
  }
}
```

### Dependency Specification

**Current Status (v0.1.2)**: Local dependencies only. Remote Git repositories planned for future releases.

```json
{
  "dependencies": {
    "local-lib": "v1.2.3", // Exact version (local)
    "math": "v0.2.0" // Stdlib module
  }
}
```

**Future Support** (planned):

- `"^1.2.0"` - Compatible with 1.x (semantic versioning)
- `"1.0.0..2.0.0"` - Version range
- `"*"` - Latest version
- Git repositories: `"github.com/user/lib": "v1.2.0"`

### Native Dependencies

For C/C++ integration:

```json
{
  "native": {
    "sources": ["native/src/implementation.c"],
    "libraries": ["ssl", "zlib"],
    "search_paths": ["/usr/local/lib", "/opt/homebrew/lib"],
    "static_libs": ["./vendor/libcustom.a"],
    "cflags": ["-O3", "-Wall", "-fPIC", "-std=c11"],
    "include_dirs": ["path/to/headers", "../../../vex-runtime/c"]
  }
}
```

**Field Descriptions**:

- `sources`: C/C++ files to compile
- `libraries`: System libraries to link (e.g., `m`, `ssl`)
- `search_paths`: Library search directories
- `static_libs`: Static library files (.a)
- `cflags`: C compiler flags
- `include_dirs`: Header include directories

### Complete Field Reference

| Field          | Type   | Required | Description                         |
| -------------- | ------ | -------- | ----------------------------------- |
| `name`         | string | ✅       | Package name                        |
| `version`      | string | ✅       | Semantic version (e.g., "1.0.0")    |
| `description`  | string | ❌       | Package description                 |
| `authors`      | array  | ❌       | Author names and emails             |
| `license`      | string | ❌       | License identifier (e.g., "MIT")    |
| `repository`   | string | ❌       | Repository URL                      |
| `dependencies` | object | ❌       | Package dependencies                |
| `main`         | string | ❌       | Entry point (default: `src/lib.vx`) |
| `bin`          | object | ❌       | Binary targets                      |
| `testing`      | object | ❌       | Test configuration                  |
| `targets`      | object | ❌       | Platform configuration              |
| `profiles`     | object | ❌       | Build profiles                      |
| `native`       | object | ❌       | C/C++ integration config            |
| `vex`          | object | ❌       | Vex-specific settings               |

**Targets Structure**:

```json
{
  "targets": {
    "default": "x64",
    "supported": ["x64", "arm64", "wasm", "wasi"]
  }
}
```

**Profiles Structure**:

```json
{
  "profiles": {
    "development": {
      "optimizationLevel": 0,
      "debugSymbols": true,
      "memProfiling": false,
      "cpuProfiling": false
    },
    "production": {
      "optimizationLevel": 3,
      "debugSymbols": false
    }
  }
}
```

**Vex Config Structure**:

```json
{
  "vex": {
    "borrowChecker": "strict" // or "permissive"
  }
}
```

**Testing Config Structure**:

```json
{
  "testing": {
    "dir": "tests", // Test directory (informational)
    "pattern": "**/*.test.vx", // Glob pattern from project root (default)
    "timeout": 30, // Test timeout in seconds (optional)
    "parallel": true // Run tests in parallel (default: true)
  }
}
```

**Test File Naming Convention**:

- Test files MUST follow the `*.test.vx` pattern
- Examples:
  - `basic.test.vx`
  - `integration.test.vx`
  - `unit.test.vx`
- **Pattern Search**: Uses glob from project root (`**/*.test.vx`)
- Custom patterns can be specified via `testing.pattern`

---

## Lock File (vex.lock)

The `vex.lock` file ensures reproducible builds by locking exact dependency versions:

```json
{
  "version": "1.0",
  "packages": {
    "github.com/user/math-lib": {
      "version": "v1.2.3",
      "checksum": "abc123...",
      "dependencies": {}
    },
    "github.com/user/http-client": {
      "version": "v2.1.0",
      "checksum": "def456...",
      "dependencies": {
        "github.com/user/math-lib": "v1.2.3"
      }
    }
  }
}
```

Lock files are automatically generated and should be committed to version control.

---

## CLI Commands

### Project Initialization

```bash
# Create new project
vex new my-project

# Initialize in existing directory
vex init
```

### Dependency Management

**Current Status (v0.1.2)**: Local dependencies only. CLI commands for remote packages planned.

```bash
# Manual dependency management (edit vex.json)
# Add to dependencies section:
# "local-lib": "v1.2.0"

# Planned commands (future):
# vex add github.com/user/math-lib@v1.2.0
# vex remove github.com/user/math-lib
# vex update
# vex list

# Currently available:
vex clean  # Clean build cache
```

### Building and Running

```bash
# Build project
vex build

# Build with specific profile
vex build --release

# Run executable
vex run

# Run specific binary
vex run --bin my-app

# Run example
vex run --example demo

# CI build (locked dependencies)
vex build --locked
```

### Development

```bash
# Check project
vex check

# Format code
vex format

# Run tests (discovers *.test.vx files)
vex test

# Run specific test file
vex test tests/basic.test.vx

# Run tests with timeout
vex test --timeout 60

# Run tests sequentially (no parallel)
vex test --no-parallel

# Generate documentation
vex doc
```

---

## Dependency Resolution

Vex uses a Go-style flat dependency resolution:

### Algorithm

1. **Collect**: Gather all direct and transitive dependencies
2. **Resolve**: Find compatible versions for all packages
3. **Download**: Fetch packages in parallel to global cache
4. **Verify**: Check SHA-256 checksums
5. **Link**: Generate build configuration

### Version Selection

- **Semantic versioning**: `^1.2.0` allows compatible updates within major version
- **Exact pinning**: `v1.2.3` locks to specific version
- **Range specification**: `1.0.0..2.0.0` for custom ranges
- **Latest**: `*` or no version specifier

### Conflict Resolution

When version conflicts occur, Vex follows these rules:

1. Prefer already resolved versions
2. Choose highest compatible version
3. Fail with clear error message if impossible

---

## Platform-Specific Code

Vex supports platform-specific implementations using file suffixes:

### Priority Order

1. `{file}.testing.vx` (when running tests)
2. `{file}.{os}.{arch}.vx` (most specific)
3. `{file}.{arch}.vx` (architecture-specific)
4. `{file}.{os}.vx` (OS-specific)
5. `{file}.vx` (generic fallback)

### Supported Platforms

**Architectures:**

- `x64` - x86-64
- `arm64` - ARM64/AArch64
- `wasm` - WebAssembly
- `wasi` - WASI
- `riscv64` - RISC-V 64-bit

**Operating Systems:**

- `linux` - Linux
- `macos` - macOS
- `windows` - Windows
- `freebsd` - FreeBSD
- `openbsd` - OpenBSD

### Example

```
src/
├── crypto.vx           # Generic implementation
├── crypto.x64.vx       # x86-64 with SIMD
├── crypto.arm64.vx     # ARM64 with NEON
├── crypto.wasm.vx      # WebAssembly version
└── crypto.testing.vx   # Test mocks
```

---

## Testing

### Test Discovery

Vex automatically discovers test files using the pattern specified in `vex.json`.

**Default Pattern**: `*.test.vx`

**Directory Structure**:

```
my-project/
├── vex.json
├── src/
│   └── lib.vx
└── tests/
    ├── basic.test.vx
    ├── integration.test.vx
    └── unit.test.vx
```

### Test File Naming

**Required Pattern**: Files must end with `.test.vx`

**Examples**:

```
✅ basic.test.vx
✅ user_auth.test.vx
✅ api_integration.test.vx
❌ basic_test.vx       (missing .test before .vx)
❌ test_basic.vx       (wrong position)
❌ basic.vx            (missing .test)
```

### Test Configuration

```json
{
  "testing": {
    "dir": "tests",
    "pattern": "*.test.vx",
    "timeout": 30,
    "parallel": true
  }
}
```

**Fields**:

- `dir`: Directory containing test files (default: `"tests"`)
- `pattern`: Glob pattern for test files (default: `"*.test.vx"`)
- `timeout`: Maximum execution time per test in seconds (optional)
- `parallel`: Run tests in parallel (default: `true`)

### Running Tests

**Discover and run all tests**:

```bash
vex test
```

**Run specific test file**:

```bash
vex test tests/basic.test.vx
```

**Run with custom timeout**:

```bash
vex test --timeout 60
```

**Run sequentially**:

```bash
vex test --no-parallel
```

### Test Organization

**Unit Tests**: Test individual functions/modules

```
tests/
├── math.test.vx
├── string.test.vx
└── utils.test.vx
```

**Integration Tests**: Test module interactions

```
tests/
├── api_integration.test.vx
├── db_integration.test.vx
└── workflow.test.vx
```

**Mixed Structure**:

```
tests/
├── unit/
│   ├── math.test.vx
│   └── string.test.vx
└── integration/
    ├── api.test.vx
    └── db.test.vx
```

### Platform-Specific Tests

Tests can also use platform-specific files:

```
tests/
├── io.test.vx              # Generic tests
├── io.test.macos.vx        # macOS-specific tests
└── io.test.linux.vx        # Linux-specific tests
```

**Priority**:

1. `test.{os}.{arch}.vx`
2. `test.{arch}.vx`
3. `test.{os}.vx`
4. `test.vx`

---

## Build Integration

### Automatic Resolution

When building, Vex automatically:

1. **Reads** `vex.json` and `vex.lock`
2. **Downloads** dependencies to `~/.vex/cache/`
3. **Verifies** checksums
4. **Generates** build configuration
5. **Compiles** with proper include paths and linking

### Cache Location

- **Global cache**: `~/.vex/cache/`
- **Project cache**: `vex-builds/`
- **Lock file**: `vex.lock`

### Build Profiles

Configure optimization levels and flags:

```json
{
  "profiles": {
    "debug": {
      "opt-level": 0,
      "debug": true
    },
    "release": {
      "opt-level": 3,
      "lto": true
    }
  }
}
```

### Native Library Integration

Link with system libraries:

```json
{
  "native": {
    "libs": ["ssl", "crypto", "z"],
    "include": ["/usr/local/include"],
    "flags": ["-O3", "-march=native"]
  }
}
```

---

**Previous**: [18_Raw_Pointers_and_FFI.md](./18_Raw_Pointers_and_FFI.md)

**Maintained by**: Vex Language Team</content>
<parameter name="filePath">/Users/mapletechnologies/Desktop/big_projects/vex_lang/Specifications/19_Package_Manager.md

# Policy System

**Version:** 0.1.2
**Last Updated:** November 2025

This document describes Vex's policy system, which provides metadata annotations for structs and fields, enabling features like serialization, validation, and code generation.

---

## Table of Contents

1. [Overview](#overview)
2. [Policy Declarations](#policy-declarations)
3. [Policy Composition](#policy-composition)
4. [Struct Application](#struct-application)
5. [Metadata Resolution](#metadata-resolution)
6. [Inline Metadata](#inline-metadata)
7. [Use Cases](#use-cases)
8. [Examples](#examples)

---

## Overview

The policy system allows you to define reusable metadata templates that can be applied to structs and their fields. This enables:

- **Serialization**: JSON, XML, database mappings
- **Validation**: Field constraints and rules
- **Code Generation**: Automatic CRUD operations, API endpoints
- **Documentation**: Field descriptions and examples
- **UI Generation**: Form layouts and input types

### Key Concepts

- **Policy**: A named collection of field metadata
- **Metadata**: Key-value annotations using backtick syntax
- **Composition**: Policies can inherit from parent policies
- **Application**: Structs apply policies using `with` clause
- **Resolution**: Metadata is merged with precedence rules

---

## Policy Declarations

### Basic Policy Syntax

```vex
policy PolicyName {
    field_name `key:"value"`,
    another_field `type:"string" required:"true"`
}
```

**Components:**

- `policy` keyword
- Policy name (identifier)
- Field declarations with metadata in backticks
- Comma-separated fields

### Metadata Syntax

Metadata uses a simple key-value format within backticks:

```vex
`key1:"value1" key2:"value2" key3:"true"`
```

**Rules:**

- Keys and values are strings
- Multiple key-value pairs separated by spaces
- Values can contain special characters
- No nested structures (flat key-value pairs)

### Example Policies

```vex
policy APIModel {
    id `json:"id" db:"id" required:"true"`,
    name `json:"name" db:"name" max_length:"100"`,
    email `json:"email" db:"email" format:"email"`,
    created_at `json:"created_at" db:"created_at" type:"datetime"`
}

policy ValidationRules {
    age `min:"0" max:"150"`,
    salary `min:"0"`,
    phone `pattern:"^\\+?[1-9]\\d{1,14}$"`
}
```

---

## Policy Composition

### Parent Policies

Policies can inherit metadata from parent policies:

```vex
policy BaseModel {
    id `db:"id" indexed:"true"`,
    created_at `db:"created_at"`,
    updated_at `db:"updated_at"`
}

policy APIModel with BaseModel {
    id `json:"id"`,  // Overrides db:"id" with json:"id"
    name `json:"name"`,
    email `json:"email"`
}
```

**Inheritance Rules:**

1. Child policies inherit all fields from parent policies
2. Child field metadata overrides parent metadata for the same field
3. Multiple inheritance is supported with comma separation

### Multiple Inheritance

```vex
policy Timestamped {
    created_at `db:"created_at"`,
    updated_at `db:"updated_at"`
}

policy SoftDelete {
    deleted_at `db:"deleted_at"`,
    is_deleted `db:"is_deleted" default:"false"`
}

policy FullModel with Timestamped, SoftDelete {
    id `json:"id" db:"id"`,
    name `json:"name" db:"name"`
}
```

**Resolution Order:**

1. First parent policies processed left-to-right
2. Child policy fields override parent fields
3. Later parents can override earlier parents

---

## Struct Application

### Basic Application

Apply policies to structs using the `with` clause:

```vex
struct User with APIModel {
    id: i32,
    name: string,
    email: string,
    created_at: i64,
}

struct Product with APIModel, ValidationRules {
    id: i32,
    name: string,
    price: f64,
    category: string,
}
```

**Effects:**

- All policy fields must exist in the struct
- Metadata is applied to matching fields
- Struct gains the combined metadata from all policies

### Field Requirements

When a policy is applied, the struct must contain all fields defined in the policy:

```vex
policy RequiredFields {
    id `required:"true"`,
    name `required:"true"`
}

// ✅ Valid - has both required fields
struct User with RequiredFields {
    id: i32,
    name: string,
    email: string,  // Extra fields allowed
}

// ❌ Invalid - missing 'name' field
struct Incomplete with RequiredFields {
    id: i32,
    // name field missing
}
```

---

## Metadata Resolution

### Merge Order

When multiple sources define metadata for the same field, they are merged with this precedence:

1. **Inline metadata** (highest precedence)
2. **Child policy metadata**
3. **Parent policy metadata** (lowest precedence)

### Example Resolution

```vex
policy Base {
    id `db:"id"`,
    name `db:"name"`
}

policy API with Base {
    id `json:"id"`,     // Overrides db:"id"
    name `json:"name"`  // Overrides db:"name"
}

struct User with API {
    id: i32 `primary_key:"true"`,  // Overrides json:"id"
    name: string,                  // Uses json:"name"
}
```

**Final metadata for `id` field:**

- `primary_key:"true"` (from inline)
- `json:"id"` (from API policy, but overridden by inline)

**Final metadata for `name` field:**

- `json:"name"` (from API policy)

---

## Inline Metadata

### Field-Level Metadata

You can add metadata directly to struct fields:

```vex
struct User with APIModel {
    id: i32 `primary_key:"true" auto_increment:"true"`,
    name: string `max_length:"100"`,
    email: string `unique:"true"`,
    created_at: i64 `default:"now()"`,
}
```

**Use Cases:**

- Field-specific overrides
- Additional constraints not in policies
- Database-specific annotations
- Validation rules

### Metadata Combination

Inline metadata is merged with policy metadata:

```vex
policy APIModel {
    id `json:"id"`,
    name `json:"name"`
}

struct User with APIModel {
    id: i32 `db:"user_id" primary_key:"true"`,  // Combines with json:"id"
    name: string `max_length:"50"`,             // Adds to json:"name"
}
```

**Result for `id`:**

- `json:"id"` (from policy)
- `db:"user_id"` (from inline)
- `primary_key:"true"` (from inline)

---

## Use Cases

### 1. API Serialization

```vex
policy JSONAPI {
    id `json:"id"`,
    name `json:"name"`,
    email `json:"email"`,
    created_at `json:"created_at"`,
    updated_at `json:"updated_at"`
}

struct User with JSONAPI {
    id: i32,
    name: string,
    email: string,
    created_at: i64,
    updated_at: i64,
}
```

### 2. Database Mapping

```vex
policy DatabaseModel {
    id `db:"id" type:"INTEGER" primary_key:"true"`,
    name `db:"name" type:"VARCHAR(100)" not_null:"true"`,
    email `db:"email" type:"VARCHAR(255)" unique:"true"`,
    created_at `db:"created_at" type:"TIMESTAMP" default:"CURRENT_TIMESTAMP"`
}

struct User with DatabaseModel {
    id: i32,
    name: string,
    email: string,
    created_at: i64,
}
```

### 3. Validation Rules

```vex
policy Validation {
    age `min:"0" max:"150"`,
    email `format:"email" required:"true"`,
    phone `pattern:"^\\+?[1-9]\\d{1,14}$"`,
    salary `min:"0"`
}

struct Employee with Validation {
    name: string,
    age: i32,
    email: string,
    phone: string,
    salary: f64,
}
```

### 4. UI Generation

```vex
policy FormUI {
    name `ui:"text" label:"Full Name" required:"true"`,
    email `ui:"email" label:"Email Address" required:"true"`,
    age `ui:"number" label:"Age" min:"0" max:"150"`,
    department `ui:"select" label:"Department" options:"Engineering,Sales,Marketing"`
}

struct Employee with FormUI {
    name: string,
    email: string,
    age: i32,
    department: string,
}
```

### 5. Multi-Format Support

```vex
policy MultiFormat {
    id `json:"id" xml:"id" db:"id"`,
    name `json:"name" xml:"name" db:"name"`,
    data `json:"data" xml:"data" db:"data" type:"JSONB"`
}

struct Document with MultiFormat {
    id: i32,
    name: string,
    data: string,  // JSON string
}
```

---

## Examples

### Complete API Model

```vex
// Base model with common fields
policy BaseModel {
    id `json:"id" db:"id" type:"INTEGER" primary_key:"true"`,
    created_at `json:"created_at" db:"created_at" type:"TIMESTAMP"`,
    updated_at `json:"updated_at" db:"updated_at" type:"TIMESTAMP"`
}

// API-specific extensions
policy APIModel with BaseModel {
    id `xml:"id"`,  // Add XML support
    name `json:"name" xml:"name" db:"name" required:"true"`,
    email `json:"email" xml:"email" db:"email" format:"email"`
}

// Validation rules
policy UserValidation {
    name `min_length:"2" max_length:"100"`,
    email `format:"email" unique:"true"`,
    age `min:"13" max:"150"`
}

// Complete user model
struct User with APIModel, UserValidation {
    id: i32,
    name: string,
    email: string,
    age: i32 `json:"age" db:"age"`,  // Additional inline metadata
    created_at: i64,
    updated_at: i64,
}

fn main(): i32 {
    let user = User {
        id: 1,
        name: "Alice Johnson",
        email: "alice@example.com",
        age: 30,
        created_at: 1699000000,
        updated_at: 1699000000,
    };

    // At compile time, the metadata is available for:
    // - JSON serialization/deserialization
    // - Database ORM operations
    // - API documentation generation
    // - Form validation
    // - UI component generation

    return 0;
}
```

### Policy Inheritance Chain

```vex
// Foundation policies
policy Identifiable {
    id `type:"uuid" required:"true"`
}

policy Timestamped with Identifiable {
    created_at `type:"datetime"`,
    updated_at `type:"datetime"`
}

policy SoftDelete with Timestamped {
    deleted_at `type:"datetime"`,
    is_deleted `type:"boolean" default:"false"`
}

// Domain policies
policy APIModel with SoftDelete {
    id `json:"id"`,
    name `json:"name"`,
    description `json:"description"`
}

policy DatabaseModel with SoftDelete {
    id `db:"id" primary_key:"true"`,
    name `db:"name" not_null:"true"`,
    description `db:"description"`
}

// Concrete usage
struct Product with APIModel, DatabaseModel {
    id: string,
    name: string,
    description: string,
    created_at: i64,
    updated_at: i64,
    deleted_at: i64,
    is_deleted: bool,
}
```

### Metadata-Driven Code Generation

```vex
policy CRUD {
    id `primary_key:"true" auto_increment:"true"`,
    name `unique:"true" index:"name_idx"`,
    created_at `default:"CURRENT_TIMESTAMP"`,
    updated_at `on_update:"CURRENT_TIMESTAMP"`
}

policy APIEndpoints {
    id `route:"/api/{id}" methods:"GET,PUT,DELETE"`,
    name `route:"/api/search" methods:"GET" query:"name"`
}

struct User with CRUD, APIEndpoints {
    id: i32,
    name: string,
    email: string,
    created_at: i64,
    updated_at: i64,
}

// This could generate:
// 1. Database table creation
// 2. CRUD repository methods
// 3. REST API endpoints
// 4. OpenAPI documentation
// 5. Frontend forms and components
```

---

**Previous**: [19_Package_Manager.md](./19_Package_Manager.md)

**Maintained by**: Vex Language Team</content>
<parameter name="filePath">/Users/mapletechnologies/Desktop/big_projects/vex_lang/Specifications/20_Policy_System.md

# Mutability and Pointers

**Version:** 0.1.2
**Last Updated:** November 2025

This document provides a comprehensive guide to Vex's mutability system and pointer types, including raw pointers, references, and memory safety guarantees.

---

## Table of Contents

1. [Mutability System](#mutability-system)
2. [References](#references)
3. [Raw Pointers](#raw-pointers)
4. [Pointer Arithmetic](#pointer-arithmetic)
5. [Memory Safety](#memory-safety)
6. [FFI Integration](#ffi-integration)
7. [Common Patterns](#common-patterns)

---

## Mutability System

### Variable Mutability

Vex uses explicit mutability markers:

```vex
let x = 42;        // Immutable (default)
let! y = 42;       // Mutable (explicit ! suffix)

x = 100;           // ERROR: Cannot assign to immutable
y = 100;           // OK: y is mutable
```

### Field Mutability

Struct fields inherit mutability from their containing variable:

```vex
struct Point {
    x: i32,
    y: i32,
}

let p = Point { x: 1, y: 2 };
// p.x = 10;        // ERROR: p is immutable

let! p2 = Point { x: 1, y: 2 };
p2.x = 10;         // OK: p2 is mutable
```

### Method Mutability

Vex uses a hybrid model for method mutability.

#### 1. Inline Methods (in `struct` or `contract`)

- **Declaration**: `fn method_name()!`
- **Behavior**: The method can mutate `self`.
- **Call**: `object.method_name()` (no `!` at call site). The compiler ensures this is only called on a mutable (`let!`) variable.

```vex
struct Counter {
    value: i32,
    fn increment()! {
        self.value = self.value + 1;
    }
}

let! c = Counter { value: 0 };
c.increment(); // OK
```

#### 2. External Methods (Golang-style)

- **Declaration**: `fn (self: &MyType!) method_name()`
- **Behavior**: The method can mutate `self`.
- **Call**: `object.method_name()` (no `!` at call site).

```vex
struct Point { x: i32, y: i32 }

fn (p: &Point) get_x(): i32 {
    return p.x;    // Immutable access
}

fn (p: &Point!) set_x(x: i32) {
    p.x = x;       // Mutable access
}

let! p = Point { x: 1, y: 2 };
let x = p.get_x();     // OK: immutable method
p.set_x(42);           // OK: mutable method
```

---

## References

### Immutable References

```vex
&T                    // Immutable reference to T
let x = 42;
let ref_x: &i32 = &x;  // Reference to x
println("{}", ref_x);  // Dereference with *
```

### Mutable References

```vex
&T!                   // Mutable reference to T
let! x = 42;
let ref_x: &i32! = &x; // Mutable reference
*ref_x = 100;         // Modify through reference
```

### Reference Rules

1. **Single Writer**: Only one mutable reference at a time
2. **No Aliasing**: Mutable references cannot coexist with other references
3. **Lifetime Bounds**: References cannot outlive their referent

```vex
let! x = 42;
let r1: &i32! = &x;
// let r2: &i32! = &x;  // ERROR: Multiple mutable borrows
// let r3: &i32 = &x;   // ERROR: Mutable and immutable borrow conflict
```

### Deref Operator

```vex
let x = 42;
let r: &i32 = &x;
println("{}", *r);     // Dereference: prints 42

let! y = 42;
let mr: &i32! = &y;
*mr = 100;             // Modify through mutable reference
```

---

## Raw Pointers

### Raw Pointer Types

```vex
*T                    // Raw immutable pointer
*T!                   // Raw mutable pointer
```

### Creation

```vex
let x = 42;
let ptr: *i32 = &x as *i32;        // Cast reference to raw pointer
let mut_ptr: *i32! = &x as *i32!;  // Cast to mutable raw pointer
```

### Dereferencing

```vex
unsafe {
    let value = *ptr;           // Dereference immutable pointer
    *mut_ptr = 100;             // Modify through mutable pointer
}
```

### Null Pointers

```vex
let null_ptr: *i32 = 0 as *i32;     // Null pointer
let is_null = ptr == 0 as *i32;     // Check for null
```

---

## Pointer Arithmetic

### Basic Arithmetic

```vex
let arr = [1, 2, 3, 4, 5];
let ptr: *i32 = &arr[0] as *i32;

unsafe {
    let second = ptr + 1;       // Points to arr[1]
    let third = ptr + 2;        // Points to arr[2]

    println("{}", *second);     // Prints 2
    println("{}", *third);      // Prints 3
}
```

### Array Iteration

```vex
fn sum_array(arr: *i32, len: usize): i32 {
    let mut sum = 0;
    let mut ptr = arr;

    for i in 0..len {
        unsafe {
            sum += *ptr;
            ptr = ptr + 1;
        }
    }

    return sum;
}
```

### Pointer Subtraction

```vex
let arr = [10, 20, 30, 40];
let start: *i32 = &arr[0] as *i32;
let end: *i32 = &arr[3] as *i32;

let distance = end - start;  // distance = 3
```

---

## Memory Safety

### Borrow Checker Integration

Raw pointers bypass borrow checker but require `unsafe` blocks:

```vex
let! x = 42;
let ref_x: &i32! = &x;

// Safe: borrow checker enforced
*ref_x = 100;

// Unsafe: raw pointer bypasses checks
let raw: *i32! = ref_x as *i32!;
unsafe {
    *raw = 200;  // No borrow checker validation
}
```

### Lifetime Safety

References have lifetime bounds, raw pointers do not:

```vex
fn safe_ref<'a>(data: &'a Vec<i32>): &'a i32 {
    return &data[0];  // Lifetime 'a enforced
}

fn unsafe_ptr(data: *Vec<i32>): *i32 {
    unsafe {
        return &(*data)[0] as *i32;  // No lifetime tracking
    }
}
```

### Common Unsafe Patterns

```vex
// Iterator invalidation
unsafe {
    let mut vec = Vec.new<i32>();
    vec.push(1);
    vec.push(2);

    let ptr = &vec[0] as *i32;
    vec.push(3);  // May reallocate, invalidating ptr

    // *ptr now dangling! Undefined behavior
}

// Use-after-free
unsafe {
    let ptr: *i32;
    {
        let x = 42;
        ptr = &x as *i32;
    }  // x dropped here

    // *ptr is now dangling! Undefined behavior
}
```

---

## FFI Integration

### C Interoperability

Raw pointers are essential for C FFI:

```vex
extern "C" {
    fn malloc(size: usize): *u8;
    fn free(ptr: *u8);
    fn memcpy(dest: *u8, src: *u8, n: usize);
}

fn allocate_buffer(size: usize): *u8 {
    unsafe {
        return malloc(size);
    }
}

fn deallocate_buffer(ptr: *u8) {
    unsafe {
        free(ptr);
    }
}
```

### Struct Layout Compatibility

C-compatible struct layout is automatic in Vex (no attributes needed):

```vex
struct CPoint {
    x: f32,
    y: f32,
}

extern "C" {
    fn create_point(x: f32, y: f32): *CPoint;
    fn get_x(point: *CPoint): f32;
}

fn use_c_library() {
    unsafe {
        let point = create_point(1.0, 2.0);
        let x = get_x(point);
        println("x: {}", x);
        // Remember to deallocate if required by C library
    }
}
```

````

### Struct Layout Compatibility

```vex
#[repr(C)]
struct CPoint {
    x: f32,
    y: f32,
}

extern "C" {
    fn create_point(x: f32, y: f32): *CPoint;
    fn get_x(point: *CPoint): f32;
}

fn use_c_library() {
    unsafe {
        let point = create_point(1.0, 2.0);
        let x = get_x(point);
        println("x: {}", x);
        // Remember to deallocate if required by C library
    }
}
````

---

## Common Patterns

### Safe Wrapper Types

```vex
struct SafePtr<T> {
    ptr: *T,
    valid: bool,
}

impl<T> SafePtr<T> {
    fn new(value: T): SafePtr<T> {
        unsafe {
            let ptr = malloc(sizeof<T>()) as *T;
            *ptr = value;
            return SafePtr { ptr: ptr, valid: true };
        }
    }

    fn get(self: &SafePtr<T>): &T {
        assert(self.valid, "Pointer is invalid");
        unsafe {
            return &*self.ptr;
        }
    }

    fn drop(self: &SafePtr!) {
        if self.valid {
            unsafe {
                free(self.ptr as *u8);
            }
            self.valid = false;
        }
    }
}
```

### Iterator Implementation

```vex
struct ArrayIter<T> {
    ptr: *T,
    end: *T,
}

impl<T> ArrayIter<T> {
    fn new(arr: &Vec<T>): ArrayIter<T> {
        unsafe {
            let start = &arr[0] as *T;
            let end = start + arr.len();
            return ArrayIter { ptr: start, end: end };
        }
    }

    fn next(self: &ArrayIter!): Option<&T> {
        if self.ptr >= self.end {
            return Option.None;
        }

        unsafe {
            let result = &*self.ptr;
            self.ptr = self.ptr + 1;
            return Option.Some(result);
        }
    }
}
```

### Manual Memory Management

```vex
fn manual_vec_demo() {
    unsafe {
        // Allocate space for 10 i32s
        let ptr = malloc(10 * sizeof<i32>()) as *i32!;

        // Initialize
        for i in 0..10 {
            *(ptr + i) = i as i32 * 2;
        }

        // Use
        for i in 0..10 {
            println("{}", *(ptr + i));
        }

        // Deallocate
        free(ptr as *u8);
    }
}
```

### Performance-Critical Code

```vex
fn fast_memcpy(dest: *u8!, src: *u8, n: usize) {
    unsafe {
        let mut d = dest;
        let mut s = src;

        // Copy in chunks of 8 bytes when possible
        while n >= 8 {
            *(d as *u64!) = *(s as *u64);
            d = d + 8;
            s = s + 8;
            n -= 8;
        }

        // Copy remaining bytes
        while n > 0 {
            *d = *s;
            d = d + 1;
            s = s + 1;
            n -= 1;
        }
    }
}
```

---

## Best Practices

### When to Use References

- **Default choice** for function parameters
- **Safe** and enforced by borrow checker
- **Zero-cost** abstractions

### When to Use Raw Pointers

- **FFI boundaries** with C libraries
- **Performance-critical** code requiring manual control
- **Unsafe operations** that bypass borrow checker
- **Low-level system programming**

### Safety Guidelines

1. **Minimize unsafe blocks** - Keep them as small as possible
2. **Validate pointers** - Check for null before dereferencing
3. **Respect lifetimes** - Don't create dangling pointers
4. **Use safe abstractions** - Wrap unsafe code in safe interfaces
5. **Test thoroughly** - Unsafe code needs extensive testing

### Common Pitfalls

```vex
// ❌ Dangling pointer
fn bad() {
    let ptr: *i32;
    {
        let x = 42;
        ptr = &x as *i32;
    }
    // ptr now dangles!
}

// ✅ Safe alternative
fn good() -> i32 {
    let x = 42;
    return x;  // Value moved out
}
```

---

**Previous**: [20_Policy_System.md](./20_Policy_System.md)  
**Next**: [22_Advanced_Topics.md](./22_Advanced_Topics.md) (planned)

**Maintained by**: Vex Language Team

# Intrinsics and Low-Level Operations

**Version:** 0.1.2
**Last Updated:** November 9, 2025

This document describes Vex's support for low-level operations, compiler intrinsics, and platform-specific features.

---

## Table of Contents

1. [Bit Manipulation Intrinsics](#bit-manipulation-intrinsics)
2. [CPU Feature Detection](#cpu-feature-detection)
3. [Memory Intrinsics](#memory-intrinsics)
4. [SIMD Intrinsics](#simd-intrinsics)
5. [Platform-Specific Features](#platform-specific-features)

---

## Bit Manipulation Intrinsics

Vex provides LLVM bit manipulation intrinsics for high-performance low-level operations.

### Available Intrinsics

#### Count Leading Zeros: `ctlz(x)`

Counts the number of leading zero bits in an integer.

```vex
let x: u32 = 0b00001111_00000000_00000000_00000000; // 0x0F000000
let zeros = ctlz(x); // Returns 4 (leading zeros before first 1)
```

**Supported Types**: `u8`, `u16`, `u32`, `u64`, `u128`

#### Count Trailing Zeros: `cttz(x)`

Counts the number of trailing zero bits in an integer.

```vex
let x: u32 = 0b11110000_00000000_00000000_00000000; // 0xF0000000
let zeros = cttz(x); // Returns 24 (trailing zeros)
```

**Supported Types**: `u8`, `u16`, `u32`, `u64`, `u128`

#### Population Count: `popcnt(x)`

Counts the number of set bits (1s) in an integer.

```vex
let x: u32 = 0b10110100_11100011_00001111_00000000; // 0xB4E30F00
let count = popcnt(x); // Returns 12 (number of 1 bits)
```

**Supported Types**: `u8`, `u16`, `u32`, `u64`, `u128`

#### Bit Reverse: `bitreverse(x)`

Reverses the bits in an integer.

```vex
let x: u8 = 0b11010011; // 0xD3
let reversed = bitreverse(x); // Returns 0b11001011 (0xCB)
```

**Supported Types**: `u8`, `u16`, `u32`, `u64`, `u128`

#### Bit Swap: `bswap(x)`

Swaps the bytes in an integer (endianness conversion).

```vex
let x: u32 = 0x12345678;
let swapped = bswap(x); // Returns 0x78563412
```

**Supported Types**: `u16`, `u32`, `u64`, `u128`

#### Overflow Checks

**Signed Addition Overflow**: `sadd_overflow(a, b)` → `(result, overflow)`

```vex
let (result, overflow) = sadd_overflow(i32::MAX, 1);
if overflow {
    // Handle overflow
}
```

**Signed Subtraction Overflow**: `ssub_overflow(a, b)` → `(result, overflow)`

**Signed Multiplication Overflow**: `smul_overflow(a, b)` → `(result, overflow)`

**Supported Types**: `i8`, `i16`, `i32`, `i64`, `i128`

### Memory Operations

#### Memory Allocation: `alloc(size)` → `*u8`

Allocates `size` bytes of memory.

```vex
unsafe {
    let ptr: *u8 = alloc(1024); // Allocate 1KB
    // Use ptr...
    free(ptr);
}
```

#### Memory Free: `free(ptr)`

Frees previously allocated memory.

#### Memory Realloc: `realloc(ptr, new_size)` → `*u8`

Resizes allocated memory block.

#### Memory Move: `memmove(dst, src, count)`

Copies `count` bytes from `src` to `dst`, handling overlapping regions.

```vex
unsafe {
    let buffer: *u8 = alloc(1024);
    // Move overlapping regions
    memmove(buffer.add(10), buffer, 100);
}
```

### Type Information

#### Size Of: `sizeof<T>()` → `usize`

Returns the size in bytes of type `T`.

```vex
let size = sizeof<i32>();     // Returns 4
let arr_size = sizeof<[i32; 10]>(); // Returns 40
```

#### Align Of: `alignof<T>()` → `usize`

Returns the alignment in bytes of type `T`.

```vex
let align = alignof<i64>();   // Returns 8
```

### Array Operations

#### Array Length: `array_len(arr)` → `usize`

Returns the length of an array.

```vex
let arr: [i32; 5] = [1, 2, 3, 4, 5];
let len = array_len(arr); // Returns 5
```

### UTF-8 Operations

#### UTF-8 Validation: `utf8_valid(str)` → `bool`

Checks if a string contains valid UTF-8.

```vex
let valid = utf8_valid("Hello, 世界!"); // Returns true
let invalid = utf8_valid(&[0xFF, 0xFF]); // Returns false
```

#### UTF-8 Character Count: `utf8_char_count(str)` → `usize`

Returns the number of Unicode characters in a UTF-8 string.

```vex
let count = utf8_char_count("Hello, 世界!"); // Returns 9 (not byte count)
```

#### UTF-8 Character At: `utf8_char_at(str, index)` → `u32`

Returns the Unicode codepoint at the specified character index.

```vex
let codepoint = utf8_char_at("Hello, 世界!", 7); // Returns '世' (19990)
```

### Launch Expression (Future)

**Syntax**: `launch func[grid_dims](args)`

Launches a function on parallel compute units (GPU, HPC clusters).

```vex
// Launch kernel on GPU
launch vector_add[1024, 1](a, b, result);

// Multi-dimensional grid
launch matrix_mul[32, 32](matrix_a, matrix_b, output);
```

**Status**: AST support exists, implementation pending.

**Note**: This is a planned feature for high-performance computing integration.

# Operator Overloading

**Version:** 0.2.0  
**Date:** November 12, 2025  
**Status:** Syntax Ready (Codegen Pending)

---

## 🎯 Core Principles

1. **Contract-based**: Operators MUST be declared in a contract first.
2. **`op` prefix mandatory**: All operator methods use `op+`, `op-`, `op*`, etc.
3. **Multiple overloads**: One contract can define multiple operator overloads with different parameter types.
4. **Explicit implementation**: No automatic derivation — the implementing type explicitly defines its operator methods.

---

## 📋 Overview

Operator overloading in Vex is implemented as a capability provided through contracts. Contracts declare `op` methods and implementing types (typically `struct`s) provide concrete methods to support operator syntax.

The operator names follow the form `op<token>`, for example `op+`, `op-`, `op==`, as method names in contract declarations and implementations.

The compiler resolves an operator expression `a + b` by:

1. Checking whether `a`'s type implements the contract that defines `op+`.
2. Looking up the overload that best matches the right-hand side argument (`b`), including builtin primitive rules and `Self` substitutions.
3. Compiling a direct call to the corresponding method (`a.op+(b)`).

---

## Syntax: Contract Declarations

```vex
contract Add {
    op+(rhs: Self): Self;                    // Same type add
    op+(rhs: i32): Self;                     // Overload with i32
    op+(rhs: f64): Self;                     // Overload with f64
}
```

Rules:

- Contract name describes the capability (`Add`, `Mul`, `Eq`).
- All operator methods must use `op` prefix.
- Overloads are allowed as distinct method signatures.
- Return type is explicit and independent per overload.

---

## Syntax: Implementation

```vex
struct Vec2 impl Add {
    x: f64,
    y: f64,
}

// Vec2 + Vec2
fn (self: Vec2) op+(other: Vec2): Vec2 {
    return Vec2 { x: self.x + other.x, y: self.y + other.y };
}

// Vec2 + scalar
fn (self: Vec2) op+(scalar: f64): Vec2 {
    return Vec2 { x: self.x + scalar, y: self.y + scalar };
}
```

Rules:

- Struct must declare `impl ContractName`.
- Implementations must match contract method signatures (including `!` for mutability when applicable).
- Multiple overloads are separate method definitions.

---

## Resolution & Semantics

- Operator expression `a + b` resolves to `a.op+(b)` if `a`'s type implements the contract with a matching signature.
- Matching uses compile-time types; no implicit conversions except for the builtin `extends` contract mapping for primitives.
- If no match is found on the left-hand type, the compiler attempts to check for `coerce` rules and builtin operators (primitives), and finally raises an error.

---

## Phase 1: Arithmetic Operators

Contracts and typical builtin extensions.

```vex
contract Add {
    op+(rhs: Self): Self;
}

contract Sub { op-(rhs: Self): Self; }
contract Mul { op*(rhs: Self): Self; }
contract Div { op/(rhs: Self): Self; }
contract Rem { op%(rhs: Self): Self; }

contract Neg { op-(): Self; }
```

Primitive types are declared to `extend` these contracts in the standard library (builtin extensions). The compiler emits the corresponding IR for primitive operations.

---

## Phase 1.5: Bitwise Operators

```vex
contract BitAnd { op&(rhs: Self): Self; }
contract BitOr  { op|(rhs: Self): Self; }
contract BitXor { op^(rhs: Self): Self; }
contract BitNot { op~(): Self; }
contract Shl    { op<<(rhs: i32): Self; }
contract Shr    { op>>(rhs: i32): Self; }
```

Builtins: integer types are `extends` these contracts.

---

## Phase 2: Comparison & Logical Operators

```vex
contract Eq  { op==(rhs: Self): bool; op!=(rhs: Self): bool; }
contract Ord { op<(rhs: Self): bool; op>(rhs: Self): bool; op<=(rhs: Self): bool; op>=(rhs: Self): bool; }
contract Not { op!(): bool; }
```

Rules and special behaviors:

- If `op==` is implemented and `op!=` is not present, the compiler auto-generates `op!=` as `!self.op==(rhs)`.
- `Ord` must implement all four methods (no auto-generation), maintaining consistent semantics.

---

## Phase 3: Compound Assignment & Index Operators

```vex
contract AddAssign { op+=(rhs: Self); }
contract SubAssign { op-=(rhs: Self); }
contract MulAssign { op*=(rhs: Self); }
contract DivAssign { op/=(rhs: Self); }
contract RemAssign { op%=(rhs: Self); }

contract Index    { type Output; op[](index: i32): Output; }
contract IndexMut { type Output; op[]=(index: i32, value: Output); }

// Bitwise assignment
contract BitAndAssign { op&=(rhs: Self); }
contract BitOrAssign  { op|=(rhs: Self); }
contract BitXorAssign { op^=(rhs: Self); }

contract ShlAssign { op<<=(rhs: i32); }
contract ShrAssign { op>>=(rhs: i32); }
```

- Compound assignment operators (`op+=`) typically mutate `self` and require `!` in implementations to indicate mutability for contracts where necessary.

---

## Mutability & Side Effects

- Operators that are mutation semantics (like `op+=`) should be defined to mutate `self` with signature including `!` where the contract or implementation requires a mutable receiver.
- The `!` syntax in method signatures indicates the method requires a mutable reference for `self`.

---

## Builtin Primitives & `extends`

- The standard library declares `extends` for primitive types and builtin contracts to provide default operator support. These are specified in `stdlib/core/builtin_contracts.vx` and related files.
- `extends` indicates a special compiler-provided mapping for primitives and is not the same as a user `impl` for user-defined types.

---

## Auto-Generation & Default Implementations

- The compiler may auto-generate comparisons (`op!=` from `op==`) when semantically correct.
- Default contract methods that have body can be used to offer generic operator compositions (e.g., `op!=` defaulting to `!op==`).

---

## Diagnostics & Errors

- Type mismatch in operator operands or return type mismatches result in compile errors.
- When multiple overloads are applicable, the compiler uses exact type match precedence, then consider generic matches and coercions.

---

## Examples

See `examples/` for several samples and tests:

- `examples/test_operator_overloading.vx`
- `examples/test_operator_unary_neg.vx`
- `examples/operator/05_all_operators.vx`
- `examples/PROPOSAL_operator_syntax.vx`

---

## Implementation Notes (Compiler)

- Parser: Operator method names are parsed as `op` token followed by punctuation (e.g., `+`, `-`, `==`).
- Resolution: Compiler looks up `impl` for contract, ensuring correct arity and parameter types.
- Codegen: For user-defined types, the compiler emits calls to the method implementation; for primitives, it emits optimized IR.

---

## Security & Safety Considerations

- Overloading assignment operators like `op=` is either forbidden or strongly discouraged due to move/copy semantics that can easily introduce surprising behavior.
- Implementations must be careful around mutability to avoid aliasing and data races in concurrent code.

---

## Future Work / Open Questions

- Cross-type operator inference rules and conversions.
- Automatic generation of `op!=` vs `op==` symmetries for compound or complex types.
- Partial ordering inference for `Ord`.

---

## Changelog

- v0.2.0: Specification draft consolidating operator overloading phases and builtin contracts.

---

> See also: `OPERATOR_OVERLOADING_SPEC.md` at repository root for the original in-depth draft and additional examples.
