# Method-Level Mutability Specification

**Version:** v0.9.1  
**Date:** 7 Kasım 2025  
**Status:** 🟢 APPROVED - Implementation Required  
**Feature:** Method-level mutability declaration via `!` suffix

---

## 📋 Overview

Vex v0.9.1 introduces **method-level mutability declaration** to simplify reasoning about method behavior and enforce compile-time safety.

### The Problem

**Old approach (v0.9):**

```vex
struct Counter {
    value: i32,

    fn increment() {
        self!.value = self!.value + 1;  // When is self! allowed?
    }
}
```

❌ **Issues:**

- Unclear when `self!` is valid inside method body
- No way to declare method intent at signature level
- Hard to enforce consistency

---

## ✅ The Solution: Method Signature Mutability

### Syntax

```vex
fn method_name(params): return_type { }     // Immutable method
fn method_name(params): return_type! { }    // Mutable method
fn method_name(params)! { }                 // Mutable method (no return)
fn method_name(params)!: return_type { }    // Mutable method (with return)
```

**Key:** The `!` suffix after parameter list declares method mutability.

---

## 📖 Complete Specification

### Rule 1: Immutable Methods

**Declaration:**

```vex
fn method_name(params): return_type { }
```

**Characteristics:**

- No `!` after parameter list
- Implicitly receives `&T` (immutable reference to self)
- Cannot modify struct fields
- Cannot call other mutable methods

**Body Rules:**

```vex
struct Data {
    x: i32,
    y: i32,

    fn get_x(): i32 {
        return self.x;           // ✅ OK - read access
    }

    fn sum(): i32 {
        return self.x + self.y;  // ✅ OK - read access
    }

    fn invalid() {
        self!.x = 42;            // ❌ COMPILE ERROR
        self!.mutate();          // ❌ COMPILE ERROR
    }
}
```

**Error Message:**

```
error: cannot use mutable reference `self!` in immutable method
  --> example.vx:12:9
   |
12 |         self!.x = 42;
   |         ^^^^^ immutable method cannot mutate self
   |
help: add `!` to method signature to make it mutable
   |
8  |     fn invalid()! {
   |               +
```

---

### Rule 2: Mutable Methods

**Declaration:**

```vex
fn method_name(params)! { }
fn method_name(params)!: return_type { }
```

**Characteristics:**

- `!` suffix after parameter list, before return type
- Implicitly receives `&T!` (mutable reference to self)
- Can modify struct fields via `self!`
- Can call other mutable methods

**Body Rules:**

```vex
struct Counter {
    value: i32,

    fn increment()! {
        self!.value = self!.value + 1;  // ✅ OK - mutation
    }

    fn add(x: i32)!: i32 {
        self!.value = self!.value + x;  // ✅ OK - mutation
        return self!.value;             // ✅ OK - read
    }

    fn reset()! {
        self!.value = 0;                // ✅ OK - mutation
    }

    fn flexible()! {
        let x = self.value;             // ✅ OK - immutable read
        self!.value = x * 2;            // ✅ OK - mutable write
    }
}
```

**Key Insight:** In mutable methods, both `self` (immutable) and `self!` (mutable) are allowed.

- Use `self.field` for read-only access
- Use `self!.field = x` for mutation

---

### Rule 3: Call Site Enforcement

**Immutable Method Call:**

```vex
let counter = Counter { value: 0 };
let x = counter.get_value();  // ✅ OK - no ! needed
```

**Mutable Method Call:**

```vex
let! counter = Counter { value: 0 };
counter.increment()!;          // ✅ OK - ! required at call site
counter.add(5)!;               // ✅ OK - ! required
```

**Error on Missing `!`:**

```vex
let! counter = Counter { value: 0 };
counter.increment();  // ❌ COMPILE ERROR

error: mutable method call requires `!` suffix
  --> example.vx:5:9
   |
5  |     counter.increment();
   |             ^^^^^^^^^ add `!` to indicate mutation
   |
help: mutable methods must be called with `!`
   |
5  |     counter.increment()!;
   |                        +
```

---

### Rule 4: Trait Method Mutability

**Trait Declaration:**

```vex
trait Writable {
    fn write(data: string);      // Immutable method
    fn flush()!;                 // Mutable method
}
```

**Implementation Must Match:**

```vex
struct FileWriter impl Writable {
    buffer: Vec<string>,

    fn write(data: string) {
        // ✅ OK - immutable (matches trait)
        print(data);
    }

    fn flush()! {
        // ✅ OK - mutable (matches trait)
        self!.buffer.clear();
    }
}
```

**Error on Mismatch:**

```vex
struct BadWriter impl Writable {
    fn write(data: string)! {  // ❌ ERROR: trait declares immutable
        // ...
    }
}

error: method mutability does not match trait declaration
  --> example.vx:5:5
   |
5  |     fn write(data: string)! {
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^ trait method is immutable
   |
note: trait method declared here
  --> example.vx:2:5
   |
2  |     fn write(data: string);
   |     ^^^^^^^^^^^^^^^^^^^^^^^ declared as immutable
```

---

## 🔍 Edge Cases

### Case 1: Chained Method Calls

```vex
struct Builder {
    value: i32,

    fn add(x: i32)!: &Builder! {
        self!.value = self!.value + x;
        return self!;
    }

    fn multiply(x: i32)!: &Builder! {
        self!.value = self!.value * x;
        return self!;
    }

    fn build(): i32 {
        return self.value;
    }
}

// Usage
let! builder = Builder { value: 10 };
let result = builder.add(5)!.multiply(2)!.build();
//                      ^             ^       ^ immutable
//                      mutable       mutable
```

---

### Case 2: Nested Struct Methods

```vex
struct Inner {
    x: i32,

    fn get(): i32 {
        return self.x;
    }

    fn set(val: i32)! {
        self!.x = val;
    }
}

struct Outer {
    inner: Inner,

    fn update()! {
        self!.inner.set(42)!;  // ✅ OK - mutable method can call mutable
    }

    fn read(): i32 {
        return self.inner.get();  // ✅ OK - immutable can call immutable
    }
}
```

---

### Case 3: Generic Methods

```vex
struct Container<T> {
    value: T,

    fn get(): &T {
        return &self.value;
    }

    fn set(new_val: T)! {
        self!.value = new_val;
    }
}
```

---

### Case 4: Closures Capturing Self

```vex
struct Counter {
    value: i32,

    fn increment_by()!: fn(i32) {
        return |x: i32| {
            self!.value = self!.value + x;  // ✅ OK if method is mutable
        };
    }

    fn get_reader(): fn(): i32 {
        return || {
            return self.value;  // ✅ OK - immutable capture
        };
    }
}
```

---

## 🔧 Implementation Details

### Parser Changes

**File:** `vex-parser/src/parser/items/structs.rs`

```rust
fn parse_method_signature(&mut self) -> Result<MethodSignature> {
    self.expect(Token::Fn)?;

    // Parse optional receiver
    let receiver = if self.current_token == Token::LeftParen {
        Some(self.parse_receiver()?)
    } else {
        None
    };

    // Parse method name
    let name = self.expect_identifier()?;

    // Parse parameters
    let params = self.parse_function_params()?;

    // ⭐ NEW: Check for mutability marker
    let is_mutable = if self.current_token == Token::Bang {
        self.advance();
        true
    } else {
        false
    };

    // Parse return type
    let return_type = if self.current_token == Token::Colon {
        self.advance();
        Some(self.parse_type()?)
    } else {
        None
    };

    Ok(MethodSignature {
        name,
        receiver,
        params,
        return_type,
        is_mutable,  // ⭐ NEW field
    })
}
```

---

### AST Changes

**File:** `vex-ast/src/lib.rs`

```rust
#[derive(Debug, Clone)]
pub struct MethodDeclaration {
    pub name: String,
    pub receiver: Option<Receiver>,
    pub params: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub is_mutable: bool,        // ⭐ NEW
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct TraitMethod {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub is_mutable: bool,        // ⭐ NEW
}
```

---

### Codegen Changes

**File:** `vex-compiler/src/codegen_ast/methods.rs`

```rust
fn compile_method_body(&mut self, method: &MethodDeclaration) {
    // Store receiver mutability for validation
    self.current_method_is_mutable = method.is_mutable;

    // Compile body statements
    for stmt in &method.body {
        self.compile_statement(stmt)?;
    }

    // Clear flag
    self.current_method_is_mutable = false;
}
```

**File:** `vex-compiler/src/codegen_ast/expressions/mod.rs`

```rust
fn compile_member_access(&mut self, expr: &MemberAccess) -> Result<Value> {
    match &expr.object {
        Expression::SelfRef { is_mutable } => {
            // ⭐ NEW: Validate mutability
            if *is_mutable && !self.current_method_is_mutable {
                return Err(CompileError::ImmutableMethodUsingSelfMut {
                    location: expr.span,
                });
            }
            // ... rest of compilation
        }
        // ...
    }
}
```

---

### Borrow Checker Changes

**File:** `vex-compiler/src/borrow_checker/immutability.rs`

```rust
fn check_method_self_usage(&mut self, method: &MethodDeclaration) -> Result<()> {
    let mut visitor = SelfUsageVisitor {
        method_is_mutable: method.is_mutable,
        errors: vec![],
    };

    visitor.visit_statements(&method.body);

    if !visitor.errors.is_empty() {
        return Err(BorrowError::InvalidSelfUsage {
            errors: visitor.errors,
        });
    }

    Ok(())
}

struct SelfUsageVisitor {
    method_is_mutable: bool,
    errors: Vec<BorrowError>,
}

impl SelfUsageVisitor {
    fn visit_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::SelfRef { is_mutable: true } => {
                if !self.method_is_mutable {
                    self.errors.push(BorrowError::MutableSelfInImmutableMethod);
                }
            }
            // ... visit other expressions
        }
    }
}
```

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
    let d = p.distance();
    assert(d == 5.0);
    return 0;
}
```

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

### Test 3: Error - Mutable Self in Immutable Method

```vex
struct Data {
    x: i32,

    fn invalid() {
        self!.x = 42;  // ❌ Should fail compilation
    }
}
```

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

### Test 5: Mixed Methods

```vex
struct Calculator {
    result: i32,

    fn get_result(): i32 {
        return self.result;
    }

    fn add(x: i32)! {
        self!.result = self!.result + x;
    }

    fn multiply(x: i32)! {
        self!.result = self!.result * x;
    }
}

fn main(): i32 {
    let! calc = Calculator { result: 5 };
    calc.add(3)!;
    calc.multiply(2)!;
    let result = calc.get_result();
    assert(result == 16);
    return 0;
}
```

---

## 📚 Migration Guide

### From v0.9 to v0.9.1

#### Step 1: Identify Mutable Methods

**Old (v0.9):**

```vex
fn method() {
    self!.field = x;  // Uses self!
}
```

**New (v0.9.1):**

```vex
fn method()! {
    self!.field = x;
}
```

#### Step 2: Update Call Sites

**Old (v0.9):**

```vex
obj.method();
```

**New (v0.9.1):**

```vex
obj.method()!;  // Add ! if method is mutable
```

#### Step 3: Update Trait Signatures

**Old (v0.9):**

```vex
trait Writable {
    fn write(data: string);
}
```

**New (v0.9.1):**

```vex
trait Writable {
    fn write(data: string);   // Immutable
    fn flush()!;              // Mutable
}
```

---

## 🎯 Summary

### What Changed

- ✅ Methods can now declare mutability with `!` suffix
- ✅ Compiler enforces `self!` usage matches method signature
- ✅ Call sites require `!` for mutable methods
- ✅ Trait methods can specify mutability

### Why This Matters

- 🔒 **Compile-time safety:** Catch mutation errors early
- 📖 **Clear intent:** Method signature shows mutability
- 🎯 **Better APIs:** Users know which methods modify data
- ✅ **Trait contracts:** Traits can enforce mutability rules

### Implementation Priority

🔴 **CRITICAL** - This feature should be implemented before v1.0

---

**Status:** 🟢 SPECIFICATION COMPLETE - Ready for implementation
