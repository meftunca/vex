# Contracts

**Version:** 0.2.0  
**Last Updated:** December 16, 2025

This document defines the contract system in the Vex programming language. Contracts provide polymorphism through shared behavior definitions.

**⚠️ BREAKING CHANGE (v0.2.0)**: The `trait` keyword has been replaced with `contract`. This change was made to better reflect Vex's unique identity and to distinguish contracts (pure interfaces) from implementation.

---

## Table of Contents

1. [Contract Definitions](#contract-definitions)
2. [Contract Implementation](#contract-implementation)
3. [Default Methods](#default-methods)
4. [Contract Bounds](#contract-bounds)
5. [Associated Types](#associated-types)
6. [Contract Inheritance](#contract-inheritance)
7. [Standard Contracts](#standard-contracts)

---

## Contract Definitions

### Basic Syntax

**Syntax**: `contract Name { methods }`

```vex
contract Display {
    show();
}

contract Comparable {
    compare(other: &Self): i32;
}
```

**Properties**:

- `contract` keyword (pure interface, signatures only)
- Name in PascalCase (convention)
- Method signatures (no body, no `fn` prefix)
- `Self` type refers to implementing type
- Can have default method implementations

### Simple Contract

```vex
contract Greet {
    say_hello();
}
```

**Note**: `interface` and `trait` keywords are deprecated in v0.2.0, use `contract` instead.

### Multiple Methods

```vex
contract Shape {
    area(): f64;
    perimeter(): f64;
    name(): string;
}
```

### Self Type

`Self` represents the type implementing the contract:

```vex
contract Cloneable {
    clone(): Self;
}

contract Comparable {
    equals(other: &Self): bool;
}
```

---

## Contract Implementation

### Method Mutability in Contracts

Contract method signatures define a contract for mutability. To declare a method that can mutate the implementing type's state, the `!` suffix is used.

**Syntax**:

- **Immutable Method**: `method_name(args...): ReturnType;`
- **Mutable Method**: `method_name(args...)!;` or `method_name(args...)!: ReturnType;`

The `!` indicates that the method requires a mutable reference to `self`, allowing for modifications.

```vex
contract Logger {
    // Immutable contract: cannot modify `self`
    log(msg: string);

    // Mutable contract: can modify `self`
    clear()!;
}
```

This contract must be respected by all implementing types.

---

## Contract Implementation

### Go-Style External Implementation (RECOMMENDED v0.2.0)

**⚠️ IMPORTANT**: Vex v0.2.0 deprecates inline struct methods and recommends Go-style external methods.

**Recommended Syntax**: External methods with `contract` as pure interface

```vex
// 1. Define contract (pure interface, no fn prefix)
contract Logger {
    log(msg: string);
    clear()!;
}

// 2. Define struct (data only)
struct ConsoleLogger {
    prefix: string,
}

// 3. Implement contract via external methods (Go-style)
fn (self: ConsoleLogger) log(msg: string) {
    println(self.prefix, ": ", msg);
}

fn (self: ConsoleLogger!) clear() {
    println("Logger cleared.");
}
```

**Benefits**:
- Keeps struct definitions small (400-line limit)
- Separates data from behavior
- More modular and testable
- Follows Go and Odin conventions

### Inline Implementation (DEPRECATED v0.2.0)

**⚠️ DEPRECATED**: Inline struct methods will be removed in a future version.

**Old Syntax**: `struct MyStruct impl MyContract { ... methods ... }`

```vex
struct ConsoleLogger impl Logger {
    prefix: string,

    // Implementation of the `log` method from the `Logger` trait.
    log(msg: string) {
        println(self.prefix, ": ", msg);
    }

    // Implementation of the mutable `clear` method.
    // The `!` is required in the implementation as well.
    clear()! {
        // This method can now mutate `self`.
        // For example, if we had a mutable field:
        // self.buffer = "";
        println("Logger cleared.");
    }
}
```

**Key Rules**:

- Contract methods **MUST** be implemented directly inside the `struct`'s body.
- The method signatures in the implementation must match the contract definition, including the `!` for mutability.

### Multiple Traits (Future)

```vex
struct FileLogger impl Logger, Closeable {
    path: string,

    // All contract methods must be in struct body
    log(msg: string) {
        // Logger implementation
    }

    clear()! {
        // Logger implementation
    }

    fn close()! {
        // Closeable implementation
    }
}
```

### Implementation Requirements

All contract methods must be implemented:

```vex
contract Shape {
    area(): f64;
    perimeter(): f64;
}

// ERROR: Missing perimeter() implementation
struct Circle impl Shape {
    radius: f64,

    area(): f64 {
        return 3.14159 * self.radius * self.radius;
    }
    // Missing perimeter()!
}
```

---

## Default Methods

### Definition

Traits can provide default implementations:

```vex
contract Logger {
    log(msg: string);        // Required (immutable)
    clear()!;                // Required (mutable)

    info(msg: string) {      // Default (immutable)
        self.log(msg);
    }

    debug(msg: string) {     // Default (immutable)
        self.log(msg);
    }
}
```

**Properties**:

- Methods with body are default methods
- Implementing types inherit default behavior
- Can be overridden if needed
- Reduces code duplication

### Inheritance

Structs automatically get default methods:

```vex
struct ConsoleLogger impl Logger {
    log(msg: string) {
        // Only implement required method
    }

    clear()! {
        // Required mutable method
    }

    // info() and debug() inherited automatically!
}

fn main(): i32 {
    let! logger = ConsoleLogger { };
    logger.log("Required method");
    logger.info("Default method");    // Works!
    logger.debug("Default method");   // Works!
    logger.clear()!;                  // Required !
    return 0;
}
```

### Overriding Defaults

Implementing types can override default methods:

```vex
struct CustomLogger impl Logger {
    log(msg: string) {
        // Required method
    }

    clear()! {
        // Required method
    }

    info(msg: string) {
        // Override default implementation
        self.log("[INFO] " + msg);
    }

    // debug() still uses default implementation
}
```

### Default Method Access

Default methods can call other contract methods:

```vex
contract Formatter {
    format(): string;  // Required

    format_bold(): string {
        return "**" + self.format() + "**";
    }

    format_italic(): string {
        return "_" + self.format() + "_";
    }
}
```

---

## Contract Bounds

### Generic Constraints (Future)

Restrict generic types to those implementing specific traits:

```vex
fn print_all<T: Display>(items: [T]) {
    // T must implement Display
    for item in items {
        item.show();
    }
}
```

**Syntax**: `T: Trait` after type parameter

### Multiple Bounds (Future)

Require multiple traits:

```vex
fn compare_and_show<T: Comparable & Display>(a: T, b: T) {
    // T must implement both traits
    let result = a.compare(b);
    a.show();
    b.show();
}
```

**Syntax**: `T: Trait1 & Trait2 & ...`

### Where Clauses ✅ COMPLETE (v0.1.2)

Complex bounds use where clause for readability:

```vex
fn print_both<T, U>(a: T, b: U): i32
where
    T: Display,
    U: Display
{
    print("T: ");
    print(a);
    print("U: ");
    print(b);
    return 0;
}

fn main(): i32 {
    let x: i32 = 42;
    let y: i32 = 100;
    print_both(x, y);
    return 0;
}
```

**Implementation Details**:

- Parser: `parse_where_clause()` in `vex-parser/src/parser/items/functions.rs:138`
- AST: `WhereClausePredicate { type_param, bounds }`
- Syntax: `where T: Trait1 & Trait2, U: Trait3`
- Test: `examples/test_where_clause.vx`
- Verified: November 9, 2025

**Limitations**:

- Struct inline methods don't support where clauses yet (see `structs.rs:195`)

### Bound on Methods (Future)

```vex
struct Container<T> {
    value: T,

    fn (self: &Container<T>!) show() where T: Display {
        self.value.show();
    }
}
```

---

## Associated Types

### Definition (Future)

Traits can have associated types:

```vex
contract Iterator {
    type Item;

    next(): Option<Self.Item>;
}
```

**Properties**:

- `type Name` declares associated type
- Implementing types specify concrete type
- Used for output types in contract methods

### Implementation (IMPLEMENTED ✅)

```vex
struct Counter impl Iterator {
    type Item = i32;

    current: i32,

    next()!: Option<i32> {
        let value = self.current;
        self.current = self.current + 1;
        return Some(value);
    }
}
```

### Generic Associated Types (Future)

```vex
contract Container {
    type Item<T>;

    fn get<T>(): Self.Item<T>;
}
```

---

## Contract Inheritance

### Supertraits

Traits can require other traits:

```vex
contract Eq {
    equals(other: &Self): bool;
}

contract Ord: Eq {
    // Ord requires Eq
    fn less_than(other: &Self): bool;
}
```

**Implementation**:

```vex
struct Number impl Ord {
    value: i32,

    // Must implement Eq methods
    fn (self: &Number!) equals(other: &Number): bool {
        return self.value == other.value;
    }

    // And Ord methods
    fn (self: &Number!) less_than(other: &Number): bool {
        return self.value < other.value;
    }
}
```

### Multiple Supertraits

```vex
contract Serializable: Display & Cloneable {
    serialize(): string;
}
```

---

## Standard Traits

### Drop Contract ✅ IMPLEMENTED

Automatic resource cleanup when value goes out of scope:

```vex
contract Drop {
    drop()!;  // Called automatically
}

struct File impl Drop {
    handle: i32,
    path: string,

    drop()! {
        // Cleanup logic - called automatically when File goes out of scope
        close_file(self.handle);
        print("Closed file: ", self.path);
    }
}

// Usage
{
    let! file = File { handle: 42, path: "data.txt" };
    // ... use file ...
}  // drop() called automatically here
```

**Status**: Fully functional, automatic Drop contract implementation detection.

### Clone Contract ✅ IMPLEMENTED

Explicit deep copying:

```vex
contract Clone {
    clone(): Self;
}

struct Point impl Clone {
    x: i32,
    y: i32,

    clone(): Point {
        return Point { x: self.x, y: self.y };
    }
}

// Usage
let p1 = Point { x: 10, y: 20 };
let p2 = p1.clone();  // Deep copy
```

**Status**: Fully functional, used for explicit copying.

### Eq Contract ✅ IMPLEMENTED

Equality comparison:

```vex
contract Eq {
    eq(other: Self): bool;
}

struct Point impl Eq {
    x: i32,
    y: i32,

    eq(other: Point): bool {
        return self.x == other.x && self.y == other.y;
    }
}

// Usage
let p1 = Point { x: 10, y: 20 };
let p2 = Point { x: 10, y: 20 };
if p1.eq(p2) {
    print("Equal!");
}
```

**Status**: Fully functional, used for custom equality.

### Ord Contract ✅ IMPLEMENTED

Ordering comparison:

```vex
contract Ord {
    cmp(other: Self): i32;
    // Returns: -1 (less), 0 (equal), 1 (greater)
}

struct Number impl Ord {
    value: i32,

    cmp(other: Number): i32 {
        if self.value < other.value {
            return -1;
        } else if self.value > other.value {
            return 1;
        }
        return 0;
    }
}

// Usage
let n1 = Number { value: 10 };
let n2 = Number { value: 20 };
let result = n1.cmp(n2);  // Returns -1
```

**Status**: Fully functional, used for ordering operations.

### Iterator Contract ✅ IMPLEMENTED

Lazy iteration protocol:

```vex
contract Iterator {
    type Item;  // Associated type

    next()!: Option<Self.Item>;  // Returns next element or None
}

struct Counter impl Iterator {
    count: i32,
    limit: i32,

    type Item = i32;

    next()!: Option<i32> {
        if self.count < self.limit {
            let current = self.count;
            self.count = self.count + 1;
            return Some(current);
        }
        return None;
    }
}

// Usage
let! counter = Counter { count: 0, limit: 5 };
loop {
    match counter.next() {
        Some(v) => print(v),
        None => break,
    }
}
```

**Status**: Fully functional with Option<T> support. Associated type `Self.Item` temporarily uses concrete type (Option<i32>) until full generic support.

### Display Contract (Future)

Format types for display:

```vex
contract Display {
    show();
}

struct Point impl Display {
    x: i32,
    y: i32,

    show() {
        print("Point(", self.x, ", ", self.y, ")");
    }
}
```

**Status**: Planned for future implementation.

---

## Examples

### Basic Trait

```vex
contract Greet {
    say_hello();
}

struct Person impl Greet {
    name: string,

    fn (self: &Person!) say_hello() {
        // Implementation
    }
}

fn main(): i32 {
    let! person = Person { name: "Alice" };
    person.say_hello();
    return 0;
}
```

### Default Methods

```vex
contract Logger {
    log(msg: string);

    info(msg: string) {
        self.log(msg);
    }

    debug(msg: string) {
        self.log(msg);
    }
}

struct ConsoleLogger impl Logger {
    prefix: string,

    fn (self: &ConsoleLogger!) log(msg: string) {
        // Only implement required method
    }
}

fn main(): i32 {
    let! logger = ConsoleLogger { prefix: "[LOG]" };
    logger.log("Required");
    logger.info("Default method");    // Inherited!
    logger.debug("Default method");   // Inherited!
    return 0;
}
```

### Multiple Methods

```vex
contract Shape {
    area(): i32;
    perimeter(): i32;
}

struct Rectangle impl Shape {
    width: i32,
    height: i32,

    fn (self: &Rectangle!) area(): i32 {
        return self.width * self.height;
    }

    fn (self: &Rectangle!) perimeter(): i32 {
        return 2 * (self.width + self.height);
    }
}

fn main(): i32 {
    let rect = Rectangle { width: 10, height: 20 };
    let a = rect.area();        // 200
    let p = rect.perimeter();   // 60
    return a;
}
```

### Overriding Defaults

```vex
contract Counter {
    count(): i32;

    count_double(): i32 {
        return self.count() * 2;
    }
}

struct SimpleCounter impl Counter {
    value: i32,

    fn (self: &SimpleCounter!) count(): i32 {
        return self.value;
    }

    // Override default
    fn (self: &SimpleCounter!) count_double(): i32 {
        return self.value * 2 + 1;  // Custom logic
    }
}
```

---

## Best Practices

### 1. Single Responsibility

```vex
// Good: Focused trait
contract Serializable {
    serialize(): string;
}

contract Deserializable {
    from_string(s: string): Self;
}

// Bad: Too many responsibilities
contract DataHandler {
    serialize(): string;
    from_string(s: string): Self;
    validate(): bool;
    transform(): Self;
}
```

### 2. Descriptive Names

```vex
// Good: Clear purpose
contract Drawable {
    draw();
}

contract Comparable {
    compare(other: &Self): i32;
}

// Bad: Vague
contract Handler {
    handle();
}
```

### 3. Use Default Methods

```vex
// Good: Provide defaults when sensible
contract Logger {
    log(msg: string);

    info(msg: string) {
        self.log("[INFO] " + msg);
    }
}

// Bad: Force implementation of similar methods
contract Logger {
    log(msg: string);
    info(msg: string);  // No default
    debug(msg: string); // No default
}
```

### 4. Small Traits

```vex
// Good: Composable traits
contract Display {
    show();
}

contract Clone {
    clone(): Self;
}

struct Data impl Display, Clone {
    // Implement both
}

// Bad: Monolithic trait
contract Everything {
    show();
    clone(): Self;
    serialize(): string;
    validate(): bool;
}
```

### 5. Document Requirements

```vex
// Document contract purpose and requirements
/// Represents types that can be displayed to the user.
/// Implementations should provide a human-readable representation.
contract Display {
    show();
}

/// Represents types that can be compared for ordering.
/// Returns: -1 if self < other, 0 if equal, 1 if self > other
contract Ord {
    compare(other: &Self): i32;
}
```

---

## Contract Features Summary

| Feature               | Syntax                 | Status     | Example               |
| --------------------- | ---------------------- | ---------- | --------------------- |
| Contract Definition      | `trait Name { }`       | ✅ Working | Method signatures     |
| Inline Implementation | `struct S impl T { }`  | ✅ Working | v1.3 syntax           |
| Default Methods       | `fn (self) { body }`   | ✅ Working | With implementation   |
| Self Type             | `Self`                 | ✅ Working | Refers to implementer |
| Multiple Methods      | Multiple fn signatures | ✅ Working | In contract body         |
| Contract Bounds          | `<T: Trait>`           | ✅ Working | Generic constraints   |
| Associated Types      | `type Item;`           | ✅ Working | Type members          |
| Supertraits           | `trait T: U { }`       | ✅ Working | Contract inheritance     |
| Where Clauses         | `where T: Trait`       | ✅ v0.1.2  | Complex bounds        |

---

## Contract System Architecture

### Current Implementation (v1.3)

```vex
// 1. Define trait
contract Logger {
    log(msg: string);
    info(msg: string) {
        self.log(msg);  // Default method
    }
}

// 2. Implement inline
struct ConsoleLogger impl Logger {
    prefix: string,

    fn (self: &ConsoleLogger!) log(msg: string) {
        // Required method implementation
    }

    // info() inherited automatically
}

// 3. Use contract methods
fn main(): i32 {
    let! logger = ConsoleLogger { prefix: "[LOG]" };
    logger.log("Direct call");
    logger.info("Default method call");
    return 0;
}
```

### Compilation Process

1. **Parse**: Contract definition → AST
2. **Register**: Store contract in `trait_defs` HashMap
3. **Implement**: Inline `impl Trait` → `trait_impls` HashMap
4. **Codegen**: Generate LLVM IR for methods
5. **Link**: Default methods compiled on-demand
6. **Call**: Method resolution at compile time (static dispatch)

### Future: Dynamic Dispatch

```vex
// Virtual table (vtable) for runtime polymorphism
fn process(logger: &dyn Logger) {
    logger.log("Dynamic dispatch");
}
```

---

**Previous**: [08_Enums.md](./08_Enums.md)  
**Next**: [10_Generics.md](./10_Generics.md)

**Maintained by**: Vex Language Team
