# Control Flow

**Version:** 0.1.0 
**Last Updated:** November 3, 2025

This document defines control flow constructs in the Vex programming language.

---

## Table of Contents

1. \1
2. \1
3. \1
4. \1
5. \1
6. \1

---

## Conditional Statements

### If Expression

**Basic Syntax**:

``````vex
if condition {
    // body
}
```

**Properties**:

- Condition must be `bool` type (no implicit conversion)
- Braces are required (no braceless syntax)
- Body is a new scope

**Example**:

``````vex
let x = 10;
if x > 5 {
    // x is greater than 5
}
```

### If-Else

``````vex
if condition {
    // true branch
} else {
    // false branch
}
```

**Answer**: 🔴 `select` statement (High Priority) - Go-style channel selection

``````vex
select {
    case msg = <-ch1:
        println("Received from ch1");
    case ch2 <- value:
        println("Sent to ch2");
    default:
        println("No channel ready");
}
```

Channel implementation ile birlikte gelecek. Şu an channels implement edilmemiş, o yüzden bu dokümanda yok. 13_Concurrency.md'de mention edilmeli.

**Example**:

``````vex
let age = 18;
if age >= 18 {
    // adult
} else {
    // minor
}
```

### If-Elif-Else Chain (v0.1)

Use `elif` for else-if chains:

[9 lines code: ```vex]

**Example**:

[10 lines code: ```vex]

**Note**: `elif` keyword introduced in v0.1 (replaces older `else if` syntax)

### Nested If

``````vex
if outer_condition {
    if inner_condition {
        // nested body
    }
}
```

**Example**:

``````vex
let age = 20;
let has_license = true;

if age >= 18 {
    if has_license {
        // can drive
    }
}
```

### If as Expression (Future)

``````vex
let value = if condition { 10 } else { 20 };
```

---

## Pattern Matching

### Match Expression

**Syntax**:

``````vex
match value {
    pattern1 => { body1 }
    pattern2 => { body2 }
    _ => { default }
}
```

**Properties**:

- Must be exhaustive (all cases covered)
- Evaluates top-to-bottom (first match wins)
- `_` is wildcard pattern (matches anything)

### Literal Patterns

``````vex
match x {
    0 => { /* zero */ }
    1 => { /* one */ }
    2 => { /* two */ }
    _ => { /* other */ }
}
```

**Example**:

[11 lines code: ```vex]

### Enum Patterns

[12 lines code: ```vex]

**Exhaustiveness Check**:

``````vex
match color {
    Color.Red => { }
    Color.Green => { }
    // ERROR: Missing Color.Blue case
}
```

### Or Patterns (v0.1)

Match multiple patterns with `|`:

``````vex
match x {
    1 | 2 | 3 => { /* low */ }
    4 | 5 | 6 => { /* medium */ }
    7 | 8 | 9 => { /* high */ }
    _ => { /* other */ }
}
```

**Example**:

``````vex
match day {
    1 | 2 | 3 | 4 | 5 => { /* weekday */ }
    6 | 7 => { /* weekend */ }
    _ => { /* invalid */ }
}
```

### Tuple Patterns

``````vex
let point = (10, 20);
match point {
    (0, 0) => { /* origin */ }
    (0, y) => { /* on y-axis */ }
    (x, 0) => { /* on x-axis */ }
    (x, y) => { /* general point */ }
}
```

**Destructuring**:

``````vex
let pair = (1, 2);
match pair {
    (a, b) => {
        // a = 1, b = 2
    }
}
```

### Struct Patterns (Future)

``````vex
struct Point { x: i32, y: i32 }

let p = Point { x: 10, y: 20 };
match p {
    Point { x: 0, y: 0 } => { /* origin */ }
    Point { x, y: 0 } => { /* on x-axis, x = 10 */ }
    Point { x, y } => { /* general, x=10, y=20 */ }
}
```

### Range Patterns (Future)

``````vex
match age {
    0..=12 => { /* child */ }
    13..=17 => { /* teen */ }
    18..=64 => { /* adult */ }
    65.. => { /* senior */ }
}
```

### Guards (Future)

Add conditions to patterns:

``````vex
match x {
    n if n < 0 => { /* negative */ }
    n if n == 0 => { /* zero */ }
    n if n > 0 => { /* positive */ }
}
```

### Data-Carrying Enum Patterns (Future)

[10 lines code: ```vex]

---

## Loops

### While Loop

**Syntax**:

``````vex
while condition {
    // body
}
```

**Example**:

``````vex
let! counter = 0;
while counter < 10 {
    counter = counter + 1;
}
```

**Infinite Loop**:

``````vex
while true {
    // runs forever (until break)
}
```

### For Loop

**Syntax**:

``````vex
for variable in start..end {
    // body
}
```

**Range-Based**:

``````vex
for i in 0..10 {
    // i = 0, 1, 2, ..., 9
}
```

**Example**:

``````vex
let! sum = 0;
for i in 1..11 {
    sum = sum + i;
}
// sum = 55 (1+2+...+10)
```

**Inclusive Range**:

``````vex
for i in 0..=10 {
    // i = 0, 1, 2, ..., 10 (includes 10)
}
```

**Operators**:

- `..` - Exclusive range: `0..10` → 0, 1, 2, ..., 9
- `..=` - Inclusive range: `0..=10` → 0, 1, 2, ..., 10

### Loop (Infinite Loop) (Future)

``````vex
loop {
    // runs forever
    if condition {
        break;
    }
}
```

**Equivalent to**:

``````vex
while true {
    // body
}
```

### For-Each (Future)

Iterate over collections:

``````vex
let numbers = [1, 2, 3, 4, 5];
for num in numbers {
    // num = 1, then 2, then 3, ...
}
```

**With Index**:

``````vex
for (index, value) in numbers.enumerate() {
    // index = 0, 1, 2, ...
    // value = 1, 2, 3, ...
}
```

---

## Control Transfer

### Break

Exit from loop early:

``````vex
let! i = 0;
while i < 10 {
    if i == 5 {
        break;  // Exit loop
    }
    i = i + 1;
}
// i = 5
```

**In Match** (Future):

``````vex
while true {
    match get_input() {
        "quit" => { break; }
        cmd => { process(cmd); }
    }
}
```

### Continue

Skip to next iteration:

``````vex
for i in 0..10 {
    if i % 2 == 0 {
        continue;  // Skip even numbers
    }
    // Only odd numbers reach here
}
```

**Example**:

``````vex
let! count = 0;
for i in 1..101 {
    if i % 3 == 0 {
        continue;  // Skip multiples of 3
    }
    count = count + 1;
}
// count = 67 (100 - 33 multiples of 3)
```

### Return

Exit from function:

``````vex
fn find(arr: [i32; 10], target: i32): i32 {
    for i in 0..10 {
        if arr[i] == target {
            return i;  // Found, exit function
        }
    }
    return -1;  // Not found
}
```

**Early Return**:

[9 lines code: ```vex]

### Labeled Breaks (Future)

Break from nested loops:

``````vex
'outer: for i in 0..10 {
    for j in 0..10 {
        if i * j > 50 {
            break 'outer;  // Break outer loop
        }
    }
}
```

---

## Error Handling

### Result Type (Future)

Use union types for error handling:

``````vex
type Result<T> = (T | error);

fn divide(a: i32, b: i32): Result<i32> {
    if b == 0 {
        return "Division by zero";
    }
    return a / b;
}
```

**Pattern Matching on Result**:

[9 lines code: ```vex]

### Option Type (Future)

Represent optional values:

[10 lines code: ```vex]

**Unwrapping**:

``````vex
let result = find([1, 2, 3], 2);
match result {
    index when index is i32 => { /* found at index */ }
    nil => { /* not found */ }
}
```

### Try-Catch (Future Consideration)

``````vex
try {
    let result = risky_operation();
    process(result);
} catch err {
    handle_error(err);
}
```

### Panic

Abort program execution:

[9 lines code: ```vex]

---

## Examples

### If-Elif-Else

[13 lines code: ```vex]

### Match with Enums

[19 lines code: ```vex]

### While Loop

``````vex
fn count_down(n: i32): i32 {
    let! counter = n;
    while counter > 0 {
        counter = counter - 1;
    }
    return counter;  // 0
}
```

### For Loop

[11 lines code: ```vex]

### Break and Continue

[9 lines code: ```vex]

---

## Defer Statement

### Syntax

**Purpose**: Execute code when function exits, regardless of how it exits.

**Status**: ✅ Fully implemented - deferred statements execute in LIFO order on function exit

**Keyword**: `defer`

``````vex
fn example() {
    defer cleanup();  // Executes when function returns
    // ... function body
}
```

### Basic Usage

[10 lines code: ```vex]

### Multiple Defer Statements

**Execution Order**: LIFO (Last In, First Out) - Reverse order of declaration

[14 lines code: ```vex]

### Resource Management

**File Handling**:

[10 lines code: ```vex]

**Memory Management**:

[9 lines code: ```vex]

**Lock Management**:

[9 lines code: ```vex]

### Defer with Closures (Future)

[9 lines code: ```vex]

### Error Handling with Defer

[10 lines code: ```vex]

### Common Patterns

**1. RAII-style Resource Management**:

[11 lines code: ```vex]

**2. Cleanup Stack**:

[12 lines code: ```vex]

**3. Timing and Logging**:

[10 lines code: ```vex]

### Comparison with Other Languages

• Feature — Vex — Go — Rust — C++
• ------------- — ------- — ------- — ------------- — ---------
| **Keyword** | `defer` | `defer` | N/A | N/A |
• **RAII** — Manual — Manual — Automatic — Manual
• **Execution** — On exit — On exit — On drop — On scope
• **Order** — LIFO — LIFO — LIFO (drop) — LIFO
• **Closures** — ✅ Yes — ✅ Yes — ✅ Yes (Drop) — ✅ Lambda

### Implementation Status

- ✅ Keyword reserved (`defer`)
- ✅ Parser support (COMPLETE - Nov 9, 2025)
- ✅ Codegen implemented (LIFO execution)
- ✅ Stack unwinding integration working
- **Priority**: ✅ COMPLETE

**Examples**: See `examples/defer_*.vx` for working demonstrations

---

### Nested Loops

[9 lines code: ```vex]

### Early Return

[21 lines code: ```vex]

---

## Best Practices

### 1. Use Match Over If Chains

[15 lines code: ```vex]

### 2. Prefer Early Returns

[25 lines code: ```vex]

### 3. Avoid Deep Nesting

[19 lines code: ```vex]

### 4. Use Descriptive Conditions

[12 lines code: ```vex]

### 5. Limit Loop Complexity

[15 lines code: ```vex]

---

---

## Select Statement (Future)

### Syntax (Go-style)

**Purpose**: Wait on multiple channel operations

[10 lines code: ```vex]

### Semantics

- **Blocks** until one case is ready
- If multiple cases ready, **randomly** chooses one
- `default` case executes immediately if no channel ready
- Without `default`, blocks forever if no channel ready

### Example: Timeout Pattern

[15 lines code: ```vex]

### Current Status

**Syntax**: ✅ `select` keyword reserved 
**Parser**: 🚧 Partial (keyword recognized, AST node exists) 
**Channels**: ✅ MPSC channels implemented (lock-free ring buffer) 
**Priority**: � Medium (Channel infrastructure complete, select syntax pending)

**Note**: Basic channel operations (`send`, `recv`, `close`) fully working. Multi-channel `select` syntax planned.

See \1 for full concurrency model.

### Switch Statement

C-style switch with integer values:

**Syntax**: `switch value { case val: { } default: { } }`

[18 lines code: ```vex]

**Properties**:

- Only works with integer types (i32, u32, etc.)
- No implicit fallthrough (unlike C)
- Must have `default` case (unlike C)
- Each case must be a compile-time constant

**Differences from C**:

- No fallthrough by default
- Requires `default` case
- Only integer types supported
- No expression cases (use `match` instead)

---

## Control Flow Summary

• Construct — Syntax — Use Case — Status
• ------------ — -------------------------- — -------------------- — ------
| If | `if cond { }` | Simple branching | ✅ |
| If-Else | `if cond { } else { }` | Binary choice | ✅ |
| If-Elif-Else | `if { } elif { } else { }` | Multiple conditions | ✅ |
| Match | `match val { pat => { } }` | Pattern matching | ✅ |
| Switch | `switch val { case ... }` | Integer switching | ✅ |
| While | `while cond { }` | Condition-based loop | ✅ |
| For | `for i in range { }` | Iteration | ✅ |
| Defer | `defer cleanup();` | LIFO cleanup | ✅ |
| Select | `select { case ... }` | Channel multiplexing | ❌ |
| Break | `break;` | Exit loop | ✅ |
| Continue | `continue;` | Skip iteration | ✅ |
| Return | `return value;` | Exit function | ✅ |

---

**Previous**: \1 
**Next**: \1

**Maintained by**: Vex Language Team
