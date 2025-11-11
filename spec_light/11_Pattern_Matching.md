# Pattern Matching

**Version:** 0.1.0 
**Last Updated:** November 3, 2025

This document defines pattern matching and destructuring in the Vex programming language.

---

## Table of Contents

1. \1
2. \1
3. \1
4. \1
5. \1
6. \1

---

## Match Expression

### Basic Syntax

**Syntax**: `match value { pattern => body }`

``````vex
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

``````vex
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

``````vex
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

``````vex
match x {
    n => {
        // n binds to x's value
    }
}
```

**Example**:

``````vex
match age {
    a => {
        // a = age
        return a * 2;
    }
}
```

### Wildcard Pattern

Match and discard value:

``````vex
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

[11 lines code: ```vex]

**Must be exhaustive**:

``````vex
// ERROR: Missing Blue
match color {
    Red => { }
    Green => { }
}
```

### Or Patterns

Match multiple patterns:

``````vex
match day {
    1 | 2 | 3 | 4 | 5 => { /* weekday */ }
    6 | 7 => { /* weekend */ }
    _ => { /* invalid */ }
}
```

**Syntax**: `pattern1 | pattern2 | ...`

**Examples**:

[10 lines code: ```vex]

---

## Destructuring

### Tuple Destructuring

Extract tuple components:

``````vex
let point = (10, 20);
match point {
    (x, y) => {
        // x = 10, y = 20
    }
}
```

**Multiple Patterns**:

``````vex
match pair {
    (0, 0) => { /* origin */ }
    (0, y) => { /* on y-axis, y is bound */ }
    (x, 0) => { /* on x-axis, x is bound */ }
    (x, y) => { /* general point */ }
}
```

**Ignoring Components**:

``````vex
match triple {
    (x, _, z) => {
        // Only x and z are bound, middle ignored
    }
}
```

### Struct Destructuring

**Status**: ✅ **COMPLETE** (v0.1.2)

Extract struct fields in pattern matching:

[9 lines code: ```vex]

**Nested Destructuring**:

[20 lines code: ```vex]

**Field Renaming**:

``````vex
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

[37 lines code: ```vex]

**Implementation Details**:

- **Parser**: `vex-parser/src/parser/patterns.rs` - Parses `Struct { field1, field2 }` syntax
- **AST**: `vex-ast/src/lib.rs` - `Pattern::Struct { name, fields }`
- **Pattern checking**: `vex-compiler/src/codegen_ast/expressions/pattern_matching.rs`
- **Pattern binding**: Extract field values and bind to variables
- **Test file**: `examples/test_struct_patterns.vx`

**Partial Destructuring** (Future):

``````vex
match person {
    Person { name, .. } => {
        // Only extract name, ignore other fields
    }
}
```

### Array/Slice Destructuring (Future)

``````vex
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

[13 lines code: ```vex]

**Complex Enums**:

[11 lines code: ```vex]

---

## Exhaustiveness Checking

### Requirement

Match expressions must handle all possible cases:

[24 lines code: ```vex]

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

[14 lines code: ```vex]

---

## Pattern Guards

### Definition

Add conditions to patterns:

``````vex
match x {
    n if n < 0 => { /* negative */ }
    n if n == 0 => { /* zero */ }
    n if n > 0 => { /* positive */ }
}
```

**Syntax**: `pattern if condition`

### Complex Guards

``````vex
match pair {
    (x, y) if x == y => { /* equal */ }
    (x, y) if x > y => { /* first larger */ }
    (x, y) => { /* second larger or equal */ }
}
```

### With Enums

``````vex
match option {
    Some(x) if x > 10 => { /* large value */ }
    Some(x) => { /* small value */ }
    None => { /* no value */ }
}
```

---

## Advanced Patterns

### Range Patterns

``````vex
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

``````vex
match &value {
    &x => {
        // x is a reference
    }
}
```

### Nested Patterns (Future)

``````vex
match nested {
    (Point { x, y }, Some(value)) => {
        // Destructure tuple and Point and Option
    }
}
```

---

## Examples

### Basic Match

[13 lines code: ```vex]

### Enum Matching

[13 lines code: ```vex]

### Tuple Destructuring

[16 lines code: ```vex]

### Or Patterns

[10 lines code: ```vex]

---

## Best Practices

### 1. Use Match for Enums

[13 lines code: ```vex]

### 2. Specific Before General

[12 lines code: ```vex]

### 3. Use Destructuring

[15 lines code: ```vex]

### 4. Avoid Deep Nesting

[20 lines code: ```vex]

### 5. Use Wildcard for Defaults

[13 lines code: ```vex]

---

## Pattern Matching Summary

• Pattern Type — Syntax — Status — Example
• ------------ — ---------------------- — -------------------- — ----------------------------
| Literal | `42`, `true`, `"text"` | ✅ Working | Exact value match |
| Variable | `x`, `name` | ✅ Working | Bind to variable |
| Wildcard | `_` | ✅ Working | Match anything |
| Enum | `Red`, `Active` | ✅ Working | Enum variant (no :: syntax) |
| Or | `1 \| 2 \| 3` | ✅ Working | Multiple patterns |
| Tuple | `(x, y)` | ✅ Working | Destructure tuples |
| Struct | `Point { x, y }` | ✅ Complete (v0.1.2) | Destructure structs |
| Array | `[a, b, c]` | ✅ Working | Fixed-size arrays |
| Slice | `[head, ...rest]` | ✅ Working | Rest patterns with `...` |
| Enum Data | `Some(x)`, `None` | ✅ Working | Data-carrying enums working |
| Range | `0..10`, `0..=10` | ✅ Working | Value ranges with .. and ..= |
| Guard | `x if x > 0` | ✅ Working | Conditional patterns |
| Reference | `&x` | 🚧 Future | Match references |

---

**Previous**: \1 
**Next**: \1

**Maintained by**: Vex Language Team
