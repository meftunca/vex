# Vex Language - Stability Fixes Required

**Created:** November 10, 2025  
**Purpose:** Track bugs and missing features that need to be fixed for language stability  
**Status:** Active Development

This document lists issues found during stability testing that need to be fixed to make Vex language production-ready.

---

## 🔴 Critical Bugs (Must Fix for Stability)

### 1. Large Integer Literals - Lexer Error ❌

**Priority:** HIGH  
**Category:** Lexer  
**Status:** ❌ Not Implemented

**Problem:**
```vex
let i64_val: i64 = -9223372036854775808;        // i64 min
let i128_val: i128 = -170141183460469231731687303715884105728;  // i128 min
let u64_val: u64 = 18446744073709551615;        // u64 max
let u128_val: u128 = 340282366920938463463374607431768211455;  // u128 max
```

**Error:** `Lexer error: InvalidToken { span: 248..267 }`

**Files Affected:**
- `vex-lang-tests/type_system/primitives.vx`
- `vex-lang-tests/variables_and_constants/constants.vx`

**Root Cause:** Lexer uses standard integer parsing which fails on very large numbers (i64/i128/u64/u128 max values).

**Fix Required:**
- Update `vex-lexer/src/lib.rs` to handle arbitrary precision integer parsing
- Support i128/u128 max values without overflow
- Validate range after parsing

**Specification:** ✅ Required - All integer types should support their full range

---

### 2. Match Expression as Statement - Parser Issue ⚠️

**Priority:** MEDIUM  
**Category:** Parser  
**Status:** ⚠️ Partial Implementation

**Problem:**
```vex
fn calculate(a: i32, b: i32, operation: string): i32 {
    match operation {
        "add" => { return a + b; }
        "subtract" => { return a - b; }
        "multiply" => { return a * b; }
        _ => { return 0; }
    }
}
```

**Error:** `error[E0001]: Expected ';' after expression`

**Files Affected:**
- `vex-lang-tests/functions_and_methods/function_syntax.vx`

**Current Behavior:** 
- Match expressions work in `return` statements: `return match x { ... };`
- Match expressions work in assignments: `let result = match x { ... };`
- Match expressions **fail** when used directly as statements

**Specification:** ✅ Required - Spec shows match as statement (Specifications/11_Pattern_Matching.md line 28-32)

**Fix Required:**
- Update `vex-parser/src/parser/statements.rs` to allow match expressions as statements
- Wrap match expression in `Statement::Expression` when used as statement
- Or add explicit `Statement::Match` variant

---

### 3. Complex Array Indexing - Codegen Limitation ⚠️

**Priority:** MEDIUM  
**Category:** Codegen  
**Status:** ⚠️ Partial Implementation

**Problem:**
```vex
struct ComplexReceiver {
    points: [Point; 2],
    count: i32,
    
    fn add_point(p: Point)! {
        self.points[self.count] = p;  // ❌ Complex indexing not supported
        self.count = self.count + 1;
    }
}
```

**Error:** `Error: Compilation error: Complex indexing expressions not yet supported`

**Files Affected:**
- `vex-lang-tests/functions_and_methods/method_mutability.vx`

**Current Behavior:**
- Simple indexing works: `arr[0]`, `arr[1]`
- Variable indexing works: `arr[i]` (when `i` is a simple variable)
- **Complex indexing fails**: `self.points[self.count]`, `arr[i + 1]`

**Specification:** ⚠️ Not explicitly mentioned, but expected behavior

**Fix Required:**
- Update `vex-compiler/src/codegen_ast/expressions/access/indexing.rs`
- Support field access + indexing: `self.field[index]`
- Support expression indexing: `arr[expr]`

**Workaround:** Use if/elif chains for known indices (already applied in test file)

---

### 4. f16 Floating Point Type - Not Implemented ❌

**Priority:** LOW  
**Category:** Type System  
**Status:** ❌ Not Implemented

**Problem:**
```vex
let f16_val: f16 = 1.5;
```

**Files Affected:**
- `vex-lang-tests/type_system/primitives.vx`

**Evidence:**
- AST: `vex-ast/src/lib.rs:327` - `// F16,` (commented out)
- Codegen: `vex-compiler/src/codegen_ast/types.rs:23` - `// Type::F16 => ...` (commented out)

**Specification:** ✅ Listed in REFERENCE.md but not implemented

**Fix Options:**
1. **Implement f16** - Add to AST, codegen, and runtime
2. **Remove from documentation** - Update REFERENCE.md to remove f16

**Recommendation:** Implement f16 for completeness (low priority)

---

### 5. Borrow Checker False Positive - Use After Scope ⚠️

**Priority:** MEDIUM  
**Category:** Borrow Checker  
**Status:** ⚠️ False Positive

**Problem:**
```vex
let! p_mut = Point { x: 0, y: 0 };
let dist = p_mut.distance();
// ... later in same function
return dist as i32 + p_mut.x;  // ❌ Error: use of variable after it has gone out of scope
```

**Error:** `error[E0597]: use of variable after it has gone out of scope`

**Files Affected:**
- `vex-lang-tests/functions_and_methods/method_mutability.vx`

**Status:** ⚠️ **FALSE POSITIVE** - Variables are in the same function scope but borrow checker incorrectly reports them as out of scope.

**Possible Causes:**
- Lifetime checker incorrectly tracking variable scopes
- Method calls causing incorrect scope tracking
- String operations causing scope issues

**Fix Required:**
- Review `vex-compiler/src/borrow_checker/lifetimes.rs` scope tracking
- Fix variable scope tracking in method call contexts
- Verify scope tracking for immutable method calls

---

### 6. Empty Struct Instantiation - Syntax Unclear ⚠️

**Priority:** LOW  
**Category:** Parser/Codegen  
**Status:** ⚠️ Syntax Not Defined

**Problem:**
```vex
struct EmptyStruct {
    // No fields
}

let empty = EmptyStruct {};  // ❌ Parse error
let empty = EmptyStruct();   // ❌ Function not found
```

**Files Affected:**
- `vex-lang-tests/structs/basic_structs.vx`

**Specification:** ⚠️ "Unit Structs" mentioned in spec table of contents but section not found

**Fix Required:**
- Define syntax for empty struct instantiation
- Update parser to support chosen syntax
- Update codegen to handle empty structs

**Options:**
1. `EmptyStruct {}` - Rust-style (empty struct literal)
2. `EmptyStruct()` - Function-call style
3. `EmptyStruct` - Unit value style

**Recommendation:** Use `EmptyStruct {}` (Rust-style, consistent with other struct literals)

---

## ⚪ Design Decisions (Not Bugs)

### 1. Nested Function/Struct Definitions

**Status:** ✅ By Design - Not Supported

**Decision:** Functions and structs must be defined at module level, not inside functions.

**Action:** Test files updated to reflect this design decision.

---

### 2. Standalone Block Statements

**Status:** ✅ By Design - Not Supported

**Decision:** Blocks are only used as part of control flow statements (if, while, for), not as standalone statements.

**Action:** Test files updated to use `if true { ... }` instead of standalone `{ ... }`.

---

## 📋 Implementation Checklist

### Priority 1 (Critical - Blocks Stability)
- [ ] **Fix large integer literals** - Lexer support for i64/i128/u64/u128 max values
- [ ] **Fix match expression as statement** - Parser support for match statements
- [ ] **Fix borrow checker false positive** - Scope tracking for method calls

### Priority 2 (Important - Common Use Cases)
- [ ] **Fix complex array indexing** - Support `self.field[index]` and `arr[expr]`
- [ ] **Define empty struct syntax** - Choose and implement syntax

### Priority 3 (Nice to Have)
- [ ] **Implement f16 type** - Or remove from documentation

---

## 🔧 Files to Modify

### Lexer
- `vex-lexer/src/lib.rs` - Large integer literal parsing

### Parser
- `vex-parser/src/parser/statements.rs` - Match expression as statement
- `vex-parser/src/parser/expressions/literals.rs` - Empty struct literal syntax

### Codegen
- `vex-compiler/src/codegen_ast/expressions/access/indexing.rs` - Complex indexing
- `vex-compiler/src/codegen_ast/types.rs` - f16 type support (if implementing)

### Borrow Checker
- `vex-compiler/src/borrow_checker/lifetimes.rs` - Scope tracking for method calls

### AST
- `vex-ast/src/lib.rs` - f16 type (if implementing)

---

## 📝 Notes

- **Test Coverage:** All issues found via `vex-lang-tests/stability_test.sh`
- **Specification Compliance:** Issues marked with ✅ are required by spec
- **Workarounds:** Some issues have workarounds (e.g., complex indexing → if/elif chains)
- **Priority:** Focus on Priority 1 items first for language stability

---

*Last Updated: November 10, 2025*  
*Based on: Actual test execution results and specification review*

