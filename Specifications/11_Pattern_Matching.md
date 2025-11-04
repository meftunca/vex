# Pattern Matching

**Version:** 0.9.0  
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
    Color::Red => { /* red */ }
    Color::Green => { /* green */ }
    Color::Blue => { /* blue */ }
}
```

**Must be exhaustive**:

```vex
// ERROR: Missing Color::Blue
match color {
    Color::Red => { }
    Color::Green => { }
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
    Status::Active | Status::Pending => { /* in progress */ }
    Status::Inactive => { /* done */ }
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

### Struct Destructuring (Future)

Extract struct fields:

```vex
struct Point { x: i32, y: i32 }

match point {
    Point { x: 0, y: 0 } => { /* origin */ }
    Point { x, y: 0 } => { /* on x-axis */ }
    Point { x, y } => { /* general */ }
}
```

**Shorthand**:

```vex
match point {
    Point { x, y } => {
        // Binds x and y from point.x and point.y
    }
}
```

**Partial Destructuring**:

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
    Option::Some(x) => {
        // x contains the wrapped value
    }
    Option::None => {
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
    Message::Move { x, y } => { /* x, y bound */ }
    Message::Write(text) => { /* text bound */ }
    Message::ChangeColor(r, g, b) => { /* r, g, b bound */ }
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
    Status::Active => { }
    Status::Inactive => { }
    Status::Pending => { }
}

// OK: Wildcard covers remaining
match status {
    Status::Active => { }
    _ => { /* Inactive and Pending */ }
}

// ERROR: Missing Status::Pending
match status {
    Status::Active => { }
    Status::Inactive => { }
}
```

### Compiler Errors

```
Error: Match is not exhaustive
  --> example.vx:10:5
   |
10 |     match status {
   |     ^^^^^ missing Status::Pending
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

### Definition (Future)

Add conditions to patterns:

```vex
match x {
    n if n < 0 => { /* negative */ }
    n if n == 0 => { /* zero */ }
    n if n > 0 => { /* positive */ }
}
```

**Syntax**: `pattern if condition`

### Complex Guards (Future)

```vex
match pair {
    (x, y) if x == y => { /* equal */ }
    (x, y) if x > y => { /* first larger */ }
    (x, y) => { /* second larger or equal */ }
}
```

### With Enums (Future)

```vex
match option {
    Option::Some(x) if x > 10 => { /* large value */ }
    Option::Some(x) => { /* small value */ }
    Option::None => { /* no value */ }
}
```

---

## Advanced Patterns

### Range Patterns (Future)

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

### At-Patterns (Future)

Bind whole value and destructure:

```vex
match point {
    p @ Point { x: 0, y } => {
        // p is the whole point
        // y is extracted
    }
}
```

**Syntax**: `variable @ pattern`

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
        Color::Red => { return 0; }
        Color::Green => { return 1; }
        Color::Blue => { return 2; }
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
    Status::Active => { }
    Status::Inactive => { }
    Status::Pending => { }
}

// Bad: Error-prone if-else chain
if status == Status::Active {
    // ...
} elif status == Status::Inactive {
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

| Pattern Type | Syntax                 | Status     | Example              |
| ------------ | ---------------------- | ---------- | -------------------- |
| Literal      | `42`, `true`, `"text"` | ✅ Working | Exact value match    |
| Variable     | `x`, `name`            | ✅ Working | Bind to variable     |
| Wildcard     | `_`                    | ✅ Working | Match anything       |
| Enum         | `Color::Red`           | ✅ Working | Enum variant         |
| Or           | `1 \| 2 \| 3`          | ✅ Working | Multiple patterns    |
| Tuple        | `(x, y)`               | ✅ Working | Destructure tuples   |
| Struct       | `Point { x, y }`       | 🚧 Future  | Destructure structs  |
| Array        | `[a, b, c]`            | 🚧 Future  | Fixed-size arrays    |
| Slice        | `[head, ..]`           | 🚧 Future  | Variable-size        |
| Enum Data    | `Some(x)`              | 🚧 Future  | Data-carrying enums  |
| Range        | `0..10`                | 🚧 Future  | Value ranges         |
| Guard        | `x if x > 0`           | 🚧 Future  | Conditional patterns |
| At-pattern   | `p @ Point { }`        | 🚧 Future  | Bind and destructure |
| Reference    | `&x`                   | 🚧 Future  | Match references     |

---

**Previous**: [10_Generics.md](./10_Generics.md)  
**Next**: [12_Memory_Management.md](./12_Memory_Management.md)

**Maintained by**: Vex Language Team
