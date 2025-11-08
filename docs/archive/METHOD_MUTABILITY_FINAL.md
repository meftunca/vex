# Method Mutability & Location - Final Architecture

**Version:** v0.9.1  
**Date:** 7 Kasım 2025  
**Status:** 🟢 APPROVED - Ready for Implementation  
**Decision:** Method-level mutability + Trait-based location rules

---

## 🎯 Executive Summary

Vex v0.9.1 introduces **two critical features** that solve all method-related ambiguities:

1. **Method-Level Mutability:** `fn method()!` declares mutation capability
2. **Trait-Based Location Rules:** Trait methods in struct body, extras flexible

**Result:** 100% clear, 100% enforceable, 0% ambiguity 🎉

---

## 📖 Part 1: Method-Level Mutability

### The Rule

```vex
fn method()   { }  // Immutable (self only, self! forbidden)
fn method()!  { }  // Mutable (self! allowed, self also works)
```

**Key Insight:** The `!` suffix declares mutability at the method signature level.

---

### Immutable Methods (no `!`)

**Declaration:**

```vex
fn method_name(params): return_type { }
```

**Rules:**

- ✅ `self.field` allowed (read-only access)
- ❌ `self!.field = x` **COMPILE ERROR**
- ❌ `self!.mutable_method()` **COMPILE ERROR**
- Receiver type: `&T` (immutable reference)

**Example:**

```vex
struct Point {
    x: i32,
    y: i32,

    fn distance(): f32 {
        return sqrt(self.x * self.x + self.y * self.y);  // ✅ OK
    }

    fn invalid() {
        self!.x = 0;  // ❌ COMPILE ERROR: Cannot use self! in immutable method
    }
}
```

---

### Mutable Methods (with `!`)

**Declaration:**

```vex
fn method_name(params)!: return_type { }
fn method_name(params)! { }
```

**Rules:**

- ✅ `self.field` allowed (read access)
- ✅ `self!.field = x` allowed (write access)
- ✅ `self!.mutable_method()` allowed
- Receiver type: `&T!` (mutable reference)

**Example:**

```vex
struct Counter {
    value: i32,

    fn increment()! {
        self!.value = self!.value + 1;  // ✅ OK
    }

    fn add(x: i32)!: i32 {
        self!.value = self!.value + x;  // ✅ OK (mutation)
        return self.value;              // ✅ OK (read also works)
    }
}
```

---

### Call Site Enforcement

**Immutable method call:**

```vex
let point = Point { x: 3, y: 4 };
let d = point.distance();  // ✅ No ! needed
```

**Mutable method call:**

```vex
let! counter = Counter { value: 0 };
counter.increment()!;      // ✅ ! required
counter.add(5)!;           // ✅ ! required
```

**Error on missing `!`:**

```vex
let! counter = Counter { value: 0 };
counter.increment();  // ❌ COMPILE ERROR

error: mutable method call requires `!` suffix
  --> example.vx:3:9
   |
3  |     counter.increment();
   |             ^^^^^^^^^ add `!` to indicate mutation
   |
help: mutable methods must be called with `!`
   |
3  |     counter.increment()!;
   |                        +
```

---

## 📖 Part 2: Trait-Based Location Rules

### The Rule

**Trait methods:** MUST be implemented in struct body  
**Extra methods:** Can be in struct body OR external (golang-style)

---

### Trait Methods (MUST be inline)

**Declaration:**

```vex
trait Display {
    fn show();        // Immutable contract
    fn update()!;     // Mutable contract
}
```

**Implementation:**

```vex
struct User impl Display {
    name: string,
    age: i32,

    // ✅ Trait methods MUST be here (in struct body)
    fn show() {
        print(self.name, " - ", self.age);
    }

    fn update()! {
        self!.age = self!.age + 1;
    }
}

// ❌ COMPILE ERROR: Trait method cannot be external
fn (u: &User) show() {
    print(u.name);
}
```

**Error message:**

```
error: trait method must be implemented in struct body
  --> user.vx:12:1
   |
12 | fn (u: &User) show() {
   | ^^^^^^^^^^^^^^^^^^^^ trait method `Display::show` must be in struct body
   |
note: trait declared here
  --> user.vx:1:1
   |
1  | trait Display {
   | ^^^^^^^^^^^^^
   |
help: move this implementation into struct body
   |
6  | struct User impl Display {
7  |     name: string,
8  |     age: i32,
9  |
10 |     fn show() {
11 |         print(self.name);
12 |     }
   |
```

---

### Extra Methods (Flexible location)

**Extra methods** (not in trait) can be:

1. **In struct body** (inline)
2. **External** (golang-style)

**Example:**

```vex
struct FileStorage {
    path: string,
    buffer: Vec<string>,

    // ===== INLINE EXTRA METHODS =====
    fn get_path(): string {           // Immutable (default)
        return self.path;
    }

    fn clear()! {                     // Mutable (explicit)
        self!.buffer.clear();
    }
}

// ===== EXTERNAL EXTRA METHODS (Golang-style) =====
fn (fs: &FileStorage) helper() {     // Immutable
    print(fs.path);
}

fn (fs: &FileStorage!) compress()! {  // Mutable
    fs!.buffer = compress_data(fs!.buffer);
}
```

---

### Default Mutability for Extra Methods

**Rule:** Extra methods are **immutable by default** (no `!`)

```vex
struct Counter {
    value: i32,

    // Extra method (no trait)
    fn get(): i32 {         // ✅ Immutable (no ! needed)
        return self.value;
    }

    fn invalid() {          // ❌ Still immutable
        self!.value = 0;    // ❌ COMPILE ERROR
    }

    fn increment()! {       // ✅ Explicitly mutable
        self!.value += 1;   // ✅ OK
    }
}
```

**Rationale:**

- **Safe by default:** Immutability is safer
- **Opt-in mutation:** `!` suffix makes mutation explicit
- **Clear intent:** Reader knows immediately if method mutates

---

## 🎯 Part 3: Trait Mutability Contracts

### Trait Signature Specifies Mutability

**Trait declaration:**

```vex
trait Storage {
    fn read(): string;       // Immutable contract
    fn write(data: string)!; // Mutable contract
    fn clear()!;             // Mutable contract
}
```

**Implementation must match:**

```vex
struct FileStorage impl Storage {
    path: string,

    fn read(): string {              // ✅ OK - matches trait (immutable)
        return self.path;
    }

    fn write(data: string)! {        // ✅ OK - matches trait (mutable)
        // ...
    }

    fn clear()! {                    // ✅ OK - matches trait (mutable)
        // ...
    }
}
```

**Error on mismatch:**

```vex
struct BadStorage impl Storage {
    fn read()!: string {  // ❌ ERROR: trait declares immutable
        // ...
    }
}
```

**Error message:**

```
error: method mutability does not match trait declaration
  --> storage.vx:5:5
   |
5  |     fn read()!: string {
   |            ^^ unexpected `!` - trait method is immutable
   |
note: trait method declared here
  --> storage.vx:2:5
   |
2  |     fn read(): string;
   |     ^^^^^^^^^^^^^^^^^^ declared as immutable (no `!`)
   |
help: remove `!` to match trait signature
   |
5  |     fn read(): string {
   |            --
```

---

## 🎯 Part 4: Complete Architecture

### Level 1: Trait (Contract)

```vex
trait Logger {
    fn log(msg: string);     // Immutable method
    fn clear()!;             // Mutable method
}
```

**Purpose:** Define public contract with explicit mutability

---

### Level 2: Struct Body (Data + Trait Implementation + Core Methods)

```vex
struct FileLogger impl Logger {
    path: string,
    buffer: Vec<string>,

    // ===== TRAIT METHODS (Required here) =====
    fn log(msg: string) {
        print(self.path, ": ", msg);
    }

    fn clear()! {
        self!.buffer.clear();
    }

    // ===== EXTRA METHODS (Optional here) =====
    fn get_path(): string {          // Immutable (default)
        return self.path;
    }

    fn flush()! {                    // Mutable (explicit)
        self!.buffer.clear();
    }
}
```

**Purpose:**

- Fulfill trait contracts (visible in struct)
- Define core struct-specific methods

---

### Level 3: External Methods (Extensions)

```vex
// Extension methods (golang-style)
fn (logger: &FileLogger) helper() {       // Immutable
    logger.log("Helper called");
}

fn (logger: &FileLogger!) compress()! {   // Mutable
    logger!.buffer = compress(logger!.buffer);
}
```

**Purpose:**

- Third-party extensions
- Organize large types across files
- Add methods without modifying struct definition

---

## 📊 Decision Matrix

| Question                 | Answer                       | Rationale                    |
| ------------------------ | ---------------------------- | ---------------------------- |
| Where are trait methods? | Struct body (inline)         | Contract fulfillment visible |
| Where are extra methods? | Struct body OR external      | Flexibility for organization |
| Default mutability?      | Immutable (no `!`)           | Safe by default              |
| How to make mutable?     | Add `!` after params         | Explicit opt-in              |
| Trait mutability?        | Specified in trait signature | Contract clarity             |
| Receiver naming?         | Flexible (self, this, any)   | User choice                  |
| Call site for mutable?   | Requires `!` suffix          | Clear mutation intent        |

---

## 🔧 Implementation Checklist

### Phase 1: Parser (~2 hours)

- [ ] **structs.rs:** Parse `fn method()!` syntax (! after params, before return type)
- [ ] **traits.rs:** Parse `fn method()!;` in trait signatures
- [ ] **AST:** Add `is_mutable: bool` to `MethodDeclaration` and `TraitMethod`
- [ ] **Validation:** Trait methods must be in struct body (not external)

### Phase 2: Codegen (~3 hours)

- [ ] **methods.rs:** Store `current_method_is_mutable` context flag
- [ ] **expressions/mod.rs:** Validate `self!` only allowed in mutable methods
- [ ] **expressions/calls/method_calls.rs:** Require `!` at call site for mutable methods
- [ ] **types.rs:** Include mutability in method signatures
- [ ] **Error:** "Cannot use `self!` in immutable method"
- [ ] **Error:** "Mutable method call requires `!` suffix"

### Phase 3: Borrow Checker (~2 hours)

- [ ] **immutability.rs:** Add `check_method_self_usage()` validation
- [ ] **borrows.rs:** Validate method mutability matches receiver type
- [ ] **Visitor:** `SelfUsageVisitor` to traverse method body and check `self!` usage
- [ ] **Error:** "Trait method cannot be implemented externally"

### Phase 4: Testing (~2 hours)

- [ ] Test: Immutable method cannot use `self!`
- [ ] Test: Mutable method can use both `self` and `self!`
- [ ] Test: Trait method mutability contracts enforced
- [ ] Test: Call site requires `!` for mutable methods
- [ ] Test: Trait methods must be in struct body
- [ ] Test: Extra methods can be external
- [ ] Test: Error messages are clear and helpful

### Phase 5: Documentation (~1 hour)

- [ ] Update `SYNTAX.md` with method mutability syntax
- [ ] Update `VEX_SYNTAX_GUIDE.md` with examples
- [ ] Update `VARIABLE_SYSTEM_V09.md` with method rules
- [ ] Update trait documentation with location rules
- [ ] Create migration guide from v0.9 to v0.9.1

**Total Estimate:** ~10 hours (1-2 days)

---

## 🧪 Test Cases

### Test 1: Basic Immutable Method

```vex
struct Point {
    x: i32,
    y: i32,

    fn distance(): f32 {
        return sqrt(self.x * self.x + self.y * self.y);
    }
}

fn main(): i32 {
    let p = Point { x: 3, y: 4 };
    assert(p.distance() == 5.0);
    return 0;
}
```

---

### Test 2: Basic Mutable Method

```vex
struct Counter {
    value: i32,

    fn increment()! {
        self!.value = self!.value + 1;
    }
}

fn main(): i32 {
    let! counter = Counter { value: 0 };
    counter.increment()!;
    assert(counter.value == 1);
    return 0;
}
```

---

### Test 3: Error - Mutable Self in Immutable Method

```vex
struct Data {
    x: i32,

    fn invalid() {
        self!.x = 42;  // ❌ Should fail compilation
    }
}

// Expected error:
// error: cannot use mutable reference `self!` in immutable method
```

---

### Test 4: Trait Method Mutability

```vex
trait Resettable {
    fn reset()!;
}

struct Counter impl Resettable {
    value: i32,

    fn reset()! {
        self!.value = 0;
    }
}

fn main(): i32 {
    let! counter = Counter { value: 42 };
    counter.reset()!;
    assert(counter.value == 0);
    return 0;
}
```

---

### Test 5: Trait Methods Must Be Inline

```vex
trait Display {
    fn show();
}

struct User {
    name: string,
}

// ❌ Should fail compilation
fn (u: &User) show() {
    print(u.name);
}

// Expected error:
// error: trait method must be implemented in struct body
```

---

### Test 6: Extra Methods Can Be External

```vex
struct Counter {
    value: i32,
}

// ✅ Extra method (not in trait) can be external
fn (c: &Counter) helper() {
    print("Value:", c.value);
}

fn (c: &Counter!) increment()! {
    c!.value = c!.value + 1;
}

fn main(): i32 {
    let! counter = Counter { value: 0 };
    counter.helper();
    counter.increment()!;
    assert(counter.value == 1);
    return 0;
}
```

---

### Test 7: Mixed Trait and Extra Methods

```vex
trait Logger {
    fn log(msg: string);
}

struct FileLogger impl Logger {
    path: string,

    // Trait method (must be here)
    fn log(msg: string) {
        print(self.path, ": ", msg);
    }

    // Extra method (can be here)
    fn get_path(): string {
        return self.path;
    }
}

// Extra method (can be external)
fn (logger: &FileLogger) helper() {
    logger.log("Helper");
}

fn main(): i32 {
    let logger = FileLogger { path: "/var/log" };
    logger.log("Test");
    logger.helper();
    return 0;
}
```

---

## 🎯 Why This Architecture is Perfect

### 1. **100% Clear**

Every method's behavior is obvious from its signature:

```vex
fn method()   // Immutable - won't change anything
fn method()!  // Mutable - will modify state
```

### 2. **100% Enforceable**

Compiler catches all violations at compile-time:

- `self!` in immutable method → Error
- Missing `!` at call site → Error
- Trait method misplacement → Error
- Mutability mismatch → Error

### 3. **0% Ambiguity**

Every question has a clear answer:

- Q: Where is trait method? → A: In struct body
- Q: Can extra method be external? → A: Yes
- Q: Is this method mutable? → A: Check for `!`
- Q: Can I use `self!`? → A: Only if method has `!`

### 4. **Safe by Default**

```vex
fn method() { }  // Default: safe (immutable)
fn method()! { } // Opt-in: mutation (explicit)
```

### 5. **Self-Documenting**

```vex
user.update_name(name)!;  // ← Reader knows: this mutates
user.get_name();          // ← Reader knows: this just reads
```

---

## 📚 Examples

### Example 1: Simple Counter

```vex
struct Counter {
    value: i32,

    fn get(): i32 {
        return self.value;
    }

    fn set(val: i32)! {
        self!.value = val;
    }

    fn increment()! {
        self!.value = self!.value + 1;
    }
}

fn main(): i32 {
    let! counter = Counter { value: 0 };
    print(counter.get());       // 0
    counter.increment()!;
    print(counter.get());       // 1
    counter.set(10)!;
    print(counter.get());       // 10
    return 0;
}
```

---

### Example 2: Trait Implementation

```vex
trait Drawable {
    fn draw();
    fn clear()!;
}

struct Canvas impl Drawable {
    pixels: Vec<i32>,

    fn draw() {
        print("Drawing", self.pixels.len(), "pixels");
    }

    fn clear()! {
        self!.pixels.clear();
    }
}

fn main(): i32 {
    let! canvas = Canvas { pixels: Vec.new() };
    canvas.draw();
    canvas.clear()!;
    return 0;
}
```

---

### Example 3: Mixed Methods

```vex
trait Storage {
    fn read(): string;
    fn write(data: string)!;
}

struct FileStorage impl Storage {
    path: string,
    buffer: Vec<string>,

    // Trait methods (in struct body)
    fn read(): string {
        return self.path;
    }

    fn write(data: string)! {
        self!.buffer.push(data);
    }

    // Extra methods (in struct body)
    fn get_path(): string {
        return self.path;
    }

    fn clear()! {
        self!.buffer.clear();
    }
}

// Extra methods (external - golang style)
fn (fs: &FileStorage) helper() {
    print("Path:", fs.path);
}

fn (fs: &FileStorage!) compress()! {
    fs!.buffer = compress_data(fs!.buffer);
}

fn main(): i32 {
    let! storage = FileStorage {
        path: "/data",
        buffer: Vec.new()
    };

    storage.write("test")!;
    print(storage.read());
    storage.helper();
    storage.compress()!;
    storage.clear()!;

    return 0;
}
```

---

## 🚀 Migration Guide (v0.9 → v0.9.1)

### Step 1: Add `!` to Mutable Methods

**Before (v0.9):**

```vex
fn increment() {
    self!.value = self!.value + 1;
}
```

**After (v0.9.1):**

```vex
fn increment()! {
    self!.value = self!.value + 1;
}
```

### Step 2: Update Call Sites

**Before (v0.9):**

```vex
counter.increment();
```

**After (v0.9.1):**

```vex
counter.increment()!;
```

### Step 3: Move Trait Methods to Struct Body

**Before (v0.9):**

```vex
struct User impl Display {
    name: string,
}

fn (u: &User) show() {
    print(u.name);
}
```

**After (v0.9.1):**

```vex
struct User impl Display {
    name: string,

    fn show() {
        print(self.name);
    }
}
```

### Step 4: Update Trait Signatures

**Before (v0.9):**

```vex
trait Writable {
    fn write(data: string);
}
```

**After (v0.9.1):**

```vex
trait Writable {
    fn write(data: string)!;  // Add ! if method mutates
}
```

---

## 🎯 Final Summary

### What We Decided

1. **Method mutability:** Declared with `!` suffix after params
2. **Trait methods:** MUST be in struct body (not external)
3. **Extra methods:** Can be in struct body OR external (golang-style)
4. **Default mutability:** Immutable (safe by default)
5. **Receiver naming:** Flexible (self, this, any name)

### Why This Works

- ✅ **Clear:** Every method's behavior is explicit
- ✅ **Safe:** Immutable by default, mutation opt-in
- ✅ **Enforceable:** Compiler validates everything
- ✅ **Flexible:** Extra methods can be organized anywhere
- ✅ **Discoverable:** Trait methods visible in struct

### Implementation Priority

🔴 **CRITICAL** - Implement before v1.0

**Estimated time:** 10 hours (1-2 days)

---

**Status:** 🟢 FINAL DECISION - Ready for implementation

**Next Steps:**

1. Implement parser changes
2. Implement codegen validation
3. Add borrow checker rules
4. Create comprehensive tests
5. Update all documentation

---

**This is the final architecture. No more changes needed. Let's implement it! 🚀**
