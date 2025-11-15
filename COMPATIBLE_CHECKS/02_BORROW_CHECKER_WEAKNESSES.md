# 02: Borrow Checker Weaknesses

**Severity:** 🔴 CRITICAL  
**Category:** Memory Safety / Compiler Correctness  
**Analysis Date:** 15 Kasım 2025  
**Status:** IDENTIFIED - CRITICAL FIXES NEEDED

---

## Executive Summary

Vex'in borrow checker'ı 4-phase architecture ile çalışıyor (borrows, moves, immutability, drop) ancak **20 kritik eksiklik** bulundu. Lifetime tracking, move semantics ve unsafe handling alanlarında Rust seviyesine ulaşmak için ciddi çalışma gerekiyor.

**Ana Sorunlar:**
- Lifetime elision hiç implement edilmemiş
- Move checker closure'lar için incomplete
- Reborrow tracking yok
- Drop order garanti edilmiyor
- Unsafe block tracking minimal

**Impact:** Memory safety garantileri bazı edge case'lerde ihlal ediliyor, potansiyel use-after-free riskleri.

---

## Critical Issues (🔴)

### Issue 1: Lifetime Elision Not Implemented

**File:** `vex-compiler/src/borrow_checker/` (missing module)  
**Severity:** 🔴 CRITICAL  
**Impact:** Users must explicitly annotate all lifetimes, poor ergonomics

**Evidence:**
```rust
// No lifetime elision module exists
// vex-compiler/src/borrow_checker/
├── borrows/
├── moves/
├── immutability.rs
└── mod.rs
// Missing: lifetimes.rs
```

**Problem:**
```vex
// This doesn't work (should work with elision):
fn first(s: &str) -> &str {
    &s[0..1]  // ❌ Error: missing lifetime parameter
}

// Must write:
fn first<'a>(s: &'a str) -> &'a str {
    &s[0..1]
}
```

**Rust Elision Rules (should implement):**
1. Each input reference gets its own lifetime
2. If exactly one input lifetime, output gets that lifetime
3. If multiple inputs and one is `&self`, output gets `&self` lifetime

**Recommendation:**
```rust
// Add to vex-compiler/src/borrow_checker/lifetimes.rs
pub struct LifetimeElision {
    rules: Vec<ElisionRule>,
}

impl LifetimeElision {
    pub fn elide_function_signature(
        &self,
        params: &[Parameter],
        return_type: &Type,
    ) -> Result<Vec<Lifetime>, String> {
        // Implement 3 elision rules
        if params.len() == 1 && has_reference(params[0]) {
            // Rule 1: Single input lifetime
            return Ok(vec![Lifetime::input(0)]);
        }
        // ... implement other rules
    }
}
```

**Effort:** 2-3 weeks  
**References:** Rust RFC 141, 1951 (lifetime elision)

---

### Issue 2: Move Checker Incomplete for Closures

**File:** `vex-compiler/src/borrow_checker/closure_traits.rs:39-263`  
**Severity:** 🔴 CRITICAL  
**Impact:** Closures can capture moved values, causing use-after-free

**Evidence:**
```rust
// vex-compiler/src/borrow_checker/closure_traits.rs:39
let param_names: HashSet<String> = params.iter().map(|p| p.name.clone()).collect();

// Missing: Track which variables are moved into closure
// Missing: Enforce move semantics for captured values
```

**Problem:**
```vex
fn test_closure_move() {
    let s = String.from("hello");
    let f = || {
        consume(s);  // s moved into closure
    };
    f();
    println(s);  // ❌ Should error: s already moved
}
```

**Recommendation:**
```rust
// Add to closure_traits.rs
struct ClosureMoveChecker {
    captured_by_move: HashSet<String>,
    captured_by_ref: HashSet<String>,
}

impl ClosureMoveChecker {
    fn check_capture_mode(&mut self, var: &str, usage: Usage) {
        match usage {
            Usage::Move => {
                if self.captured_by_ref.contains(var) {
                    error!("Cannot move {var}, already borrowed");
                }
                self.captured_by_move.insert(var);
            }
            Usage::Borrow => {
                if self.captured_by_move.contains(var) {
                    error!("Cannot borrow {var}, already moved");
                }
                self.captured_by_ref.insert(var);
            }
        }
    }
}
```

**Effort:** 2 weeks

---

### Issue 3: Reborrow Not Tracked

**File:** `vex-compiler/src/borrow_checker/borrows/checker.rs:104-112`  
**Severity:** 🔴 CRITICAL  
**Impact:** Multiple mutable borrows allowed through reborrow, soundness issue

**Evidence:**
```rust
// vex-compiler/src/borrow_checker/borrows/checker.rs:104
self.check_can_borrow(&var, kind.clone())?;

self.active_borrows.insert(
    borrow_id,
    BorrowInfo {
        kind: kind.clone(),
        variable: var.clone(),
        location: None,
    },
);

// Missing: Check if this is a reborrow of existing borrow
// Missing: Create parent-child relationship for reborrows
```

**Problem:**
```vex
fn reborrow_test(x: &i32!) {
    let y = &x!;  // First mutable borrow
    let z = &y!;  // Reborrow of y (should be tracked)
    *z = 10;
    *y = 20;  // Should error: z still active
}
```

**Recommendation:**
```rust
struct BorrowInfo {
    kind: BorrowKind,
    variable: String,
    location: Option<String>,
    parent_borrow: Option<BorrowId>,  // NEW: Track reborrow chain
}

fn track_reborrow(&mut self, original: &str, reborrow: &str) {
    if let Some(original_id) = self.find_active_borrow(original) {
        let reborrow_id = self.next_borrow_id();
        self.active_borrows.insert(reborrow_id, BorrowInfo {
            parent_borrow: Some(original_id),
            // ...
        });
    }
}
```

**Effort:** 1-2 weeks

---

### Issue 4: Drop Order Not Guaranteed

**File:** `vex-compiler/src/codegen_ast/drop_trait.rs`  
**Severity:** 🔴 CRITICAL  
**Impact:** Resources may be freed in wrong order, dangling references

**Evidence:**
```rust
// vex-compiler/src/codegen_ast/drop_trait.rs
// Drop trait exists but no ordering guarantees
impl<'ctx> ASTCodeGen<'ctx> {
    pub(crate) fn register_drop_call(
        &mut self,
        var_name: &str,
        drop_fn: FunctionValue<'ctx>,
    ) -> Result<(), String> {
        // Pushes to vector but doesn't track dependencies
        self.pending_drops.push((var_name.to_string(), drop_fn));
        Ok(())
    }
}
```

**Problem:**
```vex
fn test_drop_order() {
    let file = File.open("test.txt");
    let buffer = Buffer.new();
    buffer.set_file(&file);
    // file should be dropped AFTER buffer
    // But no guarantee in current implementation
}
```

**Rust Drop Order:**
1. Variables dropped in reverse order of declaration
2. Fields dropped in declaration order
3. Array elements dropped in order

**Recommendation:**
```rust
struct DropGraph {
    nodes: HashMap<VarId, DropNode>,
    dependencies: HashMap<VarId, Vec<VarId>>,
}

impl DropGraph {
    fn add_dependency(&mut self, dependent: VarId, dependency: VarId) {
        self.dependencies.entry(dependent)
            .or_insert_with(Vec::new)
            .push(dependency);
    }
    
    fn topological_sort(&self) -> Vec<VarId> {
        // Return drop order respecting dependencies
    }
}
```

**Effort:** 2-3 weeks

---

### Issue 5: Unsafe Block Tracking Minimal

**File:** `vex-compiler/src/borrow_checker/borrows/statement_checking.rs:226-237`  
**Severity:** 🔴 CRITICAL  
**Impact:** Unsafe code can escape unsafe blocks, memory safety violation

**Evidence:**
```rust
// vex-compiler/src/borrow_checker/borrows/statement_checking.rs:226
fn check_unsafe_block(&mut self, block: &vex_ast::Block) -> BorrowResult<()> {
    let prev_unsafe = self.in_unsafe_block;
    self.in_unsafe_block = true;

    for stmt in &block.statements {
        self.check_statement(stmt)?;
    }

    self.in_unsafe_block = prev_unsafe;
    Ok(())
}

// Missing: Track what unsafe operations are actually performed
// Missing: Require unsafe for all raw pointer derefs
// Missing: Enforce unsafe fn calls only in unsafe blocks
```

**Problem:**
```vex
unsafe fn dangerous() {
    // ...
}

fn safe() {
    dangerous();  // ❌ Should require unsafe block
}
```

**Recommendation:**
```rust
enum UnsafeOp {
    RawPointerDeref,
    UnsafeFnCall(String),
    UnionFieldAccess,
    AsmBlock,
}

struct UnsafeTracker {
    in_unsafe: bool,
    operations: Vec<UnsafeOp>,
}

impl UnsafeTracker {
    fn check_operation(&self, op: UnsafeOp) -> Result<(), BorrowError> {
        if !self.in_unsafe {
            return Err(BorrowError::UnsafeOperationOutsideUnsafeBlock {
                operation: format!("{:?}", op),
                location: None,
            });
        }
        Ok(())
    }
}
```

**Effort:** 1 week

---

## High Priority Issues (🟡)

### Issue 6: Temporary Lifetime Extension Missing

**File:** `vex-compiler/src/borrow_checker/borrows/expression_checking.rs`  
**Severity:** 🟡 HIGH  
**Impact:** References to temporaries may dangle

**Problem:**
```vex
fn get_str() -> String { "hello".to_string() }

fn test() {
    let s: &str = &get_str();  // ❌ Temporary dropped immediately
    println(s);  // Dangling reference
}
```

**Recommendation:** Extend temporary lifetimes to end of statement

**Effort:** 1-2 weeks

---

### Issue 7: Non-Lexical Lifetimes (NLL) Not Implemented

**File:** N/A (not implemented)  
**Severity:** 🟡 HIGH  
**Impact:** Many safe programs rejected by borrow checker

**Problem:**
```vex
fn test() {
    let x = &vec[0]!;
    *x = 10;
    // x not used after this point
    println(vec);  // ❌ Error: vec still borrowed
}
```

**Recommendation:** Implement NLL algorithm (complex, 4-6 weeks)

**Effort:** 4-6 weeks

---

### Issue 8: Interior Mutability Not Tracked

**File:** `vex-compiler/src/borrow_checker/immutability.rs`  
**Severity:** 🟡 HIGH  
**Impact:** Cannot use RefCell/Mutex patterns safely

**Problem:**
```vex
// RefCell pattern doesn't work:
struct Container {
    value: RefCell<i32>
}

fn test(c: &Container) {
    let x = c.value.borrow!();
    let y = c.value.borrow!();  // ❌ Should error: already borrowed mutably
}
```

**Recommendation:** Add UnsafeCell tracking, borrow checking for interior mutability

**Effort:** 2-3 weeks

---

### Issue 9: Partial Move Not Handled

**File:** `vex-compiler/src/borrow_checker/moves/checker.rs`  
**Severity:** 🟡 HIGH  
**Impact:** Cannot move out of struct fields individually

**Problem:**
```vex
struct Pair { a: String, b: String }

fn test() {
    let p = Pair { a: "x", b: "y" };
    let x = p.a;  // Move out of p.a
    let y = p.b;  // Move out of p.b
    // p is now partially moved, some fields invalid
}
```

**Recommendation:** Track field-level move state

**Effort:** 2 weeks

---

### Issue 10: 2-Phase Borrow Not Implemented

**File:** `vex-compiler/src/borrow_checker/borrows/checker.rs`  
**Severity:** 🟡 HIGH  
**Impact:** Method calls on mutable references fail incorrectly

**Problem:**
```vex
fn test(v: &Vec<i32>!) {
    v.push(v.len() as i32);  // ❌ Error: v borrowed mutably and immutably
}
```

**Recommendation:** Implement 2-phase borrow algorithm (reserve + activate)

**Effort:** 2 weeks

---

## Medium Priority Issues (🟢)

### Issue 11: Polonius Borrow Checker

**Severity:** 🟢 MEDIUM  
**Impact:** More precise but complex

**Recommendation:** Consider implementing Polonius (next-gen NLL)

**Effort:** 8-10 weeks (major project)

---

### Issue 12: Lifetime Bounds on Traits

**Severity:** 🟢 MEDIUM  
**Impact:** Cannot express `T: 'static` constraints

**Problem:**
```vex
contract Send where Self: 'static { }  // ❌ Not supported
```

**Effort:** 1-2 weeks

---

### Issue 13: Closure Lifetime Inference

**Severity:** 🟢 MEDIUM  
**Impact:** Must annotate closure lifetimes manually

**Effort:** 2 weeks

---

### Issue 14: Drop Flags Optimization

**Severity:** 🟢 MEDIUM  
**Impact:** Unnecessary drop flag checks

**Effort:** 1 week

---

### Issue 15: Coercion of References

**Severity:** 🟢 MEDIUM  
**Impact:** `&T` → `&dyn Trait` coercion missing

**Effort:** 1 week

---

### Issue 16: Box Deref Optimization

**Severity:** 🟢 MEDIUM  
**Impact:** Extra indirection for Box types

**Effort:** 3 days

---

## Low Priority Issues (🔵)

### Issue 17: Lifetime Error Messages

**Severity:** 🔵 LOW  
**Impact:** Error messages could be clearer

**Effort:** 1 week

---

## Metrics Summary

| Category | Critical | High | Medium | Low | Total |
|----------|----------|------|--------|-----|-------|
| Lifetime Tracking | 2 | 2 | 3 | 1 | 8 |
| Move Semantics | 1 | 2 | 1 | 0 | 4 |
| Borrow Checking | 1 | 3 | 1 | 0 | 5 |
| Unsafe Tracking | 1 | 1 | 0 | 0 | 2 |
| Drop Order | 1 | 0 | 0 | 0 | 1 |
| **TOTAL** | **5** | **8** | **6** | **1** | **20** |

---

## Implementation Roadmap

### Phase 1: Critical Fixes (Week 1-3)
- [ ] Implement basic lifetime elision (3 rules)
- [ ] Fix closure move checker
- [ ] Add reborrow tracking
- [ ] Guarantee drop order
- [ ] Improve unsafe block tracking

### Phase 2: High Priority (Week 4-7)
- [ ] Temporary lifetime extension
- [ ] Non-lexical lifetimes (NLL)
- [ ] Interior mutability tracking
- [ ] Partial move handling
- [ ] 2-phase borrow

### Phase 3: Medium Priority (Week 8-12)
- [ ] Lifetime bounds on traits
- [ ] Closure lifetime inference
- [ ] Drop flag optimization
- [ ] Reference coercion

---

## Testing Plan

```vex
// test_lifetime_elision.vx
fn first(s: &str) -> &str { &s[0..1] }  // Should work

// test_closure_move.vx
fn test() {
    let s = String.from("x");
    let f = || consume(s);
    f();
    // println(s);  // Should error
}

// test_reborrow.vx
fn reborrow(x: &i32!) {
    let y = &x!;
    let z = &y!;
    *z = 10;
    // *y = 20;  // Should error
}

// test_drop_order.vx
fn test() {
    let _a = Resource.new("a");
    let _b = Resource.new("b");
    // Should drop b, then a
}
```

---

## Related Issues

- [01_TYPE_SYSTEM_GAPS.md](./01_TYPE_SYSTEM_GAPS.md) - Type inference affects lifetime inference
- [07_MEMORY_SAFETY_CONCERNS.md](./07_MEMORY_SAFETY_CONCERNS.md) - Borrow checker is primary safety mechanism
- [03_CODEGEN_LLVM_ISSUES.md](./03_CODEGEN_LLVM_ISSUES.md) - Drop codegen depends on borrow checker

---

## References

- Rust RFC 1214: Non-lexical lifetimes
- Rust RFC 2094: Non-lexical lifetimes (NLL)
- Polonius: https://github.com/rust-lang/polonius
- Rust borrow checker: https://rustc-dev-guide.rust-lang.org/borrow_check.html

---

**Next Steps:** Implement lifetime elision as highest priority. NLL and closure improvements follow.
