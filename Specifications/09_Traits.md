## Deprecated: Traits spec

This file (`09_Traits.md`) is deprecated. The Vex language adopted the `contract` keyword and the canonical documentation was consolidated in `Specifications/09_Contracts.md`.

Please see: [Specifications/09_Contracts.md](./09_Contracts.md)

This file may be removed in a future release; it currently remains for historical reference.

### Definition (Future)

Contracts can have associated types:

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

### Supercontracts

Contracts can require other contracts:

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

### Multiple Supercontracts

```vex
contract Serializable: Display & Cloneable {
    serialize(): string;
}
```

---

## Standard Contracts

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

### Basic Contract

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
// Good: Focused contract
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

### 4. Small Contracts

```vex
// Good: Composable contracts
contract Display {
    show();
}

contract Clone {
    clone(): Self;
}

struct Data impl Display, Clone {
    // Implement both
}

// Bad: Monolithic contract
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
| Contract Definition   | `contract Name { }`    | ✅ Working | Method signatures     |
| Inline Implementation | `struct S impl T { }`  | ✅ Working | v1.3 syntax           |
| Default Methods       | `fn (self) { body }`   | ✅ Working | With implementation   |
| Self Type             | `Self`                 | ✅ Working | Refers to implementer |
| Multiple Methods      | Multiple fn signatures | ✅ Working | In contract body      |
| Contract Bounds       | `<T: Contract>`        | ✅ Working | Generic constraints   |
| Associated Types      | `type Item;`           | ✅ Working | Type members          |
| Supercontracts        | `contract T: U { }`    | ✅ Working | Contract inheritance  |
| Where Clauses         | `where T: Contract`    | ✅ v0.1.2  | Complex bounds        |

---

## Contract System Architecture

### Current Implementation (v1.3)

```vex
// 1. Define contract
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
3. **Implement**: Inline `impl Contract` → `contract_impls` HashMap
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
