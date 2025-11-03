# Vex Borrow Checker Implementation Plan

**Version:** v0.9  
**Priority:** 🔴 Critical (must be implemented before &T! and new())  
**Estimated Time:** 1-2 weeks  
**Status:** Planning Phase

---

## 🎯 Why Borrow Checker First?

### Current Problem

```vex
// ❌ This compiles but violates immutability!
let x = 42;
x = 50;  // No error - immutability not enforced

// ❌ This compiles but causes use-after-move!
let y = vec;
let z = vec;  // vec moved twice - no error

// ❌ This would compile but is unsafe!
let! data = vec;
let ref1 = &data!;
let ref2 = &data!;
data = new_vec;  // Modifying while borrowed - no error
```

Without borrow checker:

- ✅ Syntax looks safe (`let` vs `let!`)
- ❌ **Compiler doesn't enforce safety**
- ❌ Memory corruption possible
- ❌ Data races possible

### Dependencies

- `&T!` reference syntax **needs** borrow rules enforcement
- `new()` heap allocation **needs** lifetime tracking
- Safe concurrency **needs** ownership semantics

---

## 📋 Implementation Phases

### Phase 1: Basic Immutability Check (2-3 days)

**Goal:** Enforce `let` vs `let!` semantics

```rust
// In compiler/src/borrow_checker/immutability.rs

pub struct ImmutabilityChecker {
    immutable_vars: HashSet<String>,
    mutable_vars: HashSet<String>,
}

impl ImmutabilityChecker {
    pub fn check_assignment(&self, var: &str) -> Result<(), BorrowError> {
        if self.immutable_vars.contains(var) {
            return Err(BorrowError::AssignToImmutable {
                variable: var.to_string(),
                hint: format!("Consider using `let! {}` instead", var),
            });
        }
        Ok(())
    }
}
```

**Tests:**

```vex
// Should compile
let! x = 10;
x = 20;  // ✅

// Should fail
let y = 10;
y = 20;  // ❌ Error: cannot assign to immutable variable `y`
```

**Implementation:**

1. Track `let` declarations → immutable_vars
2. Track `let!` declarations → mutable_vars
3. On assignment `x = ...` → check if x is immutable
4. Report error with helpful message

---

### Phase 2: Move Semantics (3-4 days)

**Goal:** Prevent use-after-move

```rust
// In compiler/src/borrow_checker/moves.rs

pub struct MoveChecker {
    moved_vars: HashMap<String, Location>,  // var -> where it was moved
    current_scope: ScopeId,
}

impl MoveChecker {
    pub fn check_move(&mut self, var: &str, location: Location) -> Result<(), BorrowError> {
        if let Some(moved_at) = self.moved_vars.get(var) {
            return Err(BorrowError::UseAfterMove {
                variable: var.to_string(),
                moved_at: moved_at.clone(),
                used_at: location,
            });
        }

        // Mark as moved (for non-Copy types)
        self.moved_vars.insert(var.to_string(), location);
        Ok(())
    }
}
```

**Copy vs Move Types:**

```rust
// Automatically Copy types
- Primitives: i32, f32, bool
- Tuples of Copy types
- References: &T, &T!

// Move types
- Structs (unless #[derive(Copy)])
- Heap allocations (new())
- Vec, String, etc.
```

**Tests:**

```vex
// Copy types - OK
let x = 42;
let y = x;
let z = x;  // ✅ i32 is Copy

// Move types - Error
struct Point { x: i32, y: i32 }
let p = Point { x: 10, y: 20 };
let q = p;  // p moved
let r = p;  // ❌ Error: use of moved value `p`
```

---

### Phase 3: Borrow Rules (4-5 days)

**Goal:** Enforce Rust-style borrowing rules

```rust
// In compiler/src/borrow_checker/borrows.rs

pub struct BorrowChecker {
    active_borrows: HashMap<String, Vec<Borrow>>,
}

#[derive(Clone, Debug)]
pub struct Borrow {
    borrow_type: BorrowType,
    scope: ScopeId,
    location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BorrowType {
    Immutable,  // &T
    Mutable,    // &T!
}

impl BorrowChecker {
    pub fn add_borrow(&mut self, var: &str, borrow: Borrow) -> Result<(), BorrowError> {
        let borrows = self.active_borrows.entry(var.to_string()).or_default();

        // Rule 1: Cannot have mutable borrow if any borrows exist
        if borrow.borrow_type == BorrowType::Mutable && !borrows.is_empty() {
            return Err(BorrowError::MutableBorrowWhileBorrowed {
                variable: var.to_string(),
                existing_borrow: borrows[0].location.clone(),
                new_borrow: borrow.location.clone(),
            });
        }

        // Rule 2: Cannot have immutable borrow if mutable borrow exists
        if borrow.borrow_type == BorrowType::Immutable {
            if let Some(existing) = borrows.iter().find(|b| b.borrow_type == BorrowType::Mutable) {
                return Err(BorrowError::ImmutableBorrowWhileMutableBorrowed {
                    variable: var.to_string(),
                    mutable_borrow: existing.location.clone(),
                    new_borrow: borrow.location.clone(),
                });
            }
        }

        borrows.push(borrow);
        Ok(())
    }

    pub fn check_mutation(&self, var: &str) -> Result<(), BorrowError> {
        if let Some(borrows) = self.active_borrows.get(var) {
            if !borrows.is_empty() {
                return Err(BorrowError::MutationWhileBorrowed {
                    variable: var.to_string(),
                    borrowed_at: borrows[0].location.clone(),
                });
            }
        }
        Ok(())
    }
}
```

**The Rules:**

1. ✅ **Multiple immutable borrows OK**: `&T`, `&T`, `&T` ✅
2. ✅ **One mutable borrow XOR many immutable**: Either `&T!` OR `&T`+`&T`+...
3. ❌ **Cannot modify while borrowed**: `let! x = ...; let r = &x; x = ...; ❌`

**Tests:**

```vex
// Test 1: Multiple immutable borrows OK
let x = 42;
let r1 = &x;
let r2 = &x;
let r3 = &x;  // ✅ OK

// Test 2: Mutable borrow is exclusive
let! y = 10;
let r1 = &y!;
let r2 = &y;   // ❌ Error: cannot borrow as immutable while mutable borrow exists

// Test 3: Cannot mutate while borrowed
let! z = 20;
let r = &z;
z = 30;  // ❌ Error: cannot assign to `z` because it is borrowed
```

---

### Phase 4: Lifetime Analysis (5-6 days)

**Goal:** Prevent dangling references

```rust
// In compiler/src/borrow_checker/lifetimes.rs

pub struct LifetimeChecker {
    scopes: Vec<Scope>,
    current_scope: ScopeId,
}

pub struct Scope {
    id: ScopeId,
    parent: Option<ScopeId>,
    variables: HashMap<String, VarInfo>,
    borrows: Vec<BorrowInfo>,
}

pub struct VarInfo {
    name: String,
    lifetime: Lifetime,
}

pub struct BorrowInfo {
    borrowed_var: String,
    lifetime: Lifetime,
}

impl LifetimeChecker {
    pub fn check_return(&self, returned_ref: &str) -> Result<(), BorrowError> {
        // Check if returning a reference to local variable
        let var_scope = self.get_var_scope(returned_ref)?;
        let current_scope = self.current_scope;

        if var_scope >= current_scope {
            return Err(BorrowError::ReturnLocalReference {
                variable: returned_ref.to_string(),
            });
        }

        Ok(())
    }

    pub fn exit_scope(&mut self) {
        // Invalidate all borrows created in this scope
        if let Some(scope) = self.scopes.last_mut() {
            scope.borrows.clear();
            scope.variables.clear();
        }
        self.current_scope = self.current_scope.parent().unwrap();
    }
}
```

**Tests:**

```vex
// Test 1: Cannot return reference to local
fn bad_ref() : &i32 {
    let x = 42;
    return &x;  // ❌ Error: returns reference to local variable
}

// Test 2: Can return reference to parameter
fn good_ref(x: &i32) : &i32 {
    return x;  // ✅ OK
}

// Test 3: Scope-based lifetime
{
    let x = 10;
    let r = &x;
    // r valid here
}
// r invalid here - out of scope
```

---

## 🏗️ Architecture

### Module Structure

```
vex-compiler/src/borrow_checker/
├── mod.rs                  # Main borrow checker orchestrator
├── immutability.rs         # Phase 1: let vs let!
├── moves.rs                # Phase 2: move semantics
├── borrows.rs              # Phase 3: borrow rules
├── lifetimes.rs            # Phase 4: lifetime analysis
├── errors.rs               # Error types and messages
└── tests/                  # Unit tests
    ├── immutability_tests.rs
    ├── move_tests.rs
    ├── borrow_tests.rs
    └── lifetime_tests.rs
```

### Integration Points

**1. Parser → Borrow Checker**

```rust
// After parsing, run borrow checker
let ast = parser.parse(source)?;
let mut borrow_checker = BorrowChecker::new();
borrow_checker.check_program(&ast)?;
```

**2. Borrow Checker → Codegen**

```rust
// Only generate code if borrow check passes
if let Err(e) = borrow_checker.check(&ast) {
    return Err(CompilerError::BorrowCheckFailed(e));
}
codegen.compile(&ast)?;
```

---

## 📊 Error Messages (User-Friendly)

```vex
let x = 42;
x = 50;
```

```
error[E0384]: cannot assign twice to immutable variable `x`
  --> example.vx:2:1
   |
1  | let x = 42;
   |     - first assignment to `x`
2  | x = 50;
   | ^^^^^^ cannot assign twice to immutable variable
   |
help: consider making this binding mutable
   |
1  | let! x = 42;
   |     +
```

```vex
let! y = 10;
let r1 = &y!;
let r2 = &y;
```

```
error[E0502]: cannot borrow `y` as immutable because it is also borrowed as mutable
  --> example.vx:3:10
   |
2  | let r1 = &y!;
   |          --- mutable borrow occurs here
3  | let r2 = &y;
   |          ^^ immutable borrow occurs here
   |
   = note: mutable borrow must be exclusive
```

---

## 🧪 Testing Strategy

### Unit Tests (Per Phase)

```rust
#[test]
fn test_immutable_assignment() {
    let source = r#"
        let x = 42;
        x = 50;
    "#;

    let result = compile(source);
    assert!(matches!(result, Err(BorrowError::AssignToImmutable { .. })));
}

#[test]
fn test_use_after_move() {
    let source = r#"
        struct Point { x: i32 }
        let p = Point { x: 10 };
        let q = p;
        let r = p;
    "#;

    let result = compile(source);
    assert!(matches!(result, Err(BorrowError::UseAfterMove { .. })));
}
```

### Integration Tests

```vex
// tests/borrow_checker/full_program.vx
fn main() : i32 {
    // Test immutability
    let x = 42;
    let! y = 10;
    y = 20;  // OK

    // Test borrows
    let r1 = &y;
    let r2 = &y;  // OK - multiple immutable

    return 0;
}
```

---

## 📈 Progress Tracking

| Phase                       | Days           | Status         | Tests    | Blockers |
| --------------------------- | -------------- | -------------- | -------- | -------- |
| **Phase 1: Immutability**   | 2-3            | 🔴 Not started | 0/10     | -        |
| **Phase 2: Move Semantics** | 3-4            | 🔴 Not started | 0/15     | Phase 1  |
| **Phase 3: Borrow Rules**   | 4-5            | 🔴 Not started | 0/20     | Phase 2  |
| **Phase 4: Lifetimes**      | 5-6            | 🔴 Not started | 0/15     | Phase 3  |
| **Total**                   | **14-18 days** | **0%**         | **0/60** | -        |

---

## 🎯 Success Criteria

### Minimum Viable Borrow Checker (MVP)

- ✅ Phase 1: Immutability (must have)
- ✅ Phase 2: Move Semantics (must have)
- ✅ Phase 3: Borrow Rules (must have)
- ⚠️ Phase 4: Lifetimes (nice to have initially)

### After Implementation

- All 23 examples still compile ✅
- New examples with safety violations fail ❌
- Error messages are clear and helpful 📝
- Performance impact < 10% compile time ⚡

---

## 🔄 Integration with v0.9 Features

### Enables Safe Implementation Of:

1. **`&T!` Mutable References**

```vex
let! x = 10;
let r = &x!;  // Borrow checker ensures exclusive access
*r = 20;
```

2. **`new()` Heap Allocation**

```vex
let ptr = new(42);  // Borrow checker tracks ownership
// Automatic drop when ptr goes out of scope
```

3. **Pattern Matching with Ownership**

```vex
match option {
    Some(value) => { /* value moved here */ },
    None => { /* ... */ },
}
// option moved, cannot use again
```

---

## 📚 Resources

### Reference Implementations

- **Rust Borrow Checker:** https://doc.rust-lang.org/nomicon/borrow-checking.html
- **Polonius (New Rust BC):** https://github.com/rust-lang/polonius
- **Simple BC Tutorial:** https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html

### Vex-Specific Considerations

- Simpler than Rust: No explicit lifetimes in syntax
- Bang operator consistency: `let!`, `&T!`
- Scope-based lifetimes only (no lifetime parameters initially)

---

## 🤔 Open Questions

1. **Copy trait:** Auto-derive for simple structs? Manual annotation?
2. **Lifetime elision:** How much inference? Explicit annotations later?
3. **Interior mutability:** Cell/RefCell equivalents needed?
4. **Partial moves:** Allow moving struct fields?

---

**Next Steps:**

1. Review and approve this plan
2. Create initial borrow_checker module structure
3. Implement Phase 1 (Immutability) with tests
4. Iterate based on feedback

**Last Updated:** November 3, 2025
