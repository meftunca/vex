# Variables and Constants

**Version:** 0.1.0  
**Last Updated:** November 3, 2025

This document defines the variable and constant declaration system in Vex, including the unified variable syntax introduced in v0.1.

---

## Table of Contents

1. [Variable Declarations](#variable-declarations)
2. [Constant Declarations](#constant-declarations)
3. [Mutability System](#mutability-system)
4. [Shadowing](#shadowing)
5. [Scope and Lifetime](#scope-and-lifetime)
6. [Initialization Rules](#initialization-rules)
7. [Type Annotations](#type-annotations)

---

## Variable Declarations

### Syntax v0.1: Unified Variable System

Vex v0.1 unifies variable declarations with a single `let` keyword and explicit mutability markers:

```vex
let x = 42;              // Immutable (default, Rust-style)
let! counter = 0;        // Mutable (explicit with ! suffix)
```

**Key Changes from Previous Versions**:

- ✅ `let` for immutable variables (default)
- ✅ `let!` for mutable variables (explicit)
- ❌ `mut` keyword **removed** (deprecated in v0.1)
- ❌ `:=` operator **removed** (use `let` instead)

### Immutable Variables

**Syntax**: `let name = value;`

```vex
let age = 30;
let name = "Alice";
let pi = 3.14159;
let is_valid = true;
```

**Properties**:

- Cannot be reassigned after initialization
- Enforced by borrow checker (Phase 1: Immutability)
- Default behavior (Rust-inspired)
- Memory efficient (allows optimizations)

**Example**:

```vex
let x = 42;
// x = 100;  // ERROR: Cannot assign to immutable variable 'x'
```

### Mutable Variables

**Syntax**: `let! name = value;`

```vex
let! counter = 0;
let! balance = 1000.0;
let! status = true;
```

**Properties**:

- Can be reassigned after initialization
- Requires explicit `let!` declaration
- Enforced by borrow checker
- Forces intentional mutability

**Example**:

```vex
let! counter = 0;
counter = counter + 1;    // OK: counter is mutable
counter += 1;             // OK: compound assignment
```

**Reassignment**:

```vex
let! x = 10;
x = 20;              // OK: x is mutable
x = x * 2;           // OK: 40
```

### Multiple Declarations

Declare multiple variables in sequence:

```vex
let x = 10;
let y = 20;
let z = 30;
```

**Tuple Destructuring** (Future):

```vex
let (x, y, z) = (10, 20, 30);
let! (a, b) = (1, 2);
```

---

## Constant Declarations

### Compile-Time Constants

**Syntax**: `const NAME = value;`

```vex
const MAX_SIZE = 100;
const PI = 3.141592653589793;
const APP_NAME = "VexApp";
const DEBUG = true;
```

**Properties**:

- **Compile-Time Evaluation**: Value computed at compile time
- **Immutable**: Cannot be changed at runtime
- **No Type Inference**: Type must be determinable from literal
- **Naming Convention**: SCREAMING_SNAKE_CASE recommended
- **Global Scope**: Can be declared at module level

**Differences from Variables**:

| Feature         | `const`      | `let`      | `let!`     |
| --------------- | ------------ | ---------- | ---------- |
| Mutability      | Never        | No         | Yes        |
| Initialization  | Compile-time | Runtime    | Runtime    |
| Scope           | Any          | Block      | Block      |
| Memory          | Inlined      | Stack/Heap | Stack/Heap |
| Type Annotation | Optional     | Optional   | Optional   |

**Example**:

```vex
const MAX_USERS = 1000;
const MIN_PASSWORD_LENGTH = 8;
const DEFAULT_TIMEOUT = 30.0;

fn validate_users(count: i32): bool {
    return count <= MAX_USERS;
}
```

### Constant Expressions

Constants must be initialized with compile-time constant expressions:

**Allowed**:

```vex
const A = 42;                    // Literal
const B = 10 + 20;               // Arithmetic
const C = 100 * 2;               // Multiplication
const D = true && false;         // Boolean logic
const E = "Hello, " + "World";   // String concatenation (future)
```

**Not Allowed**:

```vex
const X = some_function();       // ERROR: Function calls
const Y = get_value();           // ERROR: Runtime value
let z = 10;
const Z = z + 5;                 // ERROR: Variable reference
```

---

## Mutability System

### Philosophy

Vex follows the **"immutable by default, mutable by choice"** principle:

1. **Safety First**: Immutability prevents accidental modifications
2. **Explicit Intent**: `let!` makes mutability visible
3. **Rust-Inspired**: Similar to Rust's `let` vs `let mut`
4. **Borrow Checker**: Enforces mutability rules at compile time

### Mutability Enforcement

The borrow checker (Phase 1) enforces mutability rules:

```vex
let x = 42;
// x = 100;  // ERROR: Cannot assign to immutable variable 'x'

let! y = 42;
y = 100;      // OK: y declared as mutable with let!
```

**Error Message**:

```
Borrow Checker Error: Cannot assign to immutable variable 'x'
  --> example.vx:3:1
   |
1  | let x = 42;
   |     - variable declared as immutable here
2  |
3  | x = 100;
   | ^^^^^^^ cannot assign to immutable variable
   |
   = help: consider declaring it as mutable: `let! x = 42;`
```

### Mutable References

The mutability of a reference is independent of the mutability of the variable it points to. Vex uses a `!` marker to denote mutable references.

**Syntax**:

- Immutable Reference: `&T`
- Mutable Reference: `&T!`

```vex
let! x = 42;             // Mutable variable
let ref_x: &i32 = &x;    // Immutable reference to a mutable variable

let! y = 100;
let ref_y: &i32! = &y;   // Mutable reference to a mutable variable
```

**Mutability Matrix**:

| Variable Declaration | Reference Type | Can Read? | Can Write to Variable Directly? | Can Write Through Reference? |
| -------------------- | -------------- | --------- | ------------------------------- | ---------------------------- |
| `let x` (immutable)  | `&x`           | ✅        | ❌                              | ❌                           |
| `let! x` (mutable)   | `&x`           | ✅        | ✅                              | ❌                           |
| `let! x` (mutable)   | `&x!`          | ✅        | ✅                              | ✅                           |

This system provides fine-grained control over how data can be accessed and modified, forming a core part of Vex's safety guarantees.

---

## Shadowing

### Definition

Shadowing allows declaring a new variable with the same name as a previous variable:

```vex
let x = 5;
let x = x + 1;    // Shadows previous x
let x = x * 2;    // Shadows again (x is now 12)
```

**Properties**:

- New variable shadows the old one in the same scope
- Old variable becomes inaccessible
- New variable can have different type
- Old variable can have different mutability

### Shadowing vs Mutation

**Shadowing** (creates new variable):

```vex
let x = 5;
let x = x + 1;    // New immutable variable
```

**Mutation** (modifies existing variable):

```vex
let! x = 5;
x = x + 1;        // Modifies existing variable
```

### Type Changes with Shadowing

Shadowing allows changing the type:

```vex
let x = "42";         // x: string
let x = 42;           // x: i32 (shadows previous x)
```

**This is not possible with mutation**:

```vex
let! x = "42";
// x = 42;  // ERROR: Type mismatch (string vs i32)
```

### Scope-Based Shadowing

Inner scopes can shadow outer variables:

```vex
let x = 10;
{
    let x = 20;   // Shadows outer x in this scope
    // x is 20 here
}
// x is 10 here (inner x out of scope)
```

**Example**:

```vex
fn example() {
    let x = 5;

    if true {
        let x = 10;     // Shadows x in if block
        // x is 10 here
    }

    // x is 5 here

    let x = x * 2;      // Shadows x in function scope (now 10)
}
```

---

## Scope and Lifetime

### Block Scope

Variables are scoped to the block they're declared in:

```vex
{
    let x = 42;
    // x is accessible here
}
// x is NOT accessible here (out of scope)
```

**Example**:

```vex
fn main(): i32 {
    let outer = 10;

    {
        let inner = 20;
        // Both outer and inner accessible
    }

    // Only outer accessible here
    // inner is out of scope

    return 0;
}
```

### Function Scope

Function parameters and variables have function scope:

```vex
fn calculate(x: i32, y: i32): i32 {
    let sum = x + y;
    let product = x * y;
    // x, y, sum, product all accessible
    return sum + product;
}
// x, y, sum, product NOT accessible here
```

### Module Scope

Constants and functions can have module scope:

```vex
const MAX_SIZE = 100;  // Module-level constant

fn helper(): i32 {
    return MAX_SIZE;   // Can access module-level const
}

fn main(): i32 {
    return helper();
}
```

### Lifetime (Future Feature)

Lifetimes track how long references are valid:

```vex
fn longest<'a>(x: &'a string, y: &'a string): &'a string {
    if x.len() > y.len() {
        return x;
    } else {
        return y;
    }
}
```

**Status**: Phase 4 of borrow checker (planned)

---

## Initialization Rules

### Must Initialize Before Use

Variables must be initialized before use:

```vex
let x: i32;
// let y = x + 5;  // ERROR: Use of uninitialized variable 'x'

let x = 42;
let y = x + 5;     // OK: x initialized
```

### Initialization in Branches

Variables initialized in all branches can be used:

```vex
let x: i32;
if condition {
    x = 10;
} else {
    x = 20;
}
// x is initialized here (both branches assign)
let y = x + 5;  // OK
```

**Partial Initialization (Error)**:

```vex
let x: i32;
if condition {
    x = 10;
}
// ERROR: x may not be initialized (else branch missing)
// let y = x + 5;  // ERROR
```

### Default Values

Vex does **not** provide default values automatically:

```vex
// No default initialization
let x: i32;  // x is uninitialized (error if used)
```

**Explicit Zero Initialization**:

```vex
let x: i32 = 0;
let y: f64 = 0.0;
let z: bool = false;
let s: string = "";
```

---

## Type Annotations

### Optional Annotations

Type annotations are optional when type can be inferred:

```vex
let x = 42;              // Inferred as i32
let y: i32 = 42;         // Explicit annotation
```

### Required Annotations

Type annotations required when inference fails:

**Empty Collections**:

```vex
// let arr = [];         // ERROR: Cannot infer type
let arr: [i32; 5] = [1, 2, 3, 4, 5];  // OK
```

**Ambiguous Numeric Types**:

```vex
let x: u64 = 100;        // Required: could be u8, u16, u32, or u64
```

**Function Pointers** (Future):

```vex
let f: fn(i32): i32 = some_function;
```

### Syntax

Type annotations follow the colon:

```vex
let name: Type = value;
let! name: Type = value;
const NAME: Type = value;
```

**Examples**:

```vex
let age: i32 = 30;
let! balance: f64 = 1000.0;
const MAX: u64 = 18446744073709551615;
let point: (i32, i32) = (10, 20);
let numbers: [i32; 5] = [1, 2, 3, 4, 5];
```

---

## Examples

### Basic Variables

```vex
fn main(): i32 {
    let x = 10;
    let y = 20;
    let sum = x + y;
    return sum;  // 30
}
```

### Mutable Counter

```vex
fn count_to_ten(): i32 {
    let! counter = 0;
    while counter < 10 {
        counter = counter + 1;
    }
    return counter;  // 10
}
```

### Constants

```vex
const MAX_RETRIES = 3;
const TIMEOUT = 5.0;

fn retry_operation(): bool {
    let! attempts = 0;
    while attempts < MAX_RETRIES {
        if try_operation() {
            return true;
        }
        attempts = attempts + 1;
    }
    return false;
}
```

### Shadowing

```vex
fn transform(): i32 {
    let x = "42";           // x: string
    let x = 42;             // x: i32 (shadowed)
    let x = x * 2;          // x: i32 = 84
    return x;
}
```

### Scope

```vex
fn scoped_example(): i32 {
    let outer = 10;
    {
        let inner = 20;
        let result = outer + inner;  // 30
    }
    // inner not accessible here
    return outer;  // 10
}
```

---

## Comparison with Other Languages

### Vex vs Rust

| Vex v0.1  | Rust            | Description       |
| --------- | --------------- | ----------------- |
| `let x`   | `let x`         | Immutable         |
| `let! x`  | `let mut x`     | Mutable           |
| `const X` | `const X: Type` | Constant          |
| `&T!`     | `&mut T`        | Mutable reference |

### Vex vs Go

| Vex v0.1       | Go             | Description          |
| -------------- | -------------- | -------------------- |
| `let x = 42`   | `x := 42`      | Variable declaration |
| `let! x = 42`  | `var x = 42`   | Mutable variable     |
| `const X = 42` | `const X = 42` | Constant             |

### Vex vs TypeScript

| Vex v0.1       | TypeScript     | Description |
| -------------- | -------------- | ----------- |
| `let x = 42`   | `const x = 42` | Immutable   |
| `let! x = 42`  | `let x = 42`   | Mutable     |
| `const X = 42` | `const X = 42` | Constant    |

---

## Best Practices

### 1. Prefer Immutability

```vex
// Good: Immutable by default
let x = 42;
let y = x * 2;

// Only use mutable when necessary
let! counter = 0;
counter = counter + 1;
```

### 2. Use Descriptive Names

```vex
// Good
let user_count = 42;
let total_price = 99.99;

// Bad
let x = 42;
let y = 99.99;
```

### 3. Initialize Close to Use

```vex
// Good: Initialize when needed
if condition {
    let result = expensive_computation();
    use_result(result);
}

// Bad: Initialize too early
let result = expensive_computation();
if condition {
    use_result(result);
}
```

### 4. Use Constants for Magic Numbers

```vex
// Good
const MAX_BUFFER_SIZE = 1024;
let buffer = allocate(MAX_BUFFER_SIZE);

// Bad
let buffer = allocate(1024);  // What is 1024?
```

### 5. Minimize Mutable State

```vex
// Good: Functional style
fn sum(numbers: [i32; 5]): i32 {
    let result = 0;
    // Use fold/reduce (future)
    return result;
}

// Bad: Excessive mutation
fn sum(numbers: [i32; 5]): i32 {
    let! result = 0;
    let! i = 0;
    while i < 5 {
        result = result + numbers[i];
        i = i + 1;
    }
    return result;
}
```

---

## Summary Table

| Declaration        | Syntax                | Mutable?     | Scope        | When to Use                   |
| ------------------ | --------------------- | ------------ | ------------ | ----------------------------- |
| Immutable Variable | `let x = value`       | No           | Block        | Default choice                |
| Mutable Variable   | `let! x = value`      | Yes          | Block        | When reassignment needed      |
| Constant           | `const X = value`     | No           | Module/Block | Compile-time values           |
| Shadowing          | `let x = ...` (again) | New variable | Block        | Type changes, transformations |

---

**Previous**: [03_Type_System.md](./03_Type_System.md)  
**Next**: [05_Functions_and_Methods.md](./05_Functions_and_Methods.md)

**Maintained by**: Vex Language Team
