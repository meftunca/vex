# Vex Language - Introduction and Overview

**Version:** 0.9.1  
**Status:** Living Specification  
**Last Updated:** January 2025

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
- Trait-based polymorphism
- Powerful generics system

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
- **Advanced Types**: Union types, intersection types, conditional types
- **Option/Result**: Builtin Some/None, Ok/Err constructors
- **Reflection**: typeof, type_id, is_int_type, is_float_type, is_pointer_type (runtime type information)

### Memory Management

- **Borrow Checker**: Compile-time ownership and borrowing analysis
  - Phase 1: Immutability checking
  - Phase 2: Move semantics
  - Phase 3: Borrow rules (1 mutable XOR N immutable references)
  - Phase 4: Lifetime analysis (in development)
- **No Garbage Collection**: Deterministic memory management
- **Defer Statement**: Go-style resource cleanup (executes on function exit)
- **Smart Pointers**: Box<T> (implemented), Rc, Arc (planned)

### Concurrency

- **Goroutines**: Lightweight concurrent execution with `go` keyword (basic runtime)
- **Async/Await**: Structured asynchronous programming (implemented)
- **Channels**: MPSC message passing with Channel<T> (fully implemented)
- **Defer**: Go-style LIFO cleanup on function exit (fully implemented)
- **Select Statement**: Multiplexing on channel operations (planned)

### Pattern Matching

- Exhaustive matching with `match` expressions
- Tuple and struct destructuring
- OR patterns with SIMD optimization
- Guard clauses for conditional matching (implemented)
- Range patterns with `..` and `..=` (implemented)
- Switch statements for integer matching (implemented)

### Traits and Interfaces

- Trait-based polymorphism
- Multiple trait implementation
- Default trait methods (implemented)
- Trait inheritance with supertraits (implemented)
- Trait bounds on generics (partial)

### Methods

- **Inline Methods**: Methods defined inside struct body (implemented)
- **Golang-Style Methods**: Methods defined outside struct with receiver syntax (implemented)
- **Receiver Syntax**: `fn (self: &Type) method_name()` or `fn (r: &Type) method_name()`
- **Struct Tags**: Go-style backtick tags for metadata (implemented)

## Syntax Highlights (v0.9.1)

### Variable Declaration

```vex
let x = 42;              // Immutable (default)
let! counter = 0;        // Mutable (explicit with !)
const MAX_SIZE = 1000;   // Compile-time constant
```

### References

```vex
&T      // Immutable reference
&T!     // Mutable reference (v0.9 syntax, not &mut T)
```

### Functions

```vex
fn add(a: i32, b: i32): i32 {
    return a + b;
}
```

### Structs with Traits

```vex
struct Point impl Display, Eq {
    x: i32,
    y: i32,

    fn (self: &Point) show() {
        // Display trait method
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

### Completed Features (v0.9)

- ✅ Core type system with extended integer/float types (i128, u128, f16)
- ✅ Functions and methods (both inline and golang-style)
- ✅ Borrow checker (Phases 1-3 complete, Phase 4 in progress)
- ✅ Closures and lambda expressions with capture mode analysis
- ✅ Pattern matching with OR patterns and guards
- ✅ Traits with default methods and multiple inheritance
- ✅ Generics with monomorphization
- ✅ Control flow (if/elif/else, while, for, match, switch)
- ✅ Reference expressions (&expr, \*ptr) with &T! syntax
- ✅ Async runtime with goroutines and channels (partial)
- ✅ Language Server Protocol (LSP) implementation
- ✅ Comprehensive standard library builtins

### In Progress

- 🚧 Lifetime analysis (Phase 4 - borrow checker phases 1-3 complete, closures implemented)
- 🚧 Data-carrying enums (Option, Result - parsed and partially codegen)
- 🚧 Advanced pattern matching features

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
- Generics: Type parameters, monomorphization, trait bounds
- Patterns: Destructuring, OR patterns, guards
- Strings: F-strings, operations, UTF-8 support
- Algorithms: Fibonacci, factorial, GCD, sorting
- Traits: Multiple traits, default methods, inheritance
- Borrow Checker: Immutability, moves, borrows, closure capture
- Async: Goroutines, channels, async/await (partial)
- Builtins: Arrays, collections, I/O, time, testing framework

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

### Traits with Default Methods

```vex
trait Logger {
    fn (self: &Self!) log(msg: string);

    fn (self: &Self!) info(msg: string) {
        self.log(msg);  // Default implementation
    }
}

struct ConsoleLogger impl Logger {
    prefix: string,

    fn (self: &ConsoleLogger!) log(msg: string) {
        // Implementation
    }
}
```

## Getting Started

### Installation

```bash
git clone https://github.com/meftunca/vex_lang
cd vex_lang
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
9. **Traits and Interfaces** - Definition, implementation, inheritance
10. **Generics** - Type parameters, constraints, monomorphization
11. **Pattern Matching** - Patterns, destructuring, guards
12. **Closures and Lambda Expressions** - Anonymous functions, capture modes
13. **Memory Management** - Ownership, borrowing, lifetimes
14. **Concurrency** - Goroutines, async/await, channels
15. **Modules and Imports** - Module system, imports, exports
16. **Standard Library** - Core modules and APIs

---

## Comparison with Rust and Go

This section documents features available in Rust and Go but not yet implemented in Vex (v0.9.0).

### Features Rust Has (Vex Doesn't Yet)

#### Language Features

| Feature                             | Rust                          | Vex v0.9              | Notes                          |
| ----------------------------------- | ----------------------------- | --------------------- | ------------------------------ |
| **Closures/Lambdas**                | ✅ `\|x\| x + 1`              | ✅ Implemented        | Full capture mode analysis     |
| **Lifetime Annotations**            | ✅ `'a, 'static`              | 🚧 Phase 4 (planned)  | Borrow checker incomplete      |
| **Trait Objects**                   | ✅ `&dyn Trait`               | ❌ Not implemented    | Dynamic dispatch pending       |
| **Async/Await Runtime**             | ✅ Full tokio support         | ✅ Basic runtime      | Core async runtime implemented |
| **Macros**                          | ✅ Declarative + Procedural   | ❌ Not implemented    | Low priority                   |
| **Const Generics**                  | ✅ `[T; N]`                   | ❌ Not implemented    | Array size flexibility         |
| **Higher-Ranked Trait Bounds**      | ✅ `for<'a>`                  | ❌ Not implemented    | Advanced feature               |
| **Associated Constants**            | ✅ `const X: i32;`            | ❌ Not implemented    | Trait-level constants          |
| **Drop Trait**                      | ✅ RAII destructors           | ❌ Not implemented    | Resource cleanup               |
| **Deref Coercion**                  | ✅ Automatic `&String → &str` | 🚧 Partial            | Auto-deref for fields pending  |
| **Type Aliases in Traits**          | ✅ `type Item = T;`           | 🚧 Future             | Associated types planned       |
| **Unsafe Blocks**                   | ✅ `unsafe { }`               | ❌ Not implemented    | FFI integration needed         |
| **Raw Pointers**                    | ✅ `*const T, *mut T`         | ❌ Not implemented    | Low-level operations           |
| **Interior Mutability**             | ✅ `Cell<T>, RefCell<T>`      | ❌ Not implemented    | Advanced pattern               |
| **Pattern Guards**                  | ✅ `Some(x) if x > 0`         | ✅ Implemented        | Fully working                  |
| **Range Patterns**                  | ✅ `1..=10`                   | ✅ Implemented        | .. and ..= operators           |
| **Slice Patterns**                  | ✅ `[first, .., last]`        | ❌ Not implemented    | Advanced matching              |
| **Tuple Struct Indexing**           | ✅ `point.0`                  | 🚧 Parsed, no codegen | Implementation pending         |
| **Impl Trait**                      | ✅ `fn f() -> impl Trait`     | ❌ Not implemented    | Return type flexibility        |
| **Existential Types**               | ✅ `type Foo = impl Trait;`   | ❌ Not implemented    | Advanced feature               |
| **GATs (Generic Associated Types)** | ✅ Stable                     | ❌ Not implemented    | Complex generics               |

#### Standard Library & Ecosystem

| Feature                    | Rust                           | Vex v0.9             | Notes                     |
| -------------------------- | ------------------------------ | -------------------- | ------------------------- |
| **Collections**            | ✅ Vec, HashMap, HashSet, etc. | ✅ Implemented       | Vec, Map, Set, Box        |
| **Iterators**              | ✅ Full Iterator trait         | 🚧 Partial           | Basic iteration works     |
| **Option Type**            | ✅ `Option<T>`                 | ✅ Partial           | Some/None constructors    |
| **Result Type**            | ✅ `Result<T, E>`              | ✅ Partial           | Ok/Err constructors       |
| **Error Handling**         | ✅ `?` operator                | ❌ Not implemented   | Syntactic sugar missing   |
| **String Slicing**         | ✅ `&str[0..5]`                | ❌ Not implemented   | String operations limited |
| **Format Macro**           | ✅ `format!()`                 | 🚧 F-strings only    | Limited interpolation     |
| **Testing Framework**      | ✅ `#[test]`                   | ✅ Basic framework   | Builtin testing module    |
| **Documentation Comments** | ✅ `///` and `//!`             | ❌ Not implemented   | No doc generation         |
| **Attribute Macros**       | ✅ `#[derive(Debug)]`          | 🚧 `@intrinsic` only | Limited attributes        |
| **Cargo Equivalent**       | ✅ Cargo package manager       | ❌ Not implemented   | No package manager        |
| **Crates.io Equivalent**   | ✅ Package registry            | ❌ Not implemented   | No ecosystem yet          |

#### Tooling

| Feature                     | Rust             | Vex v0.9           | Notes              |
| --------------------------- | ---------------- | ------------------ | ------------------ |
| **Language Server**         | ✅ rust-analyzer | ✅ vex-lsp         | Full LSP support   |
| **Formatter**               | ✅ rustfmt       | ❌ Not implemented | Manual formatting  |
| **Linter**                  | ✅ clippy        | ❌ Not implemented | No static analysis |
| **Package Manager**         | ✅ cargo         | ❌ Not implemented | Manual builds only |
| **Documentation Generator** | ✅ rustdoc       | ❌ Not implemented | No auto-docs       |
| **Benchmark Framework**     | ✅ criterion     | ❌ Not implemented | No benchmarking    |

### Features Go Has (Vex Doesn't Yet)

#### Language Features

| Feature                        | Go                               | Vex v0.9           | Notes                              |
| ------------------------------ | -------------------------------- | ------------------ | ---------------------------------- |
| **Goroutines**                 | ✅ `go func()`                   | ✅ Basic runtime   | Core goroutine runtime implemented |
| **Channels**                   | ✅ `make(chan T)`                | ✅ MPSC channels   | Multi-producer single-consumer     |
| **Select Statement**           | ✅ Multi-channel wait            | ❌ Not implemented | Channel operations needed first    |
| **Defer Statement**            | ✅ `defer cleanup()`             | ✅ Fully working   | Go-style LIFO execution            |
| **Auto-Vectorization**         | ❌ Manual SIMD                   | ✅ Automatic       | **Unique to Vex**                  |
| **Interface Satisfaction**     | ✅ Implicit                      | 🚧 Explicit `impl` | Different design choice            |
| **Type Embedding**             | ✅ Anonymous fields              | ❌ Not implemented | Composition pattern                |
| **Type Assertions**            | ✅ `x.(Type)`                    | ❌ Not implemented | Runtime type checking              |
| **Type Switches**              | ✅ `switch x.(type)`             | ❌ Not implemented | Type-based matching                |
| **Variadic Functions**         | ✅ `func f(args ...T)`           | ❌ Not implemented | Flexible parameters                |
| **Multiple Return Values**     | ✅ `func f() (T, error)`         | 🚧 Tuples work     | Same capability, different syntax  |
| **Named Return Values**        | ✅ `func f() (x int, err error)` | ❌ Not implemented | Convenience feature                |
| **Init Functions**             | ✅ `func init()`                 | ❌ Not implemented | Package initialization             |
| **Blank Identifier**           | ✅ `_` for unused                | 🚧 In match only   | Limited usage                      |
| **Short Variable Declaration** | ✅ `:=` operator                 | ❌ Removed in v0.9 | Use `let` instead                  |
| **Pointer Arithmetic**         | ✅ Via unsafe package            | ❌ Not implemented | Low-level operations               |

#### Standard Library

| Feature                    | Go                        | Vex v0.9             | Notes                                 |
| -------------------------- | ------------------------- | -------------------- | ------------------------------------- |
| **HTTP Server**            | ✅ `net/http`             | 🚧 Planned (Layer 3) | std lib incomplete                    |
| **JSON Marshal/Unmarshal** | ✅ `encoding/json`        | 🚧 Planned (Layer 3) | std lib incomplete                    |
| **File I/O**               | ✅ `os.File`              | 🚧 Basic (Layer 1)   | Limited operations                    |
| **Goroutine Scheduler**    | ✅ Built-in runtime       | ✅ Basic runtime     | Core goroutine runtime implemented    |
| **Garbage Collection**     | ✅ Concurrent GC          | ❌ Manual memory     | Design choice: no GC                  |
| **Reflection**             | ✅ `reflect` package      | ✅ Partial           | typeof, type*id, is*\*\_type builtins |
| **Context Package**        | ✅ Cancellation/timeout   | ❌ Not implemented   | Concurrency control                   |
| **Sync Package**           | ✅ Mutex, WaitGroup, etc. | 🚧 Planned (Layer 2) | std lib incomplete                    |
| **Testing Package**        | ✅ `testing`              | ✅ Basic framework   | testing module with assert functions  |
| **Database/SQL**           | ✅ `database/sql`         | ❌ Not implemented   | No DB drivers                         |
| **Template Engine**        | ✅ `text/template`        | ❌ Not implemented   | No templating                         |

#### Tooling & Ecosystem

| Feature               | Go                          | Vex v0.9           | Notes                    |
| --------------------- | --------------------------- | ------------------ | ------------------------ |
| **Go Modules**        | ✅ Built-in package manager | ❌ Not implemented | No dependency management |
| **go fmt**            | ✅ Standard formatter       | ❌ Not implemented | Manual formatting        |
| **go vet**            | ✅ Static analyzer          | ❌ Not implemented | No linting               |
| **go test**           | ✅ Built-in testing         | ❌ Not implemented | No test runner           |
| **go doc**            | ✅ Documentation viewer     | ❌ Not implemented | No doc generation        |
| **pprof**             | ✅ Profiling tools          | ❌ Not implemented | No profiling             |
| **race detector**     | ✅ `-race` flag             | ❌ Not implemented | No race detection        |
| **Cross-compilation** | ✅ Easy GOOS/GOARCH         | 🚧 LLVM targets    | Platform support limited |
| **Language Server**   | ✅ gopls                    | ❌ Not implemented | No IDE support           |

### What Vex Has That's Unique

While Vex is missing many features, it combines aspects from both languages in novel ways:

| Feature                  | Vex Approach                 | Rust                  | Go                     |
| ------------------------ | ---------------------------- | --------------------- | ---------------------- |
| **Variable Mutability**  | `let` vs `let!`              | `let` vs `let mut`    | All mutable by default |
| **Mutable References**   | `&T!` syntax                 | `&mut T`              | All references mutable |
| **Method Syntax**        | Both inline and golang-style | Impl blocks only      | Receiver syntax only   |
| **Elif Keyword**         | ✅ Native `elif`             | `else if`             | `else if`              |
| **Trait Implementation** | `struct S impl T { }` inline | Separate `impl` block | Implicit satisfaction  |
| **Union Types**          | `(T \| U)` planned           | `enum` workaround     | `interface{}`          |
| **Intersection Types**   | `(T & U)` planned            | Trait bounds          | Not available          |
| **GPU Functions**        | `gpu fn` keyword             | Via compute crates    | Via CGO                |

### Roadmap Priority

**High Priority (Blocking Production Use)**:

1. ✅ Borrow Checker Phases 1-3 (COMPLETE)
2. 🟡 Phase 4: Lifetime Analysis (in progress)
3. ✅ Closures and lambdas (COMPLETE)
4. 🟡 Option/Result types with pattern matching (parsed, partial codegen)
5. 🟡 Iterator trait and collection methods (builtin collections implemented)
6. ✅ Async runtime integration (COMPLETE - basic runtime implemented)
7. 🟡 Standard library completion (I/O, networking - extensive builtins added)

**Medium Priority (Developer Experience)**:

1. 🟡 Error handling (`?` operator)
2. 🟡 Testing framework (builtin testing framework implemented)
3. ✅ Language server protocol (LSP) (COMPLETE)
4. 🟡 Formatter and linter
5. 🟡 Package manager
6. 🟡 Documentation generator

**Low Priority (Advanced Features)**:

1. 🟢 Macros (declarative)
2. 🟢 Unsafe blocks and raw pointers
3. 🟢 Reflection and runtime type info
4. 🟢 Procedural macros
5. 🟢 Const generics

**By Design (Won't Implement)**:

- ❌ Garbage collection (manual memory management by design)
- ❌ Null pointers (use Option type instead)
- ❌ Exceptions (use Result type instead)
- ❌ Inheritance (use composition and traits)
- ❌ Function overloading (use generics instead)

### Current Limitations

**Stability**: Vex is alpha software (v0.9.1). Core features stable, advanced features evolving.

**Test Coverage**: Extensive examples across all major features. Core functionality well-tested.

**Documentation**: Language spec is comprehensive and up-to-date with implementation.

**Ecosystem**: No third-party packages, no package registry, no community crates yet.

**IDE Support**: ✅ Language Server Protocol (LSP) implemented, VS Code extension available.

**Production Readiness**: ⚠️ **ALPHA** - Core features stable, suitable for experimental projects and learning.

---

## Version History

### v0.9.1 (January 2025)

- **Type System Enhancements**:

  - Extended integer types: i128, u128
  - Extended float types: f16
  - Collections: Map<K,V>, Set<T>, Vec<T>, Box<T>, Channel<T>
  - Option/Result constructors: Some/None, Ok/Err

- **Concurrency Features**:

  - MPSC channels fully implemented (lock-free ring buffer)
  - Async runtime with basic goroutine support
  - Defer statement with LIFO execution
  - Channel operations: new, send, recv, close

- **Pattern Matching**:

  - Pattern guards with `if` clauses
  - Range patterns: `..` and `..=`
  - Switch statements for integer matching

- **Traits & Methods**:

  - Default trait methods
  - Trait inheritance with supertraits
  - Golang-style method definitions
  - Struct tags for metadata

- **Borrow Checker**:

  - Phases 1-3 complete
  - Closure capture mode analysis
  - Move semantics fully working

- **Other Features**:
  - Language Server Protocol (LSP) implementation
  - Closures with automatic capture analysis
  - Comprehensive standard library builtins
  - Reference expressions (`&expr`, `*ptr`)

### v0.9.0 (November 3, 2025)

- Unified variable system: `let` (immutable), `let!` (mutable)
- Reference syntax: `&T!` instead of `&mut T`
- Removed `mut` keyword from lexer
- Deprecated `interface` keyword (use `trait`)
- Initial implementation of core features

### v0.2.0 (Previous)

- Initial compiler implementation
- Basic type system
- Function and struct support
- Pattern matching foundations

---

**Next Document**: [02_Lexical_Structure.md](./02_Lexical_Structure.md)

**Maintained by**: Vex Language Team  
**License**: MIT
