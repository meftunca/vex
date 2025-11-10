# Vex Language Reference

**Version:** 0.1.2  
**Syntax Version:** 0.1.2  
**Last Updated:** November 10, 2025

This document is the comprehensive syntax reference for the Vex programming language. It is designed to be the authoritative source for AI agents and developers working with Vex.

---

## Table of Contents

1. [Introduction](#introduction)
2. [Lexical Structure](#lexical-structure)
3. [Type System](#type-system)
4. [Variables and Constants](#variables-and-constants)
5. [Functions and Methods](#functions-and-methods)
6. [Control Flow](#control-flow)
7. [Structs](#structs)
8. [Enums](#enums)
9. [Traits](#traits)
10. [Generics](#generics)
11. [Pattern Matching](#pattern-matching)
12. [Error Handling](#error-handling)
13. [Concurrency](#concurrency)
14. [Memory Management](#memory-management)
15. [Modules and Imports](#modules-and-imports)
16. [Policy System](#policy-system)
17. [Standard Library](#standard-library)
18. [Operators](#operators)
19. [CLI Commands and Tools](#cli-commands-and-tools)
20. [Quick Reference](#quick-reference)

---

## Introduction

Vex is a modern systems programming language combining:

- **Rust's Safety**: Memory safety through borrow checking
- **Go's Simplicity**: Clean syntax, goroutines
- **TypeScript's Expressiveness**: Advanced type system

### Key Principles

1. **NO `::` operator** - Use `.` for all member access
2. **NO `mut` keyword** - Use `let!` for mutable variables
3. **NO `->` syntax** - Use `:` for return types
4. **Explicit mutability** - `let!` and `!` suffix on methods/calls

### File Extension

- `.vx` - Vex source files

---

## Lexical Structure

> **See:** [Specifications/02_Lexical_Structure.md](../Specifications/02_Lexical_Structure.md)

### Comments

```vex
// Line comment

/*
 * Block comment
 * Multi-line
 */
```

### Keywords

**Verified in vex-lexer/src/lib.rs:**

#### Declaration Keywords

```
fn          let         const       struct      enum
trait       impl        type        extern      policy
```

#### Control Flow Keywords

```
if          else        elif        for         while
in          match       switch      case        default
return      break       continue    defer       select
```

#### Concurrency Keywords

```
async       await       go          gpu         launch
```

#### Modifier Keywords

```
export      import      from        as          with
unsafe      new         make
```

#### Type Keywords

```
true        false       nil         typeof      infer
where       extends     try
```

**IMPORTANT:**

- ❌ NO `mut` keyword (removed in v0.1)
- ❌ NO `interface` keyword (deprecated, use `trait`)
- ✅ `trait` is the correct keyword for interfaces
- ✅ `defer` is implemented (Go-style cleanup)
- ✅ `elif` is the keyword for else-if chains

### Identifiers

**Syntax:** `[a-zA-Z_][a-zA-Z0-9_]*`

```vex
variable
_private
count_123
camelCase
snake_case
PascalCase
```

**Conventions:**

- Variables/Functions: `snake_case`
- Types/Traits: `PascalCase`
- Constants: `UPPER_SNAKE_CASE`

### Operators

**Arithmetic:**

```
+    -    *    /    %
```

**Comparison:**

```
==   !=   <    <=   >    >=
```

**Logical:**

```
&&   ||   !
```

**Bitwise:**

```
&    |    ^    <<   >>
```

**Assignment:**

```
=    +=   -=   *=   /=   %=
&=   |=   ^=   <<=  >>=
```

**Range:**

```
..        // Exclusive: 0..10
..=       // Inclusive: 0..=10
```

**Member Access:**

```
.         // Dot operator (ONLY WAY to access members)
```

**⚠️ CRITICAL: NO `::` operator exists in Vex!**

```vex
// ✅ CORRECT:
Vec.new()
Option.Some(42)
Color.Red
std.io.println("hello")

// ❌ WRONG (Rust syntax, not Vex):
Vec::new()        // SYNTAX ERROR
Option::Some(42)  // SYNTAX ERROR
Color::Red        // SYNTAX ERROR
```

---

## Type System

> **See:** [Specifications/03_Type_System.md](../Specifications/03_Type_System.md)

### Primitive Types

#### Integer Types

**Signed:**

```vex
i8      // -128 to 127
i16     // -32,768 to 32,767
i32     // -2,147,483,648 to 2,147,483,647 (default)
i64     // -9,223,372,036,854,775,808 to 9,223,372,036,854,775,807
i128    // 128-bit signed
```

**Unsigned:**

```vex
u8      // 0 to 255
u16     // 0 to 65,535
u32     // 0 to 4,294,967,295
u64     // 0 to 18,446,744,073,709,551,615
u128    // 128-bit unsigned
```

#### Floating-Point Types

```vex
f16     // Half precision
f32     // Single precision
f64     // Double precision (default)
```

**Note:** `f128` (quad precision) is not supported due to platform-specific linker limitations with compiler-rt intrinsics.

#### Other Primitives

```vex
bool    // true, false
string  // UTF-8 text
byte    // Alias for u8
nil     // Unit type (empty value)
error   // Error type
```

### Compound Types

#### Arrays

**Fixed-size:**

```vex
let arr: [i32; 5] = [1, 2, 3, 4, 5];
let zeros: [i32; 10] = [0; 10];  // Repeat syntax
```

#### Tuples

```vex
let pair: (i32, string) = (42, "answer");
let triple: (f64, bool, i32) = (3.14, true, 100);
```

#### Slices (Future)

```vex
let s: [i32] = arr[1..4];  // Dynamic-size view
```

### Collection Types

**Verified in standard library:**

```vex
Vec<T>          // Dynamic array
Map<K, V>       // Hash map
Set<T>          // Hash set
Box<T>          // Heap allocation
Channel<T>      // Goroutine communication
```

**Usage:**

```vex
let vec = Vec.new();        // NOT Vec::new()
let map = Map.new();
let boxed = Box.new(42);
```

### Advanced Types

#### Union Types ✅ (v0.1.2)

```vex
type Result = i32 | string | error;
type Nullable<T> = T | nil;

let value: i32 | string = 42;
```

#### Generic Types

```vex
struct Box<T> {
    value: T,
}

enum Option<T> {
    Some(T),
    None,
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

#### Conditional Types (Future)

```vex
type ExtractArray<T> = T extends [infer U] ? U : T;
```

### Type Aliases

```vex
type UserID = u64;
type Callback = fn(i32): bool;
type Point2D = (f64, f64);
```

### Reflection Builtins ✅

**Verified in builtins:**

```vex
typeof(expr)           // Get type as string
type_id(expr)          // Get type ID
type_size(T)           // Size in bytes
type_align(T)          // Alignment in bytes
is_int_type(T)         // Check if integer
is_float_type(T)       // Check if float
is_pointer_type(T)     // Check if pointer
```

---

## Variables and Constants

> **See:** [Specifications/04_Variables_and_Constants.md](../Specifications/04_Variables_and_Constants.md)

### Variable Declarations

**Immutable (default):**

```vex
let x = 42;
let name = "Alice";
let pi = 3.14159;

// x = 100;  // ERROR: Cannot reassign immutable variable
```

**Mutable (explicit `!`):**

```vex
let! counter = 0;
let! balance = 1000.0;

counter = counter + 1;  // ✅ OK
balance = 500.0;        // ✅ OK
```

**⚠️ NO `mut` keyword:**

```vex
// ❌ WRONG (old syntax, removed):
mut counter = 0;

// ✅ CORRECT:
let! counter = 0;
```

### Type Annotations

```vex
let x: i32 = 42;
let name: string = "Bob";
let flag: bool = true;

// Type inference
let y = 42;        // Inferred as i32
let z = 3.14;      // Inferred as f64
```

### Constants

**Compile-time constants:**

```vex
const MAX_SIZE = 1024;
const PI = 3.141592653589793;
const APP_NAME = "VexApp";
const DEBUG = true;
```

**Properties:**

- Evaluated at compile-time
- Immutable (always)
- Can be global scope
- Convention: `SCREAMING_SNAKE_CASE`

### Shadowing

```vex
let x = 5;
let x = x + 1;     // Shadows previous x
let x = "text";    // Different type allowed
```

---

## Functions and Methods

> **See:** [Specifications/05_Functions_and_Methods.md](../Specifications/05_Functions_and_Methods.md)

### Function Syntax

**Basic:**

```vex
fn add(x: i32, y: i32): i32 {
    return x + y;
}

fn greet(name: string) {
    // No return type = returns nil
}
```

**Entry Point:**

```vex
fn main(): i32 {
    return 0;  // Exit code
}
```

### Method Mutability (Hibrit Model)

Vex, metodun tanımlandığı bağlama göre değişen, esnek ve pragmatik bir mutasyon sözdizimi kullanır.

#### Kural 1: Inline Metodlar (Struct ve Trait İçinde)

**Amaç:** Kod tekrarını önlemek ve `struct`/`trait` tanımlarını temiz tutmak.

- **Tanımlama:** Metodun `mutable` olduğu, imzanın sonuna eklenen `!` işareti ile belirtilir. Receiver (`self`) bu stilde implisittir ve yazılmaz.
  - `fn method_name()!`
- **Gövde Erişimi:** Metod gövdesinde, alanlara erişim ve atama doğrudan `self` üzerinden yapılır. `self!` **kullanılmaz**.
  - `self.field = new_value`
- **Çağrı:** `Mutable` metodlar, çağrı anında `!` **kullanılmadan** çağrılır. Derleyici, metodun sadece `let!` ile tanımlanmış `mutable` bir nesne üzerinde çağrıldığını compile-time'da kontrol eder.
  - `object.method_name()`

#### Kural 2: External Metodlar (Golang-Style)

**Amaç:** Metodun hangi veri tipi üzerinde çalıştığını ve `mutable` olup olmadığını receiver tanımında açıkça belirtmek.

- **Tanımlama:** Metodun `mutable` olduğu, receiver tanımındaki `&Type!` ifadesi ile belirtilir. Metod imzasının sonunda `!` **kullanılmaz**.
  - `fn (self: &MyType!) method_name()`
- **Gövde Erişimi:** Metod gövdesinde, alanlara erişim ve atama doğrudan `self` üzerinden yapılır. `self!` **kullanılmaz**.
  - `self.field = new_value`
- **Çağrı:** Çağrı sırasında `!` işareti **kullanılmaz**.
  - `object.method_name()`

### Method Definitions & Calls

#### 1. Inline (Struct İçinde)

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

#### 2. External (Golang-style)

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

---

## Control Flow

> **See:** [Specifications/06_Control_Flow.md](../Specifications/06_Control_Flow.md)

### If/Elif/Else

**Verified: `elif` keyword exists in lexer**

```vex
if condition {
    // body
}

if condition {
    // true branch
} else {
    // false branch
}

if score >= 90 {
    // A
} elif score >= 80 {
    // B
} elif score >= 70 {
    // C
} else {
    // F
}
```

**Properties:**

- Condition must be `bool` type
- Braces always required
- `elif` is the keyword (NOT `else if`)

### Match Expression

```vex
match value {
    pattern1 => { body1 }
    pattern2 => { body2 }
    _ => { default }
}
```

**Literal patterns:**

```vex
match x {
    0 => { /* zero */ }
    1 => { /* one */ }
    _ => { /* other */ }
}
```

**Enum patterns:**

```vex
match color {
    Color.Red => { /* red */ }
    Color.Green => { /* green */ }
    Color.Blue => { /* blue */ }
}
```

**Or patterns:**

```vex
match day {
    1 | 2 | 3 | 4 | 5 => { /* weekday */ }
    6 | 7 => { /* weekend */ }
    _ => { /* invalid */ }
}
```

**Tuple destructuring:**

```vex
match point {
    (0, 0) => { /* origin */ }
    (x, 0) => { /* on x-axis */ }
    (0, y) => { /* on y-axis */ }
    (x, y) => { /* general */ }
}
```

### For Loop

```vex
// C-style for loop
for let i = 0; i < 10; i = i + 1 {
    // body
}

// For-in loop (future)
for item in collection {
    // body
}
```

### While Loop

```vex
while condition {
    // body
}

let! i = 0;
while i < 10 {
    i = i + 1;
}
```

### Defer Statement ✅

**Verified: Statement::Defer exists in AST**

Go-style deferred execution (LIFO order):

```vex
fn main(): i32 {
    defer cleanup();  // Runs when function exits
    defer close_file();

    // Normal code
    return 0;
}  // Executes: close_file(), then cleanup()
```

**Properties:**

- Executes when function exits (return, panic, or end)
- LIFO order (stack-based)
- Useful for resource cleanup

**Example:**

```vex
fn process_file(path: string) {
    let file = open(path);
    defer file.close();  // Always runs, even on error

    // Process file...
    if error {
        return;  // file.close() still runs
    }
}
```

### Break/Continue

```vex
while true {
    if condition {
        break;     // Exit loop
    }
    if other {
        continue;  // Next iteration
    }
}
```

---

## Structs

> **See:** [Specifications/07_Structs_and_Data_Types.md](../Specifications/07_Structs_and_Data_Types.md)

### Basic Syntax

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

### Struct Instantiation

```vex
let p = Point { x: 10, y: 20 };

let person = Person {
    name: "Alice",
    age: 30,
    email: "alice@example.com",
};
```

### Struct Tags ✅

**Verified: Field.tag exists in AST (Go-style backticks)**

```vex
struct User {
    id: u64        `json:"id" db:"pk"`,
    username: string `json:"username" db:"username"`,
    email: string   `json:"email" db:"email"`,
    age: i32        `json:"age"`,
    is_active: bool `json:"is_active"`,
}
```

**⚠️ NOT Rust-style attributes:**

```vex
// ❌ WRONG (Rust syntax):
struct User {
    #[serde(rename = "id")]
    id: u64,
}

// ✅ CORRECT (Vex syntax):
struct User {
    id: u64 `json:"id"`,
}
```

### Generic Structs

```vex
struct Box<T> {
    value: T,
}

struct Pair<T, U> {
    first: T,
    second: U,
}

let int_box = Box<i32> { value: 42 };
let pair = Pair<i32, string> { first: 1, second: "one" };
```

---

## Enums

> **See:** [Specifications/08_Enums.md](../Specifications/08_Enums.md)

### Unit Variants

```vex
enum Color {
    Red,
    Green,
    Blue,
}

let color = Color.Red;  // Use . NOT ::
```

### Discriminant Values

```vex
enum HttpStatus {
    OK = 200,
    NotFound = 404,
    ServerError = 500,
}
```

### Tuple Variants ✅ (v0.1.2)

**Verified: Expression::EnumLiteral has `data: Vec<Expression>`**

**Single value:**

```vex
enum Option<T> {
    Some(T),
    None,
}

let x = Option.Some(42);  // Use . NOT ::
let y = Option.None;
```

**Multiple values:**

```vex
enum IpAddr {
    V4(u8, u8, u8, u8),
    V6(string),
}

let localhost = IpAddr.V4(127, 0, 0, 1);
let google = IpAddr.V4(8, 8, 8, 8);
```

**Pattern matching:**

```vex
match localhost {
    IpAddr.V4(a, b, c, d) => {
        // Extracts all 4 values
    },
    IpAddr.V6(addr) => {
        // Single value
    },
}
```

### Builtin Enums

**Option<T>:**

```vex
enum Option<T> {
    Some(T),
    None,
}

// Usage:
let some_value = Some(42);  // Direct constructor
let nothing = None;
```

**Result<T, E>:**

```vex
enum Result<T, E> {
    Ok(T),
    Err(E),
}

// Usage:
let success = Ok(42);
let failure = Err("error message");
```

---

## Traits

> **See:** [Specifications/09_Traits.md](../Specifications/09_Traits.md)

### Trait Definition

**✅ Use `trait` keyword (NOT `interface`)**

```vex
trait Display {
    fn show();
}

trait Logger {
    fn log(msg: string);     // Immutable contract
    fn clear()!;             // Mutable contract
}
```

### Trait Implementation

**Syntax:** `struct Name impl Trait`

**⚠️ NOT Rust syntax:**

```vex
// ❌ WRONG (Rust syntax):
impl Display for Point { }

// ✅ CORRECT (Vex syntax):
struct Point impl Display {
    // fields and methods
}
```

**⚠️ ALL trait methods MUST be implemented:**

When a struct implements a trait, it MUST provide implementations for ALL methods declared in the trait contract. Missing implementations will result in a compile error.

**Inline (required for trait methods):**

```vex
struct ConsoleLogger impl Logger {
    prefix: string,

    // ALL trait methods MUST be implemented in struct body
    fn log(msg: string) {
        print(self.prefix, ": ", msg);
    }

    fn clear()! {
        // Both methods required - cannot omit clear()!
    }
}
```

**❌ Missing implementation error:**

```vex
// COMPILE ERROR: Missing trait method implementation
struct BrokenLogger impl Logger {
    prefix: string,

    fn log(msg: string) {
        print(msg);
    }
    // ERROR: Missing required method 'clear()!'
}
```

**❌ Trait methods cannot be external:**

```vex
// COMPILE ERROR: Trait method must be in struct body
fn (logger: &ConsoleLogger) log(msg: string) {
    print(msg);
}
```

### Default Methods

Traits can provide default implementations for some methods. Implementing structs can override these or use the default behavior.

```vex
trait Logger {
    fn log(msg: string);  // Required - MUST be implemented

    fn info(msg: string) {  // Default implementation - optional to override
        self.log(msg);
    }

    fn debug(msg: string) {  // Another default - optional
        self.log("[DEBUG] " + msg);
    }
}
```

**Implementation with defaults:**

```vex
struct ConsoleLogger impl Logger {
    prefix: string,

    // Only required method must be implemented
    fn log(msg: string) {
        print(self.prefix, ": ", msg);
    }

    // info() and debug() inherited from trait default
}

// Can override defaults if needed:
struct CustomLogger impl Logger {
    fn log(msg: string) {
        print(msg);
    }

    // Override default implementation
    fn info(msg: string) {
        print("[INFO] ", msg);
    }

    // debug() still uses trait default
}
```

### Trait Bounds (Future)

```vex
fn print_all<T: Display>(items: [T]) {
    // T must implement Display
}
```

---

## Generics

> **See:** [Specifications/10_Generics.md](../Specifications/10_Generics.md)

### Generic Functions

```vex
fn identity<T>(x: T): T {
    return x;
}

let num = identity<i32>(42);
let text = identity("hello");  // Type inferred
```

### Generic Structs

```vex
struct Box<T> {
    value: T,
}

struct Pair<T, U> {
    first: T,
    second: U,
}

let box = Box<i32> { value: 42 };
let pair = Pair<i32, string> { first: 1, second: "one" };
```

### Generic Enums

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

### Type Constraints (Future)

```vex
fn max<T: Ord>(a: T, b: T): T {
    if a > b {
        return a;
    }
    return b;
}
```

---

## Pattern Matching

> **See:** [Specifications/11_Pattern_Matching.md](../Specifications/11_Pattern_Matching.md)

### Match Expression

```vex
match value {
    pattern => { body }
    _ => { default }
}
```

### Pattern Types

**Literals:**

```vex
match x {
    0 => { }
    42 => { }
    _ => { }
}
```

**Enums:**

```vex
match option {
    Some(value) => { /* use value */ }
    None => { }
}
```

**Tuples:**

```vex
match pair {
    (0, 0) => { }
    (x, 0) => { /* x is bound */ }
    (x, y) => { }
}
```

**Or patterns:**

```vex
match x {
    1 | 2 | 3 => { }
    4 | 5 | 6 => { }
    _ => { }
}
```

---

## Error Handling

> **See:** [Specifications/17_Error_Handling.md](../Specifications/17_Error_Handling.md)

### Option<T>

```vex
enum Option<T> {
    Some(T),
    None,
}

fn find_user(id: i32): Option<User> {
    if id == 42 {
        return Some(user);
    }
    return None;
}

// Usage:
match find_user(42) {
    Some(u) => { /* found */ }
    None => { /* not found */ }
}
```

### Result<T, E>

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

// Usage:
match divide(10, 2) {
    Ok(value) => { /* success */ }
    Err(error) => { /* failure */ }
}
```

### Error Propagation (?)

```vex
fn process(): Result<i32, string> {
    let value = risky_operation()?;  // Returns Err if failed
    return Ok(value * 2);
}
```

---

## Concurrency

> **See:** [Specifications/13_Concurrency.md](../Specifications/13_Concurrency.md)

### Goroutines

**Keyword:** `go`

```vex
fn worker(id: i32) {
    // Do work
}

fn main(): i32 {
    go worker(1);  // Spawn goroutine
    go worker(2);
    go worker(3);
    return 0;
}
```

**Status:** ✅ Parser supports, basic runtime implemented

### Async/Await

```vex
async fn fetch_data(url: string): string {
    // Async operation
    return "data";
}

async fn main(): i32 {
    let data = await fetch_data("https://api.example.com");
    return 0;
}
```

### Channels ✅

**MPSC channels fully working:**

```vex
let ch = Channel.new();

// Send
ch.send(42);

// Receive
let value = ch.recv();
```

### GPU Computing

```vex
gpu fn vector_add(a: [f32], b: [f32]): [f32] {
    // Automatically runs on GPU
}

launch vector_add(vec_a, vec_b);
```

---

## Memory Management

> **See:** [Specifications/14_Memory_Management.md](../Specifications/14_Memory_Management.md)

### Ownership

```vex
let x = Point { x: 10, y: 20 };
let y = x;  // Ownership moves to y
// x is no longer valid
```

### Borrowing

**Immutable reference:**

```vex
&T

let x = 42;
let r: &i32 = &x;
```

**Mutable reference:**

```vex
&T!

let! x = 42;
let r: &i32! = &x;
*r = 100;
```

### Borrowing Rules

1. **One mutable reference OR multiple immutable references**
2. **References cannot outlive their referent**
3. **No data races**

```vex
let! x = 42;
let r1: &i32! = &x;
// let r2: &i32! = &x;  // ERROR: Multiple mutable borrows
```

### Raw Pointers

```vex
*T      // Immutable raw pointer
*T!     // Mutable raw pointer

unsafe {
    let ptr: *i32 = &x as *i32;
    let value = *ptr;
}
```

---

## Modules and Imports

> **See:** [Specifications/15_Modules_and_Imports.md](../Specifications/15_Modules_and_Imports.md)

### Import Statements

**Namespace import:**

```vex
import * as io from "io";

fn main(): i32 {
    io.println("Hello");
    return 0;
}
```

**Named imports:**

```vex
import { println, readln } from "io";
import { get, post } from "net/http";

fn main(): i32 {
    println("Hello");
    return 0;
}
```

**Nested modules:**

```vex
import * as http from "net/http";
import { TcpStream } from "net/tcp";
```

### Export Declarations

```vex
// Private (default)
fn internal_helper(): i32 {
    return 42;
}

// Public
export fn public_api(): i32 {
    return internal_helper();
}

export struct Point {
    x: i32,
    y: i32,
}

export const MAX_SIZE = 1024;
```

---

## Policy System

> **See:** [Specifications/20_Policy_System.md](../Specifications/20_Policy_System.md)

### Policy Declaration ✅

**Verified: Policy system fully implemented**

```vex
policy APIModel {
    id `json:"id" db:"pk"`,
    name `json:"name" db:"name"`,
    email `json:"email" db:"email"`,
}
```

### Policy Inheritance

```vex
policy BaseModel {
    id `db:"id" indexed:"true"`,
    created_at `db:"created_at"`,
}

policy APIModel with BaseModel {
    id `json:"id"`,  // Overrides BaseModel
    name `json:"name"`,
}
```

### Struct Application

```vex
struct User with APIModel {
    id: u64,
    name: string,
    email: string,
}
```

### Inline Metadata

```vex
struct User {
    id: u64 `json:"id" db:"pk"`,
    name: string `json:"name" required:"true"`,
}
```

---

## Standard Library

> **See:** [Specifications/16_Standard_Library.md](../Specifications/16_Standard_Library.md)

### Core Modules

**io:**

```vex
import { println, print, readln } from "io";

println("Hello, World!");
let input = readln();
```

**fs:**

```vex
import { read_file, write_file, exists } from "fs";

let content = read_file("data.txt");
write_file("output.txt", content);
```

**net/http:**

```vex
import * as http from "net/http";

let response = http.get("https://api.example.com");
```

**json:**

```vex
import { parse, stringify } from "json";

let data = parse("{\"key\": \"value\"}");
let json_str = stringify(data);
```

### Collections

**Vec<T>:**

```vex
let vec = Vec.new();  // NOT Vec::new()
vec.push(42);
let value = vec.pop();
```

**Map<K,V>:**

```vex
let map = Map.new();
map.insert("key", "value");
let value = map.get("key");
```

**Set<T>:**

```vex
let set = Set.new();
set.insert(42);
let has = set.contains(42);
```

---

## Operators

> **See:** [Specifications/03_Type_System.md#operator-overloading](../Specifications/03_Type_System.md#operator-overloading)

### Operator Overloading

Vex supports **trait-based operator overloading**, allowing custom types to define behavior for built-in operators.

**Supported Operators:**

| Operator | Trait Method | Description      |
| -------- | ------------ | ---------------- |
| `+`      | `add`        | Addition         |
| `-`      | `sub`        | Subtraction      |
| `*`      | `mul`        | Multiplication   |
| `/`      | `div`        | Division         |
| `%`      | `rem`        | Remainder        |
| `==`     | `eq`         | Equality         |
| `!=`     | `ne`         | Inequality       |
| `<`      | `lt`         | Less than        |
| `<=`     | `le`         | Less or equal    |
| `>`      | `gt`         | Greater than     |
| `>=`     | `ge`         | Greater or equal |
| `+=`     | `add_assign` | Add assign       |
| `-=`     | `sub_assign` | Subtract assign  |
| `*=`     | `mul_assign` | Multiply assign  |
| `/=`     | `div_assign` | Divide assign    |
| `%=`     | `rem_assign` | Remainder assign |

**Example:**

```vex
trait Add<Rhs, Output> {
    fn add(rhs: Rhs): Output;
}

struct Point impl Add<Point, Point> {
    x: i32,
    y: i32,

    fn add(rhs: Point): Point {
        return Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        };
    }
}

// Usage:
let p1 = Point { x: 1, y: 2 };
let p2 = Point { x: 3, y: 4 };
let p3 = p1 + p2;  // Point { x: 4, y: 6 }
```

**Builtin Overloads:**

```vex
// String concatenation
let message = "Hello" + " " + "World";  // "Hello World"

// Vector operations
let v1 = Vec.new<i32>();
let v2 = Vec.new<i32>();
let v3 = v1 + v2;  // Concatenates vectors
```

**Properties:**

- ✅ Compile-time checked
- ✅ Type constraints enforced
- ✅ No implicit conversions
- ✅ Borrow checker integrated

### Arithmetic

| Operator | Description    | Example |
| -------- | -------------- | ------- |
| `+`      | Addition       | `a + b` |
| `-`      | Subtraction    | `a - b` |
| `*`      | Multiplication | `a * b` |
| `/`      | Division       | `a / b` |
| `%`      | Modulo         | `a % b` |

### Comparison

| Operator | Description      | Example  |
| -------- | ---------------- | -------- |
| `==`     | Equal            | `a == b` |
| `!=`     | Not equal        | `a != b` |
| `<`      | Less than        | `a < b`  |
| `<=`     | Less or equal    | `a <= b` |
| `>`      | Greater than     | `a > b`  |
| `>=`     | Greater or equal | `a >= b` |

### Logical

| Operator | Description | Example    |
| -------- | ----------- | ---------- |
| `&&`     | Logical AND | `a && b`   |
| `\|\|`   | Logical OR  | `a \|\| b` |
| `!`      | Logical NOT | `!a`       |

### Bitwise

| Operator | Description | Example  |
| -------- | ----------- | -------- |
| `&`      | Bitwise AND | `a & b`  |
| `\|`     | Bitwise OR  | `a \| b` |
| `^`      | Bitwise XOR | `a ^ b`  |
| `<<`     | Left shift  | `a << 2` |
| `>>`     | Right shift | `a >> 2` |

### Assignment

| Operator | Description     | Example  |
| -------- | --------------- | -------- |
| `=`      | Assignment      | `x = 42` |
| `+=`     | Add assign      | `x += 5` |
| `-=`     | Subtract assign | `x -= 5` |
| `*=`     | Multiply assign | `x *= 2` |
| `/=`     | Divide assign   | `x /= 2` |
| `%=`     | Modulo assign   | `x %= 3` |

### Member Access

| Operator | Description       | Example                           |
| -------- | ----------------- | --------------------------------- |
| `.`      | Member access     | `obj.field`, `Type.method()`      |
| `!`      | Mutability marker | `let!`, `method()!`, `self.field` |

**⚠️ NO `::` operator in Vex**

### Range

| Operator | Description     | Example  |
| -------- | --------------- | -------- |
| `..`     | Exclusive range | `0..10`  |
| `..=`    | Inclusive range | `0..=10` |

---

## Quick Reference

### Keywords Cheat Sheet

| Category         | Keywords                                                                              |
| ---------------- | ------------------------------------------------------------------------------------- |
| **Declaration**  | `fn`, `let`, `const`, `struct`, `enum`, `trait`, `impl`, `type`, `policy`             |
| **Control Flow** | `if`, `else`, `elif`, `match`, `for`, `while`, `return`, `break`, `continue`, `defer` |
| **Concurrency**  | `async`, `await`, `go`, `gpu`, `launch`, `select`                                     |
| **Modules**      | `import`, `export`, `from`, `as`                                                      |
| **Literals**     | `true`, `false`, `nil`                                                                |
| **Modifiers**    | `unsafe`, `extern`, `with`                                                            |

### Type Quick Reference

| Category          | Types                                                                |
| ----------------- | -------------------------------------------------------------------- |
| **Integers**      | `i8`, `i16`, `i32`, `i64`, `i128`, `u8`, `u16`, `u32`, `u64`, `u128` |
| **Floats**        | `f16`, `f32`, `f64`                                                  |
| **Other**         | `bool`, `string`, `byte`, `nil`, `error`                             |
| **Collections**   | `Vec<T>`, `Map<K,V>`, `Set<T>`, `Box<T>`, `Channel<T>`               |
| **Builtin Enums** | `Option<T>`, `Result<T,E>`                                           |

### Syntax Comparison: Vex vs Rust

| Feature                | Vex                  | Rust                      |
| ---------------------- | -------------------- | ------------------------- |
| **Mutable variable**   | `let! x = 42;`       | `let mut x = 42;`         |
| **Immutable variable** | `let x = 42;`        | `let x = 42;`             |
| **Member access**      | `Vec.new()`          | `Vec::new()`              |
| **Enum variant**       | `Option.Some(42)`    | `Option::Some(42)`        |
| **Mutable method**     | `fn set()! { }`      | `fn set(&mut self) { }`   |
| **Mutable call**       | `obj.set()!`         | `obj.set()`               |
| **Return type**        | `fn f(): i32 { }`    | `fn f() -> i32 { }`       |
| **Else-if**            | `elif`               | `else if`                 |
| **Interface**          | `trait`              | `trait`                   |
| **Struct tags**        | `` id `json:"id"` `` | `#[serde(rename = "id")]` |

### Common Patterns

**Error handling:**

```vex
match result {
    Ok(value) => { /* success */ }
    Err(error) => { /* failure */ }
}

let value = operation()?;  // Propagate error
```

**Iteration:**

```vex
for let i = 0; i < 10; i = i + 1 {
    // C-style loop
}

for item in collection {  // Future
    // For-each loop
}
```

**Resource cleanup:**

```vex
fn process() {
    let resource = acquire();
    defer resource.release();  // Always runs

    // Use resource...
}
```

**Goroutines:**

```vex
go worker(1);  // Spawn concurrent task
go worker(2);
```

**Async/await:**

```vex
async fn fetch(): string {
    let data = await http.get(url);
    return data;
}
```

---

## CLI Commands and Tools

> **See:** [Specifications/19_Package_Manager.md](../Specifications/19_Package_Manager.md)

### Vex Compiler Commands

**Verified: All commands from `vex --help`**

#### Project Management

```bash
# Create new project
vex new my-project

# Initialize in existing directory
vex init
```

#### Dependency Management

```bash
# Add dependency
vex add github.com/user/math-lib@v1.2.0
vex add github.com/user/math-lib  # Latest version

# Remove dependency
vex remove github.com/user/math-lib

# Update all dependencies
vex update

# List dependencies
vex list

# Clean cache and build artifacts
vex clean
```

#### Building and Running

**Compile:**

```bash
vex compile file.vx                    # Compile to executable
vex compile -o output file.vx          # Custom output name
vex compile -O 3 file.vx               # Optimization level (0-3)
vex compile --simd file.vx             # Enable SIMD optimizations
vex compile --gpu file.vx              # Enable GPU support
vex compile --emit-llvm file.vx        # Emit LLVM IR
vex compile --emit-spirv file.vx       # Emit SPIR-V (GPU functions)
vex compile --locked file.vx           # CI mode (requires valid lock file)
vex compile --json file.vx             # JSON diagnostics (IDE integration)
```

**Run:**

```bash
vex run file.vx                        # Compile and run
vex run file.vx -- arg1 arg2           # Pass arguments to program
vex run -c "print(42)"                 # Execute code from string
vex run --json file.vx                 # JSON diagnostics output
```

#### Code Quality

```bash
# Check syntax without compiling
vex check file.vx

# Format source code
vex format file.vx                     # Print formatted code
vex format -i file.vx                  # Format in-place
vex format --in-place file.vx          # Same as -i
```

### Package Manager (vex-pm)

**Integrated into `vex` command:**

#### Project Structure

```
my-project/
├── vex.json          # Project manifest
├── vex.lock          # Lock file (generated)
├── src/
│   ├── lib.vx        # Library entry point
│   ├── main.vx       # Executable entry point
│   └── mod.vx        # Module declarations
├── tests/            # Test files
├── examples/         # Example code
└── vex-builds/       # Build artifacts (generated)
```

#### Manifest File (vex.json)

```json
{
  "name": "my-project",
  "version": "1.0.0",
  "description": "A Vex project",
  "authors": ["Your Name <you@example.com>"],
  "license": "MIT",

  "dependencies": {
    "github.com/user/math-lib": "v1.2.0",
    "github.com/user/http-client": "^2.0.0"
  },

  "targets": {
    "debug": {
      "opt-level": 0
    },
    "release": {
      "opt-level": 3
    }
  },

  "main": "src/main.vx",

  "bin": {
    "my-app": "src/main.vx",
    "cli-tool": "src/cli.vx"
  },

  "native": {
    "include": ["vendor/include"],
    "libs": ["ssl", "crypto"],
    "flags": ["-O3", "-march=native"]
  }
}
```

#### Dependency Versions

```json
{
  "dependencies": {
    "github.com/user/lib": "v1.2.3", // Exact version
    "github.com/user/lib": "^1.2.0", // Compatible with 1.x
    "github.com/user/lib": "1.0.0..2.0.0", // Range
    "github.com/user/lib": "*" // Latest
  }
}
```

#### Lock File (vex.lock)

```json
{
  "version": "1.0",
  "packages": {
    "github.com/user/math-lib": {
      "version": "v1.2.3",
      "checksum": "abc123...",
      "dependencies": {}
    }
  }
}
```

**Properties:**

- ✅ Automatically generated
- ✅ Should be committed to version control
- ✅ Ensures reproducible builds
- ✅ SHA-256 checksums for security

### Package Manager Features

**Decentralized:**

- No central registry
- Uses Git repositories directly
- Compatible with GitHub, GitLab, etc.

**Fast:**

- Parallel downloads
- Global caching (`~/.vex/cache`)
- Incremental builds

**Secure:**

- SHA-256 checksums
- Lock file verification
- `--locked` flag for CI/CD

**Platform-aware:**

- Automatic platform-specific code selection
- Cross-compilation support
- Native library integration

### Common Workflows

**Start new project:**

```bash
vex new my-app
cd my-app
vex run
```

**Add dependencies:**

```bash
vex add github.com/vex-lang/std-http
vex add github.com/vex-lang/std-json@v2.0.0
```

**Build for production:**

```bash
vex compile -O 3 --simd src/main.vx -o my-app
```

**CI/CD build:**

```bash
vex compile --locked --json src/main.vx
```

**Format entire project:**

```bash
find src -name "*.vx" -exec vex format -i {} \;
```

---

## Critical Reminders

### ⚠️ ABSOLUTE RULES

1. **NO `::` operator** - Use `.` for everything

   ```vex
   ✅ Vec.new()
   ✅ Option.Some(42)
   ✅ Color.Red
   ❌ Vec::new()
   ❌ Option::Some(42)
   ```

2. **NO `mut` keyword** - Use `let!`

   ```vex
   ✅ let! x = 42;
   ❌ mut x = 42;
   ❌ let mut x = 42;
   ```

3. **NO `->` for return types** - Use `:`

   ```vex
   ✅ fn add(x: i32, y: i32): i32 { }
   ❌ fn add(x: i32, y: i32) -> i32 { }
   ```

4. **Use `trait` not `interface`**

   ```vex
   ✅ trait Display { }
   ❌ interface Display { }
   ```

5. **Use `elif` not `else if`**

   ```vex
   ✅ if x { } elif y { } else { }
   ❌ if x { } else if y { } else { }
   ```

6. **Trait implementation syntax** → `struct A impl B`

   ```vex
   ✅ struct Point impl Display { }
   ❌ impl Display for Point { }  // Rust syntax
   ```

7. **Struct tags use backticks, not `#[]`**

   ```vex
   ✅ id: u64 `json:"id"`,
   ❌ #[serde(rename = "id")] id: u64,
   ```

8. **Method mutability uses `!` suffix**

   ```vex
   ✅ fn set()! { self.field = value; }
   ✅ obj.set()!;
   ❌ fn set(&mut self) { }
   ```

9. **Modules use `import`/`export`, NOT Rust's `pub`/`mod`/`use`**
   ```vex
   ✅ import { println } from "io";
   ✅ export fn public_api() { }
   ❌ pub fn public_api() { }
   ❌ mod utils;
   ❌ use std::io;
   ```

### Version Information

- **Language Version:** 0.1.2
- **Syntax Version:** 0.1.2
- **Last Verified:** November 10, 2025

### Sources

This reference is verified against:

- Source code: `vex-lexer/src/lib.rs`, `vex-parser/src/`, `vex-ast/src/lib.rs`
- Specifications: `Specifications/*.md`
- Test suite: `examples/`, `stdlib-tests/`

---

**End of Reference Document**
