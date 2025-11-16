# Known Crashes & Runtime Issues

This document tracks all known crashes and runtime issues in the Vex compiler and runtime.

**Last Updated:** 15 Kasım 2025

---

## 🟡 Medium Priority Issues

### 2. Extern "C" Functions Not Found During Compilation

**Status:** ACTIVE  
**Severity:** MEDIUM  
**Layer:** Layer 0 (Compiler - FFI)  
**Discovered:** 15 Kasım 2025

**Description:**
Extern "C" function declarations in Vex modules are not being found during compilation, even though they are properly declared and registered.

**Symptoms:**

- `extern "C" { fn getenv(...) }` declarations compile
- Function calls to these externs fail with "Function X not found"
- FFI bridge registers the functions but lookup fails
- All stdlib modules using extern C are blocked

**Affected Modules:**

- `env` - getenv, setenv, unsetenv
- `process` - system, getpid, getppid
- `time` - vt_now, vt_parse_duration, etc.
- `memory` - vex_malloc, vex_free, vex_memcpy
- `strconv` - vex_i64_to_str, vex_parse_i64, etc.

**Reproduction:**

```vex
extern "C" {
    fn getenv(name: *u8): *u8;
}

export fn get(name: str): str {
    let name_ptr = name as *u8;
    let value_ptr = getenv(name_ptr);  // ERROR: Function getenv not found
    return value_ptr as str;
}
```

**Error:**

```
🔵 compile_call: type_args.len()=0
❌ Failed to compile get: Function getenv not found
```

**Analysis:**

- `compile_extern_block` registers functions in `self.functions` hashmap
- Function lookup during call compilation fails
- Possible scope issue: extern functions registered but not visible in call context
- `declare_extern_function` adds to LLVM module but symbol table lookup fails

**Related Code:**

- `vex-compiler/src/codegen_ast/ffi.rs:9` - `compile_extern_block`
- `vex-compiler/src/codegen_ast/ffi.rs:82` - `self.functions.insert()`
- `vex-compiler/src/codegen_ast/expressions/calls/function_calls.rs` - Call compilation

**Next Steps:**

1. Debug function registry lookup during call compilation
2. Check if `self.functions` is being queried correctly
3. Verify extern functions are in scope when compiling function bodies
4. Compare with working builtin function registration
5. Check if module-level vs global scope affects visibility

**Workaround:**
None - blocks all extern C usage.

### 3. Memory Module Import Crash

**Status:** ACTIVE (Related to Issue #2)  
**Severity:** MEDIUM  
**Layer:** Layer 2 (Stdlib)  
**Discovered:** 15 Kasım 2025

**Description:**
Importing and using memory module functions causes program termination by signal.

**Note:** Likely related to Issue #2 (extern C functions not found). Once extern C is fixed, this may resolve automatically.

**Symptoms:**

- Compilation succeeds
- Linking succeeds
- Program crashes before any output
- Error: "Program terminated by signal"

**Reproduction:**

```vex
import { alloc, free } from "memory";

fn main(): i32 {
    println("Testing simple allocation...");
    let ptr = alloc(64);
    println("Allocated");
    if ptr == 0 as *u8 {
        println("Failed");
        return 1;
    }
    println("Success!");
    return 0;
}
```

**Output:**

```
✅ LLVM module verification passed
  🔗 Linking with args: '-L/Users/mapletechnologies/.cargo/target/debug/build/vex-runtime-12d7ef3a177ae4ca/out -lvex_runtime -lpthread'
Error: Program terminated by signal
```

**Analysis:**

- Issue occurs even with minimal memory module usage
- May be related to Issue #1 (println crash)
- Could be extern "C" function linking issue
- Possible unsafe block handling problem

**Related Code:**

- `vex-libs/std/memory/src/lib.vx`
- `vex-runtime/c/vex_alloc.c`
- `vex-runtime/c/vex_memory.c`

**Next Steps:**

1. Test memory functions without println
2. Verify extern "C" function declarations match C signatures
3. Check if vex_malloc/vex_free are properly linked
4. Review unsafe block compilation

---

## 🟢 Low Priority / Under Investigation

### 3. Test Suite Failures (8/421 tests)

**Status:** MONITORING  
**Severity:** LOW  
**Layer:** Layer 0/1  
**Success Rate:** 98.1% (413/421 passing)

**Failing Tests:**

1. `05_generics/nested_depth10` - Deep generic nesting
2. `05_generics/nested_extreme` - Extreme generic nesting
3. `test_generics_comprehensive` - Complex generic scenarios
4. `test_op_comparison` - Comparison operators for custom types
5. `test_operator_custom_struct` - Operator overloading on structs
6. `test_trait_based_cleanup` - Trait implementation cleanup
7. `test_stdlib_check` - Stdlib integration check
8. ~~`11_advanced/raw_ptr`~~ - **FIXED** ✅ (Array to pointer casting)

**Note:** These are likely edge cases or features not yet fully implemented.

---

## ✅ Fixed Issues

### Extern "C" Functions Not Found During Compilation (FIXED)

**Status:** RESOLVED  
**Fixed:** 15 Kasım 2025

**Issue:**
Extern "C" function declarations in imported Vex modules were not being found during compilation.

**Root Cause:**
Two separate bugs:

1. `declare_extern_function` had an early return path that skipped HashMap registration when a function already existed in LLVM module
2. `resolve_and_merge_imports` didn't merge `ExternBlock` items from imported modules

**Solution:**

1. Added `self.functions.insert()` before early return in `ffi.rs:48`
2. Added `Item::ExternBlock(_)` case in import merging logic in `program.rs:82`

**Changes:**

- `vex-compiler/src/codegen_ast/ffi.rs:48` - Register in HashMap even on early return
- `vex-compiler/src/codegen_ast/program.rs:82` - Include extern blocks in import merge

**Impact:**

- ✅ Unblocked 5 stdlib modules: env, process, time, memory, strconv
- ✅ Test success rate improved: 98.1% → 98.3% (413→414 passing tests)

### raw_ptr Test - Array to Pointer Casting (FIXED)

**Status:** RESOLVED  
**Fixed:** 15 Kasım 2025

**Issue:**
Array to raw pointer casting required explicit `as *T` cast and caused double allocation.

**Solution:**

- Implemented automatic array-to-pointer casting
- Added array pointer cache (`last_compiled_array_ptr`)
- Type-safe casting for `Reference(Array)`, `Array`, and `Vec` types
- Zero-cost abstraction (no double alloca)

**Changes:**

- `vex-compiler/src/codegen_ast/expressions/special/casts.rs`
- `vex-compiler/src/codegen_ast/expressions/collections.rs`
- `vex-compiler/src/codegen_ast/statements/let_statement/variable_registration.rs`

---

## 📊 Statistics

- **Total Known Issues:** 1 active
- **Critical Issues:** 1 (println crash)
- **Medium Priority:** 0
- **Low Priority:** 1 (test failures)
- **Resolved Issues:** 2
- **Overall Test Success Rate:** 98.3%
- **Blocked Stdlib Modules:** 0

**Additional Architectural Issues:** 5 categories documented in `critique/` directory

- Critical Issue #1: Excessive unwrap/panic risks (100+ instances)
- Critical Issue #3: LLVM pointer safety (100+ unchecked operations)
- Critical Issue #4: Bounds/overflow protection (30+ vulnerable sites)
- Critical Issue #2: Performance (clone overhead - deferred)
- Critical Issue #5: Concurrency safety (async runtime - deferred)

---

## 🔧 Debug Commands

### Check for crashes:

```bash
# Compile and run with lldb
~/.cargo/target/debug/vex compile test.vx -o /tmp/test
lldb /tmp/test -o run -o bt

# Run all tests
./test_all.sh | grep "❌"

# Check specific test
~/.cargo/target/debug/vex run examples/test_name.vx 2>&1 | tail -20
```

### Memory debugging:

```bash
# Valgrind (Linux)
valgrind --leak-check=full /tmp/test_binary

# macOS leaks
leaks --atExit -- /tmp/test_binary

# Address Sanitizer
ASAN_OPTIONS=detect_leaks=1 /tmp/test_binary
```

---

## 📝 Notes

- All Layer 0/1 issues should be fixed immediately when discovered
- Layer 2 (stdlib) development continues while tracking Layer 0/1 issues
- Crashes are documented here for systematic resolution
- Update this file when new issues are discovered or resolved
