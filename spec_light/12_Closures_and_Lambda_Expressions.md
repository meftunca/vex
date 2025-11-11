# Closures and Lambda Expressions

**Version:** 0.1.0
**Last Updated:** November 3, 2025

This document defines closures and lambda expressions in the Vex programming language.

---

## Table of Contents

1. \1
2. \1
3. \1
4. \1
5. \1
6. \1

---

## Introduction

Closures are anonymous functions that can capture variables from their surrounding scope. Vex supports three types of closures with different capture semantics, similar to Rust's `Fn`, `FnMut`, and `FnOnce` traits.

### Key Features

- **Automatic Capture Mode Detection**: Compiler determines the appropriate closure trait
- **Borrow Checker Integration**: Full integration with Vex's ownership system
- **Multiple Calling**: Closures can be called multiple times (depending on capture mode)

---

## Closure Syntax

### Basic Syntax

**Syntax**: `|parameters| body` or `|parameters| { statements }`

[11 lines code: ```vex]

### Parameter Types

Parameters can be explicitly typed or inferred:

``````vex
// Explicit types
let add: fn(i32, i32): i32 = |a: i32, b: i32| a + b;

// Inferred types (common)
let multiply = |a, b| a * b;  // Types inferred from usage
```

### Return Types

Closures can return values implicitly or explicitly:

[10 lines code: ```vex]

---

## Capture Modes

Vex closures automatically determine their capture mode based on how they use captured variables:

### Callable (Fn) - Immutable Capture

Closures that only read captured variables:

``````vex
let x = 5;
let y = 10;
let add_to_x = |z| x + z;  // Captures x immutably

// Can be called multiple times
let result1 = add_to_x(3);  // 8
let result2 = add_to_x(7);  // 12
```

### CallableMut (FnMut) - Mutable Capture

Closures that mutate captured variables:

[9 lines code: ```vex]

### CallableOnce (FnOnce) - Move Capture

Closures that take ownership of captured variables:

[9 lines code: ```vex]

---

## Closure Traits

Vex defines three closure traits that correspond to capture modes:

### Callable Trait

``````vex
trait Callable<Args, Return> {
    fn call(args: Args): Return;
}
```

- Immutable capture
- Can be called multiple times
- Implemented by `Fn`-like closures

### CallableMut Trait

``````vex
trait CallableMut<Args, Return> {
    fn call(args: Args): Return;
}
```

- Mutable capture
- Can be called multiple times
- Can modify captured variables
- Implemented by `FnMut`-like closures

### CallableOnce Trait

``````vex
trait CallableOnce<Args, Return> {
    fn (self: Self) call(args: Args): Return;
}
```

- Move capture
- Can only be called once
- Takes ownership of environment
- Implemented by `FnOnce`-like closures

---

## Examples

### Higher-Order Functions

[10 lines code: ```vex]

### Event Handlers

[24 lines code: ```vex]

### Resource Management

[15 lines code: ```vex]

---

## Advanced Usage

### Nested Closures

Closures can be nested and capture from multiple scopes:

[13 lines code: ```vex]

### Closure Composition

[13 lines code: ```vex]

### Async Closures

Closures work with async functions:

[9 lines code: ```vex]

---

## Implementation Details

### Capture Analysis

The compiler performs static analysis to determine closure capture modes:

1. **Variable Usage Tracking**: Tracks how each captured variable is used
2. **Mode Inference**: Determines the most restrictive mode required
3. **Trait Assignment**: Assigns the appropriate closure trait

### Memory Management

- **Stack Allocation**: Closures are typically stack-allocated
- **Reference Counting**: Complex captures use reference counting
- **Move Semantics**: Move captures transfer ownership

### Performance

- **Zero-Cost Abstractions**: Closures compile to efficient machine code
- **Inlined Calls**: Small closures may be inlined by the compiler
- **Minimal Overhead**: Capture environment is optimized for size and speed

---

## Limitations

### Current Restrictions

- **No Generic Closures**: Closures cannot be generic over types
- **Limited Type Inference**: Some complex cases require explicit typing
- **No Closure Methods**: Cannot define methods on closure types

### Future Enhancements

- **Generic Closures**: Support for `|T| -> U` syntax
- **Async Closures**: Dedicated syntax for async closures
- **Closure Methods**: Ability to extend closure types with methods

---

**Previous**: \1
**Next**: \1

**Maintained by**: Vex Language Team 
**License**: MIT
