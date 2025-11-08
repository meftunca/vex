# Vex Language - Complete Reference

**Version:** 0.9.2
**Last Updated:** $(date '+%B %-d, %Y')

This document provides complete reference for Vex language syntax, APIs, and implementation details.

## 📖 Table of Contents

1. [Language Syntax](#language-syntax)
2. [Standard Library API](#standard-library-api)
3. [Compiler Internals](#compiler-internals)
4. [Runtime Details](#runtime-details)
5. [Tooling Reference](#tooling-reference)

## 🔤 Language Syntax

### Variables and Mutability

```vex
let x = 42;        // Immutable
let! y = 42;       // Mutable
const MAX = 100;   // Compile-time constant
```

### References

```vex
&T                 // Immutable reference
&T!                // Mutable reference
*ptr               // Dereference
&value             // Reference to value
```

### Functions

```vex
fn add(x: i32, y: i32): i32 {
    return x + y;
}

fn generic<T>(value: T): T {
    return value;
}
```

### Types

```vex
// Primitives
i8 i16 i32 i64 i128 u8 u16 u32 u64 u128
f32 f64 bool char str () (unit)

// Compounds
[i32]                    // Array
(i32, f64, bool)         // Tuple
fn(i32, i32): i32        // Function type

// Collections
Vec<i32>                 // Dynamic array
Map<str, i32>           // Hash map
Set<i32>                // Hash set
Box<i32>                // Heap allocation
Channel<i32>            // MPSC channel
```

### Control Flow

```vex
// If expression
let result = if x > 0 { x } else { -x };

// Match expression
let value = match option {
    Option.Some(v) => v,
    Option.None => 0,
};

// Loops
for i in 0..10 {
    println("{}", i);
}

while condition {
    // loop body
}
```

### Pattern Matching

```vex
match value {
    0 => println("zero"),
    1..10 => println("small"),
    n if n % 2 == 0 => println("even"),
    _ => println("other"),
}
```

### Traits and Generics

```vex
trait Display {
    fn to_string(self: &Self): str;
}

impl Display for i32 {
    fn to_string(self: &i32): str {
        // implementation
    }
}

fn print<T: Display>(value: T) {
    println("{}", value.to_string());
}
```

### Operator Overloading

```vex
trait Add<Rhs, Output> {
    fn add(self: &Self, rhs: Rhs): Output;
}

impl Add<Point, Point> for Point {
    fn add(self: &Point, rhs: Point): Point {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
```

### Concurrency

```vex
// Goroutines
go {
    println("Hello from goroutine");
};

// Channels
let! ch = Channel.new<i32>();
ch.send(42);
let value = ch.recv();
```

### Error Handling

```vex
fn divide(x: i32, y: i32): Result<i32, str> {
    if y == 0 {
        return Result.Err("Division by zero");
    }
    return Result.Ok(x / y);
}

let result = match divide(10, 0) {
    Result.Ok(value) => println("Result: {}", value),
    Result.Err(msg) => println("Error: {}", msg),
};
```

## 📚 Standard Library API

### Collections

#### Vec<T>

```vex
let! vec = Vec.new<i32>();
vec.push(1);
vec.push(2);
vec.push(3);

let len = vec.len();        // 3
let first = vec[0];         // 1
vec.remove(1);              // Remove index 1

for item in vec {
    println("{}", item);
}
```

#### Map<K, V>

```vex
let! map = Map.new<str, i32>();
map.insert("key1", 42);
map.insert("key2", 24);

let value = map.get("key1");  // Option.Some(42)
map.remove("key2");
```

#### Set<T>

```vex
let! set = Set.new<i32>();
set.insert(1);
set.insert(2);
set.insert(1);  // Duplicate, ignored

let contains = set.contains(1);  // true
```

#### Box<T>

```vex
let boxed = Box.new(42);
let value = *boxed;  // Dereference
```

### String Operations

```vex
let s1 = "Hello";
let s2 = "World";
let combined = s1 + " " + s2;  // "Hello World"

let len = combined.len();
let slice = combined[0..5];    // "Hello"
```

### I/O Operations

```vex
// File operations
let content = File.read_text("file.txt");
File.write_text("output.txt", "Hello World");

// Console I/O
println("Hello {}", name);
let input = readln();
```

### Time Operations

```vex
let now = Time.now();
let timestamp = now.unix_timestamp();

Time.sleep(1000);  // Sleep 1 second
```

### Crypto Operations

```vex
// Hashing
let hash = Crypto.sha256("data");

// Encryption
let key = Crypto.generate_key();
let encrypted = Crypto.encrypt(key, "secret");
let decrypted = Crypto.decrypt(key, encrypted);
```

## ⚙️ Compiler Internals

### Borrow Checker Phases

1. **Immutability:** `let` vs `let!` enforcement
2. **Move Semantics:** Prevent use-after-move
3. **Borrow Rules:** Reference aliasing and mutability rules
4. **Lifetime Analysis:** Reference validity across scopes

### Code Generation Pipeline

```
AST → Type Check → Borrow Check → LLVM IR → Optimization → Binary
```

### Memory Management

- **Stack Allocation:** Local variables, function frames
- **Heap Allocation:** Collections, boxed values
- **Automatic Cleanup:** RAII pattern for resources
- **No GC:** Compile-time ownership tracking

## 🏃 Runtime Details

### Async Runtime

```vex
async fn async_task() {
    // Async operations
}

fn main() {
    go async_task();
    // Main continues while async_task runs
}
```

### SIMD Operations

```vex
// Automatic vectorization for array operations
let! arr1 = [1, 2, 3, 4, 5, 6, 7, 8];
let! arr2 = [2, 3, 4, 5, 6, 7, 8, 9];

let result = arr1 + arr2;  // SIMD addition
```

### FFI Support

```vex
extern "C" {
    fn printf(format: *u8, ...);
    fn malloc(size: usize): *u8;
    fn free(ptr: *u8);
}

#[repr(C)]
struct CStruct {
    field1: i32,
    field2: f64,
}
```

## 🛠️ Tooling Reference

### vex Command

```bash
# Run file
~/.cargo/target/debug/vex run examples/hello.vx

# Compile to binary
~/.cargo/target/debug/vex compile examples/hello.vx

# Format code
~/.cargo/target/debug/vex format examples/hello.vx

# Package management
~/.cargo/target/debug/vex new my_project
~/.cargo/target/debug/vex add package@v1.0.0
```

### vexfmt Configuration

```json
{
  "indentation": "spaces",
  "indent_size": 4,
  "max_line_length": 100,
  "brace_style": "same_line"
}
```

### LSP Features

- **Syntax Highlighting:** Full Vex syntax support
- **Error Diagnostics:** Real-time compilation errors
- **Go to Definition:** Symbol navigation
- **Completion:** Intelligent code completion
- **Hover Info:** Type and documentation display

---

*This file is automatically updated by scripts/update_docs.sh*
