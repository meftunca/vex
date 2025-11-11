# Traits

Version: 0.1.0 
Last Updated: November 3, 2025

This document defines the trait system in the Vex programming language. Traits provide polymorphism through shared behavior definitions.

---

## Trait Definitions

### Basic Syntax

Syntax: `trait Name { methods }`

```vex
trait Display {
    fn show();
}

trait Comparable {
    fn compare(other: &Self): i32;
}
```

Properties:

- `trait` keyword
- Name in PascalCase (convention)
- Method signatures (no body by default)
- `Self` type refers to implementing type
- Can have default method implementations

### Simple Trait

```vex
trait Greet {
    fn say_hello();
}
```

Note: `interface` keyword is deprecated in v0.1, use `trait` instead.

### Multiple Methods

```vex
trait Shape {
    fn area(): f64;
    fn perimeter(): f64;
    fn name(): string;
}
```

### Self Type

`Self` represents the type implementing the trait:

```vex
trait Cloneable {
    fn clone(): Self;
}

trait Comparable {
    fn equals(other: &Self): bool;
}
```

---

## Trait Implementation

### Method Mutability in Traits

Trait method signatures define a contract for mutability. To declare a method that can mutate the implementing type's state, the `!` suffix is used.

Syntax:

- Immutable Method: `fn method_name(args...): ReturnType;`
- Mutable Method: `fn methodname(args...)!;` or `fn methodname(args...)!: ReturnType;`

The `!` indicates that the method requires a mutable reference to `self`, allowing for modifications.

```vex
// Kural 1 (Inline) applies to trait definitions
trait Logger {
    // Immutable contract: cannot modify `self`
    fn log(msg: string);

    // Mutable contract: can modify `self`
    fn clear()!;
}
```

This contract must be respected by all implementing types.

---

## Trait Implementation

### Inline Implementation

Vex follows an "inline" implementation model, where methods for a trait are defined directly within the `struct` body that implements it.

Syntax: `struct MyStruct impl MyTrait { ... methods ... }`

```vex
struct ConsoleLogger impl Logger {
    prefix: string,

    // Implementation of the `log` method from the `Logger` trait.
    fn log(msg: string) {
        println(self.prefix, ": ", msg);
    }

    // Implementation of the mutable `clear` method.
    // The `!` is required in the implementation as well.
    fn clear()! {
        // This method can now mutate `self`.
        // For example, if we had a mutable field:
        // self.buffer = "";
        println("Logger cleared.");
    }
}
```

Key Rules:

- Trait methods MUST be implemented directly inside the `struct`'s body.
- The method signatures in the implementation must match the trait definition, including the `!` for mutability.

### Multiple Traits (Future)

```vex
struct FileLogger impl Logger, Closeable {
    path: string,

    // All trait methods must be in struct body
    fn log(msg: string) {
        // Logger implementation
    }

    fn clear()! {
        // Logger implementation
    }

    fn close()! {
        // Closeable implementation
    }
}
```

### Implementation Requirements

All trait methods must be implemented:

```vex
trait Shape {
    fn area(): f64;
    fn perimeter(): f64;
}

// ERROR: Missing perimeter() implementation
struct Circle impl Shape {
    radius: f64,

    fn area(): f64 {
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
trait Logger {
    fn log(msg: string);        // Required (immutable)
    fn clear()!;                // Required (mutable)

    fn info(msg: string) {      // Default (immutable)
        self.log(msg);
    }

    fn debug(msg: string) {     // Default (immutable)
        self.log(msg);
    }
}
```

Properties:

- Methods with body are default methods
- Implementing types inherit default behavior
- Can be overridden if needed
- Reduces code duplication

### Inheritance

Structs automatically get default methods:

```vex
struct ConsoleLogger impl Logger {
    fn log(msg: string) {
        // Only implement required method
    }

    fn clear()! {
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
    fn log(msg: string) {
        // Required method
    }

    fn clear()! {
        // Required method
    }

    fn info(msg: string) {
        // Override default implementation
        self.log("[INFO] " + msg);
    }

    // debug() still uses default implementation
}
```

### Default Method Access

Default methods can call other trait methods:

```vex
trait Formatter {
    fn format(): string;  // Required

    fn format_bold(): string {
        return "**" + self.format() + "**";
    }

    fn format_italic(): string {
        return "_" + self.format() + "_";
    }
}
```

---

## Trait Bounds

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

Syntax: `T: Trait` after type parameter

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

Syntax: `T: Trait1 & Trait2 & ...`

### Where Clauses COMPLETE (v0.1.2)

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

Implementation Details:

- Parser: `parsewhereclause()` in `vex-parser/src/parser/items/functions.rs:138`
- AST: `WhereClausePredicate { type_param, bounds }`
- Syntax: `where T: Trait1 & Trait2, U: Trait3`
- Test: `examples/testwhereclause.vx`
- Verified: November 9, 2025

Limitations:

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
trait Iterator {
    type Item;

    fn next(): Option<Self.Item>;
}
```

Properties:

- `type Name` declares associated type
- Implementing types specify concrete type
- Used for output types in trait methods

### Implementation (IMPLEMENTED )

```vex
struct Counter impl Iterator {
    type Item = i32;

    current: i32,

    fn next()!: Option<i32> {
        let value = self.current;
        self.current = self.current + 1;
        return Some(value);
    }
}
```

### Generic Associated Types (Future)

```vex
trait Container {
    type Item<T>;

    fn get<T>(): Self.Item<T>;
}
```

---

## Trait Inheritance

### Supertraits

Traits can require other traits:

```vex
trait Eq {
    fn equals(other: &Self): bool;
}

trait Ord: Eq {
    // Ord requires Eq
    fn less_than(other: &Self): bool;
}
```

Implementation:

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
trait Serializable: Display & Cloneable {
    fn serialize(): string;
}
```

---

## Standard Traits

### Drop Trait IMPLEMENTED

Automatic resource cleanup when value goes out of scope:

```vex
trait Drop {
    fn drop()!;  // Called automatically
}

struct File impl Drop {
    handle: i32,
    path: string,

    fn drop()! {
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

Status: Fully functional, automatic Drop trait implementation detection.

### Clone Trait IMPLEMENTED

Explicit deep copying:

```vex
trait Clone {
    fn clone(): Self;
}

struct Point impl Clone {
    x: i32,
    y: i32,

    fn clone(): Point {
        return Point { x: self.x, y: self.y };
    }
}

// Usage
let p1 = Point { x: 10, y: 20 };
let p2 = p1.clone();  // Deep copy
```

Status: Fully functional, used for explicit copying.

### Eq Trait IMPLEMENTED

Equality comparison:

```vex
trait Eq {
    fn eq(other: Self): bool;
}

struct Point impl Eq {
    x: i32,
    y: i32,

    fn eq(other: Point): bool {
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

Status: Fully functional, used for custom equality.

### Ord Trait IMPLEMENTED

Ordering comparison:

```vex
trait Ord {
    fn cmp(other: Self): i32;
    // Returns: -1 (less), 0 (equal), 1 (greater)
}

struct Number impl Ord {
    value: i32,

    fn cmp(other: Number): i32 {
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

Status: Fully functional, used for ordering operations.

### Iterator Trait IMPLEMENTED

Lazy iteration protocol:

```vex
trait Iterator {
    type Item;  // Associated type

    fn next()!: Option<Self.Item>;  // Returns next element or None
}

struct Counter impl Iterator {
    count: i32,
    limit: i32,

    type Item = i32;

    fn next()!: Option<i32> {
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

Status: Fully functional with Option<T> support. Associated type `Self.Item` temporarily uses concrete type (Option<i32>) until full generic support.

### Display Trait (Future)

Format types for display:

```vex
trait Display {
    fn show();
}

struct Point impl Display {
    x: i32,
    y: i32,

    fn show() {
        print("Point(", self.x, ", ", self.y, ")");
    }
}
```

Status: Planned for future implementation.

---

## Examples

### Basic Trait

```vex
trait Greet {
    fn say_hello();
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
trait Logger {
    fn log(msg: string);

    fn info(msg: string) {
        self.log(msg);
    }

    fn debug(msg: string) {
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
trait Shape {
    fn area(): i32;
    fn perimeter(): i32;
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
trait Counter {
    fn count(): i32;

    fn count_double(): i32 {
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
trait Serializable {
    fn serialize(): string;
}

trait Deserializable {
    fn from_string(s: string): Self;
}

// Bad: Too many responsibilities
trait DataHandler {
    fn serialize(): string;
    fn from_string(s: string): Self;
    fn validate(): bool;
    fn transform(): Self;
}
```

### 2. Descriptive Names

```vex
// Good: Clear purpose
trait Drawable {
    fn draw();
}

trait Comparable {
    fn compare(other: &Self): i32;
}

// Bad: Vague
trait Handler {
    fn handle();
}
```

### 3. Use Default Methods

```vex
// Good: Provide defaults when sensible
trait Logger {
    fn log(msg: string);

    fn info(msg: string) {
        self.log("[INFO] " + msg);
    }
}

// Bad: Force implementation of similar methods
trait Logger {
    fn log(msg: string);
    fn info(msg: string);  // No default
    fn debug(msg: string); // No default
}
```

### 4. Small Traits

```vex
// Good: Composable traits
trait Display {
    fn show();
}

trait Clone {
    fn clone(): Self;
}

struct Data impl Display, Clone {
    // Implement both
}

// Bad: Monolithic trait
trait Everything {
    fn show();
    fn clone(): Self;
    fn serialize(): string;
    fn validate(): bool;
}
```

### 5. Document Requirements

```vex
// Document trait purpose and requirements
/// Represents types that can be displayed to the user.
/// Implementations should provide a human-readable representation.
trait Display {
    fn show();
}

/// Represents types that can be compared for ordering.
/// Returns: -1 if self < other, 0 if equal, 1 if self > other
trait Ord {
    fn compare(other: &Self): i32;
}
```

---

## Trait Features Summary

• Feature — Syntax — Status — Example
| Trait Definition | `trait Name { }` | Working | Method signatures |
| Inline Implementation | `struct S impl T { }` | Working | v1.3 syntax |
| Default Methods | `fn (self) { body }` | Working | With implementation |
| Self Type | `Self` | Working | Refers to implementer |
• Multiple Methods — Multiple fn signatures — Working — In trait body
| Trait Bounds | `<T: Trait>` | Working | Generic constraints |
| Associated Types | `type Item;` | Working | Type members |
| Supertraits | `trait T: U { }` | Working | Trait inheritance |
| Where Clauses | `where T: Trait` | v0.1.2 | Complex bounds |

---

## Trait System Architecture

### Current Implementation (v1.3)

```vex
// 1. Define trait
trait Logger {
    fn log(msg: string);
    fn info(msg: string) {
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

// 3. Use trait methods
fn main(): i32 {
    let! logger = ConsoleLogger { prefix: "[LOG]" };
    logger.log("Direct call");
    logger.info("Default method call");
    return 0;
}
```

### Compilation Process

1. Parse: Trait definition → AST
2. Register: Store trait in `trait_defs` HashMap
3. Implement: Inline `impl Trait` → `trait_impls` HashMap
4. Codegen: Generate LLVM IR for methods
5. Link: Default methods compiled on-demand
6. Call: Method resolution at compile time (static dispatch)

### Future: Dynamic Dispatch

```vex
// Virtual table (vtable) for runtime polymorphism
fn process(logger: &dyn Logger) {
    logger.log("Dynamic dispatch");
}
```

---

Previous: 08_Enums.md 
Next: 10_Generics.md
