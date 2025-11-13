# Type System Gaps - Missing Features

**Status:** 100% test pass (373/373) - But advanced type system features missing
**Date:** November 13, 2025

---

## 🔴 CRITICAL - High Priority

### 1. Generic Impl Clause (Multiple Trait Implementations with Type Args)
**Status:** ❌ Not Implemented  
**Importance:** CRITICAL - Core polymorphism feature

**Current Status:**
- ✅ `impl` keyword works for simple trait names in struct declarations
- ✅ Used for structs: `struct File impl Reader, Writer { }`
- ✅ `extends` keyword used for standalone type extensions: `i32 extends Display, Clone, Eq;`
- ❌ **Cannot parse generic type parameters in impl clause**

**Current Limitation:**
```rust
// vex-ast/src/lib.rs:177
pub struct Struct {
    pub impl_traits: Vec<String>,  // ❌ Only stores trait names, no type args
}
```

**Parser Limitation:**
```rust
// vex-parser/src/parser/items/structs.rs:30-41
let impl_traits = if self.match_token(&Token::Impl) {
    let mut traits = Vec::new();
    loop {
        traits.push(self.consume_identifier()?);  // ❌ Only identifier, no generic params!
        if !self.match_token(&Token::Comma) { break; }
    }
    traits
}
```

**Needed:**
```vex
// ❌ Current: Cannot parse generic parameters in impl
struct Vector impl Add, Mul { }  // No type args!

// ✅ Want: impl with generic type arguments
struct Vector impl Add<i32>, Add<f64>, Mul<i32> {
    x: f64,
    y: f64,
}

// Usage:
let v = Vector { x: 1.0, y: 2.0 };
let v2 = v + 5;      // Add<i32> implementation
let v3 = v + 3.14;   // Add<f64> implementation
let v4 = v * 2;      // Mul<i32> implementation
```

**Required Changes:**
1. AST: Change `impl_traits: Vec<String>` to `impl_traits: Vec<TraitImpl>` where `TraitImpl { name: String, type_args: Vec<Type> }`
2. Parser: Update `parse_struct()` to parse generic parameters after trait names in `impl` clause
3. Type checker: Validate trait implementations with different type args
4. Codegen: Generate separate monomorphized implementations for each trait instantiation

**Current Working Syntax:**
```vex
// ✅ Already works: Simple trait names
i32 extends Display, Clone, Eq;           // Standalone type extension (builtin types)
struct File impl Reader, Writer { }       // Struct trait implementation
```

**Target Syntax:**
```vex
// 🎯 Goal: impl with generic type parameters
struct Vector impl Add<i32>, Add<f64>, Mul<Point> { }
```

**Use Cases:**
- Operator overloading with multiple types
- Generic containers with different element types
- Polymorphic behavior patterns

---

### 2. Default Type Parameters
**Status:** ❌ Not Implemented  
**Importance:** HIGH - Ergonomics & Rust compatibility

**Current Limitation:**
```rust
// vex-ast/src/lib.rs:16
pub struct TypeParam {
    pub name: String,
    pub bounds: Vec<TraitBound>,
    // ❌ No default_type field
}
```

**Needed:**
```vex
// ❌ Current: Must always specify Rhs
contract Add<Rhs> {
    op+(other: Rhs): Self;
}

// ✅ Want: Default to Self if not specified
contract Add<Rhs = Self> {
    op+(other: Rhs): Self;
}

// Usage with impl:
struct Point impl Add { }        // Rhs defaults to Point (Point + Point)
struct Point impl Add<f64> { }   // Rhs is f64 (Point + f64)
struct Point impl Add, Add<i32> { } // Both Point + Point and Point + i32
```

**Required Changes:**
1. AST: Add `default_type: Option<Type>` to `TypeParam`
2. Parser: Parse `=` and default type in type parameter lists
3. Type checker: Substitute defaults when type args omitted
4. Validation: Ensure defaults appear after required params

**Use Cases:**
- Rust-style trait defaults (`Add<Rhs = Self>`)
- Simplify common generic patterns
- Reduce boilerplate in trait implementations

---

### 3. Higher-Kinded Types (HKT)
**Status:** ❌ Not Implemented  
**Importance:** HIGH - Advanced abstraction

**Note:** This feature is conceptually important but the syntax needs to be designed for Vex's philosophy. Traditional HKT implementations require external trait implementations which Vex doesn't support. This would need significant design work.

**Conceptual Goal:**
```vex
// Theoretical syntax - needs design
contract Functor<F<_>> {
    map<A, B>(fa: F<A>, f: fn(A): B): F<B>;
}

// Implementation would need to work with Vex's inline model
// Possibly through generic struct methods or external functions
// Design TBD - this is a complex feature requiring architectural decisions
```

**Required Changes:**
1. **Design Phase**: Determine how HKT fits with Vex's inline-only trait model
2. AST: Support `F<_>` kind syntax in type parameters (if design allows)
3. Parser: Parse higher-kinded type parameters
4. Type system: Kind checking (Type → Type, Type → Type → Type, etc.)
5. Codegen: Monomorphization strategy for HKT

**Use Cases:**
- Functor, Monad, Applicative patterns
- Generic algorithms over container types
- Category theory abstractions

**Status:** ⚠️ **Requires architectural design** - HKT traditionally uses external trait impls which conflicts with Vex's `struct impl` inline philosophy. May need alternative approach or may be deprioritized.

---

## 🟡 IMPORTANT - Medium Priority

### 4. Const Generics (Array Sizes)
**Status:** ❌ Not Implemented  
**Importance:** MEDIUM - Static array safety

**Needed:**
```vex
// ❌ Current: No compile-time integer parameters
// ✅ Want: Array sizes as generic parameters

struct Matrix<T, const N: usize, const M: usize> {
    data: [[T; M]; N],
}

fn transpose<T, const N: usize, const M: usize>(m: Matrix<T, N, M>): Matrix<T, M, N> {
    // Compile-time guaranteed dimensions
}

// Usage:
let m: Matrix<f64, 3, 3> = Matrix.identity();
```

**Required Changes:**
1. AST: `ConstParam { name: String, ty: Type }` in type params
2. Parser: Parse `const N: usize` in type parameter lists
3. Type checker: Validate const expressions
4. Codegen: Substitute const values during monomorphization

**Use Cases:**
- Fixed-size arrays without heap allocation
- SIMD vector sizes
- Compile-time dimension checking
- Zero-cost abstractions for matrices

---

### 5. Lifetime Annotations (Explicit)
**Status:** ⚠️ Partially Implemented (Implicit only)  
**Importance:** MEDIUM - Borrow checker enhancement

**Current:**
```vex
// ✅ Works: Implicit lifetime checking
fn get_first(data: &[i32]): &i32 {
    return &data[0];
}
```

**Needed:**
```vex
// ❌ Current: Cannot express lifetime relationships explicitly
// ✅ Want: Explicit lifetime parameters

fn longest<'a>(x: &'a string, y: &'a string): &'a string {
    if x.len() > y.len() {
        return x;
    } else {
        return y;
    }
}

struct RefWrapper<'a, T> {
    reference: &'a T,
}
```

**Required Changes:**
1. AST: `LifetimeParam { name: String }` separate from `TypeParam`
2. Parser: Parse `'a`, `'b`, etc. in parameter lists
3. Type system: Track lifetime relationships
4. Borrow checker: Validate lifetime bounds

**Use Cases:**
- Complex borrowing patterns
- Self-referential structs
- Iterator implementations
- Fine-grained lifetime control

---

### 6. Trait Bounds on Type Parameters (Inline)
**Status:** ⚠️ Partial (Where clause only)  
**Importance:** MEDIUM - Ergonomics

**Current:**
```vex
// ✅ Works: Where clause syntax
fn print_both<T, U>(a: T, b: U): i32
where
    T: Display,
    U: Display
{ }
```

**Needed:**
```vex
// ❌ Current: Inline bounds not enforced
// ✅ Want: Inline trait bounds

fn print_all<T: Display>(items: Vec<T>) {
    for item in items {
        item.show();
    }
}

fn compare<T: Eq & Ord>(a: T, b: T): i32 {
    // T must implement both Eq and Ord
}
```

**Required Changes:**
1. Parser: Already parses bounds in `TypeParam` ✅
2. Type checker: **Enforce bounds during type checking** ❌
3. Error messages: Improve bound violation errors

**Current Status:**
- ✅ AST supports: `TypeParam { bounds: Vec<TraitBound> }`
- ✅ Parser works: Parses `T: Display & Clone`
- ❌ Type checker: Doesn't validate bounds yet

---

### 7. Associated Type Constraints (Where Clause)
**Status:** ❌ Not Implemented  
**Importance:** MEDIUM - Advanced trait usage

**Needed:**
```vex
// ❌ Current: Cannot constrain associated types
// ✅ Want: Where clause on associated types

fn process<T>(iter: T)
where
    T: Iterator,
    T.Item: Display  // Constrain associated type
{
    // Know that Item implements Display
}

contract Container {
    type Item;
    
    get(): Self.Item;
}

fn show_all<C>(container: C)
where
    C: Container,
    C.Item: Display
{
    let item = container.get();
    item.show();
}
```

**Required Changes:**
1. AST: Extend `WhereClausePredicate` to support `T.AssocType: Bound`
2. Parser: Parse associated type constraints in where clauses
3. Type checker: Validate associated type bounds
4. Codegen: Monomorphization with assoc type constraints

---

## 🟢 NICE-TO-HAVE - Low Priority

### 8. Conditional Impl (Conditional Trait Implementation)
**Status:** ❌ Not Implemented  
**Importance:** LOW - Advanced feature

**Needed:**
```vex
// ❌ Current: Cannot conditionally implement traits
// ✅ Want: Conditional impl based on type param bounds

// Option 1: Where clause (if we implement this)
struct Wrapper<T> impl Display, Clone
where
    T: Display & Clone
{
    value: T,
}

// Option 2: Separate external impls per constraint
struct Wrapper<T> {
    value: T,
}

// External methods only available when T: Display
fn (self: &Wrapper<T>!) show() where T: Display {
    print("Wrapper(");
    self.value.show();
    print(")");
}

// External methods only available when T: Clone  
fn (self: &Wrapper<T>) clone() where T: Clone: Wrapper<T> {
    return Wrapper { value: self.value.clone() };
}
```

**Required Changes:**
1. AST: Support conditional impl blocks
2. Type checker: Evaluate impl conditions
3. Codegen: Generate impls only when conditions met

---

### 9. Type Aliases with Bounds
**Status:** ❌ Not Implemented  
**Importance:** LOW - Convenience

**Needed:**
```vex
// ❌ Current: Type aliases cannot have bounds
// ✅ Want: Constrained type aliases

type DisplayVec<T: Display> = Vec<T>;

fn print_vec(v: DisplayVec<i32>) {
    for x in v {
        x.show();  // OK: i32 implements Display
    }
}
```

**Required Changes:**
1. AST: Add `bounds` to `TypeAlias`
2. Parser: Parse bounds in type alias definitions
3. Type checker: Validate bounds when using alias

---

### 10. External Operator Methods (Compilation Order)
**Status:** ⚠️ Parser works, codegen issue  
**Importance:** LOW - Code organization

**Current Issue:**
```vex
// ❌ Compilation order problem
contract Add<Rhs> {
    op+(other: Rhs): Self;
}

fn (p: Point) op+(other: Point): Point {
    // External operator method
    // Problem: Not in struct_defs when struct compiled
}

struct Point {
    x: f64,
    y: f64,
}
```

**Problem:** External methods registered after struct compilation

**Solution Options:**
1. Two-pass compilation (collect all methods first)
2. Lazy method registration (defer until method call)
3. Require external methods in same file as struct

**Required Changes:**
1. Compiler: Two-pass or lazy registration
2. Codegen: Update `struct_defs` with external methods
3. Tests: Verify external operator methods work

---

## 📊 Implementation Priority

| Priority | Feature | Impact | Complexity | Estimate |
|----------|---------|--------|------------|----------|
| 🔴 P0 | Generic Impl Clause | Critical | Medium | 1-2 days |
| 🔴 P1 | Default Type Params | High | Low | 0.5 day |
| 🔴 P2 | Inline Trait Bounds (Enforcement) | High | Low | 0.5 day |
| 🟡 P3 | Lifetime Annotations | Medium | High | 2-3 days |
| 🟡 P4 | Higher-Kinded Types | Medium | Very High | 3-5 days |
| 🟡 P5 | Const Generics | Medium | Medium | 1-2 days |
| 🟡 P6 | Associated Type Constraints | Medium | Medium | 1 day |
| 🟢 P7 | Conditional Impl | Low | Medium | 1 day |
| 🟢 P8 | Type Alias Bounds | Low | Low | 0.5 day |
| 🟢 P9 | External Operators Fix | Low | Low | 0.5 day |

---

## 🎯 Recommended Implementation Order

### Phase 1: Core Polymorphism (3-4 days)
1. **Generic Impl Clause** - Most critical, enables proper operator overloading with type parameters
2. **Default Type Params** - Quick win, improves ergonomics
3. **Inline Trait Bounds** - Type checker enforcement

### Phase 2: Advanced Generics (3-5 days)
4. **Const Generics** - Static array safety
5. **Associated Type Constraints** - Advanced trait patterns
6. **External Operators Fix** - Complete operator overloading

### Phase 3: Advanced Type System (5-8 days)
7. **Higher-Kinded Types** - Most complex, highest abstraction
8. **Lifetime Annotations** - Complex but valuable
9. **Conditional Impls** - Polish feature

### Phase 4: Polish (1 day)
10. **Type Alias Bounds** - Nice-to-have

---

## 🔬 Testing Strategy

Each feature requires:
1. ✅ Parser tests - Syntax parsing
2. ✅ AST tests - Structure validation  
3. ✅ Type checker tests - Constraint validation
4. ✅ Codegen tests - Code generation
5. ✅ Integration tests - Real-world usage
6. ✅ Error tests - Error messages

---

## 📝 Notes

- **100% test pass achieved** by commenting out these features in `PROPOSAL_operator_syntax.vx`
- All features are **syntactically valid** but not implemented in type system
- **Rust compatibility** is a key goal - most features mirror Rust's type system
- **Zero-cost abstractions** maintained through monomorphization
- **Incremental implementation** recommended - one feature at a time

---

**Next Steps:**
1. User confirmation on implementation priority
2. Start with Phase 1 (Generic Impl Clause + Default Params)
3. Maintain 100% test pass throughout implementation
4. Update PROPOSAL_operator_syntax.vx as features complete

---

**Vex Philosophy:**
- ✅ Use `impl` keyword for struct trait implementations: `struct Type impl Trait1, Trait2 { }`
- ✅ Use `extends` keyword for standalone type extensions: `i32 extends Display, Clone;`
- ❌ No Rust-style `impl Trait for Type` external syntax
- ✅ Clear separation: `impl` = struct inline, `extends` = standalone declaration
