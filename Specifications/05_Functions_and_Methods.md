# Functions and Methods

**Version:** 0.9.0  
**Last Updated:** November 3, 2025

This document defines functions, methods, and related concepts in the Vex programming language.

---

## Table of Contents

1. [Function Declarations](#function-declarations)
2. [Method Definitions](#method-definitions)
3. [Parameters and Arguments](#parameters-and-arguments)
4. [Return Values](#return-values)
5. [Generic Functions](#generic-functions)
6. [Function Overloading](#function-overloading)
7. [Higher-Order Functions](#higher-order-functions)
8. [Special Function Types](#special-function-types)

---

## Function Declarations

### Basic Syntax

**Syntax**: `fn name(parameters): return_type { body }`

```vex
fn add(x: i32, y: i32): i32 {
    return x + y;
}

fn greet(name: string) {
    // No return type = returns nil (unit)
}

fn main(): i32 {
    return 0;  // Entry point
}
```

**Components**:

- `fn` keyword
- Function name (identifier)
- Parameter list in parentheses
- Optional return type after colon
- Function body in braces

### Simple Functions

**No Parameters**:

```vex
fn hello(): i32 {
    return 42;
}
```

**No Return Value** (returns nil):

```vex
fn print_message() {
    // Implicit return nil
}
```

**Single Expression** (explicit return required):

```vex
fn double(x: i32): i32 {
    return x * 2;
}
```

### Function Naming

**Conventions**:

- `snake_case` for function names
- Descriptive names preferred
- Verbs for actions: `calculate_sum`, `print_result`
- Predicates start with `is_`, `has_`, `can_`: `is_valid`, `has_error`

**Examples**:

```vex
fn calculate_total(items: [i32; 10]): i32 { }
fn is_prime(n: i32): bool { }
fn get_user_name(): string { }
fn validate_input(data: string): bool { }
```

### Entry Point

The `main` function is the program entry point:

```vex
fn main(): i32 {
    return 0;  // Exit code
}
```

**Properties**:

- Must return `i32` (exit code)
- No parameters (command-line args future feature)
- Program execution starts here
- Return 0 for success, non-zero for error

---

## Method Definitions

Vex supports two method definition styles:

### 1. Inline Methods (Inside Struct)

Define methods directly inside struct body:

```vex
struct Point {
    x: i32,
    y: i32,

    fn (self: &Point) distance_from_origin(): i32 {
        return self.x * self.x + self.y * self.y;
    }

    fn (self: &Point!) move_to(new_x: i32, new_y: i32) {
        self.x = new_x;
        self.y = new_y;
    }
}
```

**Properties**:

- Methods defined inside `struct { }` body
- Can access fields directly via `self`
- Organized with struct definition
- Similar to Rust's `impl` blocks

### 2. Golang-Style Methods (Outside Struct)

Define methods outside struct with receiver syntax:

```vex
struct Rectangle {
    width: i32,
    height: i32,
}

fn (r: &Rectangle) area(): i32 {
    return r.width * r.height;
}

fn (r: &Rectangle!) scale(factor: i32) {
    r.width = r.width * factor;
    r.height = r.height * factor;
}
```

**Properties**:

- Methods defined separately from struct
- Receiver is first parameter in parentheses
- Can use any receiver name (`r`, `rect`, `self`, etc.)
- Flexible organization
- Similar to Go's method syntax

### Method Receivers

**Immutable Receiver** (`&Type`):

```vex
fn (self: &Point) get_x(): i32 {
    return self.x;  // Read-only access
}
```

**Mutable Receiver** (`&Type!`):

```vex
fn (self: &Point!) set_x(new_x: i32) {
    self.x = new_x;  // Can modify fields
}
```

**Receiver Naming**:

```vex
// Any name works (not just "self")
fn (p: &Point) distance(): i32 { }
fn (r: &Rectangle) area(): i32 { }
fn (c: &Circle) circumference(): f64 { }
```

### Method Calls

**Syntax**: `object.method(args)`

```vex
let p = Point { x: 10, y: 20 };
let dist = p.distance_from_origin();

let! rect = Rectangle { width: 5, height: 10 };
rect.scale(2);
```

**Automatic Referencing**:

```vex
// Compiler automatically borrows
let p = Point { x: 1, y: 2 };
p.distance_from_origin();  // Compiler adds &p automatically
```

---

## Parameters and Arguments

### Parameter Syntax

**Type Required**:

```vex
fn add(x: i32, y: i32): i32 {
    return x + y;
}
```

**Multiple Parameters**:

```vex
fn calculate(a: i32, b: i32, c: i32, d: i32): i32 {
    return a + b + c + d;
}
```

**Different Types**:

```vex
fn create_user(name: string, age: i32, active: bool): User {
    // ...
}
```

### Parameter Passing

**By Value** (copy semantics):

```vex
fn increment(x: i32): i32 {
    return x + 1;  // x is copied
}

let a = 5;
let b = increment(a);  // a is copied, still 5
```

**By Reference** (borrow semantics):

```vex
fn increment_ref(x: &i32!): i32 {
    return *x + 1;  // Borrow x
}

let! a = 5;
let b = increment_ref(&a);
```

**Array Parameters**:

```vex
fn sum_array(numbers: [i32; 5]): i32 {
    let! total = 0;
    let! i = 0;
    while i < 5 {
        total = total + numbers[i];
        i = i + 1;
    }
    return total;
}
```

### Default Parameters (Future)

```vex
fn greet(name: string, greeting: string = "Hello"): string {
    return greeting + ", " + name;
}

let msg1 = greet("Alice");              // "Hello, Alice"
let msg2 = greet("Bob", "Hi");          // "Hi, Bob"
```

### Named Arguments (Future)

```vex
fn create_user(name: string, age: i32, city: string): User {
    // ...
}

let user = create_user(
    name: "Alice",
    age: 30,
    city: "NYC"
);
```

### Variadic Parameters (Future)

```vex
fn sum(numbers: ...i32): i32 {
    let! total = 0;
    for num in numbers {
        total = total + num;
    }
    return total;
}

let result = sum(1, 2, 3, 4, 5);  // 15
```

---

## Return Values

### Explicit Return

Use `return` statement:

```vex
fn add(x: i32, y: i32): i32 {
    return x + y;
}
```

**Multiple Return Points**:

```vex
fn abs(x: i32): i32 {
    if x < 0 {
        return -x;
    }
    return x;
}
```

### Implicit Return (Future)

Last expression without semicolon:

```vex
fn add(x: i32, y: i32): i32 {
    x + y  // No semicolon = implicit return
}
```

### No Return Value

Functions without return type return nil:

```vex
fn print_hello() {
    // Returns nil implicitly
}
```

Equivalent to:

```vex
fn print_hello(): () {
    return;
}
```

### Multiple Return Values (Future)

Use tuples:

```vex
fn div_mod(a: i32, b: i32): (i32, i32) {
    let quotient = a / b;
    let remainder = a % b;
    return (quotient, remainder);
}

let (q, r) = div_mod(17, 5);  // q=3, r=2
```

### Error Returns (Future)

Union types for error handling:

```vex
fn parse_int(s: string): (i32 | error) {
    if invalid {
        return "Parse error";
    }
    return 42;
}

let result = parse_int("123");
match result {
    i when i is i32 => { /* success */ }
    e when e is error => { /* handle error */ }
}
```

---

## Generic Functions

### Basic Generics

Define functions with type parameters:

```vex
fn identity<T>(x: T): T {
    return x;
}

let num = identity<i32>(42);      // Explicit type
let text = identity("hello");     // Type inferred
```

### Multiple Type Parameters

```vex
fn pair<T, U>(first: T, second: U): (T, U) {
    return (first, second);
}

let p1 = pair<i32, string>(42, "answer");
let p2 = pair(3.14, true);  // Type inferred
```

### Generic Constraints (Future)

Bound type parameters with traits:

```vex
fn print_all<T: Display>(items: [T]): () {
    // T must implement Display trait
}

fn compare<T: Ord>(a: T, b: T): bool {
    return a > b;  // Requires Ord trait
}
```

### Monomorphization

Generics are compiled to concrete types:

```vex
fn identity<T>(x: T): T {
    return x;
}

// Compiler generates:
// fn identity_i32(x: i32): i32 { return x; }
// fn identity_string(x: string): string { return x; }

let a = identity(42);        // Calls identity_i32
let b = identity("hello");   // Calls identity_string
```

**Properties**:

- Zero runtime overhead
- Each type instantiation generates specialized code
- Compile-time polymorphism
- No dynamic dispatch

---

## Function Overloading

**Not Supported** (by design):

```vex
// ERROR: Cannot have multiple functions with same name
fn add(x: i32, y: i32): i32 { }
fn add(x: f64, y: f64): f64 { }  // ERROR: Duplicate function
```

**Alternative: Use Generics**:

```vex
fn add<T>(x: T, y: T): T {
    return x + y;
}
```

**Alternative: Different Names**:

```vex
fn add_int(x: i32, y: i32): i32 { }
fn add_float(x: f64, y: f64): f64 { }
```

---

## Higher-Order Functions

### Function Pointers (Future)

Functions as first-class values:

```vex
type BinaryOp = fn(i32, i32): i32;

fn add(x: i32, y: i32): i32 { return x + y; }
fn multiply(x: i32, y: i32): i32 { return x * y; }

fn apply(op: BinaryOp, a: i32, b: i32): i32 {
    return op(a, b);
}

let result1 = apply(add, 10, 20);       // 30
let result2 = apply(multiply, 10, 20);  // 200
```

### Closures (Future)

Anonymous functions capturing environment:

```vex
fn make_adder(x: i32): fn(i32): i32 {
    return |y| { x + y };  // Captures x
}

let add_5 = make_adder(5);
let result = add_5(10);  // 15
```

### Map, Filter, Reduce (Future)

```vex
let numbers = [1, 2, 3, 4, 5];

let doubled = numbers.map(|x| x * 2);
// [2, 4, 6, 8, 10]

let evens = numbers.filter(|x| x % 2 == 0);
// [2, 4]

let sum = numbers.reduce(0, |acc, x| acc + x);
// 15
```

---

## Special Function Types

### Async Functions

Functions with `async` keyword for asynchronous execution:

```vex
async fn fetch_data(url: string): string {
    // Asynchronous operation
    return "data";
}

async fn main(): i32 {
    let data = await fetch_data("https://api.example.com");
    return 0;
}
```

**Properties**:

- Returns immediately (non-blocking)
- Must be awaited to get result
- Integrates with async runtime
- Future: Full async/await support

### GPU Functions

Functions executed on GPU:

```vex
gpu fn matrix_multiply(a: [f32], b: [f32]): [f32] {
    // GPU kernel code
}
```

**Properties**:

- Executed on GPU hardware
- Parallel execution
- Restricted operations
- Future: CUDA/OpenCL backend

### Extern Functions (FFI)

Link to external C/Rust functions:

```vex
extern "C" fn printf(format: string, ...): i32;
extern "C" fn malloc(size: u64): &u8!;
extern "C" fn free(ptr: &u8!);

fn main(): i32 {
    let ptr = malloc(1024);
    free(ptr);
    return 0;
}
```

### Intrinsic Functions

Compiler built-ins with special semantics:

```vex
@intrinsic
fn sizeof<T>(): u64 {
    // Compiler provides implementation
}

@intrinsic
fn alignof<T>(): u64 {
    // Compiler provides implementation
}

let size = sizeof<i32>();      // 4
let align = alignof<f64>();    // 8
```

---

## Recursion

### Direct Recursion

Function calls itself:

```vex
fn factorial(n: i32): i32 {
    if n <= 1 {
        return 1;
    }
    return n * factorial(n - 1);
}

let result = factorial(5);  // 120
```

### Tail Recursion

Last operation is recursive call:

```vex
fn factorial_tail(n: i32, acc: i32): i32 {
    if n <= 1 {
        return acc;
    }
    return factorial_tail(n - 1, n * acc);
}

let result = factorial_tail(5, 1);  // 120
```

**Future**: Tail call optimization

### Mutual Recursion

Functions call each other:

```vex
fn is_even(n: i32): bool {
    if n == 0 {
        return true;
    }
    return is_odd(n - 1);
}

fn is_odd(n: i32): bool {
    if n == 0 {
        return false;
    }
    return is_even(n - 1);
}
```

---

## Examples

### Simple Function

```vex
fn add(x: i32, y: i32): i32 {
    return x + y;
}

fn main(): i32 {
    let result = add(10, 20);
    return result;  // 30
}
```

### Method Example

```vex
struct Counter {
    value: i32,

    fn (self: &Counter) get(): i32 {
        return self.value;
    }

    fn (self: &Counter!) increment() {
        self.value = self.value + 1;
    }
}

fn main(): i32 {
    let! counter = Counter { value: 0 };
    counter.increment();
    counter.increment();
    return counter.get();  // 2
}
```

### Golang-Style Method

```vex
struct Point {
    x: i32,
    y: i32,
}

fn (p: &Point) distance_from_origin(): i32 {
    return p.x * p.x + p.y * p.y;
}

fn (p: &Point!) move_by(dx: i32, dy: i32) {
    p.x = p.x + dx;
    p.y = p.y + dy;
}

fn main(): i32 {
    let! point = Point { x: 3, y: 4 };
    let dist = point.distance_from_origin();  // 25
    point.move_by(1, 1);
    return 0;
}
```

### Generic Function

```vex
fn swap<T>(a: T, b: T): (T, T) {
    return (b, a);
}

fn main(): i32 {
    let (x, y) = swap<i32>(10, 20);     // x=20, y=10
    let (s1, s2) = swap("hello", "world");
    return 0;
}
```

### Recursive Fibonacci

```vex
fn fibonacci(n: i32): i32 {
    if n <= 1 {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

fn main(): i32 {
    let fib_10 = fibonacci(10);  // 55
    return fib_10;
}
```

---

## Best Practices

### 1. Keep Functions Small

```vex
// Good: Single responsibility
fn validate_email(email: string): bool {
    return contains(email, "@") && contains(email, ".");
}

// Bad: Too much logic
fn process_user(name: string, email: string, age: i32): () {
    // validation
    // formatting
    // database operations
    // logging
    // ...
}
```

### 2. Use Descriptive Names

```vex
// Good
fn calculate_total_price(items: [Item], tax_rate: f64): f64 { }

// Bad
fn calc(x: [Item], y: f64): f64 { }
```

### 3. Prefer Immutable Parameters

```vex
// Good: Read-only reference
fn sum_array(numbers: &[i32]): i32 { }

// Only use mutable when necessary
fn sort_array(numbers: &[i32]!) { }
```

### 4. Document Complex Functions

```vex
// Calculates nth Fibonacci number using dynamic programming
// Time complexity: O(n)
// Space complexity: O(1)
fn fibonacci_optimized(n: i32): i32 {
    // Implementation
}
```

### 5. Use Generics for Reusability

```vex
// Good: Generic, reusable
fn max<T: Ord>(a: T, b: T): T {
    if a > b { return a; }
    return b;
}

// Bad: Duplicate code
fn max_i32(a: i32, b: i32): i32 { }
fn max_f64(a: f64, b: f64): f64 { }
```

---

## Function Signature Summary

| Component          | Syntax              | Required?            | Example             |
| ------------------ | ------------------- | -------------------- | ------------------- |
| Keyword            | `fn`                | Yes                  | `fn add`            |
| Name               | identifier          | Yes                  | `add`               |
| Type Parameters    | `<T, U>`            | No                   | `<T>`               |
| Parameters         | `(name: Type, ...)` | Yes (can be empty)   | `(x: i32, y: i32)`  |
| Return Type        | `: Type`            | No (defaults to nil) | `: i32`             |
| Body               | `{ statements }`    | Yes                  | `{ return x + y; }` |
| Receiver (methods) | `(self: &Type)`     | For methods only     | `(self: &Point)`    |

---

**Previous**: [04_Variables_and_Constants.md](./04_Variables_and_Constants.md)  
**Next**: [06_Control_Flow.md](./06_Control_Flow.md)

**Maintained by**: Vex Language Team
