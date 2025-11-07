# Traits

**Version:** 0.9.0  
**Last Updated:** November 3, 2025

This document defines the trait system in the Vex programming language. Traits provide polymorphism through shared behavior definitions.

---

## Table of Contents

1. [Trait Definitions](#trait-definitions)
2. [Trait Implementation](#trait-implementation)
3. [Default Methods](#default-methods)
4. [Trait Bounds](#trait-bounds)
5. [Associated Types](#associated-types)
6. [Trait Inheritance](#trait-inheritance)
7. [Standard Traits](#standard-traits)

---

## Trait Definitions

### Basic Syntax

**Syntax**: `trait Name { methods }`

```vex
trait Display {
    fn (self: &Self!) show();
}

trait Comparable {
    fn (self: &Self!) compare(other: &Self): i32;
}
```

**Properties**:

- `trait` keyword
- Name in PascalCase (convention)
- Method signatures (no body by default)
- `Self` type refers to implementing type
- Can have default method implementations

### Simple Trait

```vex
trait Greet {
    fn (self: &Self!) say_hello();
}
```

**Note**: `interface` keyword is deprecated in v0.9, use `trait` instead.

### Multiple Methods

```vex
trait Shape {
    fn (self: &Self!) area(): f64;
    fn (self: &Self!) perimeter(): f64;
    fn (self: &Self!) name(): string;
}
```

### Self Type

`Self` represents the type implementing the trait:

```vex
trait Cloneable {
    fn (self: &Self!) clone(): Self;
}

trait Comparable {
    fn (self: &Self!) equals(other: &Self): bool;
}
```

---

## Trait Implementation

### Inline Implementation (v1.3)

Implement traits directly in struct definition:

```vex
trait Logger {
    fn (self: &Self!) log(msg: string);
}

struct ConsoleLogger impl Logger {
    prefix: string,

    fn (self: &ConsoleLogger!) log(msg: string) {
        // Implementation
    }
}
```

**Syntax**: `struct Name impl Trait { ... }`

**Properties**:

- `impl Trait` after struct name
- Methods defined in struct body
- Can implement multiple traits (future)
- Compile-time checking

### Multiple Traits (Future)

```vex
struct FileLogger impl Logger, Closeable {
    path: string,

    fn (self: &FileLogger!) log(msg: string) {
        // Logger implementation
    }

    fn (self: &FileLogger!) close() {
        // Closeable implementation
    }
}
```

### Separate Implementation (Future)

```vex
trait Display {
    fn (self: &Self!) show();
}

struct Point {
    x: i32,
    y: i32,
}

impl Display for Point {
    fn (self: &Point!) show() {
        // Implementation
    }
}
```

### Implementation Requirements

All trait methods must be implemented:

```vex
trait Shape {
    fn (self: &Self!) area(): f64;
    fn (self: &Self!) perimeter(): f64;
}

// ERROR: Missing perimeter() implementation
struct Circle impl Shape {
    radius: f64,

    fn (self: &Circle!) area(): f64 {
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
    fn (self: &Self!) log(msg: string);  // Required

    fn (self: &Self!) info(msg: string) {
        // Default implementation
        self.log(msg);
    }

    fn (self: &Self!) debug(msg: string) {
        // Default implementation
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
    fn (self: &ConsoleLogger!) log(msg: string) {
        // Only implement required method
    }

    // info() and debug() inherited automatically!
}

fn main(): i32 {
    let! logger = ConsoleLogger { };
    logger.log("Required method");
    logger.info("Default method");    // Works!
    logger.debug("Default method");   // Works!
    return 0;
}
```

### Overriding Defaults

Implementing types can override default methods:

```vex
struct CustomLogger impl Logger {
    fn (self: &CustomLogger!) log(msg: string) {
        // Required method
    }

    fn (self: &CustomLogger!) info(msg: string) {
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
    fn (self: &Self!) format(): string;  // Required

    fn (self: &Self!) format_bold(): string {
        return "**" + self.format() + "**";
    }

    fn (self: &Self!) format_italic(): string {
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

### Where Clauses (Future)

Complex bounds use where clause:

```vex
fn process<T, U>(a: T, b: U)
where
    T: Display & Comparable,
    U: Cloneable
{
    // Implementation
}
```

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

    fn (self: &Self!) next(): Option<Self::Item>;
}
```

**Properties**:

- `type Name` declares associated type
- Implementing types specify concrete type
- Used for output types in trait methods

### Implementation (Future)

```vex
struct Counter impl Iterator {
    type Item = i32;

    current: i32,

    fn (self: &Counter!) next(): Option<i32> {
        let value = self.current;
        self.current = self.current + 1;
        return Option::Some(value);
    }
}
```

### Generic Associated Types (Future)

```vex
trait Container {
    type Item<T>;

    fn (self: &Self!) get<T>(): Self::Item<T>;
}
```

---

## Trait Inheritance

### Supertraits

Traits can require other traits:

```vex
trait Eq {
    fn (self: &Self!) equals(other: &Self): bool;
}

trait Ord: Eq {
    // Ord requires Eq
    fn (self: &Self!) less_than(other: &Self): bool;
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
trait Serializable: Display & Cloneable {
    fn (self: &Self!) serialize(): string;
}
```

---

## Standard Traits

### Display Trait (Future)

Format types for display:

```vex
trait Display {
    fn (self: &Self!) show();
}

struct Point impl Display {
    x: i32,
    y: i32,

    fn (self: &Point!) show() {
        // Print point representation
    }
}
```

### Clone Trait (Future)

Explicit copying:

```vex
trait Clone {
    fn (self: &Self!) clone(): Self;
}

struct Data impl Clone {
    value: i32,

    fn (self: &Data!) clone(): Data {
        return Data { value: self.value };
    }
}
```

### Eq Trait (Future)

Equality comparison:

```vex
trait Eq {
    fn (self: &Self!) equals(other: &Self): bool;
}

struct Point impl Eq {
    x: i32,
    y: i32,

    fn (self: &Point!) equals(other: &Point): bool {
        return self.x == other.x && self.y == other.y;
    }
}
```

### Ord Trait (Future)

Ordering comparison:

```vex
trait Ord {
    fn (self: &Self!) compare(other: &Self): i32;
    // Returns: -1 (less), 0 (equal), 1 (greater)
}

struct Number impl Ord {
    value: i32,

    fn (self: &Number!) compare(other: &Number): i32 {
        if self.value < other.value {
            return -1;
        } elif self.value > other.value {
            return 1;
        } else {
            return 0;
        }
    }
}
```

### Iterator Trait (Future)

Iteration protocol:

```vex
trait Iterator {
    type Item;

    fn (self: &Self!) next(): Option<Self::Item>;
}

struct Range impl Iterator {
    type Item = i32;

    start: i32,
    end: i32,

    fn (self: &Range!) next(): Option<i32> {
        if self.start < self.end {
            let value = self.start;
            self.start = self.start + 1;
            return Option::Some(value);
        }
        return Option::None;
    }
}
```

---

## Examples

### Basic Trait

```vex
trait Greet {
    fn (self: &Self!) say_hello();
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
    fn (self: &Self!) log(msg: string);

    fn (self: &Self!) info(msg: string) {
        self.log(msg);
    }

    fn (self: &Self!) debug(msg: string) {
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
    fn (self: &Self!) area(): i32;
    fn (self: &Self!) perimeter(): i32;
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
    fn (self: &Self!) count(): i32;

    fn (self: &Self!) count_double(): i32 {
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
    fn (self: &Self!) serialize(): string;
}

trait Deserializable {
    fn from_string(s: string): Self;
}

// Bad: Too many responsibilities
trait DataHandler {
    fn (self: &Self!) serialize(): string;
    fn from_string(s: string): Self;
    fn (self: &Self!) validate(): bool;
    fn (self: &Self!) transform(): Self;
}
```

### 2. Descriptive Names

```vex
// Good: Clear purpose
trait Drawable {
    fn (self: &Self!) draw();
}

trait Comparable {
    fn (self: &Self!) compare(other: &Self): i32;
}

// Bad: Vague
trait Handler {
    fn (self: &Self!) handle();
}
```

### 3. Use Default Methods

```vex
// Good: Provide defaults when sensible
trait Logger {
    fn (self: &Self!) log(msg: string);

    fn (self: &Self!) info(msg: string) {
        self.log("[INFO] " + msg);
    }
}

// Bad: Force implementation of similar methods
trait Logger {
    fn (self: &Self!) log(msg: string);
    fn (self: &Self!) info(msg: string);  // No default
    fn (self: &Self!) debug(msg: string); // No default
}
```

### 4. Small Traits

```vex
// Good: Composable traits
trait Display {
    fn (self: &Self!) show();
}

trait Clone {
    fn (self: &Self!) clone(): Self;
}

struct Data impl Display, Clone {
    // Implement both
}

// Bad: Monolithic trait
trait Everything {
    fn (self: &Self!) show();
    fn (self: &Self!) clone(): Self;
    fn (self: &Self!) serialize(): string;
    fn (self: &Self!) validate(): bool;
}
```

### 5. Document Requirements

```vex
// Document trait purpose and requirements
/// Represents types that can be displayed to the user.
/// Implementations should provide a human-readable representation.
trait Display {
    fn (self: &Self!) show();
}

/// Represents types that can be compared for ordering.
/// Returns: -1 if self < other, 0 if equal, 1 if self > other
trait Ord {
    fn (self: &Self!) compare(other: &Self): i32;
}
```

---

## Trait Features Summary

| Feature               | Syntax                 | Status     | Example               |
| --------------------- | ---------------------- | ---------- | --------------------- |
| Trait Definition      | `trait Name { }`       | ✅ Working | Method signatures     |
| Inline Implementation | `struct S impl T { }`  | ✅ Working | v1.3 syntax           |
| Default Methods       | `fn (self) { body }`   | ✅ Working | With implementation   |
| Self Type             | `Self`                 | ✅ Working | Refers to implementer |
| Multiple Methods      | Multiple fn signatures | ✅ Working | In trait body         |
| Separate impl         | `impl T for S { }`     | 🚧 Future  | Outside struct        |
| Multiple Traits       | `impl T1, T2 { }`      | 🚧 Future  | Multiple traits       |
| Trait Bounds          | `<T: Trait>`           | 🚧 Future  | Generic constraints   |
| Associated Types      | `type Item;`           | 🚧 Future  | Type members          |
| Supertraits           | `trait T: U { }`       | ✅ Working | Trait inheritance     |
| Where Clauses         | `where T: Trait`       | 🚧 Future  | Complex bounds        |
| Dynamic Dispatch      | `&dyn Trait`           | 🚧 Future  | Runtime polymorphism  |

---

## Trait System Architecture

### Current Implementation (v1.3)

```vex
// 1. Define trait
trait Logger {
    fn (self: &Self!) log(msg: string);
    fn (self: &Self!) info(msg: string) {
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

1. **Parse**: Trait definition → AST
2. **Register**: Store trait in `trait_defs` HashMap
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
