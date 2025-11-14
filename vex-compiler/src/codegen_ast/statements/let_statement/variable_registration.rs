// Variable registration and type determination

use crate::codegen_ast::ASTCodeGen;
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

            // Cast integer literal to match target integer type width
            val = self.cast_integer_if_needed(val, t, target_llvm_type)?;

            // Cast float literals to match target float type
            val = self.cast_float_if_needed(val, target_llvm_type)?;

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
    ) -> Result<BasicValueEnum<'ctx>, String> {
        if let BasicValueEnum::IntValue(int_val) = val {
            if let BasicTypeEnum::IntType(target_int_type) = target_llvm_type {
                if int_val.get_type().get_bit_width() != target_int_type.get_bit_width() {
                    if int_val.get_type().get_bit_width() < target_int_type.get_bit_width() {
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
                        return self
                            .builder
                            .build_int_truncate(int_val, target_int_type, "lit_trunc")
                            .map_err(|e| format!("Failed to truncate literal: {}", e))
                            .map(|v| v.into());
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
    ) -> Result<BasicValueEnum<'ctx>, String> {
        if let BasicValueEnum::FloatValue(float_val) = val {
            if let BasicTypeEnum::FloatType(target_float_type) = target_llvm_type {
                if float_val.get_type() != target_float_type {
                    return self
                        .builder
                        .build_float_cast(float_val, target_float_type, "float_cast")
                        .map_err(|e| format!("Failed to cast float: {}", e))
                        .map(|v| v.into());
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
            let alloca = self.create_entry_block_alloca(name, final_var_type, is_mutable)?;
            self.build_store_aligned(alloca, val)?;
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
