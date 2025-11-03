# Vex Language Syntax Reference (v0.9)

> **IMPORTANT**: This document reflects the **v0.9 syntax** - the current implementation in the compiler.  
> Major changes: `let`/`let!` system, removal of `:=` and `var`, `&T!` mutable references.  
> Updated: November 3, 2025

---

## Table of Contents

1. [Comments](#comments)
2. [Variables & Constants](#variables--constants)
3. [Types](#types)
4. [Functions](#functions)
5. [Structs](#structs)
6. [Enums](#enums)
7. [Interfaces](#interfaces)
8. [Traits](#traits)
9. [Generics](#generics)
10. [Control Flow](#control-flow)
11. [Pattern Matching](#pattern-matching)
12. [Operators](#operators)
13. [Modules & Imports](#modules--imports)
14. [FFI (Foreign Function Interface)](#ffi-foreign-function-interface)
15. [Async & Concurrency](#async--concurrency)
16. [Unsafe](#unsafe)
17. [Attributes](#attributes)

---

## Comments

```vex
// Single-line comment

/*
   Multi-line comment
   Can span multiple lines
*/
```

---

## Variables & Constants

### Variable Declaration (Go-style)

````vex
## Variables & Constants (v0.9)

### Variable Declaration

**Immutable by Default (let)**

```vex
// Immutable variable (default)
let x = 42;
let name = "Alice";

// Type inference
let count = 100;        // i32
let pi = 3.14;          // f64
let is_ready = true;    // bool
````

**Mutable Variables (let!)**

```vex
// Explicit mutable with ! operator
let! counter = 0;
counter = counter + 1;  // OK: mutable

let! sum = 0;
sum = sum + 10;         // OK: can reassign
```

**Explicit Type Annotations**

```vex
let age: i32 = 25;
let height: f32 = 1.75;
let name: string = "Bob";
```

````

**v0.9 Changes:**
- ❌ Removed: `var x = 10;` keyword
- ❌ Removed: `:=` operator (Go-style)
- ❌ Removed: `let mut` syntax
- ✅ New: `let!` for mutable variables (bang operator)
- ✅ `let` is immutable by default

**Constants (Planned)**

```vex
// TODO: const keyword not yet implemented
// const MAX_SIZE = 100;
// const PI = 3.14159;
````

---

## Types

### Primitive Types

| Type                      | Description       | Size               |
| ------------------------- | ----------------- | ------------------ |
| `i8`, `i16`, `i32`, `i64` | Signed integers   | 8, 16, 32, 64 bits |
| `u8`, `u16`, `u32`, `u64` | Unsigned integers | 8, 16, 32, 64 bits |
| `f32`, `f64`              | Floating point    | 32, 64 bits        |
| `bool`                    | Boolean           | 1 bit              |
| `byte`                    | Alias for `u8`    | 8 bits             |
| `string`                  | UTF-8 string      | Variable           |
| `error`                   | Error type        | Variable           |

### Compound Types

#### Arrays (Fixed-size)

```vex
// Fixed-size array: [Type; Size]
arr: [i32; 5] = [1, 2, 3, 4, 5];
matrix: [[f32; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
```

#### Slices (References to arrays)

```vex
// Immutable slice: &[T]
fn sum(numbers: &[i32]): i32 {
    // ...
}

// Mutable slice: &[T]! (v0.9 syntax)
fn modify(data: &[f32]!) {
    // ...
}
```

#### Tuples

```vex
// Tuple declaration
point: (i32, i32) = (10, 20);
mixed: (string, i32, bool) = ("hello", 42, true);

// Tuple destructuring
(x, y) := point;
(name, age, _) := mixed;
```

#### References (v0.9)

```vex
// Immutable reference: &T
let x = 10;
let ref_x: &i32 = &x;

// Mutable reference: &T! (v0.9 syntax with bang operator)
let! y = 20;
let ref_y: &i32! = &y!;
// *ref_y = 30; // Dereference to modify (when implemented)
```

**v0.9 Changes:**

- ❌ Removed: `&mut T` syntax
- ✅ New: `&T!` for mutable references (consistent with `let!`)
- ✅ `&T` for immutable references

#### Pointers (Raw pointers for FFI)

```vex
// Raw pointer: *T
ptr: *byte = malloc(1024);
data: *i32 = &value as *i32;
```

### Type Aliases

```vex
type UserID = u64;
type Callback = fn(i32): i32;
type StringMap = map[string]string;
```

### Union Types (Sum types)

```vex
// Type can be one of several types
type Value = i32 | string | bool;
type Result = string | error;

fn process(v: i32 | string): i32 {
    // ...
}
```

### Intersection Types (Product types)

```vex
// Type must satisfy multiple interfaces
type ReadWriter = Reader & Writer;

fn do_io(stream: Reader & Writer): i32 {
    // ...
}
```

### Conditional Types (Type-level if/else)

```vex
// T extends U ? X : Y
type IsString<T> = T extends string ? string : i32;

// With infer keyword
type Unwrap<T> = T extends &[infer E] ? E : T;
```

**Current Status**: All type constructs are parsed and represented in AST.

---

## Functions

### Function Declaration

```vex
// Basic function
fn add(a: i32, b: i32): i32 {
    return a + b;
}

// No return type (void/unit)
fn greet(name: string) {
    println("Hello, {}!", name);
}

// Multiple return values (tuple)
fn divide(a: i32, b: i32): (i32, error) {
    if b == 0 {
        return (0, error.new("division by zero"));
    }
    return (a / b, nil);
}
```

### Method Syntax (Receiver functions)

```vex
struct Point {
    x: i32,
    y: i32,
}

// Immutable receiver: (self: &Type)
fn (self: &Point) distance(): f64 {
    return sqrt((self.x * self.x + self.y * self.y) as f64);
}

// Mutable receiver: (self: &Type!) (v0.9 syntax)
fn (self: &Point!) move_by(dx: i32, dy: i32) {
    self.x = self.x + dx;
    self.y = self.y + dy;
}
```

### Generic Functions

```vex
fn identity<T>(x: T): T {
    return x;
}

fn pair<T, U>(first: T, second: U): (T, U) {
    return (first, second);
}

// With trait bounds
fn read_and_print<T: Reader>(reader: &T!): i32 {
    // ...
}
```

### Variadic Functions (FFI only)

```vex
#[link(name = "c")]
extern {
    fn printf(fmt: *byte, ...): i32;
}
```

**Current Status**: All function syntaxes are fully supported.

---

## Structs

### Struct Definition

```vex
struct Point {
    x: i32,
    y: i32,
}

// Generic struct
struct Box<T> {
    value: T,
}

// Struct with multiple type parameters
struct Pair<T, U> {
    first: T,
    second: U,
}
```

### Struct Instantiation

```vex
// Basic literal
p := Point { x: 10, y: 20 };

// Generic struct literal
box := Box<i32> { value: 42 };

// Nested structs
outer := Outer { inner: Inner { data: "hello" } };
```

### Field Access

```vex
p := Point { x: 10, y: 20 };
x_val := p.x;
p.y = 30;
```

**Current Status**: Structs are fully implemented with generics support.

---

## Enums

### Enum Definition

```vex
// C-style enum (discriminants)
enum Status {
    Pending,
    Active,
    Complete,
}

// Rust-style enum (algebraic data types)
enum Option<T> {
    Some(T),
    None,
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

### Enum Usage

```vex
// Auto-generated constructors
status := Status_Active();  // Returns 1

// With data
result := Option_Some(42);
empty := Option_None();
```

### Pattern Matching on Enums

```vex
match result {
    Option::Some(value) => {
        println("Got value: {}", value);
    },
    Option::None => {
        println("No value");
    },
}
```

**Current Status**: Enums fully implemented. Constructor functions auto-generated by compiler.

---

## Interfaces

### Interface Definition

```vex
// Simple interface
interface Writer {
    fn write(data: i32): i32 {}
}

// Generic interface
interface Cache<K, V> {
    fn get(key: K): V {}
    fn set(key: K, value: V): i32 {}
}

// Multiple type parameters
interface Map<K, V> {
    fn insert(key: K, value: V): bool {}
    fn get(key: K): V {}
}
```

### Interface Implementation (Implicit)

```vex
struct File {
    path: string,
}

// Implement Writer interface methods
fn (self: &File) write(data: i32): i32 {
    return data;
}
```

**Current Status**: Interfaces are parsed but implementation is implicit (duck typing).

---

## Traits

### Trait Definition

```vex
trait Reader {
    fn read(self: &Self!, buf: &[byte]!): (i32 | error);

    // Default method
    fn read_all(self: &Self!): ([byte] | error) {
        // Default implementation
        return [];
    }
}

// Generic trait
trait Converter<T> {
    fn convert(self: &Self): T;
}
```

### Trait Implementation

```vex
struct File {
    fd: i32,
}

impl Reader for File {
    fn read(self: &Self!, buf: &[byte]!): (i32 | error) {
        // Implementation
        return 0;
    }
}

// Generic trait impl
impl Converter<string> for i32 {
    fn convert(self: &Self): string {
        return self.to_string();
    }
}
```

**Current Status**: Traits fully supported with default methods and generic implementations.

---

## Generics

### Generic Types

```vex
// Generic struct
struct Vec<T> {
    data: *T,
    len: usize,
    cap: usize,
}

// Multiple type parameters
struct HashMap<K, V> {
    buckets: *Bucket<K, V>,
    size: usize,
}
```

### Generic Functions

```vex
fn identity<T>(x: T): T {
    return x;
}

fn swap<T>(a: &T!, b: &T!) {
    tmp := *a;
    *a = *b;
    *b = tmp;
}
```

### Generic Constraints (Trait Bounds)

```vex
fn serialize<T: Serializable>(obj: T): string {
    return obj.to_json();
}

// Multiple bounds
fn convert<T: Reader & Writer>(stream: T): bool {
    // ...
}
```

**Current Status**: Generic types and functions fully supported. Trait bounds are parsed.

---

## Control Flow

### If/Else

```vex
if x > 0 {
    println("Positive");
} else if x < 0 {
    println("Negative");
} else {
    println("Zero");
}
```

### For Loop (C-style)

```vex
// Standard for loop
for let i = 0; i < 10; i = i + 1 {
    println("{}", i);
}

// Simplified
for i := 0; i < 10; i++ {
    println("{}", i);
}
```

### For-In Loop (Range iteration)

```vex
// Iterate over range
for i in 0..10 {
    println("{}", i);
}

// Iterate over array/slice
arr := [1, 2, 3, 4, 5];
for item in arr {
    println("{}", item);
}
```

### While Loop

```vex
let! i = 0;
while i < 10 {
    println("{}", i);
    i = i + 1;
}
```

### Loop (Infinite)

```vex
loop {
    // Infinite loop
    if condition {
        break;
    }
}
```

### Switch Statement

```vex
switch x {
    case 1:
        println("One");
    case 2, 3:
        println("Two or Three");
    default:
        println("Other");
}
```

**Current Status**: All control flow constructs are implemented.

---

## Pattern Matching

### Match Expression

```vex
result := match value {
    0 => "zero",
    1 => "one",
    2 => "two",
    _ => "many",
};
```

### Pattern Types

```vex
// Literal pattern
match x {
    42 => println("The answer"),
    _ => println("Not the answer"),
}

// Tuple pattern
match point {
    (0, 0) => println("Origin"),
    (0, y) => println("On Y axis"),
    (x, 0) => println("On X axis"),
    (x, y) => println("Point at ({}, {})", x, y),
}

// Enum pattern
match option {
    Option::Some(value) => println("Value: {}", value),
    Option::None => println("No value"),
}

// Struct pattern
match user {
    User { id: 0, .. } => println("Admin"),
    User { name, .. } => println("User: {}", name),
}
```

### Guards

```vex
match x {
    n if n > 0 => println("Positive"),
    n if n < 0 => println("Negative"),
    _ => println("Zero"),
}
```

**Current Status**: Match expressions fully implemented with all pattern types.

---

## Operators

### Arithmetic Operators

```vex
a + b   // Addition
a - b   // Subtraction
a * b   // Multiplication
a / b   // Division
a % b   // Modulo
```

### Comparison Operators

```vex
a == b  // Equal
a != b  // Not equal
a < b   // Less than
a <= b  // Less than or equal
a > b   // Greater than
a >= b  // Greater than or equal
```

### Logical Operators

```vex
a && b  // Logical AND
a || b  // Logical OR
!a      // Logical NOT
```

### Bitwise Operators

```vex
a & b   // Bitwise AND
a | b   // Bitwise OR
a ^ b   // Bitwise XOR
~a      // Bitwise NOT
a << n  // Left shift
a >> n  // Right shift
```

### Compound Assignment

```vex
x += 5;   // x = x + 5
x -= 3;   // x = x - 3
x *= 2;   // x = x * 2
x /= 4;   // x = x / 4
```

### Increment/Decrement

```vex
x++;  // Post-increment
x--;  // Post-decrement
```

### Reference & Dereference

```vex
&x    // Reference (address-of)
*ptr  // Dereference
```

### Range Operator

```vex
0..10     // Range from 0 to 10 (exclusive)
start..end
```

### Error Propagation

```vex
result := operation()?;  // Propagate error if present
```

**Current Status**: All operators are implemented.

---

## Modules & Imports

### Import Styles

```vex
// Named imports
import { io, net } from "std";

// Namespace import
import * as std from "std";

// Module import (brings everything into scope)
import "std/io";

// Type imports
import type { Reader, Writer } from "std::io";
```

### Export

```vex
// Export individual items
export fn public_function(): i32 {
    return 42;
}

export struct PublicStruct {
    field: i32,
}

export const PUBLIC_CONST: i32 = 100;

// Export list
export { io, net, http };
```

### Module Organization

```vex
// In mod.vx
export use sub_module::{Type1, Type2};
export use another::{function1, function2};

mod sub_module;
mod another;
```

**Current Status**: All import patterns are fully supported.

---

## FFI (Foreign Function Interface)

### Extern Block

```vex
// Link to C library
#[link(name = "c")]
extern {
    fn malloc(size: usize): *byte;
    fn free(ptr: *byte);
    fn printf(fmt: *byte, ...): i32;
}

// Multiple functions
#[link(name = "m")]
extern {
    fn sin(x: f64): f64;
    fn cos(x: f64): f64;
    fn sqrt(x: f64): f64;
}
```

### Type Declarations in FFI

```vex
#[link(name = "c")]
extern {
    type FILE;
    type pthread_t;
    type sockaddr;

    fn fopen(path: *byte, mode: *byte): *FILE;
    fn fclose(file: *FILE): i32;
}
```

### Calling FFI Functions

```vex
fn main(): i32 {
    ptr := unsafe { malloc(1024) };
    defer unsafe { free(ptr); };

    unsafe {
        printf(c"Hello, World!\n");
    }

    return 0;
}
```

**Current Status**: FFI is fully implemented. `unsafe` blocks required for FFI calls.

---

## Async & Concurrency

### Async Functions

```vex
async fn fetch_data(url: string): (string | error) {
    response := await http.get(url)?;
    return response.body;
}
```

### Await

```vex
result := await async_operation();
```

### Go Concurrency (Goroutines)

```vex
// Spawn concurrent task
go task_function();

// With arguments
go compute(x, y, z);
```

### Select (Async channel selection)

```vex
select {
    case result := await channel1.recv() => {
        println("From channel 1: {}", result);
    },
    case result := await channel2.recv() => {
        println("From channel 2: {}", result);
    },
}
```

**Current Status**: Async/await syntax is parsed. Runtime integration via Tokio FFI.

---

## Unsafe

### Unsafe Block

```vex
unsafe {
    ptr := malloc(1024);
    *ptr = 42;
    free(ptr);
}
```

### When to Use Unsafe

1. **FFI calls**: Calling C functions
2. **Raw pointer operations**: Dereferencing raw pointers
3. **Type punning**: Casting between incompatible types
4. **Manual memory management**: malloc/free

```vex
fn read_memory(addr: usize): i32 {
    ptr := addr as *i32;
    return unsafe { *ptr };
}
```

**Current Status**: Unsafe blocks are fully supported.

---

## Attributes

### Common Attributes

```vex
// Inline functions
#[inline]
fn fast_function(): i32 {
    return 42;
}

// Always inline
#[inline(always)]
fn critical_path(): i32 {
    return 0;
}

// Link to library
#[link(name = "pthread")]
extern {
    fn pthread_create(...): i32;
}

// Conditional compilation
#[cfg(target_os = "linux")]
fn platform_specific() {
    // Linux-only code
}

#[cfg(feature = "gpu")]
fn gpu_accelerated() {
    // GPU code
}
```

### GPU Attributes

```vex
#[gpu]
fn kernel_function(data: &[f32]!) {
    // GPU kernel code
}
```

**Current Status**: Attributes are parsed and stored in AST. Compiler respects `#[link]` and `#[inline]`.

---

## Special Features

### GPU/HPC

```vex
// GPU kernel
#[gpu]
fn vector_add(a: &[f32], b: &[f32], c: &[f32]!, n: i32) {
    idx := get_global_id();
    if idx < n {
        c[idx] = a[idx] + b[idx];
    }
}

// Launch kernel
launch vector_add[blocks, threads](a, b, c, n);
```

### Try/Catch (Error handling)

```vex
result := try operation();

// With match
match try risky_operation() {
    Ok(value) => println("Success: {}", value),
    Err(e) => println("Error: {}", e.message()),
}
```

### Defer (Resource cleanup)

```vex
fn open_file(path: string): (File | error) {
    f := File::open(path)?;
    defer f.close();  // Always runs when function exits

    // Use file
    return f;
}
```

---

## Lexer Tokens (Reference)

Currently recognized tokens (from `vex-lexer/src/lib.rs`):

### Keywords

`fn`, `let`, `struct`, `enum`, `interface`, `trait`, `impl`, `async`, `await`, `go`, `gpu`, `launch`, `try`, `return`, `if`, `else`, `for`, `while`, `in`, `import`, `export`, `from`, `as`, `true`, `false`, `nil`, `type`, `extends`, `infer`, `const`, `unsafe`, `new`, `make`, `switch`, `case`, `default`, `match`, `select`, `extern`

**v0.9 Note:** `mut` keyword removed - use `!` operator with `let!` and `&T!` instead.

### Primitive Types

`i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`, `f32`, `f64`, `bool`, `string`, `byte`, `error`

### Operators

`+`, `-`, `*`, `/`, `%`, `=`, `==`, `!=`, `<`, `<=`, `>`, `>=`, `&&`, `||`, `!`, `&`, `|`, `?`, `+=`, `-=`, `*=`, `/=`, `++`, `--`, `:=`

### Delimiters

`(`, `)`, `{`, `}`, `[`, `]`, `,`, `;`, `:`, `.`, `..`, `->`, `=>`, `_`, `#`, `...`

### Literals

- Integer: `123`, `0xFF`, `0b1010`
- Float: `3.14`, `1.0e-5`
- String: `"hello"`, `"unicode: \u{1F600}"`
- F-String: `f"Value is {x}"`
- Tag: `` `json:"id"` ``

---

## Return Type Syntax

**IMPORTANT**: Vex uses `:` for return types, NOT `->`:

```vex
// ✅ CORRECT
fn add(a: i32, b: i32): i32 {
    return a + b;
}

// ❌ INCORRECT (Rust syntax)
fn add(a: i32, b: i32) -> i32 {  // This will NOT parse!
    return a + b;
}
```

This applies everywhere:

- Function declarations: `fn name(): Type`
- Method receivers: `fn (self: &T) method(): Type`
- Trait methods: `fn method(self: &Self): Type`
- Closures: `|x: i32|: i32 { x + 1 }`

**Why?**: Vex chose TypeScript/Go style syntax for consistency with variable declarations (`x: Type`).

---

## Summary of Syntax Differences from Rust

| Feature              | Rust              | Vex                            |
| -------------------- | ----------------- | ------------------------------ |
| Return type          | `fn foo() -> i32` | `fn foo(): i32`                |
| Variable declaration | `let x = 5;`      | `x := 5;` or `let x = 5;`      |
| Mutable pointers     | `*mut T`          | `*T`                           |
| FFI extern           | `extern "C"`      | `extern`                       |
| Method receiver      | `&self`           | `self: &Self`                  |
| Slice syntax         | `&[T]`            | `&[T]` (same)                  |
| Error handling       | `Result<T, E>`    | `T \| error` or `Result<T, E>` |

---

## Compiler Support Status

✅ **Fully Supported**:

- All primitive types
- Structs (with generics)
- Enums (with generics)
- Functions (with generics)
- Methods (receiver syntax)
- Traits and implementations
- Pattern matching (match expressions)
- All control flow (if, for, while, loop, switch)
- FFI (extern blocks)
- Imports/Exports
- Attributes
- Unsafe blocks

⚠️ **Partially Supported**:

- Async/await (syntax parsed, runtime via Tokio FFI)
- Interface implementation (duck-typed, not checked)
- GPU kernels (syntax parsed, codegen in progress)

🚧 **In Progress**:

- Type checking/inference
- Borrow checker
- Trait bounds enforcement
- Generic monomorphization
- Full async runtime integration

---

## Examples

See `examples/` directory for working code samples:

- `calculator.vx` - Basic arithmetic
- `struct_test.vx` - Struct definition and usage
- `enum_constructor_test.vx` - Enum constructors
- `trait_example.vx` - Trait implementation
- `interface_test.vx` - Interface usage
- `generics_test.vx` - Generic functions and types
- `match_simple.vx` - Pattern matching
- `async_test.vx` - Async/await
- `gpu_vector_add.vx` - GPU kernel
- `ffi_malloc_test.vx` - FFI usage

---

**Last Updated**: November 3, 2025  
**Compiler Version**: v0.6 (AST-based)  
**Parser**: vex-parser v0.1  
**Lexer**: vex-lexer (logos-based)
