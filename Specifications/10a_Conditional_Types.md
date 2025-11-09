# Conditional Types (TypeScript-inspired)

**Status:** 🚧 Planned (Not Implemented)  
**Version:** Future (v1.0+)  
**Last Updated:** November 9, 2025

This document describes Vex's planned conditional type system, inspired by TypeScript's `T extends U ? X : Y` syntax for advanced type-level programming.

---

## Table of Contents

1. [Overview](#overview)
2. [Basic Syntax](#basic-syntax)
3. [Use Cases](#use-cases)
4. [Type-Level Conditionals](#type-level-conditionals)
5. [Distributive Conditional Types](#distributive-conditional-types)
6. [Infer Keyword](#infer-keyword)
7. [Comparison with TypeScript](#comparison-with-typescript)

---

## Overview

Conditional types allow types to be chosen based on a condition evaluated at compile time. This enables powerful type-level programming patterns for generic libraries and frameworks.

### Why Conditional Types?

**Problem:** Generic code often needs different behavior based on type properties:

```vex
// How to return different types based on input type?
fn process<T>(value: T): ??? {
    // If T is String, return i32 (length)
    // If T is i32, return String (formatted)
}
```

**Solution:** Conditional types express this at the type level:

```vex
type ProcessResult<T> = T extends String ? i32 : T extends i32 ? String : T;

fn process<T>(value: T): ProcessResult<T> {
    // Compiler knows the return type based on T
}
```

---

## Basic Syntax

### Type Condition Expression

```vex
type ConditionalType<T> = T extends U ? X : Y;
```

**Meaning:**

- If `T` is assignable to `U`, the type is `X`
- Otherwise, the type is `Y`

### Simple Example

```vex
type IsString<T> = T extends String ? true : false;

// Usage
type A = IsString<String>;  // true
type B = IsString<i32>;     // false
```

---

## Use Cases

### 1. Type-Based Return Types

```vex
type ReturnType<T> =
    T extends String ? i32 :
    T extends i32 ? String :
    T;

fn convert<T>(value: T): ReturnType<T> {
    // Implementation inferred by compiler
}

let x: i32 = convert("hello");      // OK: String → i32
let y: String = convert(42);        // OK: i32 → String
let z: bool = convert(true);        // OK: bool → bool
```

### 2. Extract Array Element Type

```vex
type ElementType<T> = T extends [U] ? U : never;

type A = ElementType<[i32]>;        // i32
type B = ElementType<[String]>;     // String
type C = ElementType<i32>;          // never (not an array)
```

### 3. Optional Type Unwrapping

```vex
type Unwrap<T> = T extends Option<U> ? U : T;

type A = Unwrap<Option<i32>>;       // i32
type B = Unwrap<i32>;               // i32
```

### 4. Function Return Type Extraction

```vex
type ReturnOf<T> = T extends fn(...): R ? R : never;

fn add(a: i32, b: i32): i32 { return a + b; }

type AddReturn = ReturnOf<typeof add>;  // i32
```

---

## Type-Level Conditionals

### Nested Conditionals

```vex
type TypeName<T> =
    T extends String ? "string" :
    T extends i32 ? "i32" :
    T extends bool ? "bool" :
    "unknown";

type A = TypeName<String>;  // "string"
type B = TypeName<i32>;     // "i32"
type C = TypeName<f64>;     // "unknown"
```

### Multiple Conditions

```vex
type IsNumeric<T> =
    T extends i32 ? true :
    T extends i64 ? true :
    T extends f32 ? true :
    T extends f64 ? true :
    false;

type A = IsNumeric<i32>;    // true
type B = IsNumeric<String>; // false
```

---

## Distributive Conditional Types

When `T` is a union type, conditional types **distribute** over the union:

```vex
type ToArray<T> = T extends U ? [U] : never;

type A = ToArray<String | i32>;
// Distributes to: ToArray<String> | ToArray<i32>
// Result: [String] | [i32]
```

### Filtering Union Types

```vex
type NonNullable<T> = T extends nil ? never : T;

type A = NonNullable<String | nil>;  // String
type B = NonNullable<i32 | nil>;     // i32
```

### Extracting from Unions

```vex
type ExtractStrings<T> = T extends String ? T : never;

type A = ExtractStrings<String | i32 | bool>;  // String
```

---

## Infer Keyword

The `infer` keyword allows **extracting types** from within a conditional type:

### Basic Inference

```vex
type GetReturnType<T> = T extends fn(...): infer R ? R : never;

fn foo(): i32 { return 42; }

type FooReturn = GetReturnType<typeof foo>;  // i32
```

### Array Element Inference

```vex
type Flatten<T> = T extends [infer U] ? U : T;

type A = Flatten<[i32]>;    // i32
type B = Flatten<i32>;      // i32
```

### Multiple Infers

```vex
type GetParams<T> = T extends fn(infer P1, infer P2): R ? [P1, P2] : never;

fn add(a: i32, b: i32): i32 { return a + b; }

type AddParams = GetParams<typeof add>;  // [i32, i32]
```

---

## Comparison with TypeScript

### Similarities

| Feature                | TypeScript            | Vex (Planned)             |
| ---------------------- | --------------------- | ------------------------- |
| Basic Syntax           | `T extends U ? X : Y` | `T extends U ? X : Y`     |
| Distributive Types     | ✅ Yes                | ✅ Yes (planned)          |
| Infer Keyword          | ✅ `infer R`          | ✅ `infer R` (planned)    |
| Type-level Programming | ✅ Full support       | ✅ Full support (planned) |

### Differences

| Feature       | TypeScript            | Vex (Planned)          |
| ------------- | --------------------- | ---------------------- | ------------------------ |
| Type Aliases  | `type X = ...`        | `type X = ...` (same)  |
| Trait Bounds  | Interface constraints | `T: Trait` constraints |
| Literal Types | `"string"             | "number"`              | String literals as types |
| Never Type    | `never`               | `never` (same)         |

---

## Implementation Plan

### Phase 1: Basic Conditionals (v1.0)

```vex
type IsString<T> = T extends String ? true : false;
```

**Requirements:**

- Parser: Extend type syntax to support `extends`, `?`, `:`
- Type Checker: Evaluate conditionals at compile time
- Codegen: No runtime impact (all compile-time)

### Phase 2: Infer Keyword (v1.1)

```vex
type ReturnType<T> = T extends fn(...): infer R ? R : never;
```

**Requirements:**

- Parser: Support `infer` in type expressions
- Type Checker: Extract and bind inferred types
- AST: Add `Type::Infer { name: String }`

### Phase 3: Distributive Types (v1.2)

```vex
type ToArray<T> = T extends U ? [U] : never;
type A = ToArray<String | i32>;  // [String] | [i32]
```

**Requirements:**

- Type Checker: Distribute conditionals over union types
- Optimization: Simplify nested unions

---

## Examples

### Practical Use Case: Generic API Response

```vex
trait Deserialize {
    fn deserialize(data: String): Self;
}

type ApiResponse<T> = T extends Deserialize ? Result<T, String> : never;

fn fetch<T: Deserialize>(url: String): ApiResponse<T> {
    // Conditional return type ensures T is Deserializable
}
```

### Type-Safe Event Handlers

```vex
type EventHandler<E> =
    E extends MouseEvent ? fn(MouseEvent) :
    E extends KeyEvent ? fn(KeyEvent) :
    fn(Event);

fn on<E>(event_name: String, handler: EventHandler<E>) {
    // Type-safe event handling
}
```

---

## Status

**Current Status:** 🚧 Not Implemented  
**Target Version:** v1.0  
**Priority:** MEDIUM (powerful but not essential for v1.0)

**Dependencies:**

- ✅ Type system (implemented)
- ✅ Generics (implemented)
- ✅ Trait bounds (implemented)
- ❌ Advanced type inference (planned)

---

**Previous**: [10_Generics.md](./10_Generics.md)  
**Next**: [11_Pattern_Matching.md](./11_Pattern_Matching.md)

**Maintained by**: Vex Language Team
