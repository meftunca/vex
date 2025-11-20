# Comprehensive Type System & Casting Plan - Rust Parity

**Goal:** Rust-level type safety with ergonomic auto-casting for improved DX  
**Timeline:** 8-12 weeks  
**Status:** DRAFT - Ready for implementation

---

## Executive Summary

**Problem:** Vex's type system lacks:

1. Automatic type coercion for safe operations (i8+i64 requires manual cast)
2. Runtime overflow detection (wrapping arithmetic without checks)
3. Mixed-type binary operations (i8+i64 errors or behaves unpredictably)
4. Consistent casting semantics across contexts (let, function call, binary op)

**Solution:** Implement Rust-parity type system with optional ergonomic extensions:

- **Tier 1 (Rust-strict):** No implicit conversions, all casts explicit
- **Tier 2 (Ergonomic mode):** Auto-upcasting for safe operations, strict downcasting

---

## Phase 1: Type Coercion Infrastructure (Week 1-2)

### 1.1 Define Coercion Rules (Week 1)

**File:** `vex-compiler/src/type_system/coercion_rules.rs` (NEW)

```rust
/// Type coercion rules for Vex
///
/// Rust Mode (--strict-types):
///   - No implicit conversions
///   - All casts require `as` keyword
///   - Mismatched types = compile error
///
/// Ergonomic Mode (default):
///   - Auto-upcast: i8 -> i16 -> i32 -> i64 -> i128
///   - Auto-upcast: u8 -> u16 -> u32 -> u64 -> u128
///   - Auto-upcast: f32 -> f64
///   - Cross-signedness: NEVER (i32 ↔ u32 always explicit)
///   - Downcast: NEVER implicit (i64 -> i32 requires `as`)

pub enum CoercionKind {
    /// No conversion needed
    NoOp,

    /// Safe widening (i8 -> i32, f32 -> f64)
    /// Always allowed in ergonomic mode, never in strict mode
    Upcast {
        from: Type,
        to: Type,
    },

    /// Narrowing (i64 -> i32, f64 -> f32)
    /// NEVER implicit, always requires `as`
    Downcast {
        from: Type,
        to: Type,
    },

    /// Signedness change (i32 -> u32, u64 -> i64)
    /// NEVER implicit, always requires `as`
    SignChange {
        from: Type,
        to: Type,
    },

    /// Incompatible types (String -> i32)
    /// Always error
    Incompatible {
        from: Type,
        to: Type,
    },
}

impl CoercionRules {
    /// Check if coercion is safe (no data loss possible)
    pub fn is_safe_coercion(from: &Type, to: &Type) -> bool {
        matches!(
            Self::classify_coercion(from, to),
            CoercionKind::NoOp | CoercionKind::Upcast { .. }
        )
    }

    /// Classify the coercion needed
    pub fn classify_coercion(from: &Type, to: &Type) -> CoercionKind {
        match (from, to) {
            // Same type
            (a, b) if a == b => CoercionKind::NoOp,

            // Integer upcasts (signed)
            (Type::I8, Type::I16 | Type::I32 | Type::I64 | Type::I128) => CoercionKind::Upcast { from: from.clone(), to: to.clone() },
            (Type::I16, Type::I32 | Type::I64 | Type::I128) => CoercionKind::Upcast { from: from.clone(), to: to.clone() },
            (Type::I32, Type::I64 | Type::I128) => CoercionKind::Upcast { from: from.clone(), to: to.clone() },
            (Type::I64, Type::I128) => CoercionKind::Upcast { from: from.clone(), to: to.clone() },

            // Integer upcasts (unsigned)
            (Type::U8, Type::U16 | Type::U32 | Type::U64 | Type::U128) => CoercionKind::Upcast { from: from.clone(), to: to.clone() },
            (Type::U16, Type::U32 | Type::U64 | Type::U128) => CoercionKind::Upcast { from: from.clone(), to: to.clone() },
            (Type::U32, Type::U64 | Type::U128) => CoercionKind::Upcast { from: from.clone(), to: to.clone() },
            (Type::U64, Type::U128) => CoercionKind::Upcast { from: from.clone(), to: to.clone() },

            // Float upcasts
            (Type::F32, Type::F64) => CoercionKind::Upcast { from: from.clone(), to: to.clone() },

            // Integer downcasts
            (Type::I16 | Type::I32 | Type::I64 | Type::I128, Type::I8) => CoercionKind::Downcast { from: from.clone(), to: to.clone() },
            (Type::I32 | Type::I64 | Type::I128, Type::I16) => CoercionKind::Downcast { from: from.clone(), to: to.clone() },
            (Type::I64 | Type::I128, Type::I32) => CoercionKind::Downcast { from: from.clone(), to: to.clone() },
            (Type::I128, Type::I64) => CoercionKind::Downcast { from: from.clone(), to: to.clone() },

            // Signedness changes
            (Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::I128,
             Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::U128) => {
                CoercionKind::SignChange { from: from.clone(), to: to.clone() }
            }
            (Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::U128,
             Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::I128) => {
                CoercionKind::SignChange { from: from.clone(), to: to.clone() }
            }

            // Everything else is incompatible
            _ => CoercionKind::Incompatible { from: from.clone(), to: to.clone() },
        }
    }

    /// Get the "wider" type for binary operations
    pub fn wider_type(left: &Type, right: &Type) -> Option<Type> {
        match (left, right) {
            // Same type
            (a, b) if a == b => Some(a.clone()),

            // Mixed signedness -> Error (require explicit cast)
            (Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::I128,
             Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::U128) => None,
            (Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::U128,
             Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::I128) => None,

            // Signed integers - pick wider
            (Type::I8, Type::I16 | Type::I32 | Type::I64 | Type::I128) => Some(right.clone()),
            (Type::I16 | Type::I32 | Type::I64 | Type::I128, Type::I8) => Some(left.clone()),
            (Type::I16, Type::I32 | Type::I64 | Type::I128) => Some(right.clone()),
            (Type::I32 | Type::I64 | Type::I128, Type::I16) => Some(left.clone()),
            (Type::I32, Type::I64 | Type::I128) => Some(right.clone()),
            (Type::I64 | Type::I128, Type::I32) => Some(left.clone()),
            (Type::I64, Type::I128) => Some(right.clone()),
            (Type::I128, Type::I64) => Some(left.clone()),

            // Unsigned integers - pick wider
            (Type::U8, Type::U16 | Type::U32 | Type::U64 | Type::U128) => Some(right.clone()),
            (Type::U16 | Type::U32 | Type::U64 | Type::U128, Type::U8) => Some(left.clone()),
            (Type::U16, Type::U32 | Type::U64 | Type::U128) => Some(right.clone()),
            (Type::U32 | Type::U64 | Type::U128, Type::U16) => Some(left.clone()),
            (Type::U32, Type::U64 | Type::U128) => Some(right.clone()),
            (Type::U64 | Type::U128, Type::U32) => Some(left.clone()),
            (Type::U64, Type::U128) => Some(right.clone()),
            (Type::U128, Type::U64) => Some(left.clone()),

            // Floats
            (Type::F32, Type::F64) => Some(Type::F64),
            (Type::F64, Type::F32) => Some(Type::F64),

            _ => None,
        }
    }
}
```

**Tests:**

```rust
#[test]
fn test_safe_coercions() {
    assert!(CoercionRules::is_safe_coercion(&Type::I8, &Type::I32));
    assert!(CoercionRules::is_safe_coercion(&Type::F32, &Type::F64));
    assert!(!CoercionRules::is_safe_coercion(&Type::I64, &Type::I32)); // downcast
    assert!(!CoercionRules::is_safe_coercion(&Type::I32, &Type::U32)); // sign change
}

#[test]
fn test_wider_type() {
    assert_eq!(CoercionRules::wider_type(&Type::I8, &Type::I64), Some(Type::I64));
    assert_eq!(CoercionRules::wider_type(&Type::I32, &Type::U32), None); // mixed sign
}
```

### 1.2 Compiler Configuration (Week 1)

**File:** `vex-compiler/src/config.rs`

```rust
pub struct TypeSystemConfig {
    /// Strict mode (Rust-like): No implicit conversions
    /// Ergonomic mode: Auto-upcast allowed
    pub strict_types: bool,

    /// Overflow checking in debug builds
    pub overflow_checks_debug: bool,

    /// Overflow checking in release builds
    pub overflow_checks_release: bool,
}

impl Default for TypeSystemConfig {
    fn default() -> Self {
        Self {
            strict_types: false,  // Ergonomic mode by default
            overflow_checks_debug: true,
            overflow_checks_release: false,
        }
    }
}
```

**CLI Integration:**

```bash
vex build --strict-types         # Rust-strict mode
vex build --overflow-checks      # Force overflow checks in release
vex build --no-overflow-checks   # Disable in debug (not recommended)
```

---

## Phase 2: Binary Operation Type Inference (Week 2-3)

### 2.1 Enhanced Type Inference for Binary Ops

**File:** `vex-compiler/src/codegen_ast/expressions/operators.rs`

**Current Issue:**

```vex
let a: i8 = 10;
let b: i64 = 100;
let c = a + b;  // ❌ Error or unpredictable behavior
```

**Solution:**

```rust
pub(crate) fn compile_binary_op_with_coercion(
    &mut self,
    left: &vex_ast::Expression,
    op: &vex_ast::BinaryOp,
    right: &vex_ast::Expression,
    expected_type: Option<&vex_ast::Type>,
) -> Result<BasicValueEnum<'ctx>, String> {
    // Step 1: Infer types of both operands
    let left_type = self.infer_expression_type(left)?;
    let right_type = self.infer_expression_type(right)?;

    // Step 2: Determine target type
    let target_type = if let Some(expected) = expected_type {
        // Use expected type if provided (e.g., `let x: i64 = a + b`)
        expected.clone()
    } else {
        // Otherwise, find wider type for binary op
        CoercionRules::wider_type(&left_type, &right_type)
            .ok_or_else(|| {
                format!(
                    "Cannot perform {} between incompatible types: {:?} and {:?}. \
                     Hint: Use explicit cast (e.g., `{} as {:?}`)",
                    op, left_type, right_type,
                    "operand", right_type
                )
            })?
    };

    // Step 3: Compile operands with target type
    let left_val = self.compile_expression_with_type(left, Some(&target_type))?;
    let right_val = self.compile_expression_with_type(right, Some(&target_type))?;

    // Step 4: Apply coercions if needed (in ergonomic mode)
    let left_val = self.apply_coercion_if_allowed(
        left_val,
        &left_type,
        &target_type,
        "left operand"
    )?;
    let right_val = self.apply_coercion_if_allowed(
        right_val,
        &right_type,
        &target_type,
        "right operand"
    )?;

    // Step 5: Perform operation
    match (&left_val, &right_val) {
        (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
            self.compile_integer_binary_op_with_overflow_check(l, r, op, &target_type)
        }
        (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
            self.compile_float_binary_op(l, r, op)
        }
        _ => Err(format!("Type mismatch in binary operation")),
    }
}

fn apply_coercion_if_allowed(
    &mut self,
    val: BasicValueEnum<'ctx>,
    from_type: &Type,
    to_type: &Type,
    context: &str,
) -> Result<BasicValueEnum<'ctx>, String> {
    let coercion = CoercionRules::classify_coercion(from_type, to_type);

    match coercion {
        CoercionKind::NoOp => Ok(val),

        CoercionKind::Upcast { .. } => {
            if self.config.type_system.strict_types {
                // Strict mode: reject implicit upcast
                return Err(format!(
                    "Implicit cast not allowed in strict mode: {:?} -> {:?} for {}. \
                     Use explicit cast: `expr as {:?}`",
                    from_type, to_type, context, to_type
                ));
            }

            // Ergonomic mode: perform upcast
            eprintln!("🔄 Auto-upcasting {} from {:?} to {:?}", context, from_type, to_type);
            self.build_upcast(val, from_type, to_type)
        }

        CoercionKind::Downcast { .. } | CoercionKind::SignChange { .. } => {
            Err(format!(
                "Unsafe cast {:?} -> {:?} for {}. Use explicit cast: `expr as {:?}`",
                from_type, to_type, context, to_type
            ))
        }

        CoercionKind::Incompatible { .. } => {
            Err(format!(
                "Incompatible types: {:?} and {:?} for {}",
                from_type, to_type, context
            ))
        }
    }
}
```

### 2.2 Update Literal Compilation

**File:** `vex-compiler/src/codegen_ast/expressions/literals_expressions.rs`

```rust
pub(crate) fn compile_literal_with_type(
    &mut self,
    expr: &vex_ast::Expression,
    expected_type: Option<&vex_ast::Type>,
) -> Result<BasicValueEnum<'ctx>, String> {
    match expr {
        vex_ast::Expression::IntLiteral(n) => {
            // Target-typed: infer from expected type if available
            let target_type = expected_type.unwrap_or(&Type::I32);

            // Validate literal fits in target type
            self.validate_literal_range(*n, target_type)?;

            match target_type {
                Type::I8 => Ok(self.context.i8_type().const_int(*n as u64, true).into()),
                Type::I16 => Ok(self.context.i16_type().const_int(*n as u64, true).into()),
                Type::I32 => Ok(self.context.i32_type().const_int(*n as u64, true).into()),
                Type::I64 => Ok(self.context.i64_type().const_int(*n as u64, true).into()),
                Type::I128 => Ok(self.context.i128_type().const_int(*n as u64, true).into()),
                Type::U8 => Ok(self.context.i8_type().const_int(*n as u64, false).into()),
                Type::U16 => Ok(self.context.i16_type().const_int(*n as u64, false).into()),
                Type::U32 => Ok(self.context.i32_type().const_int(*n as u64, false).into()),
                Type::U64 => Ok(self.context.i64_type().const_int(*n as u64, false).into()),
                Type::U128 => Ok(self.context.i128_type().const_int(*n as u64, false).into()),
                _ => Err(format!("Cannot infer integer literal to type {:?}", target_type)),
            }
        }
        // ... other literals
    }
}

fn validate_literal_range(&self, value: i64, target_type: &Type) -> Result<(), String> {
    let (min, max) = match target_type {
        Type::I8 => (i8::MIN as i64, i8::MAX as i64),
        Type::I16 => (i16::MIN as i64, i16::MAX as i64),
        Type::I32 => (i32::MIN as i64, i32::MAX as i64),
        Type::I64 => (i64::MIN, i64::MAX),
        Type::U8 => (0, u8::MAX as i64),
        Type::U16 => (0, u16::MAX as i64),
        Type::U32 => (0, u32::MAX as i64),
        Type::U64 => (0, i64::MAX), // Can't represent full u64 in i64
        _ => return Ok(()), // Skip validation for other types
    };

    if value < min || value > max {
        return Err(format!(
            "Integer literal {} out of range for type {:?} (range: {}..={})",
            value, target_type, min, max
        ));
    }

    Ok(())
}
```

---

## Phase 3: Runtime Overflow Detection (Week 3-4)

### 3.1 Overflow Intrinsics Integration

**File:** `vex-compiler/src/codegen_ast/expressions/binary_ops/integer_ops.rs`

```rust
fn compile_integer_binary_op_with_overflow_check(
    &mut self,
    l: IntValue<'ctx>,
    r: IntValue<'ctx>,
    op: &BinaryOp,
    target_type: &Type,
) -> Result<BasicValueEnum<'ctx>, String> {
    // Const context: use existing compile-time checks
    if l.is_const() && r.is_const() {
        return self.compile_integer_binary_op_internal(l, r, op, target_bit_width);
    }

    // Runtime context: use overflow intrinsics if enabled
    if self.should_check_overflow() {
        match op {
            BinaryOp::Add => self.compile_checked_add(l, r, target_type),
            BinaryOp::Sub => self.compile_checked_sub(l, r, target_type),
            BinaryOp::Mul => self.compile_checked_mul(l, r, target_type),
            _ => self.compile_unchecked_binary_op(l, r, op),
        }
    } else {
        // Unchecked (wrapping) arithmetic
        self.compile_unchecked_binary_op(l, r, op)
    }
}

fn compile_checked_add(
    &mut self,
    l: IntValue<'ctx>,
    r: IntValue<'ctx>,
    target_type: &Type,
) -> Result<BasicValueEnum<'ctx>, String> {
    let int_type = l.get_type();
    let bit_width = int_type.get_bit_width();

    // Call LLVM intrinsic: llvm.sadd.with.overflow.i32
    let intrinsic_name = format!("llvm.sadd.with.overflow.i{}", bit_width);

    // Result struct: { i32 result, i1 overflow }
    let result_struct_type = self.context.struct_type(
        &[int_type.into(), self.context.bool_type().into()],
        false,
    );

    let intrinsic = self.declare_llvm_intrinsic(
        &intrinsic_name,
        &[int_type.into(), int_type.into()],
        result_struct_type.into(),
    );

    let result = self.builder
        .build_call(intrinsic, &[l.into(), r.into()], "add_result")
        .map_err(|e| format!("Failed to call overflow intrinsic: {}", e))?
        .try_as_basic_value()
        .unwrap_basic();

    // Extract sum and overflow flag
    let sum = self.builder
        .build_extract_value(result.into_struct_value(), 0, "sum")
        .map_err(|e| format!("Failed to extract sum: {}", e))?
        .into_int_value();

    let overflow = self.builder
        .build_extract_value(result.into_struct_value(), 1, "overflow")
        .map_err(|e| format!("Failed to extract overflow: {}", e))?
        .into_int_value();

    // if overflow { panic!("arithmetic overflow") }
    self.emit_overflow_check(overflow, op, target_type)?;

    Ok(sum.into())
}

fn emit_overflow_check(
    &mut self,
    overflow_flag: IntValue<'ctx>,
    op: &BinaryOp,
    target_type: &Type,
) -> Result<(), String> {
    let current_fn = self.current_function
        .ok_or("Overflow check outside function context")?;

    let overflow_block = self.context.append_basic_block(current_fn, "overflow");
    let continue_block = self.context.append_basic_block(current_fn, "no_overflow");

    // Branch based on overflow flag
    self.builder
        .build_conditional_branch(overflow_flag, overflow_block, continue_block)
        .map_err(|e| format!("Failed to build overflow branch: {}", e))?;

    // Overflow block: panic
    self.builder.position_at_end(overflow_block);

    let panic_msg = format!(
        "arithmetic overflow: {} operation on {:?}",
        match op {
            BinaryOp::Add => "addition",
            BinaryOp::Sub => "subtraction",
            BinaryOp::Mul => "multiplication",
            _ => "unknown",
        },
        target_type
    );

    self.call_panic(&panic_msg)?;
    self.builder.build_unreachable()
        .map_err(|e| format!("Failed to build unreachable: {}", e))?;

    // Continue block
    self.builder.position_at_end(continue_block);

    Ok(())
}

fn should_check_overflow(&self) -> bool {
    if cfg!(debug_assertions) {
        self.config.type_system.overflow_checks_debug
    } else {
        self.config.type_system.overflow_checks_release
    }
}
```

### 3.2 Panic Runtime Support

**File:** `vex-runtime/c/vex_panic.c` (NEW)

```c
#include <stdio.h>
#include <stdlib.h>

void vex_panic(const char* message, const char* file, int line) {
    fprintf(stderr, "panic at %s:%d: %s\n", file, line, message);
    abort();
}
```

**File:** `vex-compiler/src/codegen_ast/builtins/panic.rs` (NEW)

```rust
impl<'ctx> ASTCodeGen<'ctx> {
    pub(crate) fn call_panic(&mut self, message: &str) -> Result<(), String> {
        // Get or declare vex_panic(message: *const i8, file: *const i8, line: i32)
        let panic_fn = self.get_or_declare_panic_fn();

        // Create string constant for message
        let msg_global = self.builder
            .build_global_string_ptr(message, "panic_msg")
            .map_err(|e| format!("Failed to create panic message: {}", e))?;

        // Get file/line info (TODO: propagate from span)
        let file_global = self.builder
            .build_global_string_ptr("unknown", "panic_file")
            .map_err(|e| format!("Failed to create file string: {}", e))?;

        let line_const = self.context.i32_type().const_int(0, false);

        // Call vex_panic
        self.builder
            .build_call(
                panic_fn,
                &[
                    msg_global.as_pointer_value().into(),
                    file_global.as_pointer_value().into(),
                    line_const.into(),
                ],
                "panic_call",
            )
            .map_err(|e| format!("Failed to call panic: {}", e))?;

        Ok(())
    }

    fn get_or_declare_panic_fn(&mut self) -> FunctionValue<'ctx> {
        if let Some(func) = self.module.get_function("vex_panic") {
            return func;
        }

        // Declare: void vex_panic(const char*, const char*, i32)
        let i8_ptr = self.context.i8_type().ptr_type(AddressSpace::default());
        let i32_type = self.context.i32_type();
        let void_type = self.context.void_type();

        let fn_type = void_type.fn_type(
            &[i8_ptr.into(), i8_ptr.into(), i32_type.into()],
            false,
        );

        self.module.add_function("vex_panic", fn_type, None)
    }
}
```

---

## Phase 4: Mixed-Type Arithmetic (Week 4-5)

### 4.1 Implement Auto-Widening

**File:** `vex-compiler/src/codegen_ast/expressions/binary_ops/type_alignment.rs`

**Enhancement:**

```rust
/// Build upcast from narrower to wider integer type
pub(crate) fn build_upcast(
    &mut self,
    val: BasicValueEnum<'ctx>,
    from_type: &Type,
    to_type: &Type,
) -> Result<BasicValueEnum<'ctx>, String> {
    let int_val = match val {
        BasicValueEnum::IntValue(i) => i,
        _ => return Err(format!("Cannot upcast non-integer value")),
    };

    let target_llvm_type = self.ast_type_to_llvm_basic_type(to_type)?;
    let target_int_type = match target_llvm_type {
        BasicTypeEnum::IntType(t) => t,
        _ => return Err(format!("Target type is not an integer")),
    };

    let current_width = int_val.get_type().get_bit_width();
    let target_width = target_int_type.get_bit_width();

    if current_width == target_width {
        return Ok(val);
    }

    if current_width > target_width {
        return Err(format!(
            "Cannot upcast from wider to narrower type: {:?} -> {:?}",
            from_type, to_type
        ));
    }

    // Determine if source is signed
    let is_signed = matches!(
        from_type,
        Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::I128
    );

    let extended = if is_signed {
        self.builder
            .build_int_s_extend(int_val, target_int_type, "upcast_sext")
            .map_err(|e| format!("Failed to sign-extend: {}", e))?
    } else {
        self.builder
            .build_int_z_extend(int_val, target_int_type, "upcast_zext")
            .map_err(|e| format!("Failed to zero-extend: {}", e))?
    };

    Ok(extended.into())
}
```

### 4.2 Test Cases

**File:** `tests/type_coercion/mixed_arithmetic.vx`

```vex
// Test 1: i8 + i64 (ergonomic mode)
fn test_mixed_signed() {
    let a: i8 = 10;
    let b: i64 = 100;

    // Should auto-upcast a to i64
    let c = a + b;  // Type: i64, value: 110

    assert(c == 110);

    // Explicit type annotation
    let d: i64 = a + b;
    assert(d == 110);
}

// Test 2: Cross-signedness error
fn test_mixed_sign_error() {
    let a: i32 = 10;
    let b: u32 = 100;

    // Should ERROR: mixed signedness
    // let c = a + b;  // Compile error

    // Requires explicit cast
    let c = (a as u32) + b;  // OK
    assert(c == 110);
}

// Test 3: Downcast error
fn test_downcast_error() {
    let a: i64 = 1000000;
    let b: i32 = 100;

    // Should ERROR: would downcast i64 to i32
    // let c = a + b;  // Compile error

    // Requires explicit cast or upcast
    let c = a + (b as i64);  // OK: upcast b to i64
    assert(c == 1000100);
}

// Test 4: Float widening
fn test_float_widening() {
    let a: f32 = 3.14;
    let b: f64 = 2.71;

    // Should auto-widen a to f64
    let c = a + b;  // Type: f64

    // Precision preserved
    assert(c > 5.84 && c < 5.86);
}

// Test 5: Const context
const SMALL: i8 = 10;
const BIG: i64 = 100;
const SUM: i64 = SMALL + BIG;  // Should infer SMALL as i64

fn test_const_mixed() {
    assert(SUM == 110);
}
```

---

## Phase 5: Function Call Coercion (Week 5)

### 5.1 Argument Type Coercion

**File:** `vex-compiler/src/codegen_ast/expressions/calls/function_calls.rs`

**Enhancement:**

```rust
fn compile_function_args_with_coercion(
    &mut self,
    args: &[Expression],
    param_types: &[Type],
) -> Result<Vec<BasicValueEnum<'ctx>>, String> {
    if args.len() != param_types.len() {
        return Err(format!(
            "Argument count mismatch: expected {}, got {}",
            param_types.len(),
            args.len()
        ));
    }

    let mut compiled_args = Vec::new();

    for (i, (arg, param_type)) in args.iter().zip(param_types).enumerate() {
        // Compile argument
        let arg_val = self.compile_expression(arg)?;
        let arg_type = self.infer_expression_type(arg)?;

        // Apply coercion if needed
        let coerced = self.apply_coercion_if_allowed(
            arg_val,
            &arg_type,
            param_type,
            &format!("argument {}", i + 1),
        )?;

        compiled_args.push(coerced);
    }

    Ok(compiled_args)
}
```

**Test Case:**

```vex
fn takes_i64(x: i64) {
    println("Received: {}", x);
}

fn test_auto_upcast_arg() {
    let small: i32 = 42;

    // Should auto-upcast i32 to i64
    takes_i64(small);  // OK in ergonomic mode

    // Strict mode would require:
    // takes_i64(small as i64);
}
```

---

## Phase 6: Const Folding Refactor (Week 6-7)

### 6.1 Builder-Free Const Evaluation

**Problem:** Current const evaluation uses builder, causing position conflicts.

**File:** `vex-compiler/src/const_eval/mod.rs` (NEW MODULE)

```rust
/// Const evaluator that doesn't require LLVM builder
/// Evaluates AST expressions at compile time
pub struct ConstEvaluator<'ctx> {
    context: &'ctx Context,
    module: &'ctx Module<'ctx>,
    const_values: HashMap<String, ConstValue>,
}

#[derive(Debug, Clone)]
pub enum ConstValue {
    Int(i128),
    UInt(u128),
    Float(f64),
    Bool(bool),
    String(String),
}

impl<'ctx> ConstEvaluator<'ctx> {
    pub fn eval_const_expr(
        &mut self,
        expr: &Expression,
        expected_type: &Type,
    ) -> Result<ConstValue, String> {
        match expr {
            Expression::IntLiteral(n) => {
                self.validate_int_range(*n, expected_type)?;
                Ok(ConstValue::Int(*n))
            }

            Expression::Binary { left, op, right, .. } => {
                let left_val = self.eval_const_expr(left, expected_type)?;
                let right_val = self.eval_const_expr(right, expected_type)?;
                self.eval_binary_op(&left_val, op, &right_val, expected_type)
            }

            Expression::Ident(name) => {
                self.const_values.get(name)
                    .cloned()
                    .ok_or_else(|| format!("Constant '{}' not found", name))
            }

            _ => Err(format!("Non-constant expression in const context")),
        }
    }

    fn eval_binary_op(
        &self,
        left: &ConstValue,
        op: &BinaryOp,
        right: &ConstValue,
        expected_type: &Type,
    ) -> Result<ConstValue, String> {
        match (left, right) {
            (ConstValue::Int(l), ConstValue::Int(r)) => {
                let result = match op {
                    BinaryOp::Add => l.checked_add(*r)
                        .ok_or("Integer overflow in const addition")?,
                    BinaryOp::Sub => l.checked_sub(*r)
                        .ok_or("Integer overflow in const subtraction")?,
                    BinaryOp::Mul => l.checked_mul(*r)
                        .ok_or("Integer overflow in const multiplication")?,
                    BinaryOp::Div => l.checked_div(*r)
                        .ok_or("Division by zero in const expression")?,
                    _ => return Err(format!("Unsupported const operation: {:?}", op)),
                };

                self.validate_int_range(result as i64, expected_type)?;
                Ok(ConstValue::Int(result))
            }
            _ => Err("Type mismatch in const binary operation".to_string()),
        }
    }

    fn validate_int_range(&self, value: i64, target_type: &Type) -> Result<(), String> {
        // Same validation logic as Phase 2.2
        // ...
    }

    pub fn to_llvm_const(
        &self,
        value: &ConstValue,
        target_type: &Type,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        match (value, target_type) {
            (ConstValue::Int(n), Type::I8) => {
                Ok(self.context.i8_type().const_int(*n as u64, true).into())
            }
            (ConstValue::Int(n), Type::I64) => {
                Ok(self.context.i64_type().const_int(*n as u64, true).into())
            }
            // ... other types
            _ => Err(format!("Cannot convert {:?} to {:?}", value, target_type)),
        }
    }
}
```

### 6.2 Update Constants Compilation

**File:** `vex-compiler/src/codegen_ast/constants.rs`

```rust
pub fn compile_const(&mut self, const_decl: &Const) -> Result<(), String> {
    let expected_type = const_decl.ty.as_ref()
        .ok_or("Const declaration must have explicit type")?;

    // Use const evaluator instead of expression compiler
    let mut evaluator = ConstEvaluator::new(self.context, &self.module);

    // Evaluate at compile time
    let const_value = evaluator.eval_const_expr(&const_decl.value, expected_type)?;

    // Convert to LLVM constant
    let llvm_const = evaluator.to_llvm_const(&const_value, expected_type)?;

    // Create global constant
    let global = self.module.add_global(llvm_const.get_type(), None, &const_decl.name);
    global.set_initializer(&llvm_const);
    global.set_constant(true);

    // Cache for future references
    self.module_constants.insert(const_decl.name.clone(), llvm_const);
    self.global_constants.insert(const_decl.name.clone(), global.as_pointer_value());

    Ok(())
}
```

---

## Phase 7: Testing & Validation (Week 7-8)

### 7.1 Comprehensive Test Suite

**File:** `tests/type_system/coercion_tests.vx`

```vex
// === ERGONOMIC MODE TESTS ===

// Test 1: Auto-upcast in binary ops
fn test_binary_upcast() {
    let a: i8 = 10;
    let b: i64 = 100;
    let c = a + b;
    assert(c == 110);
    assert(typeof(c) == "i64");
}

// Test 2: Auto-upcast in function args
fn takes_i64(x: i64) -> i64 { x }

fn test_function_upcast() {
    let small: i32 = 42;
    let result = takes_i64(small);
    assert(result == 42);
}

// Test 3: Const folding with mixed types
const A: i8 = 10;
const B: i64 = 100;
const C: i64 = A + B;  // A auto-upcasted

fn test_const_upcast() {
    assert(C == 110);
}

// Test 4: Downcast requires explicit cast
fn test_downcast_error() {
    let big: i64 = 1000;
    let small: i32 = 100;

    // Should fail to compile:
    // let result = big + small;  // Error: would downcast big to i32

    // Correct:
    let result = big + (small as i64);
    assert(result == 1100);
}

// Test 5: Cross-sign error
fn test_sign_error() {
    let signed: i32 = -10;
    let unsigned: u32 = 100;

    // Should fail:
    // let result = signed + unsigned;  // Error: mixed signedness

    // Correct:
    let result = (signed as u32) + unsigned;
    assert(result == 90);
}

// === OVERFLOW TESTS ===

fn test_overflow_panic() {
    let x: i32 = 2147483647;  // i32::MAX

    // Should panic in debug mode:
    // let y = x + 1;

    // Use checked arithmetic:
    let checked = sadd_overflow(x, 1);
    assert(checked.overflow == true);
}

// === STRICT MODE TESTS ===
// (Run with --strict-types flag)

fn test_strict_no_upcast() {
    let a: i8 = 10;
    let b: i64 = 100;

    // Should fail in strict mode:
    // let c = a + b;  // Error: implicit cast not allowed

    // Correct:
    let c = (a as i64) + b;
    assert(c == 110);
}
```

### 7.2 Compiler Error Messages

**Good error examples:**

```
error: cannot perform addition between incompatible types
  --> test.vx:5:13
   |
 5 |     let c = a + b;
   |             ^^^^^ cannot add `i32` and `u32`
   |
   = help: use explicit cast to convert signedness
   = note: try `(a as u32) + b` or `a + (b as i32)`

error: implicit downcast not allowed
  --> test.vx:10:13
   |
10 |     let c = big + small;
   |             ^^^^^^^^^^^ would implicitly downcast `i64` to `i32`
   |
   = help: widen the narrower type to match
   = note: try `big + (small as i64)`
   = note: or use explicit downcast: `(big as i32) + small` (may overflow)

error: arithmetic overflow
  --> test.vx:15:13
   |
15 |     let c = x + 1;
   |             ^^^^^ overflow in addition
   |
   = note: x = 2147483647 (i32::MAX)
   = help: use checked arithmetic: `sadd_overflow(x, 1)`
   = help: or use a wider type: `let x: i64 = ...`
```

---

## Phase 8: Documentation & Migration (Week 8)

### 8.1 Update Language Spec

**File:** `Specifications/03_Type_System.md`

````markdown
## Type Coercion and Casting

### Implicit Coercions (Ergonomic Mode)

Vex allows safe implicit conversions (upcasts) by default:

- **Integer widening:** i8 → i16 → i32 → i64 → i128
- **Unsigned widening:** u8 → u16 → u32 → u64 → u128
- **Float widening:** f32 → f64

Example:

```vex
let a: i8 = 10;
let b: i64 = 100;
let c = a + b;  // ✅ a auto-upcasted to i64
```
````

### Forbidden Implicit Conversions

The following require explicit `as` casts:

- **Downcast:** i64 → i32 (data loss)
- **Sign change:** i32 ↔ u32 (semantic change)
- **Float narrowing:** f64 → f32 (precision loss)

### Strict Mode

Enable with `--strict-types` for Rust-like semantics:

- No implicit conversions
- All casts require `as` keyword
- Mismatched types = compile error

```bash
vex build --strict-types
```

````

### 8.2 Migration Guide

**File:** `docs/MIGRATION_ERGONOMIC_TYPES.md`

```markdown
# Migration Guide: Ergonomic Type System

## Breaking Changes

### Before
```vex
let a: i8 = 10;
let b: i64 = 100;
let c = (a as i64) + b;  // Manual cast required
````

### After

```vex
let a: i8 = 10;
let b: i64 = 100;
let c = a + b;  // ✅ Auto-upcast (ergonomic mode)
```

## Opt-Out: Strict Mode

If you prefer Rust-strict semantics:

```toml
# vex.toml
[compiler]
strict-types = true
```

## New Compiler Flags

- `--strict-types`: Disable auto-upcast
- `--overflow-checks`: Force overflow detection in release
- `--no-overflow-checks`: Disable in debug (not recommended)

## Code Examples

See `examples/type_coercion/` for comprehensive examples.

````

---

## Phase 9: Performance Optimization (Week 9-10)

### 9.1 LLVM Optimization Passes

**Ensure:**
- Dead code elimination removes unused upcasts
- Constant folding optimizes away runtime casts
- Overflow checks eliminated when provably safe

**File:** `vex-compiler/src/optimization/passes.rs`

```rust
pub fn apply_type_coercion_optimizations(module: &Module) {
    // Remove redundant casts (i32 -> i64 -> i32)
    // Fold constant upcasts at compile time
    // Eliminate overflow checks for small constants
}
````

### 9.2 Benchmark Suite

**File:** `benches/type_coercion.rs`

```rust
// Compare performance:
// 1. Manual casts vs auto-upcast
// 2. Overflow checks enabled vs disabled
// 3. Strict mode vs ergonomic mode
```

---

## Phase 10: Edge Cases & Refinement (Week 10-12)

### 10.1 Complex Scenarios

**Arrays:**

```vex
let arr: [i64; 3] = [1i8, 2i16, 3i32];  // Auto-upcast elements
```

**Struct fields:**

```vex
struct Point {
    x: i64,
    y: i64,
}

let p = Point { x: 10i8, y: 20i16 };  // Auto-upcast
```

**Generic functions:**

```vex
fn max<T>(a: T, b: T) -> T { ... }

let result = max(10i8, 100i64);  // Should error: type mismatch
let result = max(10i8 as i64, 100i64);  // OK
```

### 10.2 Error Recovery

- Suggest casts in error messages
- Offer quick-fix in LSP
- Detect common patterns

---

## Success Metrics

### Functionality Checklist

- [ ] i8 + i64 auto-upcasts to i64
- [ ] i64 + i32 errors (downcast forbidden)
- [ ] i32 + u32 errors (sign change forbidden)
- [ ] const X: i64 = 60 \* SECOND works without casts
- [ ] Overflow panics in debug mode
- [ ] Overflow wraps in release (configurable)
- [ ] --strict-types disables auto-upcast
- [ ] Function args auto-upcast
- [ ] Let bindings auto-upcast
- [ ] Array/struct literals auto-upcast

### Performance Targets

- Auto-upcast adds <5% compile time
- Overflow checks add <2% runtime (debug)
- LLVM eliminates 90% of redundant casts
- Const folding handles all compile-time casts

### Documentation

- [ ] Type system spec updated
- [ ] Migration guide published
- [ ] 50+ test cases covering all scenarios
- [ ] LSP provides cast suggestions

---

## Risk Mitigation

### Compatibility

**Concern:** Existing code breaks with new auto-upcast  
**Mitigation:**

- Provide `--strict-types` opt-out
- Gradual migration period
- Clear deprecation warnings

### Performance

**Concern:** Overflow checks slow down tight loops  
**Mitigation:**

- Profile-guided optimization
- `--no-overflow-checks` for hot paths
- LLVM removes provably-safe checks

### Complexity

**Concern:** Type system too complex  
**Mitigation:**

- Start with simple rules (only safe upcasts)
- Comprehensive error messages
- Interactive playground with examples

---

## Timeline Summary

| Week  | Phase            | Deliverables                                |
| ----- | ---------------- | ------------------------------------------- |
| 1-2   | Infrastructure   | Coercion rules, config, binary op inference |
| 3-4   | Overflow         | Runtime checks, panic integration           |
| 4-5   | Mixed arithmetic | Auto-widening, test cases                   |
| 5     | Function calls   | Argument coercion                           |
| 6-7   | Const folding    | Builder-free evaluator                      |
| 7-8   | Testing          | Comprehensive test suite                    |
| 8     | Documentation    | Spec, migration guide                       |
| 9-10  | Optimization     | LLVM passes, benchmarks                     |
| 10-12 | Refinement       | Edge cases, error messages                  |

**Total:** 12 weeks to production-ready type system

---

## Next Steps

1. **Review this plan** with team
2. **Create tracking issues** for each phase
3. **Set up CI/CD** for new test suite
4. **Begin Phase 1** implementation

**Questions?** Open an issue or discussion.
