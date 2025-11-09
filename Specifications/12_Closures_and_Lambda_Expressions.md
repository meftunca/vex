# Closures and Lambda Expressions

**Version:** 0.9.0
**Last Updated:** November 3, 2025

This document defines closures and lambda expressions in the Vex programming language.

---

## Table of Contents

1. [Introduction](#introduction)
2. [Closure Syntax](#closure-syntax)
3. [Capture Modes](#capture-modes)
4. [Closure Traits](#closure-traits)
5. [Examples](#examples)
6. [Advanced Usage](#advanced-usage)

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

```vex
// Simple closure
let add_one = |x| x + 1;

// Multi-parameter closure
let add = |x, y| x + y;

// Block body closure
let complex = |x| {
    let temp = x * 2;
    return temp + 1;
};
```

### Parameter Types

Parameters can be explicitly typed or inferred:

```vex
// Explicit types
let add: fn(i32, i32): i32 = |a: i32, b: i32| a + b;

// Inferred types (common)
let multiply = |a, b| a * b;  // Types inferred from usage
```

### Return Types

Closures can return values implicitly or explicitly:

```vex
// Implicit return
let square = |x| x * x;

// Explicit return
let factorial = |n| {
    if n <= 1 {
        return 1;
    }
    return n * factorial(n - 1);
};
```

---

## Capture Modes

Vex closures automatically determine their capture mode based on how they use captured variables:

### Callable (Fn) - Immutable Capture

Closures that only read captured variables:

```vex
let x = 5;
let y = 10;
let add_to_x = |z| x + z;  // Captures x immutably

// Can be called multiple times
let result1 = add_to_x(3);  // 8
let result2 = add_to_x(7);  // 12
```

### CallableMut (FnMut) - Mutable Capture

Closures that mutate captured variables:

```vex
let! counter = 0;
let increment = || {
    counter = counter + 1;
    return counter;
};

// Can be called multiple times, modifies environment
let val1 = increment();  // 1, counter = 1
let val2 = increment();  // 2, counter = 2
```

### CallableOnce (FnOnce) - Move Capture

Closures that take ownership of captured variables:

```vex
let data = vec![1, 2, 3];
let processor = || {
    // Takes ownership of data
    return data.sum();
};

// Can only be called once
let result = processor();  // Moves data, closure consumed
// processor();  // ERROR: Already moved
```

---

## Closure Traits

Vex defines three closure traits that correspond to capture modes:

### Callable Trait

```vex
trait Callable<Args, Return> {
    fn call(args: Args): Return;
}
```

- Immutable capture
- Can be called multiple times
- Implemented by `Fn`-like closures

### CallableMut Trait

```vex
trait CallableMut<Args, Return> {
    fn call(args: Args): Return;
}
```

- Mutable capture
- Can be called multiple times
- Can modify captured variables
- Implemented by `FnMut`-like closures

### CallableOnce Trait

```vex
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

```vex
fn map_array<T, U>(arr: [T; 5], f: fn(T): U): [U; 5] {
    return [f(arr[0]), f(arr[1]), f(arr[2]), f(arr[3]), f(arr[4])];
}

fn main(): i32 {
    let numbers = [1, 2, 3, 4, 5];
    let doubled = map_array(numbers, |x| x * 2);
    // doubled = [2, 4, 6, 8, 10]
    return 0;
}
```

### Event Handlers

```vex
struct Button {
    label: string,
    on_click: fn(): (),
}

fn create_button(label: string, handler: fn(): ()): Button {
    return Button {
        label: label,
        on_click: handler,
    };
}

fn main(): i32 {
    let! count = 0;
    let button = create_button("Click me", || {
        count = count + 1;
    });

    // Simulate clicks
    button.on_click();  // count = 1
    button.on_click();  // count = 2

    return 0;
}
```

### Resource Management

```vex
fn with_resource<T>(resource: T, operation: fn(T): ()): () {
    defer cleanup(resource);  // Cleanup when done
    operation(resource);
}

fn main(): i32 {
    let file = open_file("data.txt");
    with_resource(file, |f| {
        // Use file
        let content = read_file(f);
        process_content(content);
    });
    // File automatically cleaned up
    return 0;
}
```

---

## Advanced Usage

### Nested Closures

Closures can be nested and capture from multiple scopes:

```vex
fn create_multiplier(factor: i32): fn(i32): i32 {
    return |x| {
        let inner_factor = factor + 1;
        return |y| x * y * inner_factor;
    };
}

fn main(): i32 {
    let multiply_by_3 = create_multiplier(3);
    let result = multiply_by_3(4);  // Returns a closure
    let final_result = result(5);   // 4 * 5 * (3 + 1) = 80
    return final_result;
}
```

### Closure Composition

```vex
fn compose<A, B, C>(f: fn(B): C, g: fn(A): B): fn(A): C {
    return |x| f(g(x));
}

fn main(): i32 {
    let add_one = |x| x + 1;
    let multiply_two = |x| x * 2;

    let add_one_then_double = compose(multiply_two, add_one);
    let result = add_one_then_double(5);  // (5 + 1) * 2 = 12

    return result;
}
```

### Async Closures

Closures work with async functions:

```vex
async fn process_async(data: string): string {
    return data.to_uppercase();
}

async fn main(): i32 {
    let processor = |data| process_async(data);
    let result = await processor("hello");
    return 0;
}
```

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

**Previous**: [11_Pattern_Matching.md](./11_Pattern_Matching.md)
**Next**: [14_Concurrency.md](./14_Concurrency.md)

**Maintained by**: Vex Language Team  
**License**: MIT
