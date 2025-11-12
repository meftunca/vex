# Vex Language Identity & Uniqueness
**Making Vex Truly Unique - Away from Rust-like Syntax**

**Created:** November 12, 2025  
**Status:** Planning Phase

---

## 🎯 Core Philosophy

Vex should be:
- ✅ **Systems programming** - Zero-cost abstractions, memory safety
- ✅ **Modern & readable** - Not cryptic like C++, not verbose like Java
- ✅ **Unique identity** - Not a Rust clone with different keywords
- ✅ **Developer friendly** - Clear, intuitive syntax
- ✅ **Pragmatic** - Best ideas from multiple languages, not dogmatic

---

## 🔥 Approved Changes (To Be Implemented)

### 1. `trait` → `contract` ✅
**Rationale:** More descriptive, unique, business-friendly

```vex
// BEFORE (Rust-like)
trait Display {
    fn show();
}

struct Point impl Display {
    fn show() { }
}

// AFTER (Vex unique!)
contract Display {
    show();
}

struct Point impl Display {
    show() { }
}
```

**Why "contract"?**
- Semantically correct: A contract that types must fulfill
- Unique: No other mainstream language uses this
- Clear: Even non-programmers understand "contract"
- "implements contract" reads naturally in English

---

### 2. Remove `fn` prefix in struct/contract methods ✅
**Rationale:** Context makes it obvious, reduces noise

```vex
// BEFORE (verbose)
struct Point {
    x: i32,
    y: i32,
    
    fn distance(other: Point): f64 {
        return sqrt((self.x - other.x)^2);
    }
    
    fn translate(dx: i32, dy: i32)! {
        self.x = self.x + dx;
    }
}

// AFTER (clean)
struct Point {
    x: i32,
    y: i32,
    
    distance(other: Point): f64 {
        return sqrt((self.x - other.x)^2);
    }
    
    translate(dx: i32, dy: i32)! {
        self.x = self.x + dx;
    }
}
```

**Rules:**
- Top-level functions: `fn name() { }` - `fn` **required**
- Contract methods: `name() { }` - `fn` **removed** ✅
- Struct methods: **DEPRECATED** - use Go-style external methods
- Static vs instance: determined by presence of `self` usage
- Mutable methods: `!` suffix

**Status:** ✅ COMPLETED - `fn` prefix removed from contracts, struct methods deprecated

---

### 3. Deprecate Inline Struct Methods → Go-Style External Methods ✅ IN PROGRESS
**Rationale:** Less noise, Vex already does this

```vex
struct Point {
    x: i32,
    y: i32,
    
    // No "self" parameter - it's implicit!
    distance(other: Point): f64 {
        return sqrt((self.x - other.x)^2);  // self available in body
    }
}
```

---

## 💡 Under Consideration

### 5. Deprecate Inline Struct Methods → Go-Style External Methods ⚠️ DEPRECATED
**Major architectural change - needs careful consideration**

**Current state:** Methods can be defined inside struct body (inline)

```vex
// Current: Inline methods in struct
struct Point {
    x: i32,
    y: i32,
    
    // Methods defined INSIDE struct
    distance(other: Point): f64 {
        return sqrt((self.x - other.x)^2);
    }
    
    translate(dx: i32, dy: i32)! {
        self.x = self.x + dx;
    }
}
```

**Proposed:** Deprecate inline methods, use Go-style external methods

```vex
// Proposed: Clean separation
struct Point {
    x: i32,
    y: i32,
}

// Contracts define interfaces (signatures only)
contract Geometry {
    distance(other: Point): f64;
    translate(dx: i32, dy: i32)!;
}

// Methods defined OUTSIDE, Go-style
fn (p: &Point) distance(other: Point): f64 {
    return sqrt((p.x - other.x)^2);
}

fn (p: &Point!) translate(dx: i32, dy: i32) {
    p.x = p.x + dx;
    p.y = p.y + dy;
}

// Struct implements contract
struct Point impl Geometry {
    x: i32,
    y: i32,
}
```

**Benefits:**
- ✅ Clear separation: data (struct) vs behavior (methods)
- ✅ Contract = interface definition (signatures only)
- ✅ Go-style methods = full implementation (outside struct)
- ✅ More flexible: methods can be in separate files/modules
- ✅ Reduces struct file size (400 line limit enforcement)
- ✅ Familiar to Go developers

**Trade-offs:**
- ⚠️ Breaking change: All existing inline methods must migrate
- ⚠️ More verbose: methods not co-located with struct
- ⚠️ Contract becomes mandatory for method signatures
- ⚠️ Loss of "everything in one place" convenience

**Migration strategy:**
1. **Phase 1:** Keep both syntaxes working (deprecation warnings)
2. **Phase 2:** Convert stdlib to use external methods
3. **Phase 3:** Update all examples and documentation
4. **Phase 4:** Remove inline method support (breaking)

**Example comparison:**

```vex
// ===== OLD STYLE (deprecated) =====
struct Logger impl Loggable {
    prefix: string,
    
    log(msg: string) {  // Inline
        print(self.prefix, ": ", msg);
    }
    
    clear()! {  // Inline
        // clear log
    }
}

// ===== NEW STYLE (proposed) =====

// 1. Contract defines interface
contract Loggable {
    log(msg: string);
    clear()!;
}

// 2. Struct only has data
struct Logger {
    prefix: string,
}

// 3. Methods defined externally (Go-style)
fn (l: &Logger) log(msg: string) {
    print(l.prefix, ": ", msg);
}

fn (l: &Logger!) clear() {
    // clear log
}

// 4. Struct declares implementation
struct Logger impl Loggable {
    prefix: string,
}
```

**Open questions:**
1. Should `struct X impl Contract` still be required?
2. Can methods exist without contract? (Go allows this)
3. How to enforce "all contract methods implemented"?
4. Impact on generic method instantiation?
5. LSP/IDE support for "jump to implementation"?

**Status:** ⚠️ **DEPRECATED** - Compiler now warns, will be removed in future version

**Current behavior:**
- ⚠️ Parser accepts inline methods but emits deprecation warnings
- ✅ Go-style external methods are the recommended approach
- 📋 Migration guide available in this document

---

### 7. Loop Syntax Simplification
**Current state:** Too many loop keywords

```vex
// Current (4 loop types)
while condition { }
for i := 0; i < 10; i = i + 1 { }
for item in collection { }
loop { }
```

**Proposed:** Keep only 3, remove traditional for

```vex
// Proposed
loop { }                    // Infinite loop
while condition { }         // Conditional loop
for item in collection { }  // Iteration loop

// Traditional for → use while instead
let! i = 0;
while i < 10 {
    // body
    i = i + 1;
}
```

**Status:** ⏸️ Needs decision

---

### 5. Variable Declaration Syntax ✅
**Decision:** Keep `let`/`let!` (familiar, clear)

```vex
let x = 5;        // Immutable
let! y = 10;      // Mutable
```

**Status:** ✅ Decided - Keep current syntax

---

## ✨ Already Unique Features (Keep These!)

### 1. No `::` operator - Use `.` everywhere
```vex
Vec.new()              // Not Vec::new()
Option.Some(42)        // Not Option::Some(42)
Result.Ok("success")   // Not Result::Ok(...)
```

### 2. Mutable method marker `!`
```vex
struct Counter {
    value: i32,
    
    increment()! {     // Mutable method
        self.value = self.value + 1;
    }
    
    get(): i32 {       // Immutable method
        return self.value;
    }
}
```

### 3. Go-style struct tags
```vex
struct User {
    id: u64        `json:"id" db:"pk"`,
    username: string `json:"username" db:"username"`,
    email: string   `json:"email" db:"email"`,
}
```

### 4. Inline trait implementation
```vex
struct Point impl Display, Clone {
    x: i32,
    y: i32,
    
    show() {
        print(self.x, self.y);
    }
    
    clone(): Point {
        return Point { x: self.x, y: self.y };
    }
}
```

### 5. Explicit returns required
```vex
fn add(x: i32, y: i32): i32 {
    return x + y;  // Explicit - no implicit returns
}
```

### 6. JS-style imports
```vex
import { Vec, String } from "std/collections";
import { println } from "std/io";
```

### 7. Helper function approach for types
```vex
// Instead of methods on enums, use helper functions
enum Option<T> {
    Some(T),
    None,
}

export fn IsSome<T>(opt: Option<T>): bool { }
export fn Unwrap<T>(opt: Option<T>): T { }

// Usage
import { IsSome, Unwrap } from "std/option";
if IsSome(value) {
    let x = Unwrap(value);
}
```

---

## 🚫 What Vex Should NOT Copy from Rust

### ❌ Avoid These Rust-isms:

1. **Match keyword** - Consider alternatives:
   - `inspect` (more unique)
   - `switch` (more familiar)
   - `when` (minimalist)

2. **Overly complex lifetime annotations**
   - Vex: automatic lifetime inference
   - No `'a`, `'static` syntax pollution

3. **`Result<T, E>` everywhere**
   - Vex: Use exceptions where appropriate
   - Result for explicit error handling only

4. **Macro syntax `macro_rules!`**
   - Vex: Clean metaprogramming if needed
   - Prefer compile-time code generation

5. **Module system `mod`, `use`, `pub(crate)`**
   - Vex: JS-style `import`/`export` (cleaner)

---

## 🎨 Unique Vex Syntax Summary (Proposed)

```vex
// Imports - JS-style
import { Vec, HashMap } from "std/collections";
import { IsSome, Unwrap } from "std/option";

// Contracts (not traits!)
contract Display {
    show();
    format(): string;
}

// Struct with inline contract implementation
struct Point impl Display {
    x: i32 `json:"x"`,
    y: i32 `json:"y"`,
    
    // No "fn" prefix in methods!
    show() {
        print("Point(", self.x, ",", self.y, ")");
    }
    
    format(): string {
        return "Point(" + self.x.to_string() + "," + self.y.to_string() + ")";
    }
    
    distance(other: Point): f64 {
        return sqrt((self.x - other.x)^2 + (self.y - other.y)^2);
    }
    
    translate(dx: i32, dy: i32)! {  // Mutable method!
        self.x = self.x + dx;
        self.y = self.y + dy;
    }
}

// Top-level function (fn required)
fn main(): i32 {
    let p1 = Point { x: 0, y: 0 };
    let! p2 = Point { x: 10, y: 10 };
    
    // Loops
    loop {
        break;  // Infinite loop
    }
    
    while condition {
        // Conditional loop
    }
    
    for item in collection {
        // Iteration
    }
    
    // No :: operator - use .
    let vec = Vec.new();
    let opt = Option.Some(42);
    
    return 0;
}
```

---

## 📋 Implementation Roadmap

### Phase 1: Core Syntax Changes (High Priority)
- [x] ✅ Lexer: Add `contract` keyword, keep `trait` as alias temporarily
- [x] ✅ Parser: Make `fn` optional in struct/contract method parsing
- [x] ✅ AST: Add `Item::Contract` alongside `Item::Trait`
- [x] ✅ Compiler: Support both during transition
- [x] ✅ Update all standard library examples
- [x] ✅ **NEW:** Go-style parameter grouping `(a, b, c: i32)` implemented

### Phase 2: Documentation & Examples
- [ ] Update REFERENCE.md with new syntax
- [ ] Update all example files
- [ ] Update stdlib files (beta_*.vx)
- [ ] Create migration guide (trait→contract, fn removal)

### Phase 3: Breaking Changes (Careful!)
- [ ] Deprecate `trait` keyword (warnings)
- [ ] Make `fn` in methods an error (not just optional)
- [ ] Update test suite
- [ ] Announce breaking changes

### Phase 4: Future Enhancements
- [x] Variable declaration: Keeping `let`/`let!`
- [ ] Go-style parameter grouping: `(a, b, c: i32)`
- [ ] Finalize loop syntax (keep 4 or reduce to 3?)
- [ ] Consider `match` alternatives (`inspect`, `switch`, `when`)

---

## 🎯 New Feature: Go-style Parameter Grouping ✅

**Status:** ✅ **IMPLEMENTED** (v0.2.0)

**Proposal:** Allow grouping consecutive parameters of the same type

```vex
// Current (verbose)
fn add(a: i32, b: i32, c: i32): i32 {
    return a + b + c;
}

fn process(x: f64, y: f64, z: f64, name: string, tag: string): void {
    // ...
}

// New (Go-style grouping) ✅ WORKS!
fn add(a, b, c: i32): i32 {
    return a + b + c;
}

fn process(x, y, z: f64, name, tag: string): void {
    // ...
}

// Works in struct methods too
struct Point {
    distance(x1, y1, x2, y2: f64): f64 {
        return sqrt((x2-x1)^2 + (y2-y1)^2);
    }
}
```

**Benefits:**
- ✅ Less repetition for common patterns
- ✅ Cleaner function signatures
- ✅ Go developers will find this familiar
- ✅ Optional - can still use full syntax
- ✅ Parser automatically expands to individual parameters in AST

**Implementation:**
- ✅ Parser: `parse_parameters()` updated with lookahead
- ✅ AST: Each param still gets its own `Param` node (transparent to compiler)
- ✅ Type checker: No changes needed
- ✅ Test coverage: `examples/test_param_grouping.vx`

**Status:** ✅ **COMPLETED** - Feature is production-ready

---

## 🤔 Open Questions

1. ~~**Variable syntax:**~~ ✅ Keeping `let`/`let!`
2. ~~**Inline methods deprecation:**~~ ⚠️ Deprecated - warnings active
3. **Contract vs Trait:** Contracts are now pure interfaces (signature only)
4. **Loop unification:** Keep 4 loops or reduce to 3?
5. **Pattern matching:** Keep `match` or use `inspect`/`switch`/`when`?
6. **Error handling:** Result-heavy or exception-friendly?
7. **Module system:** Pure JS-style or add Vex-specific features?
8. **Parameter grouping:** Implement Go-style `(a, b, c: i32)` syntax?

---

## 📋 Migration Guide: Inline Methods → Go-Style External

**Old style (DEPRECATED):**
```vex
struct Point {
    x: i32,
    y: i32,
    
    // ⚠️ DEPRECATED - Inline method
    distance(other: Point): f64 {
        return sqrt((self.x - other.x)^2);
    }
}
```

**New style (RECOMMENDED):**
```vex
// 1. Struct has only data
struct Point {
    x: i32,
    y: i32,
}

// 2. Contract defines interface
contract Geometry {
    distance(other: Point): f64;
}

// 3. Method defined externally (Go-style)
fn (p: &Point) distance(other: Point): f64 {
    return sqrt((p.x - other.x)^2 + (p.y - other.y)^2);
}

// 4. Struct declares contract implementation
struct Point impl Geometry {
    x: i32,
    y: i32,
}
```

**Benefits:**
- ✅ Clear separation: data vs behavior
- ✅ Methods can be in separate files
- ✅ Easier to maintain (400 line limit per file)
- ✅ Go developers will find it familiar
- ✅ Contracts truly define "contracts"

---

## 🤔 Open Questions (continued)

---

## 💬 Discussion Points

**What makes Vex special?**
- Safety without Rust's learning curve
- Modern syntax without TypeScript's complexity
- Systems programming without C++'s baggage
- Unique blend: Go's simplicity + Rust's safety + TypeScript's clarity

**Target audience:**
- Systems programmers who want modern syntax
- TypeScript/Go developers entering systems programming
- Rust developers who want simpler syntax
- Anyone building high-performance services

---

## 📝 Notes & Brainstorming

*Use this section for ideas that need more thought*

- Consider: Error handling with `?` operator (keep from Rust?)
- Consider: Async/await syntax (unique approach?)
- Consider: Generics syntax (keep `<T>` or use something else?)
- Consider: Null safety (Option-based or nullable types?)
- Consider: Package manager (unique approach vs cargo clone?)

---

**Remember:** Every syntax decision should ask:
1. Does this make Vex more unique?
2. Does this improve readability?
3. Is this worth the migration cost?
4. Does this align with Vex's philosophy?
