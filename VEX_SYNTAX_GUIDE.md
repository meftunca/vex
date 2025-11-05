# Vex Syntax Guide - Method Calls & Constructors

**⚠️ IMPORTANT:** Vex does NOT use `::` operator. Use `.` for all member access.

---

## ✅ Correct Syntax

### Type Constructors (Associated Functions)

Use `Type.method()` syntax:

```vex
// Builtin types
let vec = Vec.new();
let vec_with_cap = Vec.with_capacity(10);
let map = Map.new();
let set = Set.new();
let chan = Channel.new(100);
let string = String.from("hello");

// ❌ WRONG: Vec::new()
// ✅ CORRECT: Vec.new()
```

### Enum Variant Constructors

Use direct constructor names (auto-imported):

```vex
// Option variants
let some = Some(42);
let none = None;

// Result variants
let ok = Ok(100);
let err = Err("failed");

// ❌ WRONG: Option::Some(42), Result::Ok(100)
// ✅ CORRECT: Some(42), Ok(100)
```

### Instance Methods

Use `value.method()` syntax:

```vex
let! vec = Vec.new();
vec.push(1);            // Instance method
vec.push(2);
let len = vec.len();    // Instance method

let opt = Some(42);
let val = opt.unwrap(); // Instance method

// ❌ WRONG: Vec::push(vec, 1)
// ✅ CORRECT: vec.push(1)
```

### Trait Methods

Use `value.trait_method()` syntax:

```vex
let numbers = Vec.from([1, 2, 3, 4, 5]);
let doubled = numbers.iter()     // Instance method
    .map(|x| x * 2)              // Trait method
    .filter(|x| x > 5)           // Trait method
    .collect();                  // Trait method

// ❌ WRONG: Iterator::map(iter, |x| x * 2)
// ✅ CORRECT: iter.map(|x| x * 2)
```

### Module Imports

Use JavaScript-style imports:

```vex
// Namespace import
import * as fs from "std/fs";
let file = fs.open("data.txt");
fs.write(file, "Hello");

// Named import
import { open, write, close } from "std/fs";
let file = open("data.txt");
write(file, "Hello");

// ❌ WRONG: std::fs::open()
// ✅ CORRECT: fs.open() or open() (after import)
```

---

## 📋 Common Replacements

| ❌ Incorrect (Rust-style)   | ✅ Correct (Vex syntax)                          |
| --------------------------- | ------------------------------------------------ |
| `Vec::new()`                | `Vec.new()`                                      |
| `Vec::with_capacity(10)`    | `Vec.with_capacity(10)`                          |
| `Map::new()`                | `Map.new()`                                      |
| `Set::new()`                | `Set.new()`                                      |
| `String::from("hi")`        | `String.from("hi")`                              |
| `Option::Some(x)`           | `Some(x)`                                        |
| `Option::None`              | `None`                                           |
| `Result::Ok(x)`             | `Ok(x)`                                          |
| `Result::Err(e)`            | `Err(e)`                                         |
| `Channel::new(10)`          | `Channel.new(10)`                                |
| `std::collections::HashMap` | `import * as collections from "std/collections"` |

---

## 🎯 Key Rules

1. **Type constructors**: Use `.` (e.g., `Vec.new()`)
2. **Enum variants**: Direct names (e.g., `Some(x)`, `None`)
3. **Instance methods**: Use `.` (e.g., `vec.push(1)`)
4. **Trait methods**: Use `.` (e.g., `iter.map(...)`)
5. **Module access**: Use `.` after import (e.g., `fs.open()`)
6. **NO `::` operator**: Does not exist in Vex!

---

## 💡 Why This Design?

1. **Consistency**: `.` for all member access
2. **Familiarity**: JavaScript/TypeScript developers feel at home
3. **Simplicity**: One operator (`.`) instead of two (`::` and `.`)
4. **Clarity**: `Type.constructor()` vs `value.method()` distinction is clear

---

## 📚 See Also

- `MODULE_SYSTEM_SYNTAX_FIX.md` - Complete migration guide
- `BUILTIN_TYPES_ARCHITECTURE.md` - Builtin types design
- `ITERATOR_SYSTEM_DESIGN.md` - Iterator trait details
