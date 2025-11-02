# Vex v0.6 - Syntax Changes Summary

## 🎯 Major Changes Overview

### 1. Variable Declaration (Complete Redesign)

#### ❌ Old Syntax (v0.4)

```javascript
let project = "Vex";           // Immutable
let mut counter = 0;           // Mutable
let version: f64 = 0.4;        // Explicit type
```

#### ✅ New Syntax (v0.6)

```javascript
// Mutable by default
x := 10;                       // Type inferred (Go-style)
int32 y = 20;                  // Explicit type (C-style)
name := "Vex";                 // Mutable

// Immutable with const
const PI := 3.14159;           // Type inferred
const string VERSION = "0.6";  // Explicit type
```

**Rationale:** Simpler, more intuitive. Follows Go's `:=` for inference and C's type-first syntax for clarity.

---

### 2. Reference & Pointer System

#### ❌ Old Syntax

```javascript
&T        // Immutable reference
&mut T    // Mutable reference
*mut T    // Raw mutable pointer
*const T  // Raw immutable pointer
```

#### ✅ New Syntax

```javascript
// Safe references (for 99% of code)
&T        // Immutable reference (unchanged)
*T        // Mutable reference (simplified from &mut T)

// Unsafe raw pointers (for FFI/unsafe code only)
*const T  // Immutable raw pointer
*mut T    // Mutable raw pointer
```

**Rationale:** Less verbose for common case. `*T` is shorter than `&mut T` and visually distinct from `&T`.

---

### 3. Built-in Collection Types

#### ❌ Old Syntax

```javascript
map[string]string              // Go-style but inconsistent
```

#### ✅ New Syntax

```javascript
Map<K, V>                      // TypeScript-style mutable hash map
Record<K, V>                   // TypeScript-style immutable map

// Examples
type StringMap = Map<string, string>;
type Config = Record<string, string>;
```

**Rationale:** Consistent with generic type syntax. Aligns with TypeScript naming conventions.

---

### 4. Heap Allocation

#### Old & New (Updated return types)

```javascript
// Old
let v_heap = new(Vector2{...});  // Returns: &mut Vector2

// New
v_heap := new(Vector2{...});     // Returns: *Vector2
buf := make([byte], 1024);       // Returns: *[byte]
```

**Rationale:** Consistent with new reference syntax.

---

### 5. New Language Features

#### Enums (Sum Types)

```javascript
enum Option<T> {
    Some(T),
    None,
}

enum HttpMethod {
    GET,
    POST,
    PUT,
}
```

#### Constants

```javascript
const PI := 3.14159;
const int32 MAX_SIZE = 1024;
```

#### Switch Statement (Go-style)

```javascript
switch x {
    case 1:
        log.info("One");
    case 2, 3:
        log.info("Two or Three");
    default:
        log.info("Other");
}

// Type switch for enums
switch opt.(type) {
    case Option::Some(value):
        io.print(f"Value: {value}");
    case Option::None:
        io.print("No value");
}
```

#### Select Statement (Async)

```javascript
select {
    res1 = await task1() => {
        log.info(f"Task 1: {res1}");
    },
    res2 = await task2() => {
        log.info(f"Task 2: {res2}");
    },
}
```

#### Unsafe Blocks

```javascript
unsafe {
    *mut int32 ptr = &x as *mut int32;
    *ptr = 100;
}
```

#### Extern Blocks (FFI)

```javascript
extern "C" {
    fn printf(format: *const byte, ...) -> int;
}
```

#### Increment/Decrement

```javascript
for i := 0; i < 10; i++ {
    // ...
}

x := 5;
x++;  // x = 6
x--;  // x = 5
```

---

## 📊 Comparison Table

| Feature       | Old (v0.4)            | New (v0.6)           | Inspiration |
| ------------- | --------------------- | -------------------- | ----------- |
| Mutable var   | `let mut x = 10`      | `x := 10`            | Go          |
| Immutable var | `let x = 10`          | `const x := 10`      | Go/C        |
| Explicit type | `let x: i32 = 10`     | `int32 x = 10`       | C           |
| Mutable ref   | `&mut T`              | `*T`                 | Simplified  |
| Raw pointer   | `*mut T` / `*const T` | Same                 | Rust        |
| Map type      | `map[K]V`             | `Map<K,V>`           | TypeScript  |
| Enum          | ❌                    | `enum { ... }`       | Rust/TS     |
| Switch        | ❌                    | `switch { ... }`     | Go          |
| Increment     | ❌                    | `x++` / `x--`        | C/Go        |
| Unsafe        | ❌                    | `unsafe { ... }`     | Rust        |
| Extern        | ❌                    | `extern "C" { ... }` | Rust        |

---

## 🎨 Design Philosophy

### From Specification v0.6:

> Vex, Go'nun basitliğini, Rust'ın bellek güvenliği garantilerini ve TypeScript'in gelişmiş tip sistemini birleştirir.

### Key Principles:

1. **Simplicity First** - Reduce cognitive load (Go)
2. **Safety by Default** - Unsafe requires explicit opt-in (Rust)
3. **Powerful Type System** - Generics, unions, conditionals (TypeScript)
4. **Modern Hardware** - First-class SIMD/GPU support

---

## 🔧 Implementation Status

### ✅ Completed

- [x] Lexer updated with new tokens (`:=`, `++`, `--`, `const`, `unsafe`, etc.)
- [x] AST updated with new node types (VarDecl, Enum, Const, Switch, Select, etc.)
- [x] Specification v0.6 fully documented
- [x] Example file created and syntax-checked

### 🚧 Todo

- [ ] Parser implementation (lalrpop grammar)
- [ ] Type checker for new syntax
- [ ] LLVM IR generation for new features
- [ ] Standard library (Map, Record, Option, etc.)

---

## 📝 Migration Guide (v0.4 → v0.6)

### Variables

```javascript
// Before
let x = 10;
let mut y = 20;
let z: i32 = 30;

// After
const x := 10;     // or: const int32 x = 10;
y := 20;           // or: int32 y = 20;
const z: int32 = 30;  // or: const int32 z = 30;
```

### References

```javascript
// Before
fn process(data: &mut Vec<i32>) { }

// After
fn process(data: *Vec<int32>) { }
```

### Collections

```javascript
// Before
type StringMap = map[string]string;

// After
type StringMap = Map<string, string>;
```

---

## 🚀 Next Steps

1. **Parser Development**: Implement lalrpop grammar for new syntax
2. **Type System**: Handle Map<K,V>, Record<K,V>, enum types
3. **Standard Library**: Implement built-in types
4. **Compiler**: Generate LLVM IR for new constructs
5. **Testing**: Comprehensive test suite for new features

---

**Version**: 0.6
**Date**: November 2025
**Status**: Specification Complete ✅
