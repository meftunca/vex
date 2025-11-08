# Module System & Static Method Call Syntax Migration

**Date:** November 5, 2025  
**Priority:** 🔴 **CRITICAL - Breaking Change**  
**Status:** Planning Phase

---

## 🚨 Problem Statement

**Current Issue:** Documentation uses Rust-style `::` syntax (`Vec::new()`, `Option::Some()`)  
**Reality:** Vex uses JavaScript-style module system with `.` for member access

**This is a BREAKING CHANGE that affects:**

- All builtin type documentation
- Method call syntax design
- Module import/export system
- Associated function calls

---

## 📐 Vex Module System (Existing)

### Import Syntax

```vex
// Named imports
import { io, log } from "std";

// Namespace import
import * as std from "std";
std.io.println("Hello");

// Module import
import "std/io";  // All exports available in scope
```

**Key Points:**

- ✅ JavaScript-style imports
- ✅ No `::` operator (doesn't exist in lexer/parser)
- ✅ `.` for member access (not `::`)

---

## 🎯 Design Decisions Required

### Decision 1: Associated Functions (Static Methods)

**Question:** How to call "constructor-like" functions on types?

#### Option A: Type-prefixed with `.` (TypeScript-style)

```vex
let vec = Vec.new();
let opt = Option.Some(42);
let res = Result.Ok(100);
```

**Pros:**

- Consistent with `.` for member access
- Works with existing parser
- TypeScript/JavaScript familiar

**Cons:**

- Might confuse with instance methods
- Type names are not values (can't pass around)

#### Option B: Namespace import pattern

```vex
// Builtins are auto-imported
let vec = new Vec();       // Constructor-like
let opt = Some(42);        // Enum variant constructor
let res = Ok(100);         // Enum variant constructor
```

**Pros:**

- Clean, minimal syntax
- Clear distinction: `new` = type construction, bare names = variants

**Cons:**

- Global namespace pollution (Some, Ok, Err, None all global)
- Potential conflicts with user code

#### Option C: Hybrid approach (RECOMMENDED)

```vex
// Type constructors: Type.new()
let vec = Vec.new();
let map = Map.new();
let chan = Channel.new(10);

// Enum variants: Direct constructors (auto-imported)
let opt = Some(42);       // Option.Some variant
let none = None;          // Option.None variant
let res = Ok(100);        // Result.Ok variant
let err = Err("failed");  // Result.Err variant

// Instance methods: value.method()
vec.push(1);
opt.unwrap();
```

**Pros:**

- Clear distinction: `Type.constructor()` vs `variant()` vs `value.method()`
- Enum variants feel lightweight (no type prefix)
- Minimal global pollution (only common variants)

**Cons:**

- Mixed syntax styles (but intentional!)

---

### Decision 2: Method Call Syntax

**Question:** How to call methods on instances?

#### ✅ Agreed: `.` for all instance methods

```vex
let! vec = Vec.new();
vec.push(1);           // Instance method
vec.push(2);
let len = vec.len();   // Instance method

let opt = Some(42);
let val = opt.unwrap(); // Instance method
```

**No ambiguity:** Instance methods always use `.`

---

### Decision 3: Trait Methods

**Question:** How to call trait-provided methods?

#### ✅ Same as instance methods: `.`

```vex
// Iterator trait methods
let doubled = numbers.iter()
    .map(|x| x * 2)
    .filter(|x| x > 5)
    .collect();

// All trait methods use . notation
```

---

### Decision 4: Standard Library Functions

**Question:** How to access non-builtin functions?

#### Option A: Namespace import (RECOMMENDED)

```vex
import * as fs from "std/fs";

let file = fs.open("data.txt");
fs.write(file, "Hello");
fs.close(file);
```

#### Option B: Named import

```vex
import { open, write, close } from "std/fs";

let file = open("data.txt");
write(file, "Hello");
close(file);
```

**Both work!** User choice based on preference.

---

## 🔧 Implementation Plan

### Phase 1: Clarify Syntax Rules (Documentation)

**Update all docs to remove `::` syntax:**

1. **BUILTIN_TYPES_ARCHITECTURE.md**

   - `Vec::new()` → `Vec.new()`
   - `Option::Some()` → `Some()` (direct constructor)
   - `Option::None` → `None` (direct constructor)
   - `Result::Ok()` → `Ok()` (direct constructor)
   - `Result::Err()` → `Err()` (direct constructor)

2. **ITERATOR_SYSTEM_DESIGN.md**

   - `Iterator::next()` → `iter.next()` (instance method)
   - All trait methods use `.` notation

3. **VEX_RUNTIME_STDLIB_ROADMAP.md**

   - Update all examples
   - Clarify import patterns

4. **NAMING_DECISIONS.md**
   - Add section on method call syntax
   - Explain Type.new() vs variant() distinction

### Phase 2: Parser/Compiler Support

**Ensure parser handles Type.method() correctly:**

1. **Type-level "static" methods:**

   ```rust
   // vex-parser: Parse Type.method() as special case
   Expression::Call {
       func: Expression::MemberAccess {
           object: Expression::Ident("Vec"),  // Type name
           member: "new",
       },
       args: []
   }
   ```

2. **Codegen: Resolve Type.method() to constructor:**

   ```rust
   // vex-compiler: Check if object is a type name
   if self.is_type_name(&object) {
       // Generate constructor call
       self.compile_type_constructor(type_name, method, args)
   } else {
       // Regular method call
       self.compile_method_call(object, method, args)
   }
   ```

3. **Enum variant constructors (global):**
   ```rust
   // Auto-import into global scope
   self.insert_global("Some", VariantConstructor(Option, Some));
   self.insert_global("None", VariantConstructor(Option, None));
   self.insert_global("Ok", VariantConstructor(Result, Ok));
   self.insert_global("Err", VariantConstructor(Result, Err));
   ```

### Phase 3: Examples & Tests

**Update all .vx files:**

```bash
# Search for :: usage
grep -r "::" examples/*.vx

# Replace patterns
Vec::new()        → Vec.new()
Option::Some(x)   → Some(x)
Option::None      → None
Result::Ok(x)     → Ok(x)
Result::Err(e)    → Err(e)
HashMap::new()    → Map.new()
```

---

## 📝 Syntax Reference (Complete)

### Type Constructors (Associated Functions)

```vex
// Builtin types - use Type.method()
let vec = Vec.new();
let vec_cap = Vec.with_capacity(10);
let map = Map.new();
let set = Set.new();
let chan = Channel.new(100);

// String
let s = String.new();
let s_from = String.from("hello");
```

### Enum Variant Constructors (Global)

```vex
// Option variants
let some = Some(42);      // Option<i32>
let none = None;          // Option<i32>

// Result variants
let ok = Ok(100);         // Result<i32, E>
let err = Err("failed");  // Result<T, str>

// Pattern matching (no change)
match opt {
    Some(x) => println(x),
    None => println("empty"),
}
```

### Instance Methods

```vex
let! vec = Vec.new();
vec.push(1);              // Instance method
vec.push(2);
let len = vec.len();      // Instance method
let first = vec.get(0);   // Returns Option<&T>

let opt = Some(42);
let val = opt.unwrap();   // Instance method
let is_some = opt.is_some();
```

### Trait Methods

```vex
// Iterator trait
let numbers = Vec.from([1, 2, 3, 4, 5]);
let doubled = numbers.iter()  // Instance method (returns iterator)
    .map(|x| x * 2)           // Trait method
    .filter(|x| x > 5)        // Trait method
    .collect();               // Trait method
```

### Module Imports

```vex
// Namespace import
import * as fs from "std/fs";
let file = fs.open("data.txt");

// Named import
import { open, close } from "std/fs";
let file = open("data.txt");
close(file);

// Auto-imported (builtins)
let vec = Vec.new();   // No import needed
let opt = Some(42);    // No import needed
```

---

## 🔄 Migration Checklist

### Documentation

- [ ] Update `BUILTIN_TYPES_ARCHITECTURE.md` - Remove all `::`
- [ ] Update `ITERATOR_SYSTEM_DESIGN.md` - Use `.` for methods
- [ ] Update `VEX_RUNTIME_STDLIB_ROADMAP.md` - Fix all examples
- [ ] Update `NAMING_DECISIONS.md` - Add syntax section
- [ ] Update `BUILTIN_TYPES_QUICKSTART.md` - Correct syntax
- [ ] Create `MODULE_SYSTEM_GUIDE.md` - Comprehensive guide

### Parser/Compiler

- [ ] Verify `Type.method()` parsing works
- [ ] Implement type name detection in codegen
- [ ] Add builtin variant constructors to global scope
- [ ] Test associated function calls
- [ ] Test enum variant construction

### Examples & Tests

- [ ] Search all `.vx` files for `::`
- [ ] Update all examples
- [ ] Update all test files
- [ ] Add test for `Type.method()` syntax
- [ ] Add test for variant constructors

---

## 📊 Breaking Changes Summary

| Old (Incorrect)        | New (Correct)         | Category         |
| ---------------------- | --------------------- | ---------------- |
| `Vec::new()`           | `Vec.new()`           | Type constructor |
| `Vec::with_capacity()` | `Vec.with_capacity()` | Type constructor |
| `Option::Some(x)`      | `Some(x)`             | Enum variant     |
| `Option::None`         | `None`                | Enum variant     |
| `Result::Ok(x)`        | `Ok(x)`               | Enum variant     |
| `Result::Err(e)`       | `Err(e)`              | Enum variant     |
| `HashMap::new()`       | `Map.new()`           | Type constructor |
| `iter::Iterator`       | `Iterator` trait      | Module path      |

**Impact:** Documentation only (parser/compiler already correct!)

---

## 🎯 Final Syntax Rules

### 1. Type Constructors: `Type.method()`

```vex
Vec.new()
Map.new()
String.from("hello")
```

### 2. Enum Variants: Direct names

```vex
Some(42)
None
Ok(100)
Err("failed")
```

### 3. Instance Methods: `value.method()`

```vex
vec.push(1)
opt.unwrap()
iter.next()
```

### 4. Trait Methods: `value.trait_method()`

```vex
numbers.iter().map(|x| x * 2).collect()
```

### 5. Module Functions: `module.function()`

```vex
import * as fs from "std/fs";
fs.open("file.txt")
```

### 6. Named Imports: Direct names

```vex
import { open } from "std/fs";
open("file.txt")
```

---

## 🚀 Implementation Priority

1. **Phase 1 (HIGH):** Update all documentation (1-2 hours)
2. **Phase 2 (MEDIUM):** Verify parser/compiler (2-3 hours)
3. **Phase 3 (LOW):** Update examples (1 hour)

**Total:** 4-6 hours to fix all syntax issues

---

**Decision:** Use **Option C (Hybrid approach)**

- Type constructors: `Type.new()`
- Enum variants: `Some()`, `None`, `Ok()`, `Err()`
- Instance methods: `value.method()`
- Consistent with JavaScript/TypeScript
- No `::` operator needed
