# Vex Examples - v0.9

Welcome to the Vex programming language examples! These examples demonstrate the core features of Vex that are currently implemented and working.

## 📁 Directory Structure

```
examples/
├── 01_basics/           # Core language features
├── 02_functions/        # Functions, methods, recursion
├── 03_control_flow/     # If, switch, loops
├── 04_types/            # Structs, enums, tuples, aliases
├── 05_generics/         # Generic types and functions
├── 06_patterns/         # Pattern matching, destructuring
├── 07_strings/          # String operations, f-strings
└── 08_algorithms/       # Classic algorithms
```

## 🚀 Running Examples

```bash
# Using the vex compiler
~/.cargo/target/debug/vex run examples/01_basics/hello_world.vx

# Or compile to object file
~/.cargo/target/debug/vex compile examples/01_basics/hello_world.vx -o hello.o
```

## 📚 Examples by Category

### 01_basics/ - Core Language

| File             | Description           | Features               |
| ---------------- | --------------------- | ---------------------- |
| `hello_world.vx` | Simplest program      | Basic function         |
| `variables.vx`   | Variable declarations | let, let!, const       |
| `types_basic.vx` | Primitive types       | i32, f32, bool, string |

**Key Concepts:**

- ✅ `let x = 42;` - Immutable variable (default)
- ✅ `let! x = 42;` - Mutable variable (explicit with !)
- ✅ `const MAX = 100;` - Compile-time constant
- ✅ Type inference and explicit types

### 02_functions/ - Functions

| File           | Description         | Features                  |
| -------------- | ------------------- | ------------------------- |
| `basic.vx`     | Function basics     | Parameters, return values |
| `recursion.vx` | Recursive functions | Fibonacci, factorial, GCD |
| `methods.vx`   | Struct methods      | Method receivers          |

**Key Concepts:**

- ✅ Function definitions with parameters
- ✅ Return types
- ✅ Recursion support
- ✅ Method syntax with receivers

### 03_control_flow/ - Control Flow

| File         | Description            | Features                       |
| ------------ | ---------------------- | ------------------------------ |
| `if_else.vx` | Conditional statements | if, else, nested               |
| `switch.vx`  | Switch statements      | case, default, multiple values |
| `loops.vx`   | Loop constructs        | while, for                     |

**Key Concepts:**

- ✅ If-else conditionals
- ✅ Switch/case with LLVM backend
- ✅ While loops
- ✅ For loops

### Defer Statement (Resource Management) ✅ NEW

| File              | Description               | Features                 |
| ----------------- | ------------------------- | ------------------------ |
| `defer_simple.vx` | LIFO execution order      | Basic defer, 3 functions |
| `defer_test.vx`   | Comprehensive defer tests | Multiple scenarios       |

**Key Concepts:**

- ✅ **Go-style defer**: Deferred function calls execute before function returns
- ✅ **LIFO execution**: Last registered defer executes first (stack-based)
- ✅ **Automatic cleanup**: Runs on `return`, function exit, `break`, `continue`
- ✅ **Syntax**: `defer function_call();` (block syntax pending)

**Example:**

```vex
fn cleanup(): i32 { print("Cleanup"); return 0; }

fn example(): i32 {
    defer cleanup();  // Registers cleanup
    print("Work");
    return 0;         // cleanup() executes here
}
// Output: Work Cleanup
```

### 04_types/ - Type System

| File                 | Description      | Features                  |
| -------------------- | ---------------- | ------------------------- |
| `struct_basic.vx`    | Basic structs    | Definition, instantiation |
| `struct_advanced.vx` | Advanced structs | Nested, methods           |
| `enum_basic.vx`      | Simple enums     | C-style enums             |
| `tuple_basic.vx`     | Tuple types      | (T, U, V)                 |
| `type_aliases.vx`    | Type aliases     | Custom type names         |
| `references.vx`      | References       | &T, &T!                   |

**Key Concepts:**

- ✅ Struct definitions and field access
- ✅ Enum definitions (C-style)
- ✅ Tuple types (parse support)
- ✅ Type aliases
- ✅ References: `&T` (immutable), `&T!` (mutable)

### 05_generics/ - Generics

| File                 | Description            | Features                   |
| -------------------- | ---------------------- | -------------------------- |
| `functions.vx`       | Generic functions      | Type parameters            |
| `interfaces.vx`      | Generic interfaces     | Interface<T>               |
| `structs.vx`         | Generic structs        | Option<T>, Result<T>       |
| `nested_generics.vx` | Nested generic types   | Box<Box<T>>, Pair<Box<T>>  |
| `nested_simple.vx`   | Simple nested test     | Box<Box<i32>> field access |
| `nested_debug.vx`    | Debug nested w/ annots | Type annotations           |

**Key Concepts:**

- ✅ Generic functions with `<T>`
- ✅ Generic structs
- ✅ Nested generics (Box<Box<T>>)
- ✅ Interface definitions
- ✅ Monomorphization
- ⚠️ Chained field access (a.b.c) requires intermediate variables

### 06_patterns/ - Pattern Matching

| File                    | Description     | Features            |
| ----------------------- | --------------- | ------------------- |
| `struct_destructure.vx` | Struct patterns | Field destructuring |
| `tuple_destructure.vx`  | Tuple patterns  | Element extraction  |
| `enum_match.vx`         | Enum patterns   | Match expressions   |

**Key Concepts:**

- ⚠️ Pattern matching (parser support only)
- ⚠️ Destructuring (limited codegen)

### 07_strings/ - Strings

| File                   | Description        | Features         |
| ---------------------- | ------------------ | ---------------- |
| `literals.vx`          | String basics      | String literals  |
| `formatting.vx`        | F-strings          | f"Value: {x}"    |
| `string_comparison.vx` | String comparisons | ==, != operators |

**Key Concepts:**

- ✅ String literals
- ✅ String comparison (==, !=)
- ✅ F-string syntax (limited codegen)
- ✅ Global string constants

### 08_algorithms/ - Algorithms

| File           | Description             | Complexity | Returns |
| -------------- | ----------------------- | ---------- | ------- |
| `fibonacci.vx` | Nth Fibonacci           | O(2^n)     | 55      |
| `factorial.vx` | Factorial               | O(n)       | 120     |
| `gcd.vx`       | Greatest Common Divisor | O(log n)   | 6       |
| `prime.vx`     | Prime check             | O(√n)      | bool    |
| `power.vx`     | Exponentiation          | O(n)       | 1024    |

**All algorithms are working and tested!** ✅

## 🔧 v0.9 Syntax Guide

### Variables

```vex
// Immutable (default)
let x = 42;

// Mutable (explicit)
let! y = 10;
y = 20;  // OK

// Constant
const PI = 3.14;
```

### Functions

```vex
fn add(a: i32, b: i32) : i32 {
    return a + b;
}
```

### Structs

```vex
struct Point {
    x: i32,
    y: i32,
}

let p = Point { x: 10, y: 20 };
let x_val = p.x;
```

### Control Flow

```vex
// If-else
if x > 0 {
    return 1;
} else {
    return 0;
}

// Switch
switch x {
    case 1:
        return 10;
    case 2, 3:
        return 20;
    default:
        return 0;
}
```

### References

```vex
// Immutable reference
let x = 42;
let ref_x: &i32 = &x;

// Mutable reference
let! y = 10;
let ref_y: &i32! = &y!;
```

## ✅ Working Features

- ✅ Functions (basic, generic, recursive, methods)
- ✅ Variables (let, let!, const)
- ✅ Primitive types (integers, floats, bool, string)
- ✅ Structs (definition, instantiation, field access)
- ✅ Enums (C-style, basic support)
- ✅ Control flow (if/else, switch, while, for)
- ✅ Arithmetic operators (+, -, \*, /, %)
- ✅ Comparison operators (==, !=, <, >, <=, >=)
- ✅ Type inference
- ✅ Type aliases
- ✅ Generics (partial)
- ✅ Interfaces (definition, parsing)
- ✅ References (&T, &T!)
- ✅ Tuples (parsing)

## ⚠️ Partial Support

- ⚠️ Pattern matching (parser only, limited codegen)
- ⚠️ Match expressions (parsed, codegen incomplete)
- ⚠️ F-strings (parsed, limited interpolation)
- ⚠️ Generics (basic monomorphization, edge cases)
- ⚠️ Traits (parser only, no codegen)

## ❌ Not Yet Implemented

- ❌ Async/await
- ❌ Channels and concurrency
- ❌ GPU kernels
- ❌ SIMD intrinsics
- ❌ FFI (work in progress)
- ❌ Module system (imports parse only)
- ❌ Standard library (io, fs, net, etc.)
- ❌ Error handling (Result<T>, try/catch)
- ❌ Trait implementations
- ❌ Union type codegen
- ❌ Advanced pattern matching

## 📊 Test Status

| Category     | Total  | Working | Partial | Not Working |
| ------------ | ------ | ------- | ------- | ----------- |
| Basics       | 3      | 3       | 0       | 0           |
| Functions    | 3      | 3       | 0       | 0           |
| Control Flow | 3      | 3       | 0       | 0           |
| Types        | 6      | 4       | 2       | 0           |
| Generics     | 3      | 1       | 2       | 0           |
| Patterns     | 3      | 0       | 3       | 0           |
| Strings      | 2      | 1       | 1       | 0           |
| Algorithms   | 5      | 5       | 0       | 0           |
| **TOTAL**    | **28** | **20**  | **8**   | **0**       |

**Success Rate: 71% fully working, 29% partial** 🎉

## 🎯 Quick Start

1. **Hello World**

```bash
~/.cargo/target/debug/vex run examples/01_basics/hello_world.vx
```

2. **Try Variables**

```bash
~/.cargo/target/debug/vex run examples/01_basics/variables.vx
```

3. **Fibonacci**

```bash
~/.cargo/target/debug/vex run examples/08_algorithms/fibonacci.vx
# Should exit with code 55
```

4. **Check Exit Code**

```bash
~/.cargo/target/debug/vex run examples/08_algorithms/factorial.vx
echo $?  # Should print 120
```

## 📖 Learning Path

1. Start with `01_basics/` - Learn core syntax
2. Move to `02_functions/` - Understand functions and recursion
3. Try `03_control_flow/` - Master conditionals and loops
4. Explore `04_types/` - Work with structs and enums
5. Study `08_algorithms/` - See real-world examples

## 🐛 Known Issues

- Pattern matching codegen is incomplete
- Match expressions don't generate proper LLVM IR
- F-string interpolation is limited
- Generic type constraints not enforced
- Module imports are parsed but not resolved
- Error handling (Result/Option) needs codegen

## 🤝 Contributing

When adding new examples:

1. Use v0.9 syntax (`let` vs `let!`)
2. Include comments explaining the feature
3. Make examples self-contained
4. Test that they compile and run
5. Add to this README

## 📝 Notes

- All examples use v0.9 syntax (let/let! system)
- Examples return values via exit codes for testing
- IO operations commented out until std library is ready
- Focus on features that actually work in the compiler

---

**Compiler Version:** 0.2.0  
**Syntax Version:** v0.9  
**Last Updated:** 3 Kasım 2025
