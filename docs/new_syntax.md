# Vex Language Syntax Documentation

> **Complete Syntax Reference** - Based on Compiler, AST, Lexer & Parser Analysis  
> Last Updated: November 3, 2025

This document provides a comprehensive overview of the Vex language syntax as implemented in the current compiler infrastructure.

---

## Table of Contents

1. [Lexical Structure](#1-lexical-structure)
2. [Types](#2-types)
3. [Variables & Constants](#3-variables--constants)
4. [Functions](#4-functions)
5. [Structs](#5-structs)
6. [Enums](#6-enums)
7. [Interfaces & Traits](#7-interfaces--traits)
8. [Generics](#8-generics)
9. [Expressions](#9-expressions)
10. [Statements & Control Flow](#10-statements--control-flow)
11. [Pattern Matching](#11-pattern-matching)
12. [Modules & Imports](#12-modules--imports)
13. [FFI (Foreign Function Interface)](#13-ffi-foreign-function-interface)
14. [Async & Concurrency](#14-async--concurrency)
15. [Attributes](#15-attributes)
16. [Operators](#16-operators)

---

## 1. Lexical Structure

### 1.1 Comments

```vex
// Single-line comment

/*
   Multi-line comment
   Can span multiple lines
*/
```

### 1.2 Keywords

```
fn       let      mut      struct   enum     interface  trait    impl
async    await    go       gpu      launch   try       return   if
else     for      while    in       import   export    from     as
true     false    nil      type     extends  infer     const    unsafe
new      make     switch   case     default  match     select   extern
```

### 1.3 Primitive Type Keywords

```
i8   i16  i32  i64   u8   u16  u32  u64
f32  f64  bool string byte error
```

### 1.4 Operators & Delimiters

```
+    -    *    /    %    =    ==   !=   <    <=   >    >=
&&   ||   !    &    |    ?    ++   --   +=   -=   *=   /=
(    )    {    }    [    ]    ,    ;    :    .    ..   ...
->   =>   :=   _    #
```

### 1.5 Literals

#### Integer Literals

```vex
42          // Decimal
0xFF        // Hexadecimal (not fully implemented yet)
```

#### Float Literals

```vex
3.14        // Standard notation
2.5e10      // Scientific notation (planned)
```

#### String Literals

```vex
"hello"                    // Standard string
"hello\nworld"             // With escape sequences
f"value: {x}"             // Formatted string (f-string)
`json:"id" db:"pk"`       // Struct tags (backtick strings)
```

#### Boolean Literals

```vex
true
false
```

#### Nil Literal

```vex
nil         // Represents absence of value or no error
```

---

## 2. Types

### 2.1 Primitive Types

| Type     | Description      | Size (bits) |
| -------- | ---------------- | ----------- |
| `i8`     | Signed integer   | 8           |
| `i16`    | Signed integer   | 16          |
| `i32`    | Signed integer   | 32          |
| `i64`    | Signed integer   | 64          |
| `u8`     | Unsigned integer | 8           |
| `u16`    | Unsigned integer | 16          |
| `u32`    | Unsigned integer | 32          |
| `u64`    | Unsigned integer | 64          |
| `f32`    | Floating point   | 32          |
| `f64`    | Floating point   | 64          |
| `bool`   | Boolean          | 1           |
| `byte`   | Alias for `u8`   | 8           |
| `string` | UTF-8 string     | Variable    |
| `error`  | Error type       | Variable    |

### 2.2 Named Types

```vex
// Custom struct, enum, or interface name
type UserID = u64;
let user: UserID = 12345;
```

### 2.3 Generic Types

```vex
// Generic type with type arguments
Vec<i32>
Box<string>
Result<T, E>
Cache<string, User>
```

### 2.4 Array Types

```vex
// Fixed-size array: [Type; Size]
[i32; 5]        // Array of 5 integers
[[f32; 3]; 3]   // 3x3 matrix
```

### 2.5 Slice Types

```vex
// Dynamic slice (immutable and mutable)
&[i32]          // Immutable slice
&mut [i32]      // Mutable slice
```

### 2.6 Reference Types

```vex
// References (immutable and mutable)
&T              // Immutable reference
&mut T          // Mutable reference
```

### 2.7 Tuple Types

```vex
// Tuple with multiple types
(i32, string)
(f64, f64, f64)
```

### 2.8 Union Types

```vex
// TypeScript-style union
(i32 | string | nil)
(Result<T> | error)
```

### 2.9 Intersection Types

```vex
// TypeScript-style intersection
(Reader & Writer)
(Serializable & Comparable)
```

### 2.10 Conditional Types

```vex
// Advanced type system feature (TypeScript-inspired)
T extends Array ? ElementType : never
```

### 2.11 Unit Type

```vex
// Represents no value (void)
() or implicitly when no return type specified
```

---

## 3. Variables & Constants

### 3.1 Variable Declaration (Go-style)

```vex
// Type inference with :=
x := 42;
name := "Alice";
coordinates := (10.0, 20.0);
```

```vex
// With explicit type (C-style)
i32 age = 25;
string message = "Hello";
```

### 3.2 Let Bindings (Rust-style)

```vex
// Immutable by default
let x: i32 = 10;

// Mutable
let mut counter: i32 = 0;
counter = counter + 1;
```

### 3.3 Constants

```vex
// Global constants (require explicit types)
const MAX_SIZE: u32 = 1000;
const PI: f64 = 3.14159;
```

### 3.4 Type Inference

```vex
// Vex can infer types in most contexts
let x = 42;              // inferred as i32 (or i64)
let names = ["a", "b"];  // inferred as array
let result = compute();  // inferred from function return type
```

---

## 4. Functions

### 4.1 Basic Function Syntax

```vex
// Simple function
fn add(x: i32, y: i32): i32 {
    return x + y;
}

// No return value (Unit type)
fn print_hello() {
    io.print("Hello\n");
}

// With error return
fn main(): error {
    return nil;
}
```

### 4.2 Generic Functions

```vex
// Generic function with type parameters
fn identity<T>(x: T): T {
    return x;
}

// Multiple type parameters
fn pair<T, U>(first: T, second: U): (T, U) {
    return (first, second);
}
```

### 4.3 Async Functions

```vex
// Async function (returns Future)
async fn fetch_data(url: string): (string | error) {
    let response = await http.get(url);
    return response.body;
}
```

### 4.4 GPU Functions

```vex
// GPU kernel function
gpu fn vector_add(a: &[f32], b: &[f32], result: &mut [f32]) {
    let idx = threadIdx.x + blockIdx.x * blockDim.x;
    result[idx] = a[idx] + b[idx];
}
```

### 4.5 Methods (Receiver Functions)

```vex
struct Vector2 {
    x: f32,
    y: f32,
}

// Immutable method
fn (self: &Vector2) magnitude(): f32 {
    return sqrt(self.x * self.x + self.y * self.y);
}

// Mutable method
fn (self: &mut Vector2) normalize() {
    let mag = self.magnitude();
    self.x = self.x / mag;
    self.y = self.y / mag;
}
```

### 4.6 Function Attributes

```vex
// Inline attribute
#[inline]
fn fast_compute(x: i32): i32 {
    return x * 2;
}

// Conditional compilation
#[cfg(target_os = "linux")]
fn linux_specific() {
    // ...
}

// Always inline
#[inline(always)]
fn critical_path() {
    // ...
}
```

---

## 5. Structs

### 5.1 Struct Definition

```vex
// Simple struct
struct Point {
    x: i32,
    y: i32,
}

// With default values (planned)
struct Config {
    timeout: u32,
    retries: u8,
}
```

### 5.2 Generic Structs

```vex
// Generic struct with type parameters
struct Box<T> {
    value: T,
}

struct Pair<T, U> {
    first: T,
    second: U,
}
```

### 5.3 Struct Tags (Go-style)

```vex
struct User {
    id: u64 `json:"id" db:"user_id"`,
    name: string `json:"name"`,
    email: string `json:"email" validate:"email"`,
}
```

### 5.4 Struct Literals

```vex
// Basic struct literal
let p = Point { x: 10, y: 20 };

// With type arguments
let b = Box<i32> { value: 42 };

// Nested structs
let rect = Rectangle {
    top_left: Point { x: 0, y: 0 },
    bottom_right: Point { x: 100, y: 100 },
};
```

---

## 6. Enums

### 6.1 Unit Enums (C-style)

```vex
// Simple enum without data
enum Color {
    Red,
    Green,
    Blue,
}

// Usage
let c = Color::Red;
```

### 6.2 Data-Carrying Enums (Rust-style)

```vex
// Enum variants with associated data
enum Option<T> {
    Some(T),
    None,
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

### 6.3 Generic Enums

```vex
// Multiple type parameters
enum Either<L, R> {
    Left(L),
    Right(R),
}
```

### 6.4 Enum Pattern Matching

```vex
fn handle_result(r: Result<i32, error>): i32 {
    match r {
        Ok(value) => value,
        Err(e) => 0,
    }
}
```

---

## 7. Interfaces & Traits

### 7.1 Interface Definition

```vex
// Simple interface
interface Writer {
    fn write(data: &[byte]): (usize | error);
}

// Generic interface
interface Cache<K, V> {
    fn get(key: K): (V | nil);
    fn set(key: K, value: V): error;
    fn delete(key: K);
}
```

### 7.2 Trait Definition

```vex
// Rust-style trait
trait Display {
    fn to_string(self: &Self): string;
}

trait Converter<T> {
    fn convert(self: &Self): T;
}
```

### 7.3 Trait Implementation

```vex
struct Point {
    x: i32,
    y: i32,
}

// Implement trait for type
impl Display for Point {
    fn to_string(self: &Point): string {
        return f"Point({self.x}, {self.y})";
    }
}
```

---

## 8. Generics

### 8.1 Generic Type Parameters

```vex
// In structs
struct Container<T> {
    items: [T],
}

// In enums
enum Result<T, E> {
    Ok(T),
    Err(E),
}

// In functions
fn map<T, U>(items: &[T], f: fn(T): U): [U] {
    // ...
}

// In interfaces
interface Iterator<T> {
    fn next(self: &mut Self): (T | nil);
}
```

### 8.2 Multiple Type Parameters

```vex
// Multiple parameters
struct HashMap<K, V> {
    // ...
}

fn zip<T, U>(a: &[T], b: &[U]): [(T, U)] {
    // ...
}
```

---

## 9. Expressions

### 9.1 Literals

```vex
42                          // Integer
3.14                        // Float
"hello"                     // String
f"value: {x}"              // Formatted string
true, false                 // Boolean
nil                         // Nil
```

### 9.2 Identifiers

```vex
x
variable_name
_private
MAX_SIZE
```

### 9.3 Binary Operations

```vex
a + b                       // Addition
a - b                       // Subtraction
a * b                       // Multiplication
a / b                       // Division
a % b                       // Modulo
a == b                      // Equality
a != b                      // Inequality
a < b, a <= b               // Comparison
a > b, a >= b               // Comparison
a && b                      // Logical AND
a || b                      // Logical OR
```

### 9.4 Unary Operations

```vex
-x                          // Negation
!flag                       // Logical NOT
&x                          // Reference
*ptr                        // Dereference
```

### 9.5 Function Calls

```vex
// Regular function call
add(10, 20)

// With generics
identity<i32>(42)

// Nested calls
process(transform(data))
```

### 9.6 Method Calls

```vex
// Method on object
obj.method(args)

// Chained methods
buffer.write(data).flush()
```

### 9.7 Field Access

```vex
// Struct field
point.x
user.profile.name

// Nested access
config.database.connection.timeout
```

### 9.8 Index Access

```vex
// Array/slice indexing
arr[0]
matrix[i][j]

// Map access
map[key]
```

### 9.9 Array Literals

```vex
[1, 2, 3, 4, 5]
["hello", "world"]
[[1, 2], [3, 4]]
```

### 9.10 Tuple Literals

```vex
(1, "hello", true)
(x, y, z)
```

### 9.11 Struct Literals

```vex
// Named fields
Point { x: 10, y: 20 }

// With generics
Box<i32> { value: 42 }

// Nested
Rectangle {
    top_left: Point { x: 0, y: 0 },
    dimensions: Size { w: 100, h: 50 },
}
```

### 9.12 Range Expressions

```vex
0..10                       // Exclusive range
1..=100                     // Inclusive range (planned)
```

### 9.13 Reference & Dereference

```vex
&x                          // Immutable reference
&mut x                      // Mutable reference
*ptr                        // Dereference
```

### 9.14 Await Expression

```vex
await future
await async_function()
```

### 9.15 Go Expression (Concurrency)

```vex
go task(args)               // Spawn concurrent task
```

### 9.16 Try Expression

```vex
try risky_operation()       // Error propagation
```

### 9.17 Match Expression

```vex
match value {
    pattern1 => expression1,
    pattern2 => expression2,
    _ => default,
}
```

### 9.18 Launch Expression (GPU)

```vex
// Launch GPU kernel
launch kernel[blocks, threads](args)
```

### 9.19 New & Make

```vex
// Heap allocation
new(value)

// Slice creation
make([T], size)
make([i32], 100)
```

### 9.20 Type Cast

```vex
expr as Type
value as i64
result as f32
```

### 9.21 Postfix Operations

```vex
x++                         // Post-increment
x--                         // Post-decrement
```

---

## 10. Statements & Control Flow

### 10.1 Expression Statement

```vex
// Expression followed by semicolon
compute();
x + y;
```

### 10.2 Assignment

```vex
// Simple assignment
x = 10;

// Compound assignment
x += 5;
x -= 3;
x *= 2;
x /= 4;
```

### 10.3 Return Statement

```vex
return;                     // Return nothing
return value;               // Return value
return nil;                 // Return nil (no error)
```

### 10.4 If Statement

```vex
// Simple if
if x > 0 {
    print("positive");
}

// If-else
if x > 0 {
    print("positive");
} else {
    print("non-positive");
}

// If-else if-else
if x > 0 {
    print("positive");
} else if x < 0 {
    print("negative");
} else {
    print("zero");
}
```

### 10.5 For Loop

```vex
// C-style for loop
for let i = 0; i < 10; i++ {
    print(i);
}

// For-in loop (range)
for i in 0..10 {
    print(i);
}

// For-in loop (collection)
for item in collection {
    process(item);
}
```

### 10.6 While Loop

```vex
while condition {
    // body
}

// Example
while x < 100 {
    x = x * 2;
}
```

### 10.7 Switch Statement (Go-style)

```vex
switch value {
    case 1:
        print("one");
    case 2, 3:
        print("two or three");
    default:
        print("other");
}

// Type switch (planned)
switch x.(type) {
    case i32:
        print("integer");
    case string:
        print("string");
}
```

### 10.8 Break & Continue

```vex
// Break out of loop
for i in 0..100 {
    if i == 50 {
        break;
    }
}

// Continue to next iteration
for i in 0..100 {
    if i % 2 == 0 {
        continue;
    }
    print(i);
}
```

### 10.9 Block Statement

```vex
{
    let x = 10;
    let y = 20;
    print(x + y);
}
```

### 10.10 Unsafe Block

```vex
unsafe {
    // Unsafe operations
    let ptr = malloc(100);
    free(ptr);
}
```

---

## 11. Pattern Matching

### 11.1 Match Expression

```vex
let result = match value {
    pattern1 => expression1,
    pattern2 => expression2,
    _ => default_expression,
};
```

### 11.2 Pattern Types

#### Wildcard Pattern

```vex
match x {
    _ => default_action(),
}
```

#### Literal Pattern

```vex
match x {
    0 => print("zero"),
    1 => print("one"),
    42 => print("answer"),
    _ => print("other"),
}
```

#### Identifier Binding

```vex
match result {
    x => print(x),  // Bind value to x
}
```

#### Tuple Pattern

```vex
match point {
    (0, 0) => print("origin"),
    (x, 0) => print(f"on x-axis: {x}"),
    (0, y) => print(f"on y-axis: {y}"),
    (x, y) => print(f"point: ({x}, {y})"),
}
```

#### Struct Pattern

```vex
match point {
    Point { x: 0, y: 0 } => print("origin"),
    Point { x, y } => print(f"point: ({x}, {y})"),
}
```

#### Enum Pattern

```vex
match option {
    Some(value) => print(value),
    None => print("no value"),
}

match result {
    Ok(data) => process(data),
    Err(e) => handle_error(e),
}
```

### 11.3 Pattern Guards

```vex
match x {
    n if n > 0 => print("positive"),
    n if n < 0 => print("negative"),
    _ => print("zero"),
}
```

---

## 12. Modules & Imports

### 12.1 Import Syntax

#### Named Imports

```vex
// Import specific items
import { io, log } from "std";
import { HashMap, HashSet } from "std/collections";

// Usage
io.print("hello");
log.info("message");
```

#### Namespace Import

```vex
// Import entire module with alias
import * as std from "std";
import * as collections from "std/collections";

// Usage
std.io.print("hello");
collections.HashMap<K, V>;
```

#### Module Import

```vex
// Import module into scope
import "std/io";
import "std/net";

// All module contents available
```

### 12.2 Export Syntax

```vex
// Export specific items
export { MyStruct, MyFunction, MyInterface };

// Export inline (planned)
export fn public_function() {
    // ...
}

export struct PublicStruct {
    // ...
}
```

---

## 13. FFI (Foreign Function Interface)

### 13.1 Extern Block

```vex
// Declare external C functions
extern "C" {
    fn malloc(size: usize) -> *byte;
    fn free(ptr: *byte);
    fn printf(fmt: *byte, ...) -> i32;
}
```

### 13.2 Attributes on Extern

```vex
// Link attribute
#[link(name = "c")]
extern "C" {
    fn strlen(s: *byte) -> usize;
}

// Platform-specific
#[cfg(unix)]
#[link(name = "pthread")]
extern "C" {
    fn pthread_create(...) -> i32;
}
```

### 13.3 Variadic Functions

```vex
// Variadic C function
extern "C" {
    fn printf(fmt: *byte, ...) -> i32;
}

// Usage
unsafe {
    printf("Value: %d\n".as_ptr(), 42);
}
```

### 13.4 Unsafe FFI Calls

```vex
fn main() {
    unsafe {
        let ptr = malloc(100);
        // use ptr
        free(ptr);
    }
}
```

---

## 14. Async & Concurrency

### 14.1 Async Functions

```vex
// Async function
async fn fetch_user(id: u64): (User | error) {
    let response = await http.get(f"/users/{id}");
    return parse_user(response);
}
```

### 14.2 Await Expression

```vex
// Await async operation
let user = await fetch_user(123);

// In async context
async fn process() {
    let result1 = await operation1();
    let result2 = await operation2();
    return combine(result1, result2);
}
```

### 14.3 Go Keyword (Goroutines)

```vex
// Spawn concurrent task
go task(args);

// Example
fn worker(id: i32) {
    print(f"Worker {id} started");
}

fn main() {
    go worker(1);
    go worker(2);
    go worker(3);
}
```

### 14.4 Select Statement (Async)

```vex
// Wait on multiple async operations
select {
    result = await operation1():
        handle_result1(result);

    data = await operation2():
        handle_result2(data);

    timeout = await timer(5000):
        print("timeout");
}
```

### 14.5 GPU Launch

```vex
// Define GPU kernel
gpu fn vector_add(a: &[f32], b: &[f32], c: &mut [f32]) {
    let idx = threadIdx.x + blockIdx.x * blockDim.x;
    c[idx] = a[idx] + b[idx];
}

// Launch kernel
fn main() {
    let blocks = 256;
    let threads = 256;
    launch vector_add[blocks, threads](a, b, c);
}
```

---

## 15. Attributes

### 15.1 Function Attributes

```vex
// Inline hint
#[inline]
fn small_function() { }

// Always inline
#[inline(always)]
fn critical_function() { }

// Conditional compilation
#[cfg(target_os = "linux")]
fn linux_only() { }

#[cfg(feature = "experimental")]
fn experimental_feature() { }
```

### 15.2 Struct/Type Attributes

```vex
#[repr(C)]
struct CCompatible {
    x: i32,
    y: i32,
}

#[derive(Debug, Clone)]
struct AutoImplemented {
    // ...
}
```

### 15.3 Module Attributes

```vex
#[cfg(unix)]
extern "C" {
    fn unix_specific();
}

#[link(name = "m")]
extern "C" {
    fn sqrt(x: f64) -> f64;
}
```

---

## 16. Operators

### 16.1 Arithmetic Operators

| Operator | Description    | Example |
| -------- | -------------- | ------- |
| `+`      | Addition       | `a + b` |
| `-`      | Subtraction    | `a - b` |
| `*`      | Multiplication | `a * b` |
| `/`      | Division       | `a / b` |
| `%`      | Modulo         | `a % b` |

### 16.2 Comparison Operators

| Operator | Description           | Example  |
| -------- | --------------------- | -------- |
| `==`     | Equal                 | `a == b` |
| `!=`     | Not equal             | `a != b` |
| `<`      | Less than             | `a < b`  |
| `<=`     | Less than or equal    | `a <= b` |
| `>`      | Greater than          | `a > b`  |
| `>=`     | Greater than or equal | `a >= b` |

### 16.3 Logical Operators

| Operator | Description | Example    |
| -------- | ----------- | ---------- |
| `&&`     | Logical AND | `a && b`   |
| `\|\|`   | Logical OR  | `a \|\| b` |
| `!`      | Logical NOT | `!a`       |

### 16.4 Assignment Operators

| Operator | Description          | Example   |
| -------- | -------------------- | --------- |
| `=`      | Assignment           | `x = 10`  |
| `:=`     | Declaration & assign | `x := 10` |
| `+=`     | Add and assign       | `x += 5`  |
| `-=`     | Subtract and assign  | `x -= 3`  |
| `*=`     | Multiply and assign  | `x *= 2`  |
| `/=`     | Divide and assign    | `x /= 4`  |

### 16.5 Increment/Decrement

| Operator | Description    | Example |
| -------- | -------------- | ------- |
| `++`     | Post-increment | `x++`   |
| `--`     | Post-decrement | `x--`   |

### 16.6 Reference Operators

| Operator | Description       | Example  |
| -------- | ----------------- | -------- |
| `&`      | Reference         | `&x`     |
| `&mut`   | Mutable reference | `&mut x` |
| `*`      | Dereference       | `*ptr`   |

### 16.7 Range Operators

| Operator | Description     | Example  |
| -------- | --------------- | -------- |
| `..`     | Exclusive range | `0..10`  |
| `..=`    | Inclusive range | `0..=10` |

### 16.8 Other Operators

| Operator | Description         | Example        |
| -------- | ------------------- | -------------- |
| `.`      | Field/method access | `obj.field`    |
| `::`     | Enum variant access | `Color::Red`   |
| `?`      | Error propagation   | `try_op()?`    |
| `as`     | Type cast           | `x as f64`     |
| `...`    | Variadic (FFI)      | `fn(fmt, ...)` |

### 16.9 Operator Precedence (Highest to Lowest)

1. Field access (`.`), Method call, Array index (`[]`), Function call (`()`)
2. Unary operators (`-`, `!`, `&`, `*`)
3. Multiplicative (`*`, `/`, `%`)
4. Additive (`+`, `-`)
5. Relational (`<`, `<=`, `>`, `>=`)
6. Equality (`==`, `!=`)
7. Logical AND (`&&`)
8. Logical OR (`||`)
9. Assignment (`=`, `+=`, `-=`, etc.)

---

## Appendix: Example Programs

### Hello World

```vex
import { io, log } from "std";

fn main(): error {
    log.info("Vex v0.6 running");
    io.print("Hello, Vex!\n");
    return nil;
}
```

### Struct & Methods

```vex
struct Vector2 {
    x: f32,
    y: f32,
}

fn (self: &Vector2) magnitude(): f32 {
    return sqrt(self.x * self.x + self.y * self.y);
}

fn main(): i32 {
    let v = Vector2 { x: 3.0, y: 4.0 };
    let mag = v.magnitude();
    return 0;
}
```

### Generics

```vex
struct Box<T> {
    value: T,
}

fn identity<T>(x: T): T {
    return x;
}

fn main(): i32 {
    let b = Box<i32> { value: 42 };
    let result = identity<i32>(100);
    return 0;
}
```

### Enums & Pattern Matching

```vex
enum Option<T> {
    Some(T),
    None,
}

fn unwrap_or<T>(opt: Option<T>, default: T): T {
    match opt {
        Some(value) => value,
        None => default,
    }
}

fn main(): i32 {
    let x = Some(42);
    let value = unwrap_or(x, 0);
    return value;
}
```

### Async/Await

```vex
import { io } from "std";

async fn read_file(path: string): (string | error) {
    let content = await io.read_to_string(path);
    return content;
}

async fn main(): error {
    let data = await read_file("config.toml");
    io.print(data);
    return nil;
}
```

### FFI Example

```vex
extern "C" {
    fn printf(fmt: *byte, ...) -> i32;
    fn strlen(s: *byte) -> usize;
}

fn main(): i32 {
    unsafe {
        printf("Hello from Vex!\n".as_ptr());
        let msg = "Vex Language";
        let len = strlen(msg.as_ptr());
        printf("Length: %zu\n".as_ptr(), len);
    }
    return 0;
}
```

---

## Notes on Implementation Status

This syntax documentation reflects the current state of the Vex compiler infrastructure as of November 2025:

- **Fully Implemented**: Basic types, functions, structs, enums, control flow, imports, FFI basics
- **Partially Implemented**: Generics (parsing complete, codegen in progress), async/await (runtime under development)
- **In Development**: Advanced type system features (conditional types, intersection), full trait system
- **Planned**: Some advanced features mentioned in examples

Always refer to the test examples in the `examples/` directory for working code samples.

---

**End of Documentation**
