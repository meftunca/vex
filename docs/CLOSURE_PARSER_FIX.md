# Closure Parser Fix - November 2025

## Problem

Closure parsing was failing with error: `Expected '|' after closure parameters at position 211`

Example that failed:

```vex
let double = |x: i32| x * 2;
```

## Root Cause

In `vex-parser/src/parser/types.rs`, the `parse_type()` function was treating `|` (pipe) as a **union type operator** (for types like `i32 | i64`).

When parsing closure parameter type `|x: i32|`:

1. Parser consumed opening `|`
2. Parsed identifier `x`
3. Saw `:` colon
4. Called `parse_type()` to parse `i32`
5. **BUG**: `parse_type()` parsed `i32`, then saw `|` and thought it was a union type
6. Continued parsing, creating `Union([I32, Named("x")])`
7. Consumed the closing `|` as part of the union type
8. Next check for closing `|` failed because it was already consumed

Debug output showed:

```
🟣 Parsed type: Union([I32, Named("x")]), next token: Star
🟣 Expecting closing |, current token: Star
```

## Solution

Use `parse_type_primary()` instead of `parse_type()` in closure parameter parsing.

**File**: `vex-parser/src/parser/expressions.rs` (line ~738)

**Before**:

```rust
let param_type = if self.match_token(&Token::Colon) {
    self.parse_type()?  // BUG: Treats | as union operator
} else {
    Type::Inferred
}
```

**After**:

```rust
let param_type = if self.match_token(&Token::Colon) {
    // Use parse_type_primary to avoid parsing | as union type operator
    self.parse_type_primary()?
} else {
    Type::Inferred
}
```

## Difference Between Functions

- **`parse_type()`**: Parses full type expressions including unions (`T1 | T2`), intersections (`T1 & T2`), and conditionals
- **`parse_type_primary()`**: Parses only primary types (primitives, named, arrays, references) WITHOUT union/intersection operators

The `parse_type_primary()` function is specifically designed for contexts where `|` and `&` should NOT be treated as type operators.

## Related Fix

Also fixed function type syntax in same file (line 26):

- **Before**: `Token::Arrow` (Rust syntax `fn(i32) -> i32`)
- **After**: `Token::Colon` (Vex syntax `fn(i32): i32`)

## Test Results

✅ `examples/02_functions/closure_simple.vx` - Parses successfully (codegen TODO)
✅ `examples/02_functions/higher_order.vx` - **WORKS PERFECTLY** (returns 35)

## Current Status

**Parser**: ✅ Fixed - Closures parse correctly with type annotations
**Codegen**: ❌ Not implemented - `compile_closure()` needs implementation (~5 days)
**Borrow Checker**: ⚠️ Has bug with closure parameter scoping (false positive "out of scope" error)

## Next Steps

1. Fix borrow checker closure scoping bug
2. Implement `compile_closure()` in `vex-compiler/src/codegen_ast/expressions/special.rs`
3. Add environment capture mechanism
4. Implement closure traits (`Fn`, `FnMut`, `FnOnce`)
5. Add closure type inference

## Files Modified

- `vex-parser/src/parser/expressions.rs` - Line 738: Use `parse_type_primary()`
- `vex-parser/src/parser/types.rs` - Line 26: Change `Arrow` to `Colon` for function types
- `examples/02_functions/higher_order.vx` - Fixed function type syntax

## Lessons Learned

- Always consider context when parsing operators that have multiple meanings
- The `|` operator means:
  - Closure delimiter: `|x| { body }`
  - Union type operator: `i32 | i64`
  - Bitwise OR: `a | b`
- Use `parse_type_primary()` in any context where union/intersection types should not be parsed
- The parser already had the right function, just needed to use it!

---

**Date**: November 2025  
**Impact**: Unblocks closure implementation (TODO #1 critical feature)
