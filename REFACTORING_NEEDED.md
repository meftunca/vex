# Vex File Size Refactoring Plan

**Policy:** Rust files MUST NOT exceed 400 lines  
**Updated:** November 5, 2025  
**Status:** 18 files exceed limit

---

## 📊 Quick Summary

| Priority    | Range         | Files      | Status                   |
| ----------- | ------------- | ---------- | ------------------------ |
| 🔴 CRITICAL | 1000+ lines   | 6          | **Split now**            |
| 🟡 HIGH     | 700-999 lines | 6          | Split before adding code |
| 🟠 MEDIUM   | 500-699 lines | 6          | Monitor, split at 600+   |
| 🟢 SAFE     | < 500 lines   | All others | Monitor only             |

**Total affected:** 18 files need refactoring  
**Check command:** `find vex-*/src -name "*.rs" -exec wc -l {} \; | awk '$1 > 400' | sort -rn`

---

## � Priority 1: Critical (1000+ lines) - SPLIT NOW

### 1. codegen_ast/expressions/mod.rs (1418 lines)

**Problem:** Monolithic expression compiler  
**Target:** 350 lines

**Action:**

```
expressions/
├── mod.rs (~350 lines)           # Dispatcher only
└── unary_control.rs (~350 lines) # NEW: Unary/if/match/block/cast
```

**Move to unary_control.rs:**

- `compile_unary_op()` - !, -, &, &!
- `compile_if_expression()`, `compile_match_expression()`
- `compile_block()`, `compile_cast()`

**Lines saved:** 1418 → 700 (50% reduction)

---

### 2. codegen_ast/statements.rs (1408 lines)

**Problem:** All statement types in one file  
**Target:** 350 lines

**Action:**

```
statements/
├── mod.rs (~350 lines)            # Let + dispatcher
├── control_flow.rs (~350 lines)   # NEW: while/for/loop/return
├── defer_break.rs (~300 lines)    # NEW: defer/break/continue
└── type_injection.rs (~400 lines) # NEW: Generic type helpers
```

**Lines saved:** 1408 → 1400 (4 files, better organization)

---

### 3. codegen_ast/functions.rs (1353 lines)

**Problem:** Function + generics + closures mixed  
**Target:** 400 lines

**Action:**

```
functions/
├── mod.rs (~400 lines)            # Core function compilation
├── generics.rs (~400 lines)       # NEW: Monomorphization
└── closure_env.rs (~350 lines)    # NEW: Closure environment
```

**Lines saved:** 1353 → 1150 (3 files)

---

### 4. codegen_ast/expressions/pattern_matching.rs (957 lines)

**Problem:** All pattern types together  
**Target:** 300 lines

**Action:**

```
expressions/pattern_matching/
├── mod.rs (~300 lines)            # Pattern dispatcher
├── destructuring.rs (~350 lines)  # NEW: Struct/tuple/array
└── enum_guards.rs (~300 lines)    # NEW: Enum + guards
```

**Lines saved:** 957 → 950 (3 files, clearer structure)

---

### 5. codegen_ast/builtins/builtin_types.rs (917 lines)

**Problem:** All builtin constructors in one file  
**Target:** 250 lines

**Action:**

```
builtins/builtin_types/
├── mod.rs (~250 lines)            # Registry
├── option_result.rs (~350 lines)  # NEW: Option/Result
└── collections.rs (~350 lines)    # NEW: Vec/Box/Tuple
```

**Lines saved:** 917 → 950 (3 files, easier to find)

---

### 6. parser/expressions.rs (902 lines)

**Problem:** All expression parsing logic  
**Target:** 300 lines

**Action:**

```
parser/expressions/
├── mod.rs (~300 lines)            # Dispatcher
├── primary.rs (~300 lines)        # NEW: Literals/identifiers
└── operators.rs (~300 lines)      # NEW: Binary/unary
```

**Lines saved:** 902 → 900 (3 files)

---

## 🟡 High Priority (700-999 lines) - Split Before Adding Code

### 7. calls.rs (820 lines) → Target: <400 lines

**Split Plan:**

```
expressions/calls/
├── mod.rs (~300 lines)            # Call dispatcher
├── method_calls.rs (~350 lines)   # NEW - Method call logic
└── generic_args.rs (~200 lines)   # NEW - Generic argument handling
```

---

### 8. access.rs (762 lines) → Target: <400 lines

**Split Plan:**

```
expressions/access/
├── mod.rs (~300 lines)            # Field access dispatcher
├── indexing.rs (~250 lines)       # NEW - Array/Vec indexing
└── chained.rs (~250 lines)        # NEW - Chained access (a.b.c)
```

---

### 9. parser/items.rs (757 lines) → Target: <400 lines

**Split Plan:**

```
parser/items/
├── mod.rs (~300 lines)            # Item dispatcher + functions
└── types.rs (~400 lines)          # NEW - Struct/enum/trait parsing
```

---

### 10. special.rs (723 lines) → Target: <400 lines

**Split Plan:**

```
expressions/special/
├── mod.rs (~300 lines)            # Dispatcher + closures
└── ranges_async.rs (~400 lines)   # NEW - Range + async/await
```

---

## 🟢 Medium Priority (600-700 lines)

### 11. lifetimes.rs (692 lines) → Split to 400 lines

### 12. moves.rs (625 lines) → Split to 400 lines

### 13. borrows.rs (610 lines) → Split to 400 lines

### 14. types.rs (597 lines) → Split to 400 lines

**Borrow Checker Split Plan:**

```
borrow_checker/
├── lifetimes/
│   ├── mod.rs (200 lines)
│   ├── inference.rs (250 lines)
│   └── validation.rs (250 lines)
├── moves/
│   ├── mod.rs (200 lines)
│   ├── tracking.rs (200 lines)
│   └── validation.rs (250 lines)
└── borrows/
    ├── mod.rs (200 lines)
    ├── tracking.rs (200 lines)
    └── validation.rs (250 lines)
```

---

## � Medium Priority (500-699 lines) - Monitor & Split at 600+

### 11. lifetimes.rs (692 lines) → Already near limit

### 12. moves.rs (625 lines) → Monitor

### 13. borrows.rs (610 lines) → Monitor

### 14. types.rs (597 lines) → Monitor

**Action:** Split when adding significant new code (>50 lines)

---

## 🟢 Low Priority (400-499 lines) - Monitor Only

- mod.rs (493) - OK for now
- compilation.rs (484) - OK for now
- types.rs (parser) (451) - OK for now
- control_flow.rs (439) - OK for now

**Action:** Keep under 500 lines total

---

## 📊 Refactoring Priority Order

### Phase 1 (Critical - Do First)

1. ⚠️ expressions/mod.rs (1418 → 400)
2. ⚠️ statements.rs (1408 → 400)
3. ⚠️ functions.rs (1353 → 400)

### Phase 2 (High Priority)

4. pattern_matching.rs (957 → 400)
5. builtin_types.rs (917 → 400)
6. parser/expressions.rs (902 → 400)

### Phase 3 (Before Adding Features)

7. calls.rs (820 → 400)
8. access.rs (762 → 400)
9. parser/items.rs (757 → 400)
10. special.rs (723 → 400)

### Phase 4 (Borrow Checker - As Needed)

11. lifetimes.rs (692 → 400)
12. moves.rs (625 → 400)
13. borrows.rs (610 → 400)

---

## 🎯 Implementation Strategy

### **CRITICAL RULE:** Split BEFORE adding code if file > 350 lines

**Process:**

1. **Before implementing new features:**

   ```bash
   # Check target file size
   wc -l path/to/file.rs

   # If > 350 lines → Split FIRST
   # If 300-350 lines → Can add small features (<50 lines)
   # If < 300 lines → OK to add code
   ```

2. **When file hits 380 lines:**

   - 🛑 STOP adding code immediately
   - Split into 2-3 modules
   - Then continue feature work

3. **Gradual refactoring:**

   - Refactor 1-2 files per week
   - Always test after splitting: `cargo build && ./test_all.sh`
   - Update imports carefully

4. **Split testing checklist:**
   ```bash
   cargo build          # Must compile
   ./test_all.sh        # All tests must pass
   git diff --stat      # Verify only targeted files changed
   ```

---

## ✅ Success Metrics

**Target:** All files < 400 lines (300-350 ideal)  
**Current:** 18 files exceed 400 lines (down from 31)  
**Progress:** `find . -name "*.rs" -exec wc -l {} \; | awk '$1 > 400'`

**Benefits:**

- ✅ AI edits entire file in 1-2 tool calls (vs 7-10 for large files)
- ✅ Human code review takes <5 minutes per file
- ✅ Clear separation of concerns
- ✅ Merge conflicts 80% easier to resolve
- ✅ New contributors onboard faster
