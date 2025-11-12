# Vex Core Strategy - Foundation First

**Date:** November 12, 2025  
**Status:** ACTIVE - Critical Path  
**Goal:** Build solid foundation before adding features

---

## 🎯 The Two Critical Blockers

### 1. Compiler Builtin vs Stdlib Confusion
**Problem:** Unclear separation between what should be:
- Compiler intrinsic (LLVM IR)
- Runtime library (C/Rust)
- Standard library (Vex code)

**Impact:** 
- Can't complete stdlib
- Inconsistent behavior
- Poor performance
- Maintenance nightmare

### 2. Missing Contract System Infrastructure
**Problem:** No systematic way to handle:
- `Display` / `ToString` (string conversion)
- `Debug` (debug printing)
- `Clone` (deep copy)
- Operator overloading (`Add`, `Sub`, `Mul`, `Eq`)

**Impact:**
- Can't write ergonomic code
- No operator overloading
- Poor interoperability
- Every type is isolated

---

## 📐 Architecture Decision: Three Layers

### Layer 1: Compiler Intrinsics (LLVM IR)
**What belongs here:** Zero-cost, fundamental operations

```
✅ Primitive types (i32, f64, bool, string)
✅ Basic operators (+, -, *, /, %, ==, !=, <, >, &&, ||)
✅ Memory operations (size_of, align_of, offset_of)
✅ Control flow (if, loop, match, break, continue, return)
✅ References (&T, &T!)
✅ Arrays (fixed size [T; N])
✅ Tuples (T1, T2, T3)
✅ Function pointers
✅ Type casting (as operator)
```

**Implementation:** Direct LLVM IR generation in `vex-compiler`

---

### Layer 2: Runtime Library (C ABI)
**What belongs here:** Efficient, reusable implementations

**IMPORTANT:** These are **globally available** - auto-imported in every Vex file (prelude).

```
✅ Core Collections (GLOBAL PRELUDE):
   - Vec<T>          // Dynamic array - always available
   - Map<K, V>       // Hash map - always available
   - Set<T>          // Hash set - always available
   - String          // UTF-8 string - always available

✅ Smart Pointers (GLOBAL PRELUDE):
   - Box<T>          // Heap allocation - always available
   - Rc<T>           // [future] Reference counting
   - Arc<T>          // [future] Atomic ref count

✅ Result Types (GLOBAL PRELUDE):
   - Option<T>       // Some(T) | None - always available
   - Result<T, E>    // Ok(T) | Err(E) - always available

✅ Basic I/O (GLOBAL PRELUDE):
   - print()         // Print without newline
   - println()       // Print with newline
   - eprint()        // Error output without newline
   - eprintln()      // Error output with newline
   - dbg!()          // Debug macro (uses Debug contract)

✅ Utility Functions (GLOBAL PRELUDE):
   - panic()         // Panic with message
   - assert!()       // Runtime assertion
   - todo!()         // Mark unimplemented code
   - unreachable!()  // Mark unreachable code

✅ I/O (Explicit Import):
   - File            // import { File } from "io"
   - stdin/stdout    // import { stdin, stdout } from "io"
   - Network         // [future]

✅ Concurrency (Explicit Import):
   - Thread          // import { Thread } from "thread"
   - Channel<T>      // import { Channel } from "sync"
   - Mutex<T>        // [future]

✅ System (Explicit Import):
   - Time/Duration   // import { Time } from "time"
   - Path/PathBuf    // import { Path } from "fs"
   - Environment     // import { env } from "env"
```

**Prelude Strategy:**
- **Global (no import):** Vec, Map, Set, String, Box, Option, Result, print/println, panic/assert
- **Must import:** Everything else (File, Thread, Time, etc.)

**Rationale:**
- Collections used in 90%+ of programs → global
- Option/Result fundamental to error handling → global
- print/println/dbg needed for debugging → global
- panic/assert/todo needed for development → global
- Specialized features (File, Thread) → explicit import

**Implementation:** 
- C runtime: `vex-runtime/` (currently exists)
- Rust runtime: `vex-libs/runtime/` (future, better safety)
- Vex headers: Extern declarations

**ABI Contract:**
```c
// Clear, stable C ABI
void* vex_vec_new(size_t elem_size);
void vex_vec_push(void* vec, void* elem);
void* vex_vec_get(void* vec, size_t index);
size_t vex_vec_len(void* vec);
void vex_vec_free(void* vec);
```

---

### Layer 3: Standard Library (Vex Code)
**What belongs here:** High-level, composable abstractions

```
✅ Algorithms:
   - sort, search, binary_search
   - map, filter, reduce
   - zip, enumerate, take, skip

✅ Data structures:
   - LinkedList<T>
   - BinaryTree<T>
   - Graph<T>

✅ Utilities:
   - math functions (sqrt, pow, sin, cos)
   - string manipulation
   - formatting helpers

✅ Patterns:
   - Iterator adapters
   - Builder pattern
   - Visitor pattern
```

**Implementation:** Pure Vex code using Layer 1 + 2

---

## 🔧 Contract System Design

### Core Contracts (Must Implement)

#### 1. Display Contract
```vex
// User-facing string representation
contract Display {
    fn to_string(): string;
}

// Builtin contract extensions (declared in stdlib, implemented by compiler)
// vex-libs/std/core/builtin_contracts.vx
i32 extends Display, Clone, Eq, Debug, Hash;
f64 extends Display, Clone, Eq, Debug;
bool extends Display, Clone, Eq, Debug, Hash;
string extends Display, Clone, Eq, Debug, Hash;

// Compiler auto-generates implementations:
// i32.to_string() → "42"
// f64.to_string() → "3.14"
// bool.to_string() → "true"/"false"

// User implementations (existing syntax)
struct Point impl Display {
    x: i32,
    y: i32,
    
    fn to_string(): string {
        return "Point(" + self.x.to_string() + ", " + self.y.to_string() + ")";
    }
}
```

**Compiler Integration:**
```rust
// In codegen: when calling .to_string()
if has_display_impl(type) {
    call_contract_method("Display", "to_string", receiver)
} else {
    compile_error!("Type does not implement Display")
}
```

---

#### 2. Debug Contract
```vex
// Developer-facing representation (with type info)
contract Debug {
    fn debug(): string;
}

// Builtin contract extensions (declared in stdlib)
// vex-libs/std/core/builtin_contracts.vx
i32 extends Debug;
f64 extends Debug;
bool extends Debug;
string extends Debug;

// Compiler auto-generates:
// i32(42).debug() → "i32(42)"
// f64(3.14).debug() → "f64(3.14)"
// bool(true).debug() → "bool(true)"
// string("hello").debug() → "string(\"hello\")"

// User implementations
struct Point impl Debug {
    x: i32,
    y: i32,
    
    fn debug(): string {
        return "Point { x: " + self.x.debug() + ", y: " + self.y.debug() + " }";
    }
}

// Auto-derive (future feature)
struct Config impl Debug;  // Compiler generates debug() method
```

---

#### 3. Clone Contract
```vex
// Deep copy
contract Clone {
    fn clone(): Self;
}

// Builtin contract extensions (declared in stdlib)
// vex-libs/std/core/builtin_contracts.vx
i32 extends Clone;
f64 extends Clone;
bool extends Clone;
// Primitives are Copy types - clone is bitwise copy

// User implementation
struct Point impl Clone {
    x: i32,
    y: i32,
    
    fn clone(): Point {
        return Point { x: self.x, y: self.y };
    }
}

// Future: Auto-derive for simple structs
struct Vec2 impl Clone;  // Compiler generates clone() if all fields are Clone
```

---

#### 4. Eq Contract (Equality)
```vex
contract Eq {
    fn eq(other: &Self): bool;
    fn ne(other: &Self): bool {
        return !self.eq(other);  // Default implementation
    }
}

// Compiler uses this for == and != operators
let a = Point { x: 1, y: 2 };
let b = Point { x: 1, y: 2 };
if a == b {  // Calls a.eq(&b)
    println("Equal!");
}
```

---

#### 5. Ord Contract (Ordering)
```vex
contract Ord {
    fn cmp(other: &Self): i32;  // -1, 0, 1
    
    // Default implementations
    fn lt(other: &Self): bool { return self.cmp(other) < 0; }
    fn le(other: &Self): bool { return self.cmp(other) <= 0; }
    fn gt(other: &Self): bool { return self.cmp(other) > 0; }
    fn ge(other: &Self): bool { return self.cmp(other) >= 0; }
}

// Compiler uses this for <, <=, >, >= operators
```

---

#### 6. Operator Contracts
```vex
contract Add {
    fn add(other: Self): Self;
}

contract Sub {
    fn sub(other: Self): Self;
}

contract Mul {
    fn mul(other: Self): Self;
}

contract Div {
    fn div(other: Self): Self;
}

// Usage:
struct Vec2 impl Add, Sub, Mul {
    x: f64,
    y: f64,
    
    fn add(other: Vec2): Vec2 {
        return Vec2 { x: self.x + other.x, y: self.y + other.y };
    }
    
    fn sub(other: Vec2): Vec2 {
        return Vec2 { x: self.x - other.x, y: self.y - other.y };
    }
    
    fn mul(other: Vec2): Vec2 {
        return Vec2 { x: self.x * other.x, y: self.y * other.y };
    }
}

// Now works:
let a = Vec2 { x: 1.0, y: 2.0 };
let b = Vec2 { x: 3.0, y: 4.0 };
let c = a + b;  // Calls a.add(b)
```

---

## 🚀 Implementation Roadmap

### Phase 1: Contract Infrastructure (Week 1)
**Goal:** Make contracts work at compiler level

#### Day 1-2: Contract Dispatch System
- [ ] **Builtin contract extensions**
  - Create `vex-libs/std/core/builtin_contracts.vx`
  - Add `extends` keyword to lexer/parser
  - Syntax: `i32 extends Display, Clone, Eq, Debug;`
  - Compiler recognizes these and provides implementations
  - Clean, documented, visible in stdlib

- [ ] **Contract registry** in compiler
  - Track builtin extensions (`i32 extends Display`)
  - Track user implementations (`struct Point impl Display`)
  - Store method signatures
  - Validate implementations
  
- [ ] **Method dispatch** in codegen
  - `x.to_string()` → lookup Display impl → call method
  - Cache dispatch for performance
  
- [ ] **Builtin contract impls** (in Rust compiler code)
  - Create `vex-compiler/src/builtin_contracts.rs`
  - Codegen for `i32.to_string()`, `i32.clone()`, `i32.eq()`, etc.
  - Triggered by `extends` declarations in builtin_contracts.vx
  - Pure compiler magic - no Vex function bodies needed

**Files to create/modify:**
```
vex-compiler/src/
├── builtin_contracts.rs     [NEW] - Codegen for builtin contract methods
├── contract_registry.rs     [NEW] - Track extensions & implementations
├── codegen_ast/
│   ├── contracts.rs         [NEW] - Contract dispatch logic
│   └── expressions/
│       └── method_calls.rs  [MODIFY] - Check builtin first, then user impls

vex-parser/src/
├── lexer/tokens.rs          [MODIFY] - Add Token::Extends
└── parser/items/
    └── contracts.rs         [NEW] - Parse "Type extends Contract, ..."

vex-libs/std/core/
└── builtin_contracts.vx     [NEW] - Declare builtin extensions
```

**Builtin contract strategy:**
- **Declaration:** `i32 extends Display, Clone, Eq;` in Vex stdlib
- **Implementation:** Compiler codegen in Rust
- **Dispatch:** Method calls check builtin registry first
- **User types:** Use existing `struct Type impl Contract` syntax

**Test:**
```vex
struct Point impl Display {
    x: i32,
    y: i32,
    fn to_string(): string { return "Point"; }
}

fn main(): i32 {
    let p = Point { x: 10, y: 20 };
    println(p.to_string());  // Should work
    return 0;
}
```

---

#### Day 3-4: Operator Overloading
- [ ] **Add/Sub/Mul/Div contracts** in stdlib
- [ ] **Operator → contract mapping** in compiler
  - `a + b` → check for Add contract → call `a.add(b)`
  - `a - b` → check for Sub contract → call `a.sub(b)`
  
- [ ] **Error messages** when contract missing
  - "Cannot add Point and Point: Point does not implement Add"

**Files to modify:**
```
vex-compiler/src/codegen_ast/expressions/
└── binary_ops.rs  [MODIFY] - Add contract dispatch

vex-libs/std/ops/
└── src/lib.vx     [NEW] - Define Add, Sub, Mul, Div contracts
```

**Test:**
```vex
import { Add } from "ops";

struct Vec2 impl Add {
    x: f64,
    y: f64,
    
    fn add(other: Vec2): Vec2 {
        return Vec2 { x: self.x + other.x, y: self.y + other.y };
    }
}

fn main(): i32 {
    let a = Vec2 { x: 1.0, y: 2.0 };
    let b = Vec2 { x: 3.0, y: 4.0 };
    let c = a + b;  // Should work!
    println(c.x);   // 4.0
    return 0;
}
```

---

#### Day 5: Debug Contract
- [ ] **Debug contract** definition
- [ ] **Builtin impls** for primitives
- [ ] **dbg!() macro** (prints with debug info)

**Test:**
```vex
fn main(): i32 {
    let x = 42;
    dbg!(x);  // Prints: "i32(42)"
    return 0;
}
```

---

### Phase 2: Builtin Standardization (Week 2)

#### Day 1-2: Builtin Inventory & Cleanup
- [ ] **Audit current builtins**
  - List everything in compiler
  - List everything in runtime
  - Find duplicates/conflicts
  
- [ ] **Create builtin registry**
  - Document each builtin
  - Specify signature
  - Mark deprecated ones

**Output:** `docs/BUILTINS.md` - complete reference

---

#### Day 3-4: Runtime API Redesign
- [ ] **Stable C ABI** for runtime
  - Clear function names: `vex_vec_*`, `vex_string_*`
  - Consistent error handling
  - Memory ownership rules
  
- [ ] **Vex extern declarations**
  - Create `vex-libs/runtime.vx` with all extern fns
  - Type-safe wrappers

**Example:**
```vex
// vex-libs/runtime.vx
extern "C" {
    fn vex_vec_new(elem_size: u64): *void;
    fn vex_vec_push(vec: *void, elem: *void);
    fn vex_vec_free(vec: *void);
}

// vex-libs/std/collections/vec.vx
struct Vec<T> {
    ptr: *void,
}

// Methods use Vex syntax: fn (self: Type) method_name()
fn (self: Vec<T>) new(): Vec<T> {
    return Vec { ptr: vex_vec_new(size_of::<T>()) };
}

fn (self: Vec<T>) push(item: T)! {
    vex_vec_push(self.ptr, &item as *void);
}
```

---

#### Day 5: Stdlib Reorganization
- [ ] **Module structure**
  ```
  vex-libs/std/
  ├── prelude/       # AUTO-IMPORTED (Vec, HashMap, Option, Result, Box)
  ├── core/          # Contracts, fundamental types
  ├── ops/           # Operator contracts
  ├── fmt/           # Display, Debug, formatting
  ├── collections/   # Advanced collections (LinkedList, BTreeMap)
  ├── io/            # File, console I/O
  ├── sync/          # Thread, Channel, Mutex
  ├── iter/          # Iterator contracts
  └── time/          # Time, Duration
  ```

- [ ] **Prelude (auto-imported globally)**
  ```vex
  // These are ALWAYS available, no import needed:
  
  // Collections
  - Vec<T>
  - Map<K, V>
  - Set<T>
  - String
  - Box<T>
  
  // Result types
  - Option<T> (Some, None)
  - Result<T, E> (Ok, Err)
  
  // I/O functions
  - print(), println()
  - eprint(), eprintln()
  - dbg!()
  
  // Utility macros
  - panic!()
  - assert!()
  - todo!()
  - unreachable!()
  ```

- [ ] **Compiler prelude injection**
  - Every Vex file automatically gets: `use std::prelude::*;`
  - No explicit import needed for core types
  - Can be disabled with: `#![no_prelude]` (advanced use)

---

### Phase 3: Core Contracts (Week 3)

#### Day 1-2: Iterator Contract
```vex
contract Iterator {
    type Item;  // Associated type
    fn next(): Option<Item>!;
}

contract IntoIterator {
    type Item;
    type Iter: Iterator<Item = Item>;
    fn into_iter(): Iter;
}

// Now for-in loops work!
for item in vec {  // Calls vec.into_iter().next()
    println(item);
}
```

---

#### Day 3-4: From/Into Contracts
```vex
contract From<T> {
    fn from(value: T): Self;
}

contract Into<T> {
    fn into(): T;
}

// Auto-impl Into if From exists (compiler magic)

// Builtin conversions (compiler provides)
// i32 → f64, i32 → i64, etc. are automatic

// User conversions
struct Meters {
    value: f64,
}

struct Feet impl From<Meters> {
    value: f64,
    
    fn from(m: Meters): Feet {
        return Feet { value: m.value * 3.28084 };
    }
}

let m = Meters { value: 10.0 };
let f: Feet = m.into();  // Works!
```

---

#### Day 5: Default Contract
```vex
contract Default {
    fn default(): Self;
}

// Usage in constructors:
struct Config impl Default {
    timeout: i32,
    retries: i32,
}

fn (self: Config) default(): Config {
    return Config { timeout: 30, retries: 3 };
}

let cfg = Config.default();  // Static method call
```

---

## 📊 Success Metrics

### Week 1 (Contract Infrastructure)
- [ ] ✅ Display contract works for primitives and user structs
- [ ] ✅ Operator overloading (Add, Sub, Mul) works
- [ ] ✅ All existing tests still pass
- [ ] ✅ 5 new contract tests passing

### Week 2 (Builtin Cleanup)
- [ ] ✅ Complete builtin documentation
- [ ] ✅ Stable C runtime ABI
- [ ] ✅ Clean module structure
- [ ] ✅ Zero compiler warnings

### Week 3 (Core Contracts)
- [ ] ✅ Iterator for-in loops work
- [ ] ✅ From/Into conversions work
- [ ] ✅ Default contract implemented
- [ ] ✅ Can write real programs without workarounds

---

## 🎯 The Ultimate Test

After Phase 3, this code should **just work** (no imports for core types):

```vex
// NO IMPORTS NEEDED - Vec, HashMap, Option, Result are in prelude!

struct Point impl Display, Debug, Clone, Eq, Add {
    x: f64,
    y: f64,
    
    fn to_string(): string {
        return "(" + self.x.to_string() + ", " + self.y.to_string() + ")";
    }
    
    fn debug(): string {
        return "Point { x: " + self.x.to_string() + ", y: " + self.y.to_string() + " }";
    }
    
    fn clone(): Point {
        return Point { x: self.x, y: self.y };
    }
    
    fn eq(other: &Point): bool {
        return self.x == other.x && self.y == other.y;
    }
    
    fn add(other: Point): Point {
        return Point { x: self.x + other.x, y: self.y + other.y };
    }
}

fn main(): i32 {
    // Vec is global - no import!
    let mut points = Vec.new();
    points.push(Point { x: 1.0, y: 2.0 });
    points.push(Point { x: 3.0, y: 4.0 });
    
    // Map is global too!
    let mut map = Map.new();
    map.insert("origin", Point { x: 0.0, y: 0.0 });
    
    // Option/Result are global!
    let maybe_point: Option<Point> = Some(Point { x: 5.0, y: 6.0 });
    
    match maybe_point {
        Some(p) => println(p),
        None => println("No point"),
    }
    
    // Utility functions are global!
    assert!(points.len() == 2, "Should have 2 points");
    
    for p in points {
        println(p);        // Uses Display
        dbg!(p);          // Uses Debug (global macro)
        
        let p2 = p.clone();
        let p3 = p + p2;  // Uses Add
        
        if p == p3 {      // Uses Eq
            println("Equal!");
        }
    }
    
    return 0;
}
```

**This is the standard we're aiming for.**

---

## 🌍 Prelude Design Philosophy

### What Goes in Prelude?
**Rule:** Types used in >50% of programs

✅ **YES - Global Prelude:**
- Vec, Map, Set (collections are everywhere)
- String (text is everywhere)
- Box (heap allocation is common)
- Option, Result (error handling is everywhere)
- print, println, eprint, eprintln (I/O is everywhere)
- dbg! (debugging is everywhere)
- panic!, assert!, todo!, unreachable! (development helpers)

❌ **NO - Explicit Import:**
- File, Path (I/O is specialized)
- Thread, Channel (concurrency is specialized)
- Time, Duration (timing is specialized)
- Advanced collections (BTreeMap, BTreeSet, LinkedList)

### Prelude Implementation

**Compiler automatically injects:**
```vex
// Every .vx file implicitly starts with:
use std::prelude::*;
```

**Prelude exports:**
```vex
// vex-libs/std/prelude/mod.vx
pub use collections::{Vec, Map, Set};
pub use string::String;
pub use smart_ptrs::Box;
pub use result::{Option, Some, None, Result, Ok, Err};
pub use io::{print, println, eprint, eprintln};
pub use macros::{dbg, panic, assert, todo, unreachable};
```

**User can disable:**
```vex
#![no_prelude]  // For advanced users, stdlib authors
```

---

## 🚧 Current Blockers (Must Fix First)

Before starting Phase 1:

### 1. Fix Failing Tests (1-2 days)
- 12 tests currently failing
- Need 100% pass rate before major changes
- **Action:** Finish TODO.md items

### 2. Contract Syntax Validation
- Ensure `struct X impl Contract` parses correctly
- Verify contract method dispatch works
- **Action:** Write minimal test case

### 3. Module System Check
- Verify `import { X } from "module"` works
- Test cross-module contract implementations
- **Action:** Test existing imports

---

## 📝 Decision Log

### November 12, 2025
- ✅ Identified two critical blockers
- ✅ Designed three-layer architecture
- ✅ Created contract system blueprint
- ✅ Roadmap for 3-week implementation
- 🎯 **Next:** Fix failing tests, then start Phase 1

---

## ⚠️ Rules Going Forward

1. **No new features** until contract system is solid
2. **No shortcuts** - implement properly or not at all
3. **Test everything** - every contract, every builtin
4. **Document as you go** - update BUILTINS.md, CONTRACTS.md
5. **One thing at a time** - finish Phase 1 before Phase 2

---

**Maintained By:** Vex Core Team  
**Status:** Foundation Phase - Critical Priority  
**Next Milestone:** Week 1 - Contract Infrastructure Complete
