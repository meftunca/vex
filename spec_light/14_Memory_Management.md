# Memory Management

**Version:** 0.1.2 
**Last Updated:** November 2025

This document defines memory management, ownership, and borrowing in the Vex programming language.

---

## Table of Contents

1. \1
2. \1
3. \1
4. \1
5. \1
6. \1

---

## Ownership System

### Core Principles

Vex uses **ownership-based memory management** without garbage collection:

1. **Each value has exactly one owner**
2. **When the owner goes out of scope, the value is dropped**
3. **Ownership can be transferred (moved)**
4. **Values can be borrowed temporarily**

### Ownership Transfer (Move Semantics)

``````vex
let x = Point { x: 10, y: 20 };
let y = x;  // Ownership moves from x to y
// x is no longer valid!
```

**After Move**:

``````vex
let x = Point { x: 10, y: 20 };
let y = x;
// ERROR: x has been moved
// let z = x;
```

### Copy Types

Some types implement implicit copy (primitives):

``````vex
let x = 42;
let y = x;  // x is copied, not moved
// Both x and y are valid
```

**Copy Types**:

- All integer types: i8-i64, u8-u64
- Floating-point types: f32, f64
- Boolean: bool
- Tuples of copy types: `(i32, i32)`

**Move Types**:

- String: `string`
- Arrays: `[T; N]` (unless T is Copy)
- Structs: All user-defined structs
- Enums: All enums with data

---

## Borrowing Rules

### Immutable Borrowing

**Syntax**: `&T`

``````vex
let x = 42;
let ref_x: &i32 = &x;  // Borrow x immutably
```

**Properties**:

- Can have multiple immutable borrows
- Cannot modify through immutable reference
- Original owner cannot modify while borrowed

**Example**:

``````vex
fn print_value(x: &i32) {
    // Can read x, cannot modify
}

let value = 42;
print_value(&value);
// value still accessible here
```

### Mutable Borrowing

**Syntax**: `&T!` (v0.1 syntax)

``````vex
let! x = 42;
let ref_x: &i32! = &x;  // Borrow x mutably
```

**Properties**:

- Can have only ONE mutable borrow at a time
- Cannot have immutable borrows while mutably borrowed
- Can modify through mutable reference

**Example**:

``````vex
fn increment(x: &i32!) {
    *x = *x + 1;  // Modify through reference
}

let! value = 42;
increment(&value);
// value is now 43
```

### The Core Rule

**"One mutable XOR many immutable"**:

[15 lines code: ```vex]

### Borrowing Examples

**Read-Only Access**:

``````vex
fn calculate_area(rect: &Rectangle): i32 {
    return rect.width * rect.height;
}

let r = Rectangle { width: 10, height: 20 };
let area = calculate_area(&r);
// r still valid
```

**Mutation Through Reference**:

``````vex
fn scale_rectangle(rect: &Rectangle!, factor: i32) {
    rect.width = rect.width * factor;
    rect.height = rect.height * factor;
}

let! r = Rectangle { width: 10, height: 20 };
scale_rectangle(&r, 2);
// r is now { width: 20, height: 40 }
```

---

## Borrow Checker

### Four-Phase System (v0.1.2)

Vex implements a **four-phase borrow checker**:

### Phase 1: Immutability Checking ✅

Enforces `let` vs `let!` semantics:

``````vex
let x = 42;
// x = 100;  // ERROR: Cannot assign to immutable variable

let! y = 42;
y = 100;     // OK: y is mutable
```

**Test Coverage**: 7 tests passing

### Phase 2: Move Semantics ✅

Prevents use-after-move:

``````vex
let point = Point { x: 10, y: 20 };
let moved = point;
// let error = point;  // ERROR: point has been moved
```

**Test Coverage**: 5 tests passing

### Phase 3: Borrow Rules ✅

Enforces reference rules:

``````vex
let! x = 42;
let r1: &i32! = &x;
// let r2: &i32! = &x;  // ERROR: Cannot have two mutable borrows
```

**Test Coverage**: 5 tests passing

### Phase 4: Lifetime Analysis ✅

**Status**: ✅ **COMPLETE** (v0.1.2)

**Purpose**: Track reference validity across scopes and prevent dangling references

Lifetime analysis prevents common memory safety bugs:

- **Dangling references**: References to deallocated memory
- **Use-after-free**: Using memory after it's been freed
- **Return local reference**: Returning references to local variables

**How It Works**:

The lifetime checker tracks:

1. **Variable scopes**: When variables are created and destroyed
2. **Reference tracking**: Which references point to which variables
3. **Scope validation**: Ensures references don't outlive their referents
4. **Return value analysis**: Prevents returning references to locals

**Examples**:

[33 lines code: ```vex]

**Implementation Details**:

- **Checker**: `vex-compiler/src/borrow_checker/lifetimes.rs`
- **Scope tracking**: Maintains variable scope depth (0=global, 1=function, 2+=blocks)
- **Reference map**: Tracks which references point to which variables
- **Global variables**: Extern functions and constants never go out of scope
- **Builtin registry**: Identifies builtin functions for special handling
- **Test file**: `examples/test_lifetimes.vx`

**Test Coverage**: 8+ tests passing (v0.1.2)

### Borrow Checker Errors

**Immutability Violation**:

[10 lines code: (unknown)]

**Use After Move**:

[10 lines code: (unknown)]

**Multiple Mutable Borrows**:

```
Borrow Checker Error: Cannot borrow 'x' as mutable more than once
  --> example.vx:3:17
   |
2  | let r1: &i32! = &x;
   |                 -- first mutable borrow occurs here
3  | let r2: &i32! = &x;
   |                 ^^ second mutable borrow occurs here
```

---

## Lifetimes

### Concept (Phase 4 - Future)

Lifetimes track how long references are valid:

``````vex
fn example<'a>(x: &'a i32): &'a i32 {
    return x;  // Returned reference lives as long as input
}
```

### Lifetime Annotations

Vex automatically infers lifetimes in all cases, so explicit annotations are rarely needed.

---

## Memory Layout

### Stack Allocation

Most values allocated on stack:

``````vex
let x = 42;            // Stack: 4 bytes
let point = Point {    // Stack: 8 bytes (2 × i32)
    x: 10,
    y: 20,
};
```

**Stack Properties**:

- Fast allocation/deallocation
- Automatic cleanup (scope-based)
- Limited size
- LIFO (Last In, First Out)

### Heap Allocation (Future)

Dynamic allocation for variable-size data:

``````vex
let buffer = Box::new([0; 1024]);  // Heap allocation
let text = String::from("hello");  // Heap string
```

**Heap Properties**:

- Slower than stack
- Manual management (ownership)
- Unlimited size (system dependent)
- Fragmentation possible

### Memory Alignment

Types align to natural boundaries:

``````vex
struct Example {
    a: i8,    // 1 byte, aligned to 1
    b: i32,   // 4 bytes, aligned to 4
    c: i16,   // 2 bytes, aligned to 2
}
// Size: 12 bytes (with padding)
```

**Alignment Rules**:

- i8: 1-byte alignment
- i16: 2-byte alignment
- i32: 4-byte alignment
- i64: 8-byte alignment
- f32: 4-byte alignment
- f64: 8-byte alignment

---

## Resource Management

### RAII Pattern (Future)

Resources tied to object lifetime:

[10 lines code: ```vex]

### Manual Cleanup

Current approach - explicit cleanup:

``````vex
fn process_file(path: string) {
    let file = open_file(path);
    // Use file
    close_file(file);  // Manual cleanup
}
```

### Defer Statement (Future - Go-style)

``````vex
fn process() {
    let file = open("data.txt");
    defer close(file);  // Executes when function returns

    // Use file
    // close(file) called automatically
}
```

---

## Best Practices

### 1. Prefer Immutable Bindings

``````vex
// Good: Immutable by default
let x = 42;
let data = load_data();

// Only use mutable when necessary
let! counter = 0;
counter = counter + 1;
```

### 2. Use References for Large Data

[9 lines code: ```vex]

### 3. Borrow, Don't Move

[13 lines code: ```vex]

### 4. Minimize Mutable State

[11 lines code: ```vex]

### 5. Clear Ownership

[13 lines code: ```vex]

---

## Memory Management Summary

• Feature — Status — Description
• ----------------------- — ------------ — ---------------------------
• **Ownership** — ✅ Working — Each value has one owner
• **Move Semantics** — ✅ Phase 2 — Transfer ownership
• **Copy Types** — ✅ Working — Primitive types auto-copy
| **Immutable Borrow** | ✅ Phase 3 | `&T` reference |
| **Mutable Borrow** | ✅ Phase 3 | `&T!` reference |
• **Borrow Checker** — ✅ Phase 1-4 — Compile-time checking
• **Lifetimes** — ✅ Phase 4 — Reference validity tracking
• **Drop Trait** — ❌ Future — RAII destructors
• **Box Type** — ❌ Future — Heap allocation
• **Reference Counting** — ❌ Future — Rc/Arc types
• **Interior Mutability** — ❌ Future — Cell/RefCell

### Test Coverage

- **Phase 1 (Immutability)**: 7/7 tests passing ✅
- **Phase 2 (Move Semantics)**: 5/5 tests passing ✅
- **Phase 3 (Borrow Rules)**: 5/5 tests passing ✅
- **Phase 4 (Lifetimes)**: 5/5 tests passing ✅ (v0.1.2)
- **Total**: 22/22 borrow checker tests passing (100%)

---

## Examples

### Ownership Transfer

``````vex
fn main(): i32 {
    let x = Point { x: 10, y: 20 };
    let y = x;  // x moved to y
    // x is invalid now
    return y.x;  // 10
}
```

### Immutable Borrowing

[11 lines code: ```vex]

### Mutable Borrowing

[9 lines code: ```vex]

### Borrow Checker Error

``````vex
fn main(): i32 {
    let x = 42;
    x = 100;  // ERROR: Cannot assign to immutable variable
    return 0;
}
```

---

**Previous**: \1 
**Next**: \1

**Maintained by**: Vex Language Team
