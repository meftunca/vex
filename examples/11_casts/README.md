# Type Casting Tests

Type casting with `as` operator implementation tests.

## ✅ Passing Tests

| Test File               | Description              | Exit Code | Status                  |
| ----------------------- | ------------------------ | --------- | ----------------------- |
| `cast_simple.vx`        | Basic i32→i64→i32 chain  | 42        | ✅                      |
| `debug_cast.vx`         | Function return cast     | 42        | ✅                      |
| `test_cast_infer.vx`    | Cast with type inference | 42        | ✅                      |
| `test_basic.vx`         | Basic variable test      | 42        | ✅                      |
| `cast_comprehensive.vx` | Multiple cast operations | 110       | ✅                      |
| `cast_edge_cases.vx`    | Edge case scenarios      | 101       | ⚠️ (negative num issue) |

## ⚠️ Known Issues

| Test File          | Issue                     | Exit Code | Notes                                         |
| ------------------ | ------------------------- | --------- | --------------------------------------------- |
| `cast_numeric.vx`  | i64 literal store problem | 202       | Codegen stores i64 literals as i32            |
| `test_negative.vx` | Negative number handling  | 214       | Unary minus parsing issue (not cast-specific) |

## 🎯 Supported Cast Operations

### Integer Casts

- ✅ **Widening**: `i32 → i64` (sign extension, sext)
- ✅ **Narrowing**: `i64 → i32` (truncation, trunc)
- ✅ **Signed ↔ Unsigned**: `i32 ↔ u32` (bitcast)
- ✅ **Width Change**: `u32 → u64`, `i8 → i32` (sext/zext/trunc)

### Float Casts

- ✅ **Widening**: `f32 → f64` (extension, fext)
- ✅ **Narrowing**: `f64 → f32` (truncation, ftrunc)

### Mixed Numeric Casts

- ✅ **Signed Int → Float**: `i32 → f64` (sitofp)
- ✅ **Unsigned Int → Float**: `u32 → f64` (TODO: use uitofp)
- ✅ **Float → Signed Int**: `f64 → i32` (fptosi, truncates decimal)
- ✅ **Float → Unsigned Int**: `f64 → u32` (fptoui, truncates decimal)

### Pointer Casts

- ✅ **Pointer → Pointer**: `*T → *U` (unsafe, for FFI)

## 📝 Usage Examples

```vex
// Integer widening (safe)
let x: i32 = 42;
let y: i64 = x as i64;  // Sign extension: 42 → 42

// Integer narrowing (lossy)
let a: i64 = 1000;
let b: i32 = a as i32;  // Truncation: 1000 → 1000 (fits)

// Float conversion
let c: i32 = 100;
let d: f64 = c as f64;  // 100 → 100.0

// Float truncation
let e: f64 = 42.7;
let f: i32 = e as i32;  // 42.7 → 42 (decimal truncated)

// Cast chains
let result = ((42 as i64) as f64) as i32;  // 42
```

## 🔬 Implementation Details

**Parser**: `parse_cast()` in `vex-parser/src/parser/expressions.rs`

- Precedence: Between multiplicative and unary
- Syntax: `expr as TargetType`

**Codegen**: `compile_cast_expression()` in `vex-compiler/src/codegen_ast/expressions/special.rs`

- LLVM instructions: sext, trunc, fext, ftrunc, sitofp, fptosi, pointer_cast
- 139 lines of cast logic

## 🚧 Future Work

- [ ] Fix i64 literal storage in Let statements
- [ ] Add unsafe blocks for pointer casts
- [ ] Implement cast warnings for lossy conversions
- [ ] Handle NaN, infinity, overflow edge cases
- [ ] Add `as!` for checked casts (panic on overflow)
