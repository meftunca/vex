# Enums (Enumerated Types)

**Version:** 0.1.0 
**Last Updated:** November 3, 2025

This document defines enumerated types (enums) in the Vex programming language.

---

## Table of Contents

1. \1
2. \1
3. \1
4. \1
5. \1
6. \1

---

## Enum Definitions

### Basic Syntax

**Syntax**: `enum Name { variants }`

[11 lines code: ```vex]

**Properties**:

- `enum` keyword
- Name in PascalCase (convention)
- Variants in braces
- Comma-separated variants
- Each variant is a distinct value

### Unit Variants

Simplest form - variants with no associated data:

``````vex
enum Direction {
    North,
    South,
    East,
    West,
}
```

**Usage**:

``````vex
let dir = Direction::North;
let opposite = Direction::South;
```

### Explicit Discriminants

Assign integer values to variants:

[11 lines code: ```vex]

**Auto-Increment**:

``````vex
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

[10 lines code: ```vex]

**Discriminant Values**:

``````vex
enum Status {
    Active = 0,
    Inactive = 1,
    Pending = 2,
}
```

### Data-Carrying Variants (Future)

Variants that hold additional data:

[9 lines code: ```vex]

**Complex Data**:

``````vex
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

[12 lines code: ```vex]

**Multi-Value Tuple Variants** ✅ NEW (v0.1.2):

[16 lines code: ```vex]

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

[10 lines code: ```c]

**Advanced Examples**:

[20 lines code: ```vex]

**Type Constraints**:

- All tuple fields must have concrete types (no inference)
- Generic types are supported: `Some(T)`, `V4(T, T, T, T)`
- Recursive types allowed: `Node(i32, Box<Node>)`
- No tuple size limit (practical limit: 255 fields)

### Struct Variants (Future)

``````vex
enum Shape {
    Circle { radius: f64 },
    Rectangle { width: f64, height: f64 },
    Triangle { base: f64, height: f64 },
}
```

---

## Pattern Matching

### Basic Match

[20 lines code: ```vex]

### Exhaustiveness

Match must cover all variants:

[18 lines code: ```vex]

### Wildcard Pattern

Use `_` to match remaining cases:

``````vex
// Specific pattern
match status {
    Active => { /* handle active */ }
    _ => { /* handle all other cases */ }
}
```

### Or Patterns

Match multiple variants:

``````vex
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

```````vex
enum Option<T> {
    Some(T),
    None,
}

```
let value = Some(42);
match value {
 Some(x) => {
 // x = 42
 }
 None => {
 // No value
 }
}
```````

```

**Named Fields**:

[9 lines code: ```vex]

---

## Methods on Enums

### Inline Methods

Define methods inside enum body:

[21 lines code: ```vex]

### Golang-Style Methods

Define methods outside enum:

[20 lines code: ```vex]

### Associated Functions (Future)

[16 lines code: ```vex]

---

## Generic Enums

### Single Type Parameter

``````vex
enum Option<T> {
    Some(T),
    None,
}

let some_int = Some(42);
let some_str = Some("hello");
let nothing: Option<i32> = None;
```

### Multiple Type Parameters

``````vex
enum Result<T, E> {
    Ok(T),
    Err(E),
}

let success: Result<i32, string> = Ok(42);
let failure: Result<i32, string> = Err("error");
```

### Constraints (Future)

``````vex
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

``````vex
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

``````vex
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

``````vex
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

[19 lines code: ```vex]

**Builtin Constructors**:

``````vex
let value = Some(42);        // Creates Option<i32>
let nothing = None<i32>();   // Explicit type annotation for None
```

**Implementation Status**: ✅ Complete - constructors and pattern matching fully working

### Result Type

Error handling without exceptions with builtin constructors:

[17 lines code: ```vex]

**Builtin Constructors**:

``````vex
let success = Ok(42);                  // Creates Result<i32, E>
let failure = Err("error message");    // Creates Result<T, string>
```

**Implementation Status**: ✅ Complete - constructors and pattern matching fully working

### State Machine

Model states with enums:

[23 lines code: ```vex]

### Event System

[13 lines code: ```vex]

---

## Examples

### Basic Enum

[20 lines code: ```vex]

### Enum with Values

[18 lines code: ```vex]

### Enum with Methods

[32 lines code: ```vex]

### Or Patterns

[24 lines code: ```vex]

---

## Best Practices

### 1. Use Enums for Fixed Sets

[13 lines code: ```vex]

### 2. Prefer Descriptive Names

[15 lines code: ```vex]

### 3. Use Match for Exhaustiveness

[14 lines code: ```vex]

### 4. Group Related Variants

[14 lines code: ```vex]

### 5. Use Methods for Common Operations

[20 lines code: ```vex]

---

## Enum Features Summary

• Feature — Syntax — Status — Example
• --------------- — -------------------- — ----------- — ------------------------
| Unit Variants | `Red, Green, Blue` | ✅ Working | C-style enums |
| Explicit Values | `Active = 0` | ✅ Working | Discriminants |
| Pattern Match | `match enum { }` | ✅ Working | Exhaustive |
| Or Patterns | `A \| B => { }` | ✅ Working | Multiple variants |
• Inline Methods — Inside enum body — ✅ Working — Methods on enums
• Golang Methods — Outside enum — ✅ Working — Separate definition
| Data-Carrying | `Some(T), None` | ✅ Complete | Option/Result work fully |
| Tuple Variants | `Some(T)` (single) | ✅ v0.1.2 | Single value tuples |
| Multi-Tuple | `V4(u8, u8, u8, u8)` | 🚧 Future | Multiple values |
| Struct Variants | `Move { x, y }` | 🚧 Future | Named fields |
| Generic Enums | `Option<T>` | ✅ Complete | Type parameters working |

---

**Previous**: \1 
**Next**: \1

**Maintained by**: Vex Language Team
