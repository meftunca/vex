// Expression code generation
// This module dispatches expression compilation and coordinates submodules

use super::ASTCodeGen;
use crate::diagnostics::{error_codes, Diagnostic, ErrorLevel, Span};
use inkwell::types::BasicTypeEnum;
use inkwell::values::BasicValueEnum;
use vex_ast::*;

// Submodules
mod access;
mod binary_ops;
mod calls;
mod control;
mod literals;
pub(crate) mod pattern_matching;
mod special;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Main expression compiler - dispatches to specialized methods
    pub(crate) fn compile_expression(
        &mut self,
        expr: &Expression,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        match expr {
            Expression::IntLiteral(n) => {
                Ok(self.context.i32_type().const_int(*n as u64, false).into())
            }

            Expression::BigIntLiteral(s) => {
                // Parse large integer literals for i128/u128
                // Remove any prefix (0x, 0b, 0o) and parse accordingly
                if s.starts_with("0x") || s.starts_with("0X") {
                    // Hexadecimal
                    let hex_str = &s[2..];
                    if u128::from_str_radix(hex_str, 16).is_ok() {
                        Ok(self
                            .context
                            .i128_type()
                            .const_int_from_string(
                                hex_str,
                                inkwell::types::StringRadix::Hexadecimal,
                            )
                            .unwrap()
                            .into())
                    } else {
                        Err(format!("Invalid hexadecimal BigIntLiteral: {}", s))
                    }
                } else if s.starts_with("0b") || s.starts_with("0B") {
                    // Binary
                    let bin_str = &s[2..];
                    if u128::from_str_radix(bin_str, 2).is_ok() {
                        Ok(self
                            .context
                            .i128_type()
                            .const_int_from_string(bin_str, inkwell::types::StringRadix::Binary)
                            .unwrap()
                            .into())
                    } else {
                        Err(format!("Invalid binary BigIntLiteral: {}", s))
                    }
                } else if s.starts_with("0o") || s.starts_with("0O") {
                    // Octal
                    let oct_str = &s[2..];
                    if u128::from_str_radix(oct_str, 8).is_ok() {
                        Ok(self
                            .context
                            .i128_type()
                            .const_int_from_string(oct_str, inkwell::types::StringRadix::Octal)
                            .unwrap()
                            .into())
                    } else {
                        Err(format!("Invalid octal BigIntLiteral: {}", s))
                    }
                } else {
                    // Decimal
                    if s.parse::<u128>().is_ok() {
                        Ok(self
                            .context
                            .i128_type()
                            .const_int_from_string(s, inkwell::types::StringRadix::Decimal)
                            .unwrap()
                            .into())
                    } else {
                        Err(format!("Invalid decimal BigIntLiteral: {}", s))
                    }
                }
            }

            Expression::FloatLiteral(f) => Ok(self.context.f64_type().const_float(*f).into()),

            Expression::BoolLiteral(b) => {
                Ok(self.context.bool_type().const_int(*b as u64, false).into())
            }

            Expression::StringLiteral(s) => {
                // Create global string constant
                let global_str = self
                    .builder
                    .build_global_string_ptr(s, "str")
                    .map_err(|e| format!("Failed to create string: {}", e))?;
                Ok(global_str.as_pointer_value().into())
            }

            Expression::FStringLiteral(s) => {
                // For now, handle F-strings as formatted strings with interpolation
                self.compile_fstring(s)
            }

            Expression::Ident(name) => {
                // Check if this is a builtin function first (before variables/functions)
                // This ensures builtins like print() work inside method bodies
                if self.builtins.is_builtin(name) {
                    // This is a builtin function - return a dummy function pointer
                    // The actual builtin will be called via compile_call()
                    // For now, return a zero pointer (builtins are handled specially)
                    return Err(format!(
                        "Builtin function '{}' cannot be used as a value (must be called directly)",
                        name
                    ));
                }

                // Check global constants FIRST (never cleared during function compilation)
                if let Some(ptr) = self.global_constants.get(name) {
                    let ty = self
                        .global_constant_types
                        .get(name)
                        .ok_or_else(|| format!("Type for constant {} not found", name))?;

                    // Load the constant value
                    let loaded = self.build_load_aligned(*ty, *ptr, name)?;
                    return Ok(loaded);
                }

                // Check if this is a function pointer parameter first
                if let Some(fn_ptr) = self.function_params.get(name) {
                    // Return function pointer directly
                    return Ok((*fn_ptr).into());
                }

                // Check if this is a variable (includes regular parameters)
                if let Some(ptr) = self.variables.get(name) {
                    let ty = self
                        .variable_types
                        .get(name)
                        .ok_or_else(|| format!("Type for variable {} not found", name))?;

                    if name == "result" {
                        eprintln!("[DEBUG result] Variable 'result' type: {:?}", ty);
                        eprintln!(
                            "[DEBUG result] Is in variable_struct_names: {}",
                            self.variable_struct_names.contains_key(name)
                        );
                    }

                    // IMPORTANT: For struct variables, return the pointer directly (zero-copy)
                    // Struct types in LLVM are already pointers (ast_type_to_llvm returns pointer for structs)
                    if self.variable_struct_names.contains_key(name) {
                        // This is a struct variable - return pointer without loading
                        if name == "result" {
                            eprintln!("[DEBUG result] Returning pointer without loading");
                        }
                        return Ok((*ptr).into());
                    }

                    // Use alignment-aware load to fix memory corruption
                    let loaded = self.build_load_aligned(*ty, *ptr, name)?;

                    if name == "t" {
                        eprintln!(
                            "[DEBUG VAR LOAD] Loaded value is_struct: {}",
                            loaded.is_struct_value()
                        );
                        eprintln!(
                            "[DEBUG VAR LOAD] Loaded value is_pointer: {}",
                            loaded.is_pointer_value()
                        );
                    }

                    return Ok(loaded);
                }

                // Check if this is a global function name (for function pointers)
                if let Some(func_val) = self.functions.get(name) {
                    // Return function as a pointer value
                    return Ok(func_val.as_global_value().as_pointer_value().into());
                }

                // Variable/function not found - find similar names for suggestion
                let mut candidates: Vec<String> = self.variables.keys().cloned().collect();
                candidates.extend(self.functions.keys().cloned());

                use crate::diagnostics::fuzzy;
                let suggestions = fuzzy::find_similar_names(name, &candidates, 0.7, 3);

                let mut help_msg =
                    "Check that the name is spelled correctly and is in scope".to_string();
                if !suggestions.is_empty() {
                    help_msg = format!("did you mean `{}`?", suggestions.join("`, `"));
                }

                self.diagnostics.emit(Diagnostic {
                    level: ErrorLevel::Error,
                    code: error_codes::UNDEFINED_VARIABLE.to_string(),
                    message: format!("Cannot find variable or function `{}` in this scope", name),
                    span: Span::unknown(),
                    notes: vec![],
                    help: Some(help_msg),
                    suggestion: None,
                });
                Err(format!("Variable or function {} not found", name))
            }

            Expression::Binary {
                span_id: _,
                left,
                op,
                right,
            } => self.compile_binary_op(left, op, right),

            Expression::Unary {
                span_id: _,
                op,
                expr,
            } => self.compile_unary_op(op, expr),

            Expression::Call {
                func,
                type_args,
                args,
                ..
            } => self.compile_call(func, type_args, args),

            Expression::MethodCall {
                receiver,
                method,
                type_args,
                args,
                is_mutable_call,
            } => self.compile_method_call(receiver, method, type_args, args, *is_mutable_call),

            Expression::Index { object, index } => self.compile_index(object, index),

            Expression::Array(elements) => self.compile_array_literal(elements),

            Expression::ArrayRepeat(value, count) => {
                self.compile_array_repeat_literal(value, count)
            }

            Expression::MapLiteral(entries) => self.compile_map_literal(entries),

            Expression::TupleLiteral(elements) => self.compile_tuple_literal(elements),

            Expression::StructLiteral {
                name,
                type_args,
                fields,
            } => self.compile_struct_literal(name, type_args, fields),

            Expression::FieldAccess { object, field } => self.compile_field_access(object, field),

            Expression::PostfixOp { expr, op } => self.compile_postfix_op(expr, op),

            Expression::Await(expr) => {
                // Await expression: suspend coroutine and yield to scheduler
                // 1. Compile the future expression
                // 2. Check if it's ready (for now, assume always ready - TODO: poll)
                // 3. Call worker_await_after to yield control
                // 4. Return CORO_STATUS_YIELDED

                let _future_val = self.compile_expression(expr)?;

                // Get current WorkerContext (first parameter of resume function)
                let current_fn = self
                    .current_function
                    .ok_or_else(|| "Await outside of function".to_string())?;

                // Check if we're in an async function (resume function has WorkerContext* param)
                let is_in_async = current_fn
                    .get_name()
                    .to_str()
                    .map(|n| n.ends_with("_resume"))
                    .unwrap_or(false);

                if !is_in_async {
                    return Err("Await can only be used inside async functions".to_string());
                }

                // Get WorkerContext parameter (first param)
                let ctx_param = current_fn
                    .get_nth_param(0)
                    .ok_or_else(|| "Missing WorkerContext parameter".to_string())?
                    .into_pointer_value();

                // Call worker_await_after(ctx, 0) to yield
                let worker_await_fn = self.get_or_declare_worker_await();
                self.builder
                    .build_call(
                        worker_await_fn,
                        &[
                            ctx_param.into(),
                            self.context.i64_type().const_int(0, false).into(),
                        ],
                        "await_yield",
                    )
                    .map_err(|e| format!("Failed to call worker_await_after: {}", e))?;

                // Return CORO_STATUS_YIELDED (1)
                let yielded_status = self.context.i32_type().const_int(1, false);
                self.builder
                    .build_return(Some(&yielded_status))
                    .map_err(|e| format!("Failed to build await return: {}", e))?;

                // For type system compatibility, return a dummy value
                // (this code is unreachable after return)
                Ok(self.context.i8_type().const_int(0, false).into())
            }

            Expression::Nil => {
                // Return zero/null for nil
                Ok(self.context.i8_type().const_int(0, false).into())
            }

            Expression::Match { value, arms } => self.compile_match_expression(value, arms),

            Expression::Block {
                statements,
                return_expr,
            } => self.compile_block_expression(statements, return_expr),

            Expression::QuestionMark(expr) => {
                // ? operator: Unwrap Result, propagate Err
                // Desugar: let x = expr? => match expr { Ok(v) => v, Err(e) => return Err(e) }

                // Compile the Result expression
                let result_val = self.compile_expression(expr)?;

                // Check if this is a Result/Option enum (has tag + data struct)
                if !result_val.is_struct_value() {
                    return Err("? operator can only be used on Result/Option enums".to_string());
                }

                // Result is a struct value, but we need to work with it on stack
                // Allocate temporary space and store it
                let result_ptr = self
                    .builder
                    .build_alloca(result_val.get_type(), "result_tmp")
                    .map_err(|e| format!("Failed to allocate result temp: {}", e))?;

                self.builder
                    .build_store(result_ptr, result_val)
                    .map_err(|e| format!("Failed to store result: {}", e))?;

                // Extract tag (field 0)
                let tag_ptr = self
                    .builder
                    .build_struct_gep(result_val.get_type(), result_ptr, 0, "tag_ptr")
                    .map_err(|e| format!("Failed to get tag pointer: {}", e))?;

                let tag = self
                    .builder
                    .build_load(self.context.i32_type(), tag_ptr, "tag")
                    .map_err(|e| format!("Failed to load tag: {}", e))?
                    .into_int_value();

                // Extract data (field 1)
                let data_ptr = self
                    .builder
                    .build_struct_gep(result_val.get_type(), result_ptr, 1, "data_ptr")
                    .map_err(|e| format!("Failed to get data pointer: {}", e))?;

                // Create blocks for Ok and Err paths
                let ok_block = self.context.append_basic_block(
                    self.current_function.ok_or("? operator outside function")?,
                    "try_ok",
                );
                let err_block = self.context.append_basic_block(
                    self.current_function.ok_or("? operator outside function")?,
                    "try_err",
                );
                let merge_block = self.context.append_basic_block(
                    self.current_function.ok_or("? operator outside function")?,
                    "try_merge",
                );

                // Check if tag == 0 (Ok variant)
                let is_ok = self
                    .builder
                    .build_int_compare(
                        inkwell::IntPredicate::EQ,
                        tag,
                        self.context.i32_type().const_int(0, false),
                        "is_ok",
                    )
                    .map_err(|e| format!("Failed to compare tag: {}", e))?;

                // Branch: if Ok goto ok_block, else goto err_block
                self.builder
                    .build_conditional_branch(is_ok, ok_block, err_block)
                    .map_err(|e| format!("Failed to build conditional branch: {}", e))?;

                // Ok block: unwrap data and continue
                self.builder.position_at_end(ok_block);
                let data_type = self.context.i32_type(); // TODO: Infer from Result<T, E>
                let ok_value = self
                    .builder
                    .build_load(data_type, data_ptr, "ok_value")
                    .map_err(|e| format!("Failed to load ok value: {}", e))?;
                self.builder
                    .build_unconditional_branch(merge_block)
                    .map_err(|e| format!("Failed to branch to merge: {}", e))?;

                // Err block: early return with Err
                self.builder.position_at_end(err_block);

                // Execute deferred statements before early return
                self.execute_deferred_statements()?;

                // Return the error Result value
                self.builder
                    .build_return(Some(&result_val))
                    .map_err(|e| format!("Failed to build error return: {}", e))?;

                // Merge block: continue with unwrapped value
                self.builder.position_at_end(merge_block);

                Ok(ok_value)
            }

            Expression::Reference { is_mutable, expr } => {
                // Take a reference to an expression: &expr or &expr!
                // This creates a pointer to the value
                match expr.as_ref() {
                    Expression::Ident(name) => {
                        // For identifiers, return the pointer directly (don't load)
                        let ptr = self
                            .variables
                            .get(name)
                            .ok_or_else(|| format!("Variable {} not found", name))?;
                        Ok((*ptr).into())
                    }
                    _ => {
                        // For other expressions, compile them, store in temporary, return pointer
                        let value = self.compile_expression(expr)?;
                        let value_type = value.get_type();
                        let temp_name = if *is_mutable {
                            "ref_temp_mut"
                        } else {
                            "ref_temp"
                        };
                        let temp_ptr =
                            self.builder
                                .build_alloca(value_type, temp_name)
                                .map_err(|e| {
                                    format!("Failed to allocate reference temporary: {}", e)
                                })?;
                        self.builder
                            .build_store(temp_ptr, value)
                            .map_err(|e| format!("Failed to store reference temporary: {}", e))?;
                        Ok(temp_ptr.into())
                    }
                }
            }

            Expression::Deref(expr) => {
                // Dereference a pointer: *ptr
                // Try to infer the inner type from the expression
                match expr.as_ref() {
                    Expression::Ident(name) => {
                        // For identifiers, we can load using the stored LLVM type
                        let ptr = self
                            .variables
                            .get(name)
                            .ok_or_else(|| format!("Variable {} not found", name))?;
                        let var_type = self
                            .variable_types
                            .get(name)
                            .ok_or_else(|| format!("Type for variable {} not found", name))?;

                        // Load the pointer value first (variables store the reference)
                        let ptr_loaded = self
                            .builder
                            .build_load(*var_type, *ptr, &format!("{}_ptr", name))
                            .map_err(|e| format!("Failed to load pointer variable: {}", e))?;

                        if !ptr_loaded.is_pointer_value() {
                            return Err(format!(
                                "Cannot dereference non-pointer variable {}",
                                name
                            ));
                        }

                        // Now dereference the pointer
                        // For references, the inner type is what we need to load
                        // Since we don't track AST types, we'll use a heuristic:
                        // Try common types (i32, i64, f64, bool)
                        // TODO: Add proper AST type tracking for variables
                        let deref_ptr = ptr_loaded.into_pointer_value();

                        // Try to determine the pointee type
                        // For now, default to i32 (most common case)
                        let inner_type = self.context.i32_type();
                        let loaded = self
                            .builder
                            .build_load(inner_type, deref_ptr, "deref")
                            .map_err(|e| format!("Failed to dereference pointer: {}", e))?;
                        Ok(loaded)
                    }
                    _ => {
                        // For other expressions, compile and dereference
                        let ptr_value = self.compile_expression(expr)?;
                        if !ptr_value.is_pointer_value() {
                            return Err("Cannot dereference non-pointer value".to_string());
                        }
                        let ptr = ptr_value.into_pointer_value();

                        // Default to i32 for now
                        let inner_type = self.context.i32_type();
                        let loaded = self
                            .builder
                            .build_load(inner_type, ptr, "deref")
                            .map_err(|e| format!("Failed to dereference pointer: {}", e))?;
                        Ok(loaded)
                    }
                }
            }

            Expression::ChannelReceive(channel_expr) => {
                // Channel receive operator: <-ch
                // Desugar to method call: ch.recv()
                let recv_call = Expression::MethodCall {
                    receiver: channel_expr.clone(),
                    method: "recv".to_string(),
                    type_args: vec![],
                    args: vec![],
                    is_mutable_call: true,
                };
                self.compile_expression(&recv_call)
            }

            Expression::EnumLiteral {
                enum_name,
                variant,
                data,
            } => {
                // Phase 0.4: Handle builtin enums (Option, Result) specially
                eprintln!("🟡 Compiling EnumLiteral: {}::{}", enum_name, variant);
                if enum_name == "Option" || enum_name == "Result" {
                    eprintln!("   → Calling compile_builtin_enum_literal");
                    return self.compile_builtin_enum_literal(enum_name, variant, data);
                }

                // For now, treat enums as tagged unions (C-style for unit variants)
                // Unit variants (no data): Just return the tag as i32
                // Data-carrying variants: Need struct with tag + data (TODO: full implementation)

                // Look up enum definition
                if let Some(enum_def) = self.enum_ast_defs.get(enum_name) {
                    // Find variant index
                    let variant_index = enum_def
                        .variants
                        .iter()
                        .position(|v| &v.name == variant)
                        .ok_or_else(|| {
                        format!("Variant {} not found in enum {}", variant, enum_name)
                    })?;

                    // Check if enum has ANY data-carrying variants
                    let enum_has_data = enum_def.variants.iter().any(|v| !v.data.is_empty());

                    if data.is_empty() && !enum_has_data {
                        // Pure unit enum (all variants are unit): return tag as i32 for compatibility
                        // (Variables expect i32, return statements expect i32)
                        let tag = self
                            .context
                            .i32_type()
                            .const_int(variant_index as u64, false);
                        Ok(tag.into())
                    } else {
                        // Mixed enum (has data variants): create struct { tag: i32, data: T }
                        // For unit variants in mixed enums, use enum's data type from definition

                        let (data_value, actual_data_type) = if !data.is_empty() {
                            // Variant has data: compile all expressions
                            if data.len() == 1 {
                                // Single-value tuple: compile directly
                                let val = self.compile_expression(&data[0])?;
                                let ty = val.get_type();
                                (val, ty)
                            } else {
                                // Multi-value tuple: create struct with all values
                                let mut field_values = Vec::new();
                                let mut field_types = Vec::new();

                                for expr in data {
                                    let val = self.compile_expression(expr)?;
                                    let ty = val.get_type();
                                    field_values.push(val);
                                    field_types.push(ty);
                                }

                                // Create tuple struct type
                                let tuple_struct_type =
                                    self.context.struct_type(&field_types, false);

                                // Build tuple value
                                let mut tuple_val = tuple_struct_type.get_undef();
                                for (i, val) in field_values.iter().enumerate() {
                                    tuple_val = self
                                        .builder
                                        .build_insert_value(
                                            tuple_val,
                                            *val,
                                            i as u32,
                                            &format!("tuple_field_{}", i),
                                        )
                                        .map_err(|e| {
                                            format!("Failed to insert tuple field: {}", e)
                                        })?
                                        .into_struct_value();
                                }

                                (tuple_val.into(), tuple_struct_type.into())
                            }
                        } else {
                            // Unit variant in mixed enum: use enum's largest data type with zero value
                            let enum_llvm_type =
                                self.ast_type_to_llvm(&Type::Named(enum_name.clone()));
                            if let BasicTypeEnum::StructType(struct_ty) = enum_llvm_type {
                                // Extract data field type (index 1)
                                let data_field_type = struct_ty
                                    .get_field_type_at_index(1)
                                    .ok_or_else(|| "Enum struct missing data field".to_string())?;
                                // Create zero/undef value for data field
                                let zero_value = match data_field_type {
                                    BasicTypeEnum::IntType(int_ty) => int_ty.const_zero().into(),
                                    BasicTypeEnum::FloatType(float_ty) => {
                                        float_ty.const_zero().into()
                                    }
                                    BasicTypeEnum::PointerType(ptr_ty) => {
                                        ptr_ty.const_null().into()
                                    }
                                    _ => {
                                        return Err(format!(
                                            "Unsupported enum data type: {:?}",
                                            data_field_type
                                        ))
                                    }
                                };
                                (zero_value, data_field_type)
                            } else {
                                return Err(format!(
                                    "Expected struct type for mixed enum {}",
                                    enum_name
                                ));
                            }
                        };

                        // Create struct type: { i32, T } (consistent with ast_type_to_llvm)
                        let tag_type = self.context.i32_type();
                        let struct_type = self
                            .context
                            .struct_type(&[tag_type.into(), actual_data_type], false);

                        // Allocate struct on stack
                        let struct_ptr = self
                            .builder
                            .build_alloca(struct_type, "enum_data_carrier")
                            .map_err(|e| format!("Failed to allocate enum struct: {}", e))?;

                        // Store tag at index 0 (i32 type - consistent with struct definition)
                        let tag = self
                            .context
                            .i32_type()
                            .const_int(variant_index as u64, false);
                        let tag_ptr = self
                            .builder
                            .build_struct_gep(struct_type, struct_ptr, 0, "enum_tag_ptr")
                            .map_err(|e| format!("Failed to get tag pointer: {}", e))?;
                        self.builder
                            .build_store(tag_ptr, tag)
                            .map_err(|e| format!("Failed to store tag: {}", e))?;

                        // Store data at index 1
                        let data_ptr = self
                            .builder
                            .build_struct_gep(struct_type, struct_ptr, 1, "enum_data_ptr")
                            .map_err(|e| format!("Failed to get data pointer: {}", e))?;
                        self.builder
                            .build_store(data_ptr, data_value)
                            .map_err(|e| format!("Failed to store data: {}", e))?;

                        // Load and return the struct value
                        let struct_value = self
                            .builder
                            .build_load(struct_type, struct_ptr, "enum_with_data")
                            .map_err(|e| format!("Failed to load enum struct: {}", e))?;

                        Ok(struct_value)
                    }
                } else {
                    Err(format!("Enum {} not found", enum_name))
                }
            }

            Expression::Closure {
                params,
                return_type,
                body,
                capture_mode,
            } => self.compile_closure(params, return_type, body, capture_mode),

            Expression::Cast { expr, target_type } => {
                self.compile_cast_expression(expr, target_type)
            }

            Expression::Range { start, end } => self.compile_range(start, end, false),

            Expression::RangeInclusive { start, end } => self.compile_range(start, end, true),

            Expression::TypeConstructor {
                type_name,
                type_args: _,
                args,
            } => {
                // Type constructor: Vec(), Point(10, 20)
                // Desugar to static method call: Type.new(args)
                let method_call = Expression::MethodCall {
                    receiver: Box::new(Expression::Ident(type_name.clone())),
                    method: "new".to_string(),
                    type_args: vec![], // No generic args for simple constructor syntax
                    args: args.clone(),
                    is_mutable_call: false,
                };

                // Compile as static method call (handled in compile_method_call)
                self.compile_expression(&method_call)
            }

            _ => {
                let expr_str = format!("{:?}", expr);
                self.diagnostics.emit(Diagnostic {
                    level: ErrorLevel::Error,
                    code: error_codes::NOT_IMPLEMENTED.to_string(),
                    message: "This expression type is not yet implemented".to_string(),
                    span: Span::unknown(),
                    notes: vec![format!("Expression: {}", expr_str)],
                    help: Some("This feature is planned for a future release".to_string()),
                    suggestion: None,
                });
                Err(format!("Expression not yet implemented: {:?}", expr))
            }
        }
    }

    /// Compile Range or RangeInclusive expressions
    fn compile_range(
        &mut self,
        start: &Option<Box<Expression>>,
        end: &Option<Box<Expression>>,
        _inclusive: bool,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Default values: start=0, end=max_i64
        let zero_i64 = self.context.i64_type().const_int(0, false);
        let max_i64 = self.context.i64_type().const_int(i64::MAX as u64, false);

        // Compile start expression or use 0
        let start_i64 = if let Some(s) = start {
            let start_val = self.compile_expression(s)?;
            if start_val.is_int_value() {
                let int_val = start_val.into_int_value();
                if int_val.get_type().get_bit_width() < 64 {
                    self.builder
                        .build_int_s_extend(int_val, self.context.i64_type(), "start_ext")
                        .map_err(|e| format!("Failed to extend start: {}", e))?
                } else {
                    int_val
                }
            } else {
                return Err("Range start must be an integer".to_string());
            }
        } else {
            zero_i64
        };

        // Compile end expression or use max_i64
        let end_i64 = if let Some(e) = end {
            let end_val = self.compile_expression(e)?;
            if end_val.is_int_value() {
                let int_val = end_val.into_int_value();
                if int_val.get_type().get_bit_width() < 64 {
                    self.builder
                        .build_int_s_extend(int_val, self.context.i64_type(), "end_ext")
                        .map_err(|e| format!("Failed to extend end: {}", e))?
                } else {
                    int_val
                }
            } else {
                return Err("Range end must be an integer".to_string());
            }
        } else {
            max_i64
        };

        // Create Range struct: { start: i64, end: i64, current: i64 }
        let range_struct_type = self.context.struct_type(
            &[
                self.context.i64_type().into(),
                self.context.i64_type().into(),
                self.context.i64_type().into(),
            ],
            false,
        );

        // Build struct value
        let mut range_val = range_struct_type.get_undef();
        range_val = self
            .builder
            .build_insert_value(range_val, start_i64, 0, "range_start")
            .map_err(|e| format!("Failed to insert start: {}", e))?
            .into_struct_value();
        range_val = self
            .builder
            .build_insert_value(range_val, end_i64, 1, "range_end")
            .map_err(|e| format!("Failed to insert end: {}", e))?
            .into_struct_value();
        range_val = self
            .builder
            .build_insert_value(range_val, start_i64, 2, "range_current")
            .map_err(|e| format!("Failed to insert current: {}", e))?
            .into_struct_value();

        Ok(range_val.into())
    }
}
