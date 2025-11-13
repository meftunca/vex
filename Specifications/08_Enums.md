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
