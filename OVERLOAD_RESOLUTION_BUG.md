# Overload Resolution Bug - Math Library Internal Calls

**Status:** 🔴 CRITICAL - Blocking 36 tests  
**Date:** 22 Kasım 2025  
**Priority:** P0 - Breaks math library and all dependent code

## Problem Summary

Function overload resolution fails for **library-internal function calls** where arguments are function parameters, not compiled LLVM values. This causes wrong overloads to be selected, resulting in LLVM IR type mismatches.

### Error Signature
```
Error: Compilation error: Invalid LLVM IR generated: 
"DestTy too big for Trunc\n  %trunc_to_expected = trunc i32 %div to i64"
```

This means: Trying to truncate i32 → i64 (impossible, truncation only reduces size)

## Root Cause

**Location:** `vex-compiler/src/codegen_ast/expressions/calls/function_calls.rs`

When compiling math library functions like `gcd(i64, i64)`:
```vex
fn gcd(a: i64, b: i64) -> i64 {
    let x = abs(a);  // ❌ Calls abs(i32) instead of abs(i64)!
    // ...
}
```

**Why it fails:**

1. **Argument compilation order:** When processing `abs(a)`, the function tries to determine argument types to select correct overload
2. **Empty arg_basic_vals:** At this point, `a` hasn't been compiled to LLVM yet - it's still an AST Expression::Ident
3. **Fallback to wrong path:** Code falls through to "complex function expression" path, treating `abs` as a closure/function pointer
4. **Type inference from call site:** Infers return type as i32 (default), ignoring parameter's actual i64 type
5. **Wrong overload selected:** Calls `abs_i32_1` instead of `abs_i64_1`
6. **LLVM error:** Tries to cast i32 result → i64 via truncation (invalid)

## Code Flow

```
compile_call("abs", [Ident("a")])
  ↓
arg_basic_vals is empty (args not compiled yet)
  ↓
Skips overload resolution (line ~597: "else if !arg_basic_vals.is_empty()")
  ↓
Falls to line 1083: "Complex function expression" path
  ↓
compile_expression(Ident("abs")) → returns function pointer
  ↓
Infers type from call arguments: "i32 (i32)" (default)
  ↓
Calls wrong overload: abs_i32_1 instead of abs_i64_1
  ↓
Return type mismatch: i32 vs expected i64
  ↓
LLVM error: Cannot truncate i32 to i64
```

## Affected Tests (36 failed)

### Math/Namespace Tests
- `test_namespace_abs_only` ❌
- `test_namespace_function` ❌
- `test_math_constants` ❌
- `test_math_const_assign` ❌
- `repro_abs` ❌
- `test_abs_only` ❌

### Overload Tests  
- `test_multi_overload` ❌
- `test_multi_overload2` ❌
- `test_generic_overload` ❌

### Import Tests (depend on math lib)
- `test_import_complete` ❌
- `test_import_basic` ❌
- `test_import_export_complete` ❌
- `test_import_features` ❌
- `test_simple_import` ❌
- `test_re_export` ❌

### Casting/Coercion (depend on correct overloads)
- `test_all_coercion_cases` ❌
- `test_binary_coercion` ❌
- `test_casting_complete` ❌
- `test_casting_gaps` ❌
- `test_signed_unsigned` ❌
- `test_return_downcast` ❌

### Other Affected
- `test_constants` ❌
- `test_generics_comprehensive` ❌
- `test_string_constructor` ❌
- `test_special_primitives` ❌
- `test_index_write` ❌
- `test_index_operator` ❌
- `test_match_if_expr` ❌
- `test_op_eq` ❌
- `test_op_ord` ❌
- `test_mut_method_minimal` ❌
- `test_parse_static` ❌
- `test_parser_static` ❌
- `test_simple_static` ❌
- `05_generics/nested_extreme` ❌
- `10_builtins/string_simple_test` ❌

**Total:** 36/500 tests failing (92.8% pass rate)

## Expected Behavior

```vex
fn gcd(a: i64, b: i64) -> i64 {
    let x = abs(a);  // ✅ Should call abs_i64_1
}
```

Should generate:
```llvm
%call = call i64 @abs_i64_1(i64 %a)  ; ✅ Correct
```

Currently generates:
```llvm
%call = call i32 @abs_i32_1(i64 %a)  ; ❌ Wrong overload
%trunc_to_expected = trunc i32 %call to i64  ; ❌ Invalid cast
```

## Solution Requirements

**Must fix argument type inference for uncompiled AST expressions:**

1. **Check `variable_ast_types` map FIRST** - contains function parameters with their declared types
2. **If Expression::Ident(name):** Look up `variable_ast_types.get(name)` to get actual parameter type
3. **Use AST type for overload resolution** - before falling back to LLVM type inference
4. **Priority order:**
   - `variable_ast_types` (AST types from declarations)
   - `variable_types` (LLVM types from compilation)  
   - Type inference from expression structure
   - Default types (last resort)

## Fix Location

**File:** `vex-compiler/src/codegen_ast/expressions/calls/function_calls.rs`  
**Lines:** ~545-740 (overload resolution section)

**Current broken logic:**
```rust
let fn_val_opt = if self.variables.contains_key(func_name) {
    None  // ❌ Treats overloaded functions as variables!
} else if !arg_basic_vals.is_empty() {
    // Overload resolution - BUT SKIPPED when args not compiled yet!
}
```

**Required fix:**
```rust
// Check for overloaded functions BEFORE checking variables
let has_overloads = /* count function_defs with matching prefix */;

if has_overloads {
    // Infer types from AST expressions (final_args), not LLVM values
    for (i, arg_expr) in final_args.iter().enumerate() {
        let ast_type = if let Expression::Ident(var_name) = arg_expr {
            self.variable_ast_types.get(var_name)  // ✅ Get parameter type
                .or_else(|| /* fallback to variable_types */)
        } else {
            self.infer_expression_type(arg_expr)
        };
        // Use ast_type for mangling
    }
}
```

## Related Files

- `vex-compiler/src/codegen_ast/expressions/calls/method_calls.rs` - Namespace calls (partially working)
- `vex-compiler/src/codegen_ast/functions/compile.rs:354` - Where `variable_ast_types` is populated
- `vex-libs/std/math/src/lib.vx` - Math library with overloaded functions

## Temporary Workarounds

**None available.** Cannot use math library functions that call other overloaded functions internally.

## Test to Verify Fix

```bash
~/.cargo/target/debug/vex run examples/test_namespace_abs_only.vx
# Should output: 42 (not LLVM error)

./test_all.sh | grep "❌"
# Should show 0 failed tests (or <36 if partial fix)
```

## Previous Attempts

1. ✅ Added namespace detection in `method_calls.rs` - Works for `math.abs(x)` from user code
2. ✅ Added `variable_ast_types.get()` priority in argument compilation - Works for let variables  
3. ❌ Tried checking `is_function_or_extern` before variables - Incomplete, still falls through
4. ❌ Attempted to infer types from compiled args - Doesn't work when args not compiled yet

**Next step:** Implement AST-based type inference for overload resolution that works BEFORE argument compilation.
