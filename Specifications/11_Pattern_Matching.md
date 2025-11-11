# Pattern Matching

**Version:** 0.1.0  
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
    Red => { /* red */ }
    Green => { /* green */ }
    Blue => { /* blue */ }
}
```

**Must be exhaustive**:

```vex
// ERROR: Missing Blue
match color {
    Red => { }
    Green => { }
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
    Active | Pending => { /* in progress */ }
    Inactive => { /* done */ }
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

### Struct Destructuring

**Status**: ✅ **COMPLETE** (v0.1.2)

Extract struct fields in pattern matching:

```vex
struct Point { x: f32, y: f32 }

match point {
    Point { x, y } => {
        // x and y are bound from point.x and point.y
        print(x);
        print(y);
    }
}
```

**Nested Destructuring**:

```vex
struct Line {
    start: Point,
    end: Point
}

match line {
    Line { start, end } => {
        match start {
            Point { x: x1, y: y1 } => {
                match end {
                    Point { x: x2, y: y2 } => {
                        // Access nested fields
                        print(x1);
                        print(y2);
                    }
                };
            }
        };
    }
}
```

**Field Renaming**:

```vex
match point {
    Point { x: px, y: py } => {
        // Bind point.x to px, point.y to py
        print(px);
        print(py);
    }
}
```

**Use Cases**:

- Extract specific fields from structs
- Validate struct values with guards
- Destructure function parameters (future)
- Pattern matching in match expressions

**Examples**:

```vex
fn distance(p: Point): f32 {
    match p {
        Point { x, y } => {
            return (x * x + y * y);  // Simplified distance
        }
    };
}

fn origin_check(p: Point): bool {
    match p {
        Point { x, y } => {
            if x == 0.0 && y == 0.0 {
                return true;
            } else {
                return false;
            };
        }
    };
}

fn quadrant(p: Point): i32 {
    match p {
        Point { x, y } => {
            if x > 0.0 && y > 0.0 {
                return 1;
            } else if x < 0.0 && y > 0.0 {
                return 2;
            } else if x < 0.0 && y < 0.0 {
                return 3;
            } else if x > 0.0 && y < 0.0 {
                return 4;
            } else {
                return 0;  // On axis
            };
        }
    };
}
```

**Implementation Details**:

- **Parser**: `vex-parser/src/parser/patterns.rs` - Parses `Struct { field1, field2 }` syntax
- **AST**: `vex-ast/src/lib.rs` - `Pattern::Struct { name, fields }`
- **Pattern checking**: `vex-compiler/src/codegen_ast/expressions/pattern_matching.rs`
- **Pattern binding**: Extract field values and bind to variables
- **Test file**: `examples/test_struct_patterns.vx`

**Partial Destructuring** (Future):

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
    Some(x) => {
        // x contains the wrapped value
    }
    None => {
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
    Move { x, y } => { /* x, y bound */ }
    Write(text) => { /* text bound */ }
    ChangeColor(r, g, b) => { /* r, g, b bound */ }
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
    Active => { }
    Inactive => { }
    Pending => { }
}

// OK: Wildcard covers remaining
match status {
    Active => { }
    _ => { /* Inactive and Pending */ }
}

// ERROR: Missing Pending
match status {
    Active => { }
    Inactive => { }
}
```

### Compiler Errors

```
Error: Match is not exhaustive
  --> example.vx:10:5
   |
10 |     match status {
   |     ^^^^^ missing Pending
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

### Definition

Add conditions to patterns:

```vex
match x {
    n if n < 0 => { /* negative */ }
    n if n == 0 => { /* zero */ }
    n if n > 0 => { /* positive */ }
}
```

**Syntax**: `pattern if condition`

### Complex Guards

```vex
match pair {
    (x, y) if x == y => { /* equal */ }
    (x, y) if x > y => { /* first larger */ }
    (x, y) => { /* second larger or equal */ }
}
```

### With Enums

```vex
match option {
    Some(x) if x > 10 => { /* large value */ }
    Some(x) => { /* small value */ }
    None => { /* no value */ }
}
```

---

## Advanced Patterns

### Range Patterns

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
        Red => { return 0; }
        Green => { return 1; }
        Blue => { return 2; }
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
    Active => { }
    Inactive => { }
    Pending => { }
}

// Bad: Error-prone if-else chain
if status == Active {
    // ...
} elif status == Inactive {
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

| Pattern Type | Syntax                 | Status               | Example                      |
| ------------ | ---------------------- | -------------------- | ---------------------------- |
| Literal      | `42`, `true`, `"text"` | ✅ Working           | Exact value match            |
| Variable     | `x`, `name`            | ✅ Working           | Bind to variable             |
| Wildcard     | `_`                    | ✅ Working           | Match anything               |
| Enum         | `Red`, `Active`        | ✅ Working           | Enum variant (no :: syntax)  |
| Or           | `1 \| 2 \| 3`          | ✅ Working           | Multiple patterns            |
| Tuple        | `(x, y)`               | ✅ Working           | Destructure tuples           |
| Struct       | `Point { x, y }`       | ✅ Complete (v0.1.2) | Destructure structs          |
| Array        | `[a, b, c]`            | ✅ Working           | Fixed-size arrays            |
| Slice        | `[head, ...rest]`      | ✅ Working           | Rest patterns with `...`     |
| Enum Data    | `Some(x)`, `None`      | ✅ Working           | Data-carrying enums working  |
| Range        | `0..10`, `0..=10`      | ✅ Working           | Value ranges with .. and ..= |
| Guard        | `x if x > 0`           | ✅ Working           | Conditional patterns         |
| Reference    | `&x`                   | 🚧 Future            | Match references             |

---

**Previous**: [10_Generics.md](./10_Generics.md)  
**Next**: [12_Closures_and_Lambda_Expressions.md](./12_Closures_and_Lambda_Expressions.md)

**Maintained by**: Vex Language Team
