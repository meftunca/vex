# Critical Issue #3: LLVM Pointer Operations Safety Concerns

**Severity:** 🔴 HIGH  
**Category:** Memory Safety / Undefined Behavior  
**Discovered:** 15 Kasım 2025  
**Status:** DOCUMENTED - AUDIT REQUIRED

---

## Executive Summary

Vex compiler generates LLVM IR with **100+ unsafe pointer operations** (`build_alloca`, `build_load`, `build_store`, `build_gep`) without comprehensive safety checks. While LLVM verifies structural correctness, runtime alignment violations, null pointer dereferences, and use-after-free bugs can still occur.

**Risk:** Silent data corruption, segmentation faults, or undefined behavior in compiled Vex programs.

---

## Affected Components

### 🔴 Critical Safety Gaps

**1. Stack Allocations** (`build_alloca` - 50+ instances)

- `vex-compiler/src/codegen_ast/expressions/collections.rs:64,188,230,260,316`
- `vex-compiler/src/codegen_ast/expressions/pattern_matching/*.rs`
- `vex-compiler/src/codegen_ast/expressions/references.rs:112`
- **Risk:** Stack overflow, alignment violations, lifetime issues

**2. Pointer Dereferencing** (`build_load` - 40+ instances)

- `vex-compiler/src/codegen_ast/expressions/references.rs:179-255`
- `vex-compiler/src/codegen_ast/statements/assignment.rs:210,242,291`
- `vex-compiler/src/codegen_ast/scope_management.rs:80,105,117,138,156,179,199`
- **Risk:** Null pointer dereference, dangling pointer access

**3. Memory Writes** (`build_store` - 40+ instances)

- `vex-compiler/src/codegen_ast/statements/assignment.rs:56,138,295,310,375,382,506,513,520`
- `vex-compiler/src/codegen_ast/expressions/collections.rs:107,196,269,332,367,416`
- **Risk:** Write to invalid memory, data races in unsafe code

**4. Pointer Arithmetic** (`build_gep` - 15+ instances)

- `vex-compiler/src/codegen_ast/expressions/collections.rs:241,357,406`
- `vex-compiler/src/codegen_ast/expressions/access/indexing.rs:242`
- **Risk:** Buffer overflow, out-of-bounds access

---

## Detailed Analysis

### Pattern 1: Unchecked Stack Allocation Size

```rust
// vex-compiler/src/codegen_ast/expressions/collections.rs:64
let array_ptr = self.builder
    .build_alloca(array_type, "arrayliteral")
//  ^^^^^^^^^^^^^ No size validation!
    .map_err(|e| format!("Failed to allocate array: {}", e))?;
```

**Problem:**

- No limit on array size in Vex source code
- User can write: `let arr: [i32; 1000000000] = [0; 1000000000];`
- Allocates 4GB on stack → **immediate stack overflow**

**Current Behavior:**

```vex
fn stack_overflow() {
    let huge: [i64; 10000000] = [0; 10000000]; // 80MB stack alloc
    println("If you see this, stack didn't overflow"); // Never prints
}
```

**Observed Result:**

- Segmentation fault (no error message)
- Stack corruption in surrounding frames
- Debugger shows invalid stack pointer

**Safety Gap:**

- LLVM allows arbitrary `alloca` size
- No compiler limit on stack allocation
- No runtime check before allocation

### Pattern 2: Load Without Null Check

```rust
// vex-compiler/src/codegen_ast/expressions/references.rs:231-255
let loaded = match inner_type {
    BasicTypeEnum::IntType(it) => self.builder
        .build_load(it, ptr, "deref")
//      ^^^^^^^^^^^ ptr could be NULL!
        .map_err(|e| format!("Failed to dereference pointer: {}", e))?,
    // ... other types
};
```

**Problem:**

- No validation that `ptr` is non-null
- No check if `ptr` points to valid memory
- Rust's `*ptr` checks in unsafe blocks, LLVM IR doesn't

**Exploitation Example:**

```vex
fn null_deref() {
    let null_ptr: *i32 = 0 as *i32;
    unsafe {
        let value = *null_ptr; // Compiles without warning
        println("{}", value);   // Segfault at runtime
    }
}
```

**LLVM IR Generated:**

```llvm
%1 = inttoptr i64 0 to ptr
%2 = load i32, ptr %1  ; NULL POINTER DEREFERENCE (no check!)
```

### Pattern 3: GEP Bounds Checking Missing

```rust
// vex-compiler/src/codegen_ast/expressions/collections.rs:357
let elem_ptr = self.builder.build_gep(
    elem_type,
    array_ptr,
    &[
        self.context.i32_type().const_int(0, false),
        self.context.i32_type().const_int(i as u64, false),
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
//      No check if i < array.len()!
    ],
    &format!("array_elem_{}", i),
)?;
```

**Problem:**

- GEP (GetElementPtr) computes pointer without bounds check
- LLVM assumes pointer arithmetic is valid
- Out-of-bounds access is undefined behavior

**Exploitation Example:**

```vex
fn buffer_overflow() {
    let arr: [i32; 5] = [1, 2, 3, 4, 5];
    unsafe {
        let ptr = arr.as_ptr();
        let out_of_bounds = *(ptr.offset(10)); // UB! No bounds check
        println("{}", out_of_bounds); // Reads random memory
    }
}
```

**LLVM IR Generated:**

```llvm
%arr = alloca [5 x i32]
%ptr = getelementptr [5 x i32], ptr %arr, i32 0, i32 10
; BUFFER OVERFLOW (index 10 >= length 5)
%val = load i32, ptr %ptr
```

### Pattern 4: Alignment Violations

```rust
// vex-compiler/src/codegen_ast/memory_management.rs:93-108
pub(crate) fn build_store_aligned(
    &self,
    ptr: PointerValue<'ctx>,
    value: BasicValueEnum<'ctx>,
    _alignment: u32, // ⚠️ Parameter accepted but ignored!
) -> Result<(), String> {
    self.builder
        .build_store(ptr, value)
//      ^^^^^^^^^^^^ No alignment enforcement!
        .map_err(|e| format!("Failed to store value: {}", e))?;
    Ok(())
}
```

**Problem:**

- Function claims to handle alignment but doesn't use the parameter
- LLVM may generate unaligned loads/stores
- Causes crashes on ARM, slower on x86

**Alignment Requirements:**
| Type | Required Alignment | Current | Risk |
|------|-------------------|---------|------|
| i8 | 1 byte | ✅ OK | Low |
| i16 | 2 bytes | ⚠️ ? | Medium |
| i32 | 4 bytes | ⚠️ ? | Medium |
| i64 | 8 bytes | ⚠️ ? | **High** |
| f64 | 8 bytes | ⚠️ ? | **High** |
| ptr | 8 bytes (64-bit) | ⚠️ ? | **Critical** |

**Example Misaligned Access:**

```vex
struct Packed {
    a: u8,
    b: u64, // Misaligned if struct is packed!
}

fn misaligned() {
    let p: Packed = Packed { a: 1, b: 42 };
    let ptr = &p.b as *u64;
    unsafe {
        let value = *ptr; // May crash on ARM, slow on x86
    }
}
```

### Pattern 5: Use-After-Free in Scope Cleanup

```rust
// vex-compiler/src/codegen_ast/scope_management.rs:80-82
let vec_val = self.builder
    .build_load(vec_ptr_type, var_ptr, "vec_cleanup_load")
    .map_err(|e| format!("Failed to load Vec for cleanup: {}", e))?;
```

**Problem:**

- Loads value from `var_ptr` for cleanup
- No guarantee `var_ptr` is still valid
- If variable was moved earlier, this is use-after-free

**Scenario:**

```vex
fn use_after_move() {
    let v = Vec.new();
    let moved = v; // v is now invalid
    // Scope exit tries to cleanup v → use-after-free!
}
```

**Expected:** Borrow checker prevents this  
**Reality:** Edge cases may slip through

---

## Root Cause Analysis

### Why These Gaps Exist

1. **LLVM Trust Model** - Assumes input IR is safe, Vex compiler must guarantee this
2. **Missing LLVM Metadata** - Not emitting `!nonnull`, `!align`, `!dereferenceable` attributes
3. **No Runtime Checks** - Debug builds should have bounds/null checks, release can omit
4. **Unsafe Code Audit Incomplete** - Vex's `unsafe` blocks not thoroughly reviewed
5. **No Sanitizer Testing** - Not running with AddressSanitizer, MemorySanitizer, UBSan

### Comparison with Other Compilers

**rustc (Rust compiler):**

- Emits `!nonnull` for references
- `!align` for alignment guarantees
- `!dereferenceable(N)` for safe pointer range
- AddressSanitizer integration

**Swift compiler:**

- Runtime null checks in debug mode
- Bounds checking for arrays
- Alignment assertions

**Vex Current State:**

- No LLVM metadata emission
- No runtime safety checks
- No sanitizer integration

---

## Proposed Solutions

### Solution 1: LLVM Metadata Emission (High Impact, Low Risk)

**Add safety metadata to all pointer operations:**

```rust
// vex-compiler/src/codegen_ast/memory_management.rs
pub(crate) fn build_load_safe(
    &self,
    ty: BasicTypeEnum<'ctx>,
    ptr: PointerValue<'ctx>,
    name: &str,
) -> Result<BasicValueEnum<'ctx>, String> {
    // Check alignment
    let align = self.get_type_alignment(ty);

    // Emit load with metadata
    let load_inst = self.builder
        .build_load(ty, ptr, name)
        .map_err(|e| format!("Failed to load: {}", e))?;

    // Add alignment metadata
    if let Some(inst) = load_inst.as_instruction_value() {
        inst.set_alignment(align)?;
    }

    Ok(load_inst)
}
```

**Emit for all references:**

```llvm
; Before:
%1 = load i64, ptr %ptr

; After:
%1 = load i64, ptr %ptr, align 8, !nonnull !0, !dereferenceable !1
!0 = !{i64 0}
!1 = !{i64 8}
```

### Solution 2: Stack Allocation Limits (Medium Impact, Low Risk)

**Add configurable stack size limit:**

```rust
// vex-compiler/src/config.rs
pub struct CodegenConfig {
    /// Maximum bytes for single stack allocation (default 1MB)
    pub max_stack_alloc: usize,
}

// vex-compiler/src/codegen_ast/memory_management.rs
pub(crate) fn build_alloca_checked(
    &mut self,
    ty: BasicTypeEnum<'ctx>,
    name: &str,
) -> Result<PointerValue<'ctx>, String> {
    let size = self.get_type_size(ty);

    if size > self.config.max_stack_alloc {
        return Err(format!(
            "Stack allocation too large: {} bytes (limit: {})",
            size, self.config.max_stack_alloc
        ));
    }

    self.builder.build_alloca(ty, name)
        .map_err(|e| format!("Failed to allocate: {}", e))
}
```

**Emit compile-time error:**

```
error: stack allocation exceeds limit
  --> test.vx:2:9
   |
2  |     let huge: [i64; 10000000] = [0; 10000000];
   |         ^^^^ allocation size: 80,000,000 bytes (limit: 1,048,576)
   |
help: use heap allocation instead:
   |
2  |     let huge = vec![0; 10000000];
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^
```

### Solution 3: Runtime Null Checks (High Impact, Medium Risk)

**Add null checks in debug builds:**

```rust
// vex-compiler/src/codegen_ast/expressions/references.rs
pub(crate) fn compile_dereference_checked(
    &mut self,
    ptr: PointerValue<'ctx>,
    ty: BasicTypeEnum<'ctx>,
    name: &str,
) -> Result<BasicValueEnum<'ctx>, String> {
    if self.config.debug_mode {
        // Emit runtime null check
        self.emit_null_check(ptr, "null pointer dereference")?;
    }

    self.builder.build_load(ty, ptr, name)
        .map_err(|e| format!("Failed to dereference: {}", e))
}

fn emit_null_check(
    &mut self,
    ptr: PointerValue<'ctx>,
    error_msg: &str,
) -> Result<(), String> {
    let ptr_int = self.builder.build_ptr_to_int(
        ptr,
        self.context.i64_type(),
        "ptr_as_int"
    )?;

    let is_null = self.builder.build_int_compare(
        IntPredicate::EQ,
        ptr_int,
        self.context.i64_type().const_zero(),
        "is_null"
    )?;

    let null_block = self.context.append_basic_block(self.current_fn, "null_ptr");
    let safe_block = self.context.append_basic_block(self.current_fn, "ptr_valid");

    self.builder.build_conditional_branch(is_null, null_block, safe_block)?;

    // null_block: call panic with message
    self.builder.position_at_end(null_block);
    self.emit_panic(error_msg)?;
    self.builder.build_unreachable()?;

    // safe_block: continue execution
    self.builder.position_at_end(safe_block);
    Ok(())
}
```

### Solution 4: Bounds Checking for Arrays (High Impact, Medium Risk)

**Add runtime bounds checks:**

```rust
// vex-compiler/src/codegen_ast/expressions/access/indexing.rs
fn compile_array_index_checked(
    &mut self,
    array_ptr: PointerValue<'ctx>,
    index: IntValue<'ctx>,
    array_len: u64,
) -> Result<PointerValue<'ctx>, String> {
    if self.config.debug_mode || self.config.bounds_checks {
        // Emit: if (index >= len) panic("index out of bounds")
        let len_val = self.context.i64_type().const_int(array_len, false);
        let out_of_bounds = self.builder.build_int_compare(
            IntPredicate::UGE,
            index,
            len_val,
            "oob_check"
        )?;

        let oob_block = self.context.append_basic_block(self.current_fn, "index_oob");
        let safe_block = self.context.append_basic_block(self.current_fn, "index_ok");

        self.builder.build_conditional_branch(out_of_bounds, oob_block, safe_block)?;

        self.builder.position_at_end(oob_block);
        let msg = format!("index out of bounds: length {}", array_len);
        self.emit_panic(&msg)?;
        self.builder.build_unreachable()?;

        self.builder.position_at_end(safe_block);
    }

    // Normal GEP
    self.builder.build_gep(/* ... */)
}
```

### Solution 5: Sanitizer Integration (Medium Impact, Low Risk)

**Add CI jobs with sanitizers:**

```yaml
# .github/workflows/sanitizers.yml
name: Memory Sanitizers

on: [push, pull_request]

jobs:
  asan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Build with AddressSanitizer
        run: |
          export RUSTFLAGS="-Z sanitizer=address"
          cargo +nightly build --target x86_64-unknown-linux-gnu
      - name: Run tests
        run: cargo +nightly test

  msan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Build with MemorySanitizer
        run: |
          export RUSTFLAGS="-Z sanitizer=memory"
          cargo +nightly build --target x86_64-unknown-linux-gnu
      - name: Run tests
        run: cargo +nightly test

  ubsan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Build with UndefinedBehaviorSanitizer
        run: |
          export RUSTFLAGS="-Z sanitizer=undefined"
          cargo +nightly build
      - name: Run tests
        run: cargo +nightly test
```

---

## Implementation Plan

### Phase 1: Audit & Measurement (Week 1)

- [ ] Inventory all `build_alloca/load/store/gep` calls
- [ ] Run existing tests with AddressSanitizer
- [ ] Identify immediate crashes
- [ ] Create safety test suite

### Phase 2: Metadata Emission (Week 2)

- [ ] Add alignment metadata to all loads/stores
- [ ] Emit `!nonnull` for Vex references
- [ ] Add `!dereferenceable` for sized pointers
- [ ] Verify LLVM optimizations improve

### Phase 3: Stack Safety (Week 3)

- [ ] Implement max stack allocation check
- [ ] Add compile-time size validation
- [ ] Update error messages
- [ ] Test with large array programs

### Phase 4: Runtime Checks (Week 4-5)

- [ ] Add null pointer checks in debug mode
- [ ] Implement array bounds checking
- [ ] Make checks configurable (--safe-mode flag)
- [ ] Benchmark performance impact

### Phase 5: Sanitizer Integration (Week 6)

- [ ] Add ASAN/MSAN/UBSAN CI jobs
- [ ] Fix all detected issues
- [ ] Document sanitizer usage
- [ ] Add to release testing checklist

---

## Metrics for Success

**Before Fixes:**

- Unchecked pointer operations: 100+
- LLVM metadata coverage: 0%
- Sanitizer failures: Unknown
- Runtime safety checks: None

**After Fixes Target:**

- Checked pointer operations: 100%
- LLVM metadata coverage: 95%+
- Sanitizer clean runs: All tests pass
- Runtime checks: Debug mode (100%), Release mode (configurable)

---

## Performance Impact Analysis

### Expected Overhead (Debug Builds)

| Check Type      | Overhead              | Acceptable?           |
| --------------- | --------------------- | --------------------- |
| Null check      | ~2 instructions       | ✅ Yes (debug only)   |
| Bounds check    | ~5 instructions       | ✅ Yes (opt-out flag) |
| Alignment check | ~1 instruction        | ✅ Yes (compile-time) |
| LLVM metadata   | 0 (optimization hint) | ✅ Yes (always)       |

### Release Build Strategy

```toml
[profile.release]
# No runtime checks by default
debug-assertions = false

[profile.safe-release]
# Slower but safer release build
inherits = "release"
debug-assertions = true
overflow-checks = true
```

---

## Alternative Approaches Considered

### Approach A: Software Fault Isolation (SFI)

**Rejected:** Too heavyweight, significant runtime cost

### Approach B: Shadow Memory Tracking

**Rejected:** Complex implementation, memory overhead

### Approach C: Static Analysis Only

**Deferred:** Complement, not replace runtime checks

---

## Related Issues

- **KNOWN_CRASHES.md #1** - println() bus error (may be pointer corruption)
- **KNOWN_CRASHES.md #3** - Memory module crash (likely null pointer)
- **Critical Issue #4** - Bounds checking (directly related)

---

## References

- [LLVM Metadata](https://llvm.org/docs/LangRef.html#metadata)
- [AddressSanitizer](https://clang.llvm.org/docs/AddressSanitizer.html)
- [Rust Sanitizers](https://doc.rust-lang.org/beta/unstable-book/compiler-flags/sanitizer.html)
- [Alignment Requirements](https://en.cppreference.com/w/cpp/language/object#Alignment)

---

**Next Steps:** Begin Phase 1 audit with AddressSanitizer runs.
