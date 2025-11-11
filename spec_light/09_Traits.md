# Traits

**Version:** 0.1.0 
**Last Updated:** November 3, 2025

This document defines the trait system in the Vex programming language. Traits provide polymorphism through shared behavior definitions.

---

## Table of Contents

1. \1
2. \1
3. \1
4. \1
5. \1
6. \1
7. \1

---

## Trait Definitions

### Basic Syntax

**Syntax**: `trait Name { methods }`

``````vex
trait Display {
    fn show();
}

trait Comparable {
    fn compare(other: &Self): i32;
}
```

**Properties**:

- `trait` keyword
- Name in PascalCase (convention)
- Method signatures (no body by default)
- `Self` type refers to implementing type
- Can have default method implementations

### Simple Trait

``````vex
trait Greet {
    fn say_hello();
}
```

**Note**: `interface` keyword is deprecated in v0.1, use `trait` instead.

### Multiple Methods

``````vex
trait Shape {
    fn area(): f64;
    fn perimeter(): f64;
    fn name(): string;
}
```

### Self Type

`Self` represents the type implementing the trait:

``````vex
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

**Syntax**:

- **Immutable Method**: `fn method_name(args...): ReturnType;`
- **Mutable Method**: `fn method_name(args...)!;` or `fn method_name(args...)!: ReturnType;`

The `!` indicates that the method requires a mutable reference to `self`, allowing for modifications.

``````vex
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

**Syntax**: `struct MyStruct impl MyTrait { ... methods ... }`

[17 lines code: ```vex]

**Key Rules**:

- Trait methods **MUST** be implemented directly inside the `struct`'s body.
- The method signatures in the implementation must match the trait definition, including the `!` for mutability.

### Multiple Traits (Future)

[16 lines code: ```vex]

### Implementation Requirements

All trait methods must be implemented:

[14 lines code: ```vex]

---

## Default Methods

### Definition

Traits can provide default implementations:

[12 lines code: ```vex]

**Properties**:

- Methods with body are default methods
- Implementing types inherit default behavior
- Can be overridden if needed
- Reduces code duplication

### Inheritance

Structs automatically get default methods:

[20 lines code: ```vex]

### Overriding Defaults

Implementing types can override default methods:

[16 lines code: ```vex]

### Default Method Access

Default methods can call other trait methods:

[11 lines code: ```vex]

---

## Trait Bounds

### Generic Constraints (Future)

Restrict generic types to those implementing specific traits:

``````vex
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

``````vex
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

[18 lines code: ```vex]

**Implementation Details**:

- Parser: `parse_where_clause()` in `vex-parser/src/parser/items/functions.rs:138`
- AST: `WhereClausePredicate { type_param, bounds }`
- Syntax: `where T: Trait1 & Trait2, U: Trait3`
- Test: `examples/test_where_clause.vx`
- Verified: November 9, 2025

**Limitations**:

- Struct inline methods don't support where clauses yet (see `structs.rs:195`)

### Bound on Methods (Future)

``````vex
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

``````vex
trait Iterator {
    type Item;

    fn next(): Option<Self.Item>;
}
```

**Properties**:

- `type Name` declares associated type
- Implementing types specify concrete type
- Used for output types in trait methods

### Implementation (IMPLEMENTED ✅)

[11 lines code: ```vex]

### Generic Associated Types (Future)

``````vex
trait Container {
    type Item<T>;

    fn get<T>(): Self.Item<T>;
}
```

---

## Trait Inheritance

### Supertraits

Traits can require other traits:

``````vex
trait Eq {
    fn equals(other: &Self): bool;
}

trait Ord: Eq {
    // Ord requires Eq
    fn less_than(other: &Self): bool;
}
```

**Implementation**:

[13 lines code: ```vex]

### Multiple Supertraits

``````vex
trait Serializable: Display & Cloneable {
    fn serialize(): string;
}
```

---

## Standard Traits

### Drop Trait ✅ IMPLEMENTED

Automatic resource cleanup when value goes out of scope:

[20 lines code: ```vex]

**Status**: Fully functional, automatic Drop trait implementation detection.

### Clone Trait ✅ IMPLEMENTED

Explicit deep copying:

[16 lines code: ```vex]

**Status**: Fully functional, used for explicit copying.

### Eq Trait ✅ IMPLEMENTED

Equality comparison:

[19 lines code: ```vex]

**Status**: Fully functional, used for custom equality.

### Ord Trait ✅ IMPLEMENTED

Ordering comparison:

[22 lines code: ```vex]

**Status**: Fully functional, used for ordering operations.

### Iterator Trait ✅ IMPLEMENTED

Lazy iteration protocol:

[30 lines code: ```vex]

**Status**: Fully functional with Option<T> support. Associated type `Self.Item` temporarily uses concrete type (Option<i32>) until full generic support.

### Display Trait (Future)

Format types for display:

[12 lines code: ```vex]

**Status**: Planned for future implementation.

---

## Examples

### Basic Trait

[17 lines code: ```vex]

### Default Methods

[27 lines code: ```vex]

### Multiple Methods

[24 lines code: ```vex]

### Overriding Defaults

[20 lines code: ```vex]

---

## Best Practices

### 1. Single Responsibility

[16 lines code: ```vex]

### 2. Descriptive Names

[13 lines code: ```vex]

### 3. Use Default Methods

[15 lines code: ```vex]

### 4. Small Traits

[20 lines code: ```vex]

### 5. Document Requirements

[12 lines code: ```vex]

---

## Trait Features Summary

• Feature — Syntax — Status — Example
• --------------------- — ---------------------- — ---------- — ---------------------
| Trait Definition | `trait Name { }` | ✅ Working | Method signatures |
| Inline Implementation | `struct S impl T { }` | ✅ Working | v1.3 syntax |
| Default Methods | `fn (self) { body }` | ✅ Working | With implementation |
| Self Type | `Self` | ✅ Working | Refers to implementer |
• Multiple Methods — Multiple fn signatures — ✅ Working — In trait body
| Trait Bounds | `<T: Trait>` | ✅ Working | Generic constraints |
| Associated Types | `type Item;` | ✅ Working | Type members |
| Supertraits | `trait T: U { }` | ✅ Working | Trait inheritance |
| Where Clauses | `where T: Trait` | ✅ v0.1.2 | Complex bounds |

---

## Trait System Architecture

### Current Implementation (v1.3)

[26 lines code: ```vex]

### Compilation Process

1. **Parse**: Trait definition → AST
2. **Register**: Store trait in `trait_defs` HashMap
3. **Implement**: Inline `impl Trait` → `trait_impls` HashMap
4. **Codegen**: Generate LLVM IR for methods
5. **Link**: Default methods compiled on-demand
6. **Call**: Method resolution at compile time (static dispatch)

### Future: Dynamic Dispatch

``````vex
// Virtual table (vtable) for runtime polymorphism
fn process(logger: &dyn Logger) {
    logger.log("Dynamic dispatch");
}
```

---

**Previous**: \1 
**Next**: \1

**Maintained by**: Vex Language Team
