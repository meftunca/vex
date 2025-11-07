# Type System

**Version:** 0.9.0  
**Last Updated:** November 3, 2025

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

| Type  | Size     | Range                                                   | Description                     |
| ----- | -------- | ------------------------------------------------------- | ------------------------------- |
| `i8`  | 8 bits   | -128 to 127                                             | 8-bit signed integer            |
| `i16` | 16 bits  | -32,768 to 32,767                                       | 16-bit signed integer           |
| `i32` | 32 bits  | -2,147,483,648 to 2,147,483,647                         | 32-bit signed integer (default) |
| `i64` | 64 bits  | -9,223,372,036,854,775,808 to 9,223,372,036,854,775,807 | 64-bit signed integer           |
| `i128`| 128 bits | -2^127 to 2^127-1                                        | 128-bit signed integer          |

**Default**: Integer literals without type annotation default to `i32`.

**Examples**:

```vex
let small: i8 = 127;
let medium: i16 = 32000;
let normal = 42;           // i32 (default)
let large: i64 = 9223372036854775807;
```

#### Unsigned Integers

| Type  | Size     | Range                           | Description             |
| ----- | -------- | ------------------------------- | ----------------------- |
| `u8`  | 8 bits   | 0 to 255                        | 8-bit unsigned integer  |
| `u16` | 16 bits  | 0 to 65,535                     | 16-bit unsigned integer |
| `u32` | 32 bits  | 0 to 4,294,967,295              | 32-bit unsigned integer |
| `u64` | 64 bits  | 0 to 18,446,744,073,709,551,615 | 64-bit unsigned integer |
| `u128`| 128 bits | 0 to 2^128-1                     | 128-bit unsigned integer |

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

// Length (future)
let len = str.len();

// Indexing (future)
let first_char = str[0];

// Slicing (future)
let substring = str[0..5];
```

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
- `&Type!` - Mutable reference (v0.9 syntax)

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

#### Map Type

Associative arrays with key-value pairs:

**Syntax**: `Map<K, V>` (builtin type)

```vex
let ages: Map<string, i32> = Map::new();
ages.insert("Alice", 30);
ages.insert("Bob", 25);

let alice_age = ages.get("Alice");  // Some(30)
```

**Properties**:

- **Generic**: Parameterized by key and value types
- **Hash-based**: Fast lookup O(1) average case
- **Heap allocated**: Managed by runtime
- **Keys**: Must implement hash and equality

#### Set Type

Collections of unique values:

**Syntax**: `Set<T>` (builtin type)

```vex
let numbers: Set<i32> = Set::new();
numbers.insert(1);
numbers.insert(2);
numbers.insert(1);  // Duplicate, ignored

let has_one = numbers.contains(1);  // true
```

**Properties**:

- **Generic**: Parameterized by element type
- **Unique elements**: No duplicates allowed
- **Hash-based**: Fast membership testing
- **Heap allocated**: Managed by runtime

### Vec Type

Dynamic arrays with growable size:

**Syntax**: `Vec<T>` (builtin type)

```vex
let numbers: Vec<i32> = Vec::new();
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
- **Contiguous**: Elements stored contiguously
- **Cache-friendly**: Better performance than linked lists

**Operations**:

```vex
let v = Vec::new<i32>();     // Create empty Vec
v.push(42);                  // Add element
let val = v.get(0);          // Get element
let len = v.len();           // Get length
v.free();                    // Free memory (manual for now)
```

### Box Type

Heap-allocated single values:

**Syntax**: `Box<T>` (builtin type)

```vex
let boxed = Box::new(42);     // Heap allocate i32
let value = Box::unbox(boxed); // Extract value
```

**Properties**:

- **Heap allocated**: Single value on heap
- **Ownership**: Moves ownership to heap
- **Pointer**: Returns pointer to heap value
- **Manual free**: Requires explicit deallocation

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

let color = Color::Red;
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
    Color::Red => { }
    Color::Green => { }
    Color::Blue => { }
}
```

### Type Aliases

Create alternative names for types:

```vex
type UserID = u64;
type Point2D = (i32, i32);
type Callback = fn(i32): i32;  // Function pointer (future)
```

**Usage**:

```vex
let id: UserID = 12345;
let point: Point2D = (10, 20);
```

**Generic Type Aliases** (Future):

```vex
type Result<T> = (T | error);
type Option<T> = (T | nil);
```

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

Types that can be one of several alternatives:

**Syntax**: `(Type1 | Type2 | ...)`

```vex
type NumberOrString = (i32 | string);

let value: NumberOrString = 42;
let value2: NumberOrString = "hello";
```

**Use Cases**:

- Flexible function parameters
- Error handling with `(T | error)`
- Optional values with `(T | nil)`

**Pattern Matching**:

```vex
match value {
    i when i is i32 => { }      // Type guard (future)
    s when s is string => { }
}
```

### Intersection Types

Types that combine multiple traits:

**Syntax**: `(Trait1 & Trait2 & ...)`

```vex
type Comparable = (Eq & Ord);
type Serializable = (Display & ToString);
```

**Trait Bounds**:

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
fn add(a: i32, b: i32): i32 {
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

| Function                          | Return Type | Description                           |
| --------------------------------- | ----------- | ------------------------------------- |
| `typeof<T>(value: T)`             | `string`    | Get type name                         |
| `type_id<T>(value: T)`            | `u64`       | Get unique numeric type identifier    |
| `type_size<T>(value: T)`          | `u64`       | Get type size in bytes                |
| `type_align<T>(value: T)`         | `u64`       | Get type alignment in bytes           |
| `is_int_type<T>(value: T)`        | `bool`      | Check if value is integer type        |
| `is_float_type<T>(value: T)`      | `bool`      | Check if value is floating-point type |
| `is_pointer_type<T>(value: T)`    | `bool`      | Check if value is pointer type        |

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

### Trait Compatibility

Types are compatible if they implement required traits:

```vex
trait Display {
    fn (self: &Self!) show();
}

fn print_it<T: Display>(value: T) {
    value.show();  // OK: T implements Display
}
```

### Lifetime Compatibility (Future)

References with compatible lifetimes can be used interchangeably:

```vex
fn longest<'a>(x: &'a string, y: &'a string): &'a string {
    if x.len() > y.len() { x } else { y }
}
```

---

## Type System Summary

| Category   | Examples                                       | Size               | Notes               |
| ---------- | -------------------------------------- | ------------------ | ------------------- |
| Integers   | i8, i16, i32, i64, i128, u8, u16, u32, u64, u128 | 1-16 bytes         | Fixed size          |
| Floats     | f16, f32, f64                          | 2-8 bytes          | IEEE 754            |
| Boolean    | bool                                 | 1 byte             | true/false          |
| String     | string                               | 16 bytes (ptr+len) | UTF-8, heap         |
| Arrays     | [T; N]                               | N \* sizeof(T)     | Stack, fixed        |
| Tuples     | (T, U, ...)                          | Sum of sizes       | Stack               |
| References | &T, &T!                              | 8 bytes (64-bit)   | Pointers            |
| Collections| Map<K,V>, Set<T>, Vec<T>              | Variable (heap)    | Dynamic/Hash        |
| Smart Ptrs | Box<T>, Channel<T>                   | 8 bytes (ptr)      | Heap-allocated      |
| Structs    | User-defined                         | Sum + padding      | Nominal             |
| Enums      | User-defined                         | Tag + data         | Discriminated union |

---

**Previous**: [02_Lexical_Structure.md](./02_Lexical_Structure.md)  
**Next**: [04_Variables_and_Constants.md](./04_Variables_and_Constants.md)

**Maintained by**: Vex Language Team
