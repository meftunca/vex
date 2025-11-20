// Variable registration and type determination

use crate::codegen_ast::ASTCodeGen;
use crate::type_system::coercion_rules::{
    classify_coercion, coercion_policy, format_coercion_error, format_coercion_warning,
    CoercionPolicy,
};
use inkwell::types::BasicTypeEnum;
use inkwell::values::BasicValueEnum;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Determine final variable type from value, annotation, and inferred struct name
    pub(crate) fn determine_final_type(
        &mut self,
        ty: Option<&Type>,
        mut val: BasicValueEnum<'ctx>,
        value: &Expression,
        struct_name_from_expr: &Option<String>,
    ) -> Result<(Type, BasicTypeEnum<'ctx>), String> {
        // Determine type from value or explicit type
        let (var_type, llvm_type) = if let Some(t) = ty {
            let target_llvm_type = self.ast_type_to_llvm(t);

            // ⭐ CRITICAL: Check for cross-type casts (float <-> int)
            // These require explicit casting and should not be implicit
            if let BasicValueEnum::FloatValue(_) = val {
                if matches!(target_llvm_type, BasicTypeEnum::IntType(_)) {
                    return Err(
                        "cannot implicitly cast float to integer: truncation may occur\n\nhelp: use an explicit cast: `value as i32`".to_string()
                    );
                }
            }
            if let BasicValueEnum::IntValue(_) = val {
                if matches!(target_llvm_type, BasicTypeEnum::FloatType(_)) {
                    // Note: int -> float is usually safe but can lose precision for large integers
                    // For now we allow it, but we could make it explicit too
                }
            }

            // Cast integer literal to match target integer type width
            val = self.cast_integer_if_needed(val, t, target_llvm_type, value)?;

            // Cast float literals to match target float type
            val = self.cast_float_if_needed(val, target_llvm_type, value)?;

            (t.clone(), target_llvm_type)
        } else {
            // ⭐ SPECIAL CASE: Check if this is a cast expression - use target type
            if let Expression::Cast { target_type, .. } = value {
                let target_llvm_type = self.ast_type_to_llvm(target_type);
                (target_type.clone(), target_llvm_type)
            }
            // ⭐ SPECIAL CASE: Check if this is a tuple literal
            else if let Some(tuple_struct_type) = self.last_compiled_tuple_type {
                // For tuple literals, create a Tuple type
                // DON'T unwrap/consume - we need it later in register_struct_or_tuple_variable
                (Type::Named("Tuple".to_string()), tuple_struct_type.into())
            }
            // ⭐ CRITICAL: For MethodCall expressions, infer return type from AST
            else if matches!(value, Expression::MethodCall { .. }) {
                // Use AST-level type inference instead of LLVM type inference
                match self.infer_expression_type(value) {
                    Ok(inferred_type) => {
                        let inferred_llvm_type = self.ast_type_to_llvm(&inferred_type);
                        eprintln!("  ✅ MethodCall inferred type: {:?}", inferred_type);
                        (inferred_type, inferred_llvm_type.into())
                    }
                    Err(e) => {
                        eprintln!("  ⚠️  MethodCall type inference failed: {}", e);
                        // Fallback to LLVM inference
                        let inferred_llvm_type = val.get_type();
                        let inferred_ast_type =
                            self.infer_ast_type_from_llvm(inferred_llvm_type)?;
                        (inferred_ast_type, inferred_llvm_type)
                    }
                }
            } else {
                // Infer type from LLVM value
                let inferred_llvm_type = val.get_type();

                // If we have struct_name_from_expr, prefer it over LLVM type inference
                // (struct literals return pointers which can't be distinguished)
                let inferred_ast_type = if let Some(struct_name) = struct_name_from_expr {
                    Type::Named(struct_name.clone())
                } else {
                    self.infer_ast_type_from_llvm(inferred_llvm_type)?
                };

                (inferred_ast_type, inferred_llvm_type)
            }
        };

        // Track struct type name and finalize variable type
        let final_var_type =
            self.finalize_variable_type(&var_type, struct_name_from_expr, value)?;

        // Determine final LLVM type
        let final_llvm_type = self.determine_final_llvm_type(&final_var_type, llvm_type);

        Ok((final_var_type, final_llvm_type))
    }

    fn cast_integer_if_needed(
        &self,
        val: BasicValueEnum<'ctx>,
        target_type: &Type,
        target_llvm_type: BasicTypeEnum<'ctx>,
        value_expr: &Expression,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        if let BasicValueEnum::IntValue(int_val) = val {
            if let BasicTypeEnum::IntType(target_int_type) = target_llvm_type {
                let current_width = int_val.get_type().get_bit_width();
                let target_width = target_int_type.get_bit_width();

                // ⚠️ CRITICAL: Check sign change even with same width (i32 → u32)
                // Infer source type from expression AST
                let source_type_from_expr = self.infer_expression_type(value_expr).ok();
                
                if let Some(src_type) = source_type_from_expr {
                    // Check for forbidden sign changes
                    let coercion_kind = classify_coercion(&src_type, target_type);
                    let policy = coercion_policy(coercion_kind, self.is_in_unsafe_block);
                    
                    match policy {
                        CoercionPolicy::Error => {
                            return Err(format_coercion_error(&src_type, target_type, coercion_kind));
                        }
                        CoercionPolicy::Warn => {
                            eprintln!("⚠️  warning: {}", format_coercion_warning(&src_type, target_type));
                        }
                        CoercionPolicy::Allow => {}
                    }
                }

                if current_width != target_width {
                    if current_width < target_width {
                        // ✅ UPCAST: Safe widening conversion
                        let is_unsigned = matches!(
                            target_type,
                            Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::U128
                        );
                        return if is_unsigned {
                            self.builder
                                .build_int_z_extend(int_val, target_int_type, "lit_zext")
                                .map_err(|e| format!("Failed to zero-extend literal: {}", e))
                                .map(|v| v.into())
                        } else {
                            self.builder
                                .build_int_s_extend(int_val, target_int_type, "lit_sext")
                                .map_err(|e| format!("Failed to sign-extend literal: {}", e))
                                .map(|v| v.into())
                        };
                    } else {
                        // ❌ DOWNCAST: Narrowing cast (data loss possible)
                        // Use coercion_rules to determine if this is allowed

                        // Helper function to check if expression is a compile-time constant
                        fn is_compile_time_constant(expr: &Expression) -> bool {
                            match expr {
                                Expression::IntLiteral(_) | Expression::BigIntLiteral(_) => true,
                                Expression::Unary {
                                    op: UnaryOp::Neg,
                                    expr,
                                    ..
                                } => is_compile_time_constant(expr),
                                Expression::Binary { left, right, .. } => {
                                    is_compile_time_constant(left)
                                        && is_compile_time_constant(right)
                                }
                                _ => false,
                            }
                        }

                        let is_compile_time_const = is_compile_time_constant(value_expr);

                        // Infer source type from current bit width
                        let source_type = match current_width {
                            8 => Type::I8,
                            16 => Type::I16,
                            32 => Type::I32,
                            64 => Type::I64,
                            128 => Type::I128,
                            _ => return Err(format!("Unsupported integer width: {}", current_width)),
                        };

                        // Check coercion policy
                        let coercion_kind = classify_coercion(&source_type, target_type);
                        let policy = coercion_policy(coercion_kind, self.is_in_unsafe_block);

                        match policy {
                            CoercionPolicy::Error => {
                                return Err(format_coercion_error(&source_type, target_type, coercion_kind));
                            }
                            CoercionPolicy::Warn => {
                                // Emit warning for unsafe downcast
                                eprintln!("⚠️  warning: {}", format_coercion_warning(&source_type, target_type));
                                // Allow truncation in unsafe{}
                            }
                            CoercionPolicy::Allow => {
                                // Safe upcast, should not reach here in downcast branch
                            }
                        }

                        // ✅ SAFE: Compile-time constant - verify value fits
                        if is_compile_time_const {
                            // For constants, verify the value fits in target type
                            let literal_value = int_val.get_sign_extended_constant();
                            let fits_in_target = if let Some(const_val) = literal_value {
                                // Check if value is within target type's range
                                let is_unsigned = matches!(
                                    target_type,
                                    Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::U128
                                );

                                if is_unsigned {
                                    // Unsigned types: check if value is non-negative and fits
                                    match target_width {
                                        8 => const_val >= 0 && const_val <= 255,         // u8
                                        16 => const_val >= 0 && const_val <= 65535,      // u16
                                        32 => const_val >= 0 && const_val <= 4294967295, // u32
                                        _ => const_val >= 0, // u64, u128 - check non-negative
                                    }
                                } else {
                                    // Signed types: check range
                                    match target_width {
                                        8 => const_val >= -128 && const_val <= 127,      // i8
                                        16 => const_val >= -32768 && const_val <= 32767, // i16
                                        32 => const_val >= -2147483648 && const_val <= 2147483647, // i32
                                        _ => true, // i64, i128 - always fits from i32
                                    }
                                }
                            } else {
                                // If we can't get constant value, allow the truncation
                                // (this shouldn't happen for literals, but be safe)
                                true
                            };

                            if fits_in_target {
                                // ✅ Value fits - safe to truncate
                                return self
                                    .builder
                                    .build_int_truncate(int_val, target_int_type, "lit_trunc")
                                    .map_err(|e| format!("Failed to truncate literal: {}", e))
                                    .map(|v| v.into());
                            } else {
                                // ❌ Literal value too large for target type
                                return Err(format!(
                                    "integer literal {} is too large for type `{:?}`",
                                    literal_value.unwrap_or(0),
                                    target_type
                                ));
                            }
                        } else {
                            // Runtime value with explicit type annotation
                            // ✅ ALLOW: Programmer explicitly specified target type in let statement
                            // This is their responsibility to ensure correctness
                            return self
                                .builder
                                .build_int_truncate(int_val, target_int_type, "downcast")
                                .map_err(|e| format!("Failed to truncate to target type: {}", e))
                                .map(|v| v.into());
                        }
                    }
                }
            }
        }
        Ok(val)
    }

    fn cast_float_if_needed(
        &self,
        val: BasicValueEnum<'ctx>,
        target_llvm_type: BasicTypeEnum<'ctx>,
        value_expr: &Expression,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        if let BasicValueEnum::FloatValue(float_val) = val {
            if let BasicTypeEnum::FloatType(target_float_type) = target_llvm_type {
                if float_val.get_type() != target_float_type {
                    let is_source_double = float_val.get_type() == self.context.f64_type();
                    let is_target_double = target_float_type == self.context.f64_type();

                    if is_source_double && !is_target_double {
                        // f64 -> f32: Precision loss possible
                        // Helper to check if expression is a compile-time constant
                        fn is_float_constant(expr: &Expression) -> bool {
                            match expr {
                                Expression::FloatLiteral(_) => true,
                                Expression::Unary {
                                    op: UnaryOp::Neg,
                                    expr,
                                    ..
                                } => is_float_constant(expr),
                                Expression::Binary { left, right, .. } => {
                                    is_float_constant(left) && is_float_constant(right)
                                }
                                _ => false,
                            }
                        }

                        let is_compile_time_const = is_float_constant(value_expr);

                        if is_compile_time_const {
                            // ✅ LITERAL: Safe to truncate (value known)
                            return self
                                .builder
                                .build_float_trunc(float_val, target_float_type, "lit_ftrunc")
                                .map_err(|e| format!("Failed to truncate float literal: {}", e))
                                .map(|v| v.into());
                        } else {
                            // Runtime value with explicit type annotation
                            // ✅ ALLOW: Programmer explicitly specified f32 in let statement
                            return self
                                .builder
                                .build_float_trunc(float_val, target_float_type, "fdowncast")
                                .map_err(|e| format!("Failed to truncate float: {}", e))
                                .map(|v| v.into());
                        }
                    } else if !is_source_double && is_target_double {
                        // ✅ f32 -> f64: Safe upcast
                        return self
                            .builder
                            .build_float_ext(float_val, target_float_type, "float_ext")
                            .map_err(|e| format!("Failed to extend float: {}", e))
                            .map(|v| v.into());
                    }
                }
            }
        }
        Ok(val)
    }

    fn finalize_variable_type(
        &mut self,
        var_type: &Type,
        struct_name_from_expr: &Option<String>,
        value: &Expression,
    ) -> Result<Type, String> {
        eprintln!("🔍 finalize_variable_type called");
        eprintln!("  var_type: {:?}", var_type);
        eprintln!("  struct_name_from_expr: {:?}", struct_name_from_expr);
        eprintln!("  expression: {:?}", value);

        // ⭐ NEW: Extract generic type args from static method calls
        // Vec<i32>.new() parses as: MethodCall { receiver: Ident("Vec"), method: "new", type_args: [I32] }
        if let Expression::MethodCall {
            receiver,
            method,
            type_args,
            ..
        } = value
        {
            eprintln!("  → Is MethodCall, method: {}", method);
            eprintln!("  → Receiver: {:?}", receiver);
            eprintln!("  → Type args in MethodCall: {:?}", type_args);

            // Vec<i32>.new() has type_args in the MethodCall itself, receiver is just Ident("Vec")
            if let Expression::Ident(type_name) = &**receiver {
                eprintln!("  → Receiver is Ident: {}", type_name);
                if !type_args.is_empty() && (method == "new" || method == "with_capacity") {
                    let result = self.build_generic_type(type_name, type_args);
                    eprintln!("  → Built generic type: {:?}", result);
                    return Ok(result);
                }
            }
        }

        // Type finalization - actual variable_struct_names insertion happens in register_struct_or_tuple_variable
        match var_type {
            Type::Box(_) => {
                // Note: Don't insert here, will be done in register_struct_or_tuple_variable
                Ok(var_type.clone())
            }
            Type::Vec(_) => {
                // Note: Don't insert here, will be done in register_struct_or_tuple_variable
                Ok(var_type.clone())
            }
            Type::Channel(_) => {
                // Note: Don't insert here, will be done in register_struct_or_tuple_variable
                Ok(var_type.clone())
            }
            Type::Option(_) => {
                // Note: Don't insert here, will be done in register_struct_or_tuple_variable
                Ok(var_type.clone())
            }
            Type::Result(_, _) => {
                // Note: Don't insert here, will be done in register_struct_or_tuple_variable
                Ok(var_type.clone())
            }
            Type::Generic {
                name: struct_name,
                type_args,
            } => match self.instantiate_generic_struct(struct_name, type_args) {
                Ok(_mangled_name) => {
                    // Note: Don't insert here, will be done in register_struct_or_tuple_variable
                    Ok(Type::Generic {
                        name: struct_name.clone(),
                        type_args: type_args.clone(),
                    })
                }
                Err(_) => Ok(var_type.clone()),
            },
            Type::Named(struct_name) => {
                self.finalize_named_type(struct_name, struct_name_from_expr, value)
            }
            Type::Array(_, _) => {
                // Arrays don't need special struct name tracking
                Ok(var_type.clone())
            }
            Type::Slice(_, _) => {
                // Slices don't need finalization, keep as-is
                Ok(var_type.clone())
            }
            Type::RawPtr { .. } => {
                // Raw pointers don't need finalization, keep as-is
                Ok(var_type.clone())
            }
            Type::Reference(_, _) => {
                // References don't need finalization
                Ok(var_type.clone())
            }
            // Primitive types don't need finalization
            Type::I8
            | Type::I16
            | Type::I32
            | Type::I64
            | Type::I128
            | Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::U128
            | Type::F16
            | Type::F32
            | Type::F64
            | Type::Bool
            | Type::String => Ok(var_type.clone()),
            _ => {
                if let Some(type_name) = struct_name_from_expr {
                    self.finalize_inferred_type(type_name)
                } else {
                    self.finalize_from_expression(value)
                }
            }
        }
    }

    fn finalize_named_type(
        &mut self,
        struct_name: &str,
        struct_name_from_expr: &Option<String>,
        value: &Expression,
    ) -> Result<Type, String> {
        if struct_name == "AnonymousStruct" {
            if let Some(type_name) = struct_name_from_expr {
                return self.handle_anonymous_struct_resolution(type_name);
            } else if let Expression::EnumLiteral { enum_name, .. } = value {
                self.variable_enum_names
                    .insert(format!("temp"), enum_name.clone());
                return Ok(Type::Named(enum_name.clone()));
            }
        } else if self.struct_defs.contains_key(struct_name) {
            self.variable_struct_names
                .insert(format!("temp"), struct_name.to_string());
        } else if self.enum_ast_defs.contains_key(struct_name) {
            self.variable_enum_names
                .insert(format!("temp"), struct_name.to_string());
        }

        Ok(Type::Named(struct_name.to_string()))
    }

    fn handle_anonymous_struct_resolution(&mut self, type_name: &str) -> Result<Type, String> {
        let is_builtin = matches!(
            type_name,
            "Vec"
                | "Box"
                | "String"
                | "Map"
                | "Range"
                | "RangeInclusive"
                | "Slice"
                | "Option"
                | "Result"
        );

        if is_builtin {
            self.variable_struct_names
                .insert(format!("temp"), type_name.to_string());
            Ok(Type::Named(type_name.to_string()))
        } else if self.struct_defs.contains_key(type_name) {
            self.variable_struct_names
                .insert(format!("temp"), type_name.to_string());
            Ok(Type::Named(type_name.to_string()))
        } else if self.enum_ast_defs.contains_key(type_name) {
            self.variable_enum_names
                .insert(format!("temp"), type_name.to_string());
            Ok(Type::Named(type_name.to_string()))
        } else {
            Ok(Type::Named(type_name.to_string()))
        }
    }

    fn finalize_inferred_type(&mut self, type_name: &str) -> Result<Type, String> {
        let is_mangled_generic = type_name.starts_with("Vec_")
            || type_name.starts_with("Box_")
            || type_name.starts_with("Map_")
            || type_name.starts_with("HashMap_")
            || type_name.starts_with("HashSet_")
            || type_name.starts_with("Set_");

        let is_builtin = matches!(
            type_name,
            "Vec" | "Box" | "String" | "Map" | "Set" | "Slice" | "Result" | "Option"
        ) || is_mangled_generic;

        if is_builtin || self.struct_defs.contains_key(type_name) {
            self.variable_struct_names
                .insert(format!("temp"), type_name.to_string());
            Ok(Type::Named(type_name.to_string()))
        } else if self.enum_ast_defs.contains_key(type_name) {
            self.variable_enum_names
                .insert(format!("temp"), type_name.to_string());
            Ok(Type::Named(type_name.to_string()))
        } else {
            Ok(Type::Named(type_name.to_string()))
        }
    }

    fn finalize_from_expression(&mut self, value: &Expression) -> Result<Type, String> {
        match value {
            Expression::StructLiteral {
                name: struct_name, ..
            } => {
                if self.struct_defs.contains_key(struct_name) {
                    self.variable_struct_names
                        .insert(format!("temp"), struct_name.clone());
                    Ok(Type::Named(struct_name.clone()))
                } else {
                    Ok(Type::Named("AnonymousStruct".to_string()))
                }
            }
            Expression::EnumLiteral { enum_name, .. } => {
                if self.enum_ast_defs.contains_key(enum_name) {
                    self.variable_enum_names
                        .insert(format!("temp"), enum_name.clone());
                    Ok(Type::Named(enum_name.clone()))
                } else {
                    Ok(Type::Named("AnonymousEnum".to_string()))
                }
            }
            Expression::Call { func, .. } => self.finalize_from_call(func),
            Expression::FieldAccess { object, field } => {
                self.finalize_from_field_access(object, field)
            }
            _ => Ok(Type::Named("AnonymousStruct".to_string())),
        }
    }

    fn finalize_from_call(&mut self, func: &Expression) -> Result<Type, String> {
        if let Expression::FieldAccess { object, .. } = func {
            if let Expression::Ident(enum_name) = object.as_ref() {
                if self.enum_ast_defs.contains_key(enum_name) {
                    self.variable_enum_names
                        .insert(format!("temp"), enum_name.clone());
                    return Ok(Type::Named(enum_name.clone()));
                }
            }
        } else if let Expression::Ident(func_name) = func {
            if let Some(func_def) = self.function_defs.get(func_name) {
                if let Some(Type::Named(type_name)) = &func_def.return_type {
                    if self.struct_defs.contains_key(type_name) {
                        self.variable_struct_names
                            .insert(format!("temp"), type_name.clone());
                        return Ok(Type::Named(type_name.clone()));
                    } else if self.enum_ast_defs.contains_key(type_name) {
                        self.variable_enum_names
                            .insert(format!("temp"), type_name.clone());
                        return Ok(Type::Named(type_name.clone()));
                    }
                }
            }
        }
        Ok(Type::Named("AnonymousStruct".to_string()))
    }

    fn finalize_from_field_access(
        &mut self,
        object: &Expression,
        field: &str,
    ) -> Result<Type, String> {
        let object_struct_name = self.get_expression_struct_name(object)?;

        if let Some(struct_name) = object_struct_name {
            let field_type_opt = self.struct_defs.get(&struct_name).and_then(|struct_def| {
                struct_def
                    .fields
                    .iter()
                    .find(|(f, _)| f == field)
                    .map(|(_, t)| t.clone())
            });

            if let Some(field_type) = field_type_opt {
                match field_type {
                    Type::Named(field_struct_name) => {
                        if self.struct_defs.contains_key(&field_struct_name) {
                            self.variable_struct_names
                                .insert(format!("temp"), field_struct_name.clone());
                            return Ok(Type::Named(field_struct_name));
                        }
                    }
                    Type::Generic {
                        name: field_struct_name,
                        type_args,
                    } => {
                        if let Ok(mangled_name) =
                            self.instantiate_generic_struct(&field_struct_name, &type_args)
                        {
                            self.variable_struct_names
                                .insert(format!("temp"), mangled_name.clone());
                            return Ok(Type::Generic {
                                name: field_struct_name,
                                type_args,
                            });
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(Type::Named("AnonymousStruct".to_string()))
    }

    fn determine_final_llvm_type(
        &self,
        final_var_type: &Type,
        llvm_type: BasicTypeEnum<'ctx>,
    ) -> BasicTypeEnum<'ctx> {
        match final_var_type {
            Type::Named(type_name) => {
                if type_name == "Tuple"
                    || self.enum_ast_defs.contains_key(type_name)
                    || self.struct_defs.contains_key(type_name)
                    || type_name == "Range"
                    || type_name == "RangeInclusive"
                {
                    llvm_type
                } else {
                    self.ast_type_to_llvm(final_var_type)
                }
            }
            Type::Generic { .. } => llvm_type,
            _ => llvm_type,
        }
    }

    /// Register variable in symbol table with proper type tracking
    pub(crate) fn register_variable(
        &mut self,
        name: &str,
        val: BasicValueEnum<'ctx>,
        final_var_type: &Type,
        final_llvm_type: BasicTypeEnum<'ctx>,
        is_mutable: bool,
    ) -> Result<(), String> {
        let is_tuple_literal = self.last_compiled_tuple_type.is_some();
        let is_builtin_pointer = matches!(
            final_var_type,
            Type::Vec(_) | Type::Box(_) | Type::Channel(_)
        ) || matches!(final_var_type, Type::Named(n) if n.starts_with("Vec_") || n.starts_with("Box_") || n == "Channel");

        let is_array = matches!(final_var_type, Type::Array(_, _));
        let is_slice = matches!(final_var_type, Type::Slice(_, _));

        // ⭐ NEW: Check if type is a primitive type (should NOT be treated as struct)
        let is_primitive = matches!(
            final_var_type,
            Type::I32 | Type::I64 | Type::F32 | Type::F64 | Type::Bool | Type::String
        ) || matches!(final_var_type, Type::Named(n) if n == "i32" || n == "i64" || n == "f32" || n == "f64" || n == "bool");

        let is_struct_or_tuple = !is_primitive
            && (matches!(final_var_type,
                Type::Named(type_name) if type_name == "Tuple"
                    || self.struct_defs.contains_key(type_name)
                    || type_name == "Option"
                    || type_name == "Result"
                    || type_name == "Vec"  // Builtin Vec without type annotation
                    || type_name == "Box"  // Builtin Box without type annotation
                    || type_name == "Map"
                    || type_name == "Set"
                    || type_name == "Channel"
            ) || matches!(final_var_type, Type::Option(_) | Type::Result(_, _))
                || matches!(final_var_type, Type::Generic { .. })  // ⭐ CRITICAL: Generic types are structs!
                || is_tuple_literal);

        eprintln!(
            "📝 Variable '{}': is_struct_or_tuple={}, is_builtin_pointer={}, final_var_type={:?}",
            name, is_struct_or_tuple, is_builtin_pointer, final_var_type
        );

        if is_struct_or_tuple || is_builtin_pointer || is_array || is_slice {
            self.register_struct_or_tuple_variable(
                name,
                val,
                final_var_type,
                final_llvm_type,
                is_tuple_literal,
                is_mutable,
            )?;
        } else {
            self.register_regular_variable(name, val, final_var_type, final_llvm_type, is_mutable)?;
        }

        // Track closure variables
        if let BasicValueEnum::PointerValue(fn_ptr) = val {
            if let Some(env_ptr) = self.closure_envs.get(&fn_ptr) {
                self.closure_variables
                    .insert(name.to_string(), (fn_ptr, *env_ptr));
            }
        }

        // Register for automatic cleanup if type implements Drop
        // IMPORTANT: Use variable_concrete_types if available (for generic inference)
        let concrete_type = self
            .variable_concrete_types
            .get(name)
            .unwrap_or(final_var_type);

        let type_name_for_drop = match concrete_type {
            Type::Named(name) => Some(name.clone()),
            Type::Generic { name, type_args } => {
                // Build mangled name for generic types: Vec<i32> → Vec_i32
                let type_suffix = type_args
                    .iter()
                    .map(|t| self.type_to_string(t))
                    .collect::<Vec<_>>()
                    .join("_");
                Some(format!("{}_{}", name, type_suffix))
            }
            Type::Vec(inner_type) => {
                let type_suffix = self.type_to_string(inner_type);
                Some(format!("Vec_{}", type_suffix))
            }
            Type::Box(inner_type) => {
                let type_suffix = self.type_to_string(inner_type);
                Some(format!("Box_{}", type_suffix))
            }
            _ => None,
        };

        if let Some(type_name) = type_name_for_drop {
            // Check if type implements Drop trait
            if self.type_implements_drop(&type_name)
                || type_name.starts_with("Vec")
                || type_name.starts_with("Box")
            {
                self.register_for_cleanup(name.to_string(), type_name);
            }
        }

        Ok(())
    }

    fn register_struct_or_tuple_variable(
        &mut self,
        name: &str,
        val: BasicValueEnum<'ctx>,
        final_var_type: &Type,
        final_llvm_type: BasicTypeEnum<'ctx>,
        is_tuple_literal: bool,
        is_mutable: bool,
    ) -> Result<(), String> {
        if let BasicValueEnum::PointerValue(data_ptr) = val {
            self.variables.insert(name.to_string(), data_ptr);
            self.variables.insert(name.to_string(), data_ptr);

            if is_tuple_literal {
                if let Some(struct_ty) = self.last_compiled_tuple_type {
                    self.variable_types
                        .insert(name.to_string(), struct_ty.into());
                    self.variable_ast_types
                        .insert(name.to_string(), final_var_type.clone());
                    self.tuple_variable_types
                        .insert(name.to_string(), struct_ty);
                    self.variable_struct_names
                        .insert(name.to_string(), "Tuple".to_string());
                    self.last_compiled_tuple_type = None;
                } else {
                    return Err("Tuple literal didn't set last_compiled_tuple_type".to_string());
                }
            } else {
                self.variable_types
                    .insert(name.to_string(), final_llvm_type);
                self.variable_ast_types
                    .insert(name.to_string(), final_var_type.clone());

                // Track struct name for field access
                match final_var_type {
                    Type::Named(type_name) => {
                        eprintln!(
                            "📝 Registering variable '{}' with struct name: {}",
                            name, type_name
                        );
                        self.variable_struct_names
                            .insert(name.to_string(), type_name.clone());
                    }
                    Type::Vec(inner_ty) => {
                        let mangled = format!("Vec_{}", self.type_to_string(inner_ty.as_ref()));
                        self.variable_struct_names.insert(name.to_string(), mangled);
                    }
                    Type::Box(inner_ty) => {
                        let mangled = format!("Box_{}", self.type_to_string(inner_ty.as_ref()));
                        self.variable_struct_names.insert(name.to_string(), mangled);
                    }
                    Type::Channel(_inner_ty) => {
                        // Channel is not generic in LLVM representation
                        self.variable_struct_names
                            .insert(name.to_string(), "Channel".to_string());
                    }
                    Type::Option(inner_ty) => {
                        let mangled = format!("Option_{}", self.type_to_string(inner_ty.as_ref()));
                        self.variable_struct_names.insert(name.to_string(), mangled);
                    }
                    Type::Result(ok_ty, err_ty) => {
                        let mangled = format!(
                            "Result_{}_{}",
                            self.type_to_string(ok_ty.as_ref()),
                            self.type_to_string(err_ty.as_ref())
                        );
                        self.variable_struct_names.insert(name.to_string(), mangled);
                    }
                    Type::Slice(_, _) => {
                        // Slice doesn't need struct name tracking
                        // Type is already tracked in variable_types as struct
                    }
                    Type::Generic {
                        name: struct_name,
                        type_args,
                    } => {
                        // Build mangled name for generic structs
                        let type_suffix = type_args
                            .iter()
                            .map(|t| self.type_to_string(t))
                            .collect::<Vec<_>>()
                            .join("_");
                        let mangled_name = format!("{}_{}", struct_name, type_suffix);

                        eprintln!(
                            "📝 Registering Generic variable '{}' with struct name: {}",
                            name, mangled_name
                        );

                        // Instantiate if not already done
                        if !self.struct_defs.contains_key(&mangled_name) {
                            let _ = self.instantiate_generic_struct(struct_name, type_args);
                        }

                        self.variable_struct_names
                            .insert(name.to_string(), mangled_name);
                    }
                    _ => {}
                }
            }
        } else if let BasicValueEnum::StructValue(_struct_val) = val {
            eprintln!("📝 Struct value received for '{}', creating alloca", name);
            let alloca = self.create_entry_block_alloca(name, final_var_type, is_mutable)?;
            self.build_store_aligned(alloca, val)?;
            self.variables.insert(name.to_string(), alloca);
            self.variable_types.insert(name.to_string(), val.get_type());

            // Track struct name for field access
            match final_var_type {
                Type::Named(type_name) => {
                    eprintln!(
                        "📝 Registering struct variable '{}' with name: {}",
                        name, type_name
                    );
                    self.variable_struct_names
                        .insert(name.to_string(), type_name.clone());
                }
                Type::Vec(inner_ty) => {
                    let mangled = format!("Vec_{}", self.type_to_string(inner_ty.as_ref()));
                    eprintln!(
                        "📝 Registering Vec variable '{}' with struct name: {}",
                        name, mangled
                    );
                    self.variable_struct_names.insert(name.to_string(), mangled);
                }
                Type::Box(inner_ty) => {
                    let mangled = format!("Box_{}", self.type_to_string(inner_ty.as_ref()));
                    self.variable_struct_names.insert(name.to_string(), mangled);
                }
                Type::Generic {
                    name: struct_name,
                    type_args,
                } => {
                    if let Ok(mangled_name) =
                        self.instantiate_generic_struct(struct_name, type_args)
                    {
                        self.variable_struct_names
                            .insert(name.to_string(), mangled_name);
                    }
                }
                _ => {}
            }
        } else if let BasicValueEnum::ArrayValue(_array_val) = val {
            // Array values need to be stored in an alloca
            // Try to use cached pointer from array literal compilation
            let alloca = if let Some(cached_ptr) = self.last_compiled_array_ptr.take() {
                // Use the pointer from array literal (avoids double alloca)
                cached_ptr
            } else {
                // Fallback: create new alloca (for non-literal arrays)
                let alloca = self.create_entry_block_alloca(name, final_var_type, is_mutable)?;
                self.build_store_aligned(alloca, val)?;
                alloca
            };

            self.variables.insert(name.to_string(), alloca);
            self.variable_types
                .insert(name.to_string(), final_llvm_type);
            self.variable_ast_types
                .insert(name.to_string(), final_var_type.clone());
        } else {
            return Err(format!(
                "Struct/Tuple/Array literal should return pointer, struct value, or array value, got {:?}",
                val
            ));
        }

        Ok(())
    }

    fn build_generic_type(&self, type_name: &str, type_args: &[Type]) -> Type {
        match type_name {
            "Vec" if type_args.len() == 1 => Type::Vec(Box::new(type_args[0].clone())),
            "Box" if type_args.len() == 1 => Type::Box(Box::new(type_args[0].clone())),
            "Option" if type_args.len() == 1 => Type::Option(Box::new(type_args[0].clone())),
            "Result" if type_args.len() == 2 => Type::Result(
                Box::new(type_args[0].clone()),
                Box::new(type_args[1].clone()),
            ),
            _ => Type::Generic {
                name: type_name.to_string(),
                type_args: type_args.to_vec(),
            },
        }
    }

    fn register_regular_variable(
        &mut self,
        name: &str,
        val: BasicValueEnum<'ctx>,
        final_var_type: &Type,
        final_llvm_type: BasicTypeEnum<'ctx>,
        is_mutable: bool,
    ) -> Result<(), String> {
        let alloca = self.create_entry_block_alloca(name, final_var_type, is_mutable)?;
        self.build_store_aligned(alloca, val)?;
        self.variables.insert(name.to_string(), alloca);
        self.variable_types
            .insert(name.to_string(), final_llvm_type);
        self.variable_ast_types
            .insert(name.to_string(), final_var_type.clone());

        Ok(())
    }
}
