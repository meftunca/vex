// statements/let_statement.rs
// let statement + recursive injection of type args

use super::ASTCodeGen;
use inkwell::types::BasicTypeEnum;
use inkwell::values::BasicValueEnum;
use vex_ast::*;
impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile a `let` statement
    pub(crate) fn compile_let_statement(
        &mut self,
        is_mutable: bool,
        name: &String,
        ty: Option<&Type>,
        value: &Expression,
    ) -> Result<(), String> {
        // v0.1: is_mutable determines if variable is mutable (let vs let!)
        // FIRST: Determine struct name from expression BEFORE compiling
        // (because after compilation, we lose the expression structure)
        // eprintln!(
        //     "🔷 Let statement: var={}, has_type_annotation={}",
        //     name,
        //     ty.is_some()
        // );
        let struct_name_from_expr = if ty.is_none() {
            eprintln!("  → Type inference needed, analyzing expression...");
            match value {
                Expression::StructLiteral {
                    name: s_name,
                    type_args,
                    ..
                } => {
                    // eprintln!("  → StructLiteral: {}", s_name);

                    // Handle generic struct literals: Box<i32> -> Box_i32
                    if !type_args.is_empty() {
                        // Instantiate the generic struct to get the mangled name
                        match self.instantiate_generic_struct(s_name, type_args) {
                            Ok(mangled_name) => Some(mangled_name),
                            Err(_) => None,
                        }
                    } else if self.struct_defs.contains_key(s_name) {
                        Some(s_name.clone())
                    } else {
                        None
                    }
                }
                Expression::MethodCall {
                    receiver, method, ..
                } => {
                    // eprintln!("  → MethodCall expression");
                    // eprintln!("    → Method: {}", method);

                    // Check for static method calls: Type.new() -> Type
                    if let Expression::Ident(potential_type_name) = receiver.as_ref() {
                        // Check if this is a type name (PascalCase)
                        let is_type_name = potential_type_name
                            .chars()
                            .next()
                            .map(|c| c.is_uppercase())
                            .unwrap_or(false);

                        // Check if NOT a variable (static methods are on types)
                        let is_not_variable = !self.variables.contains_key(potential_type_name);

                        if is_type_name && is_not_variable {
                            // Static method call like Vec.new(), Box.new()
                            // Return the type name as the struct name
                            // eprintln!("    ✅ Static method {}.{}() -> {}", potential_type_name, method, potential_type_name);
                            Some(potential_type_name.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                    // If not a static method, continue with instance method logic
                    .or_else(|| {
                        // Special handling for builtin method calls that return specific types
                        let builtin_return_type = if let Expression::Ident(var_name) =
                            receiver.as_ref()
                        {
                            if let Some(struct_name) = self.variable_struct_names.get(var_name) {
                                // Check for Vec.as_slice() -> Slice
                                if struct_name == "Vec" && method == "as_slice" {
                                    // eprintln!("    ✅ Vec.as_slice() -> Slice");
                                    Some("Slice".to_string())
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        };

                        if builtin_return_type.is_some() {
                            builtin_return_type
                        } else {
                            // Get struct type from receiver
                            let struct_name = if let Expression::Ident(var_name) = receiver.as_ref()
                            {
                                if var_name == "self" {
                                    self.variable_struct_names.get(var_name).cloned()
                                } else {
                                    self.variable_struct_names.get(var_name).cloned()
                                }
                            } else {
                                None
                            };

                            if let Some(struct_name) = struct_name {
                                let method_func_name = format!("{}_{}", struct_name, method);
                                if let Some(func_def) = self.function_defs.get(&method_func_name) {
                                    if let Some(Type::Named(s_name)) = &func_def.return_type {
                                        Some(s_name.clone())
                                    } else if let Some(Type::Option(_)) = &func_def.return_type {
                                        Some("Option".to_string())
                                    } else if let Some(Type::Result(_, _)) = &func_def.return_type {
                                        Some("Result".to_string())
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        }
                    })
                }
                Expression::TypeConstructor {
                    type_name,
                    type_args,
                    ..
                } => {
                    // TypeConstructor: Vec(), Box(42), etc.
                    // This desugars to Type.new() but we handle it here for type inference
                    // eprintln!("  ✅ TypeConstructor: {} -> {}", type_name, type_name);

                    // Handle generic type constructors: Vec<i32>() -> Vec_i32
                    if !type_args.is_empty() {
                        // Instantiate the generic type to get the mangled name
                        if let Ok(mangled_name) =
                            self.instantiate_generic_struct(type_name, type_args)
                        {
                            Some(mangled_name)
                        } else {
                            Some(type_name.clone())
                        }
                    } else {
                        Some(type_name.clone())
                    }
                }
                Expression::Range { .. } => {
                    // eprintln!("  → Range expression (0..10)");
                    Some("Range".to_string())
                }
                Expression::RangeInclusive { .. } => {
                    // eprintln!("  → RangeInclusive expression (0..=10)");
                    Some("RangeInclusive".to_string())
                }
                Expression::Array(_) | Expression::ArrayRepeat(_, _) => {
                    // eprintln!("  → Array literal or repeat");
                    None // Arrays don't have struct names, they're stack-allocated
                }
                Expression::Call {
                    span_id: _,
                    func,
                    type_args,
                    ..
                } => {
                    // eprintln!("  → Call expression");
                    if let Expression::Ident(func_name) = func.as_ref() {
                        // eprintln!("    → Function: {}", func_name);

                        // Phase 0.4b: Check for builtin constructors
                        match func_name.as_str() {
                            "vec_new" | "vec_with_capacity" => {
                                // eprintln!("    ✅ Builtin vec_new() -> Vec");
                                Some("Vec".to_string())
                            }
                            "box_new" => {
                                // eprintln!("    ✅ Builtin box_new() -> Box");
                                Some("Box".to_string())
                            }
                            "string_new" | "string_from" => {
                                // eprintln!("    ✅ Builtin string_new/string_from() -> String");
                                Some("String".to_string())
                            }
                            "map_new" | "map_with_capacity" | "hashmap_new" => {
                                // eprintln!("    ✅ Builtin map_new() -> Map");
                                Some("Map".to_string())
                            }
                            "set_new" | "set_with_capacity" => {
                                // eprintln!("    ✅ Builtin set_new() -> Set");
                                Some("Set".to_string())
                            }
                            "range_new" => {
                                // eprintln!("    ✅ Builtin range_new() -> Range");
                                Some("Range".to_string())
                            }
                            "range_inclusive_new" => {
                                // eprintln!("    ✅ Builtin range_inclusive_new() -> RangeInclusive");
                                Some("RangeInclusive".to_string())
                            }
                            "channel_new" => {
                                // eprintln!("    ✅ Builtin channel_new() -> Channel");
                                Some("Channel".to_string())
                            }
                            _ => {
                                // Regular function
                                if let Some(func_def) = self.function_defs.get(func_name) {
                                    // eprintln!(
                                    //     "    → Found func_def, return_type: {:?}",
                                    //     func_def.return_type
                                    // );

                                    // Check if this is a generic function with explicit type args
                                    if !func_def.type_params.is_empty() && !type_args.is_empty() {
                                        // eprintln!("    → Generic function with explicit type args");

                                        // Build mangled function name
                                        let type_names: Vec<String> = type_args
                                            .iter()
                                            .map(|t| self.type_to_string(t))
                                            .collect();
                                        let mangled_func =
                                            format!("{}_{}", func_name, type_names.join("_"));
                                        // eprintln!("    → Mangled func name: {}", mangled_func);

                                        // Look up instantiated function
                                        if let Some(inst_func_def) =
                                            self.function_defs.get(&mangled_func)
                                        {
                                            // eprintln!(
                                            //     "    → Found instantiated func, return_type: {:?}",
                                            //     inst_func_def.return_type
                                            // );

                                            // Get struct name from return type
                                            if let Some(Type::Named(s_name)) =
                                                &inst_func_def.return_type
                                            {
                                                if self.struct_defs.contains_key(s_name) {
                                                    // eprintln!(
                                                    //     "    ✅ Instantiated returns struct: {}",
                                                    //     s_name
                                                    // );
                                                    Some(s_name.clone())
                                                } else {
                                                    None
                                                }
                                            } else if let Some(Type::Generic {
                                                name: gen_name,
                                                type_args: gen_args,
                                            }) = &inst_func_def.return_type
                                            {
                                                // Return type is still Generic (e.g., HashMap<str, i32>)
                                                // Mangle it to get the instantiated struct name
                                                if !gen_args.is_empty() {
                                                    let gen_name_clone = gen_name.clone();
                                                    let gen_args_clone = gen_args.clone();

                                                    let arg_names: Vec<String> = gen_args
                                                        .iter()
                                                        .map(|t| self.type_to_string(t))
                                                        .collect();
                                                    let mangled_struct = format!(
                                                        "{}_{}",
                                                        gen_name,
                                                        arg_names.join("_")
                                                    );
                                                    // eprintln!(
                                                    //     "    ✅ Generic return mangled to: {}",
                                                    //     mangled_struct
                                                    // );

                                                    // ⭐ Instantiate the generic struct with its methods
                                                    if let Err(e) = self.instantiate_generic_struct(
                                                        &gen_name_clone,
                                                        &gen_args_clone,
                                                    ) {
                                                        // eprintln!("    ⚠️  Failed to instantiate struct: {}", e);
                                                    } else {
                                                        eprintln!(
                                                            "    ✅ Struct instantiated: {}",
                                                            mangled_struct
                                                        );
                                                    }

                                                    Some(mangled_struct)
                                                } else {
                                                    None
                                                }
                                            } else {
                                                None
                                            }
                                        } else {
                                            eprintln!("    ⚠️  Instantiated func not found yet (will be created during call)");

                                            // Predict the mangled struct name from return type
                                            if let Some(Type::Generic { name: gen_name, .. }) =
                                                &func_def.return_type
                                            {
                                                eprintln!("    → Generic return: {}", gen_name);

                                                // Mangle struct name with actual type args
                                                let mangled_struct = format!(
                                                    "{}_{}",
                                                    gen_name,
                                                    type_names.join("_")
                                                );
                                                eprintln!(
                                                    "    → Predicted struct name: {}",
                                                    mangled_struct
                                                );

                                                Some(mangled_struct)
                                            } else {
                                                None
                                            }
                                        }
                                    } else if let Some(Type::Named(s_name)) = &func_def.return_type
                                    {
                                        eprintln!("    → Named return type: {}", s_name);
                                        if self.struct_defs.contains_key(s_name) {
                                            eprintln!("    ✅ Struct: {}", s_name);
                                            Some(s_name.clone())
                                        } else if self.enum_ast_defs.contains_key(s_name) {
                                            eprintln!("    ✅ Enum: {}", s_name);
                                            Some(s_name.clone())
                                        } else {
                                            eprintln!("    ❌ Type not found: {}", s_name);
                                            None
                                        }
                                    } else if let Some(Type::Generic { name: gen_name, .. }) =
                                        &func_def.return_type
                                    {
                                        eprintln!("    → Generic return type: {}", gen_name);
                                        if self.enum_ast_defs.contains_key(gen_name) {
                                            eprintln!("    ✅ Generic Enum: {}", gen_name);
                                            Some(gen_name.clone())
                                        } else if self.struct_defs.contains_key(gen_name) {
                                            eprintln!("    ✅ Generic Struct: {}", gen_name);
                                            Some(gen_name.clone())
                                        } else {
                                            eprintln!("    → Return type is not Named/Generic enum/struct");
                                            None
                                        }
                                    } else if let Some(Type::Result(_, _)) = &func_def.return_type {
                                        eprintln!("    → Result return type");
                                        Some("Result".to_string())
                                    } else if let Some(Type::Option(_)) = &func_def.return_type {
                                        eprintln!("    → Option return type");
                                        Some("Option".to_string())
                                    } else {
                                        eprintln!(
                                            "    → Return type is not Named/Generic/Result/Option"
                                        );
                                        None
                                    }
                                } else {
                                    eprintln!("    ❌ func_def not found");
                                    None
                                }
                            }
                        }
                    } else {
                        eprintln!("    → Not an Ident");
                        None
                    }
                }
                Expression::FieldAccess { object, field } => {
                    eprintln!(
                        "  → FieldAccess expression: {}.{}",
                        if let Expression::Ident(n) = object.as_ref() {
                            n
                        } else {
                            "?"
                        },
                        field
                    );
                    // Get struct type from field access
                    self.get_field_struct_type(object, field).ok().flatten()
                }
                Expression::MapLiteral(_) => {
                    eprintln!("  → MapLiteral expression");
                    Some("Map".to_string())
                }
                Expression::Binary {
                    left, op, right, ..
                } => {
                    eprintln!("  → Binary expression");

                    // First check for operator overloading (struct with trait impl)
                    let operator_result = if let Ok(left_type) = self.infer_expression_type(left) {
                        if let Type::Named(type_name) = &left_type {
                            let (trait_name, _method_name) = self.binary_op_to_trait(op);
                            if !trait_name.is_empty() {
                                if let Some(_) = self.has_operator_trait(type_name, trait_name) {
                                    eprintln!(
                                        "    ✅ Operator overload {} → {}",
                                        type_name, trait_name
                                    );
                                    Some(type_name.clone())
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    if operator_result.is_some() {
                        operator_result
                    } else if matches!(op, BinaryOp::Add) {
                        // Check for Vec + Vec concatenation
                        // Try to infer from left operand
                        if let Ok(left_type) = self.infer_expression_type(left) {
                            if let Type::Generic { name, type_args } = left_type {
                                if name == "Vec" {
                                    // Check if right is also Vec
                                    if let Ok(right_type) = self.infer_expression_type(right) {
                                        if let Type::Generic {
                                            name: right_name, ..
                                        } = right_type
                                        {
                                            if right_name == "Vec" {
                                                eprintln!("    ✅ Vec + Vec → Vec");
                                                // Get mangled name with type args: Vec<i32> -> Vec_i32
                                                if !type_args.is_empty() {
                                                    let mangled = format!(
                                                        "Vec_{}",
                                                        self.type_to_string(&type_args[0])
                                                    );
                                                    eprintln!("    → Mangled: {}", mangled);
                                                    Some(mangled)
                                                } else {
                                                    Some("Vec".to_string())
                                                }
                                            } else {
                                                None
                                            }
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                Expression::EnumLiteral { enum_name, .. } => {
                    eprintln!("  → EnumLiteral expression: {}", enum_name);
                    // For Option, Result, etc. - track as struct
                    if enum_name == "Option" || enum_name == "Result" {
                        Some(enum_name.clone())
                    } else {
                        None
                    }
                }
                _ => {
                    eprintln!("  → Other expression type");
                    None
                }
            }
        } else {
            // eprintln!("  → Type annotation present, skipping inference");
            None
        };

        // eprintln!("  → struct_name_from_expr: {:?}", struct_name_from_expr);

        // Array size validation: if type annotation is [T; N], verify array literal has N elements
        if let Some(Type::Array(_, annotated_size)) = ty {
            // Check array literal size
            let actual_size = match value {
                Expression::Array(elements) => Some(elements.len()),
                Expression::ArrayRepeat(_, repeat_size_expr) => {
                    if let Expression::IntLiteral(n) = repeat_size_expr.as_ref() {
                        Some(*n as usize)
                    } else {
                        None
                    }
                }
                _ => None,
            };

            if let Some(actual_size) = actual_size {
                if actual_size != *annotated_size {
                    return Err(format!(
                        "Array size mismatch: literal has {} elements but type annotation specifies [T; {}]",
                        actual_size, annotated_size
                    ));
                }
                eprintln!(
                    "  ✅ Array size validation passed: {} elements",
                    actual_size
                );
            }
        }

        // Recursively inject type args for nested generic structs
        let adjusted_value = if let Some(ref type_annotation) = ty {
            self.inject_type_args_recursive(value, type_annotation)?
        } else {
            value.clone()
        };

        // Special case: Array literal → Vec<T> conversion
        let mut val = if let Some(Type::Vec(elem_type)) = ty {
            if matches!(adjusted_value, Expression::Array(_)) {
                eprintln!(
                    "  🔧 Array→Vec conversion: Vec<{}>",
                    self.type_to_string(elem_type)
                );
                // Convert array literal to Vec
                if let Expression::Array(elements) = &adjusted_value {
                    self.compile_vec_from_array_literal(elements, elem_type)?
                } else {
                    self.compile_expression(&adjusted_value)?
                }
            } else {
                self.compile_expression(&adjusted_value)?
            }
        } else if let Some(Type::Array(elem_type, _)) = ty {
            // Special case: Array with type annotation [T; N]
            // For large arrays, compile directly into destination to avoid load/store
            if let Expression::ArrayRepeat(value_expr, count_expr) = &adjusted_value {
                eprintln!(
                    "  🔧 Array repeat with annotation: [{}; N]",
                    self.type_to_string(elem_type)
                );

                // Allocate buffer first
                let alloca =
                    self.create_entry_block_alloca(name, ty.as_ref().unwrap(), is_mutable)?;

                // Compile array repeat directly into buffer
                self.compile_array_repeat_into_buffer(value_expr, count_expr, elem_type, alloca)?;

                // Register variable and return early
                let llvm_type = self.ast_type_to_llvm(ty.as_ref().unwrap());
                self.variables.insert(name.clone(), alloca);
                self.variable_types.insert(name.clone(), llvm_type);
                return Ok(());
            } else if let Expression::Array(elements) = &adjusted_value {
                // Array literal optimization for large arrays
                if elements.len() > 100 {
                    eprintln!(
                        "  🔧 Large array literal with annotation: [{}; {}]",
                        self.type_to_string(elem_type),
                        elements.len()
                    );

                    // Allocate buffer first
                    let alloca =
                        self.create_entry_block_alloca(name, ty.as_ref().unwrap(), is_mutable)?;

                    // Compile array literal directly into buffer
                    self.compile_array_literal_into_buffer(elements, elem_type, alloca)?;

                    // Register variable and return early
                    let llvm_type = self.ast_type_to_llvm(ty.as_ref().unwrap());
                    self.variables.insert(name.clone(), alloca);
                    self.variable_types.insert(name.clone(), llvm_type);
                    return Ok(());
                } else {
                    self.compile_expression(&adjusted_value)?
                }
            } else {
                self.compile_expression(&adjusted_value)?
            }
        } else {
            self.compile_expression(&adjusted_value)?
        };

        // Determine type from value or explicit type
        let (var_type, llvm_type) = if let Some(t) = ty {
            let target_llvm_type = self.ast_type_to_llvm(t);

            // Cast integer literal to match target integer type width
            if let BasicValueEnum::IntValue(int_val) = val {
                if let BasicTypeEnum::IntType(target_int_type) = target_llvm_type {
                    if int_val.get_type().get_bit_width() != target_int_type.get_bit_width() {
                        if int_val.get_type().get_bit_width() < target_int_type.get_bit_width() {
                            // Sign extend for wider types
                            val = self
                                .builder
                                .build_int_s_extend(int_val, target_int_type, "lit_sext")
                                .map_err(|e| format!("Failed to extend literal: {}", e))?
                                .into();
                        } else {
                            // Truncate for narrower types
                            val = self
                                .builder
                                .build_int_truncate(int_val, target_int_type, "lit_trunc")
                                .map_err(|e| format!("Failed to truncate literal: {}", e))?
                                .into();
                        }
                    }
                }
            }

            (t.clone(), target_llvm_type)
        } else {
            // Infer type from LLVM value
            let inferred_llvm_type = val.get_type();
            let inferred_ast_type = self.infer_ast_type_from_llvm(inferred_llvm_type)?;
            (inferred_ast_type, inferred_llvm_type)
        };

        // Track struct type name if this is a named struct type or inferred from expression
        let final_var_type = if let Type::Box(inner_ty) = &var_type {
            let mangled = format!("Box_{}", self.type_to_string(inner_ty.as_ref()));
            eprintln!(
                "  ✅ Tracking Box type (from annotation): Box -> {}",
                mangled
            );
            self.variable_struct_names
                .insert(name.clone(), mangled.clone());
            var_type.clone()
        } else if let Type::Vec(inner_ty) = &var_type {
            let mangled = format!("Vec_{}", self.type_to_string(inner_ty.as_ref()));
            eprintln!(
                "  ✅ Tracking Vec type (from annotation): Vec -> {}",
                mangled
            );
            self.variable_struct_names
                .insert(name.clone(), mangled.clone());
            var_type.clone()
        } else if let Type::Channel(inner_ty) = &var_type {
            let mangled = format!("Channel_{}", self.type_to_string(inner_ty.as_ref()));
            eprintln!(
                "  ✅ Tracking Channel type (from annotation): Channel -> {}",
                mangled
            );
            self.variable_struct_names
                .insert(name.clone(), "Channel".to_string());
            var_type.clone()
        } else if let Type::Option(inner_ty) = &var_type {
            let mangled = format!("Option_{}", self.type_to_string(inner_ty.as_ref()));
            eprintln!(
                "  ✅ Tracking Option type (from annotation): Option -> {}",
                mangled
            );
            self.variable_struct_names
                .insert(name.clone(), mangled.clone());
            var_type.clone()
        } else if let Type::Result(ok_ty, err_ty) = &var_type {
            let mangled = format!(
                "Result_{}_{}",
                self.type_to_string(ok_ty.as_ref()),
                self.type_to_string(err_ty.as_ref())
            );
            eprintln!(
                "  ✅ Tracking Result type (from annotation): Result -> {}",
                mangled
            );
            self.variable_struct_names
                .insert(name.clone(), mangled.clone());
            var_type.clone()
        } else if let Type::Generic {
            name: struct_name,
            type_args,
        } = &var_type
        {
            match self.instantiate_generic_struct(struct_name, type_args) {
                Ok(mangled_name) => {
                    eprintln!(
                        "  ✅ Tracking Generic type (from annotation): {} -> {}",
                        struct_name, mangled_name
                    );
                    self.variable_struct_names
                        .insert(name.clone(), mangled_name.clone());
                    Type::Generic {
                        name: struct_name.clone(),
                        type_args: type_args.clone(),
                    }
                }
                Err(_) => var_type.clone(),
            }
        } else if let Type::Named(struct_name) = &var_type {
            if struct_name == "AnonymousStruct" {
                if let Some(type_name) = struct_name_from_expr.as_ref() {
                    eprintln!("  → AnonymousStruct resolved to: {}", type_name);
                    if type_name == "Vec"
                        || type_name == "Box"
                        || type_name == "String"
                        || type_name == "Map"
                        || type_name == "Range"
                        || type_name == "RangeInclusive"
                        || type_name == "Slice"
                    {
                        eprintln!("  ✅ Tracking as builtin type: {}", type_name);
                        self.variable_struct_names
                            .insert(name.clone(), type_name.clone());
                        Type::Named(type_name.clone())
                    } else if type_name == "Option" || type_name == "Result" {
                        eprintln!("  ✅ Tracking as builtin enum: {}", type_name);
                        self.variable_struct_names
                            .insert(name.clone(), type_name.clone());
                        Type::Named(type_name.clone())
                    } else if self.struct_defs.contains_key(type_name) {
                        eprintln!("  ✅ Tracking as struct (from expr): {}", type_name);
                        self.variable_struct_names
                            .insert(name.clone(), type_name.clone());
                        Type::Named(type_name.clone())
                    } else if self.enum_ast_defs.contains_key(type_name) {
                        eprintln!("  ✅ Tracking as enum (from expr): {}", type_name);
                        self.variable_enum_names
                            .insert(name.clone(), type_name.clone());
                        Type::Named(type_name.clone())
                    } else {
                        var_type.clone()
                    }
                } else if let Expression::EnumLiteral { enum_name, .. } = value {
                    self.variable_enum_names
                        .insert(name.clone(), enum_name.clone());
                    Type::Named(enum_name.clone())
                } else {
                    var_type.clone()
                }
            } else if struct_name == "Vec"
                || struct_name == "Box"
                || struct_name == "String"
                || struct_name == "Map"
                || struct_name == "Range"
                || struct_name == "RangeInclusive"
            {
                eprintln!("  ✅ Direct Named type is builtin: {}", struct_name);
                self.variable_struct_names
                    .insert(name.clone(), struct_name.clone());
                var_type.clone()
            } else if self.struct_defs.contains_key(struct_name) {
                self.variable_struct_names
                    .insert(name.clone(), struct_name.clone());
                var_type.clone()
            } else if self.enum_ast_defs.contains_key(struct_name) {
                self.variable_enum_names
                    .insert(name.clone(), struct_name.clone());
                var_type.clone()
            } else {
                var_type.clone()
            }
        } else if let Some(type_name) = struct_name_from_expr {
            eprintln!("  → Checking type_name: {}", type_name);
            eprintln!(
                "    → Is struct: {}",
                self.struct_defs.contains_key(&type_name)
            );
            eprintln!(
                "    → Is enum: {}",
                self.enum_ast_defs.contains_key(&type_name)
            );

            // Check for mangled generic types (Vec_i32, Box_String, HashMap_str_i32, etc.)
            let is_mangled_generic = type_name.starts_with("Vec_")
                || type_name.starts_with("Box_")
                || type_name.starts_with("Map_")
                || type_name.starts_with("HashMap_")
                || type_name.starts_with("HashSet_")
                || type_name.starts_with("Set_");

            if type_name == "Vec"
                || type_name == "Box"
                || type_name == "String"
                || type_name == "Map"
                || type_name == "Set"
                || type_name == "Slice"
                || type_name == "Result"
                || type_name == "Option"
                || is_mangled_generic
            {
                eprintln!("  ✅ Tracking as builtin type: {}", type_name);
                self.variable_struct_names
                    .insert(name.clone(), type_name.clone());

                // Track Drop trait implementations
                if self.type_implements_drop(&type_name) {
                    if let Some(scope) = self.scope_stack.last_mut() {
                        eprintln!("  📋 Adding {} to Drop scope (type: {})", name, type_name);
                        scope.push((name.clone(), type_name.clone()));
                    }
                }

                Type::Named(type_name)
            } else if self.struct_defs.contains_key(&type_name) {
                eprintln!("  ✅ Tracking as struct: {}", type_name);
                self.variable_struct_names
                    .insert(name.clone(), type_name.clone());

                // Track Drop trait implementations
                if self.type_implements_drop(&type_name) {
                    if let Some(scope) = self.scope_stack.last_mut() {
                        eprintln!("  📋 Adding {} to Drop scope (type: {})", name, type_name);
                        scope.push((name.clone(), type_name.clone()));
                    }
                }

                Type::Named(type_name)
            } else if self.enum_ast_defs.contains_key(&type_name) {
                eprintln!("  ✅ Tracking as enum: {} = {}", name, type_name);
                self.variable_enum_names
                    .insert(name.clone(), type_name.clone());
                Type::Named(type_name)
            } else {
                eprintln!("  ❌ Type {} not found!", type_name);
                Type::Named(type_name)
            }
        } else if ty.is_none() {
            match value {
                Expression::StructLiteral {
                    name: struct_name, ..
                } => {
                    if self.struct_defs.contains_key(struct_name) {
                        self.variable_struct_names
                            .insert(name.clone(), struct_name.clone());
                        Type::Named(struct_name.clone())
                    } else {
                        var_type.clone()
                    }
                }
                Expression::EnumLiteral { enum_name, .. } => {
                    if self.enum_ast_defs.contains_key(enum_name) {
                        self.variable_enum_names
                            .insert(name.clone(), enum_name.clone());
                        Type::Named(enum_name.clone())
                    } else {
                        var_type.clone()
                    }
                }
                Expression::Call {
                    span_id: _, func, ..
                } => {
                    if let Expression::FieldAccess { object, field: _ } = func.as_ref() {
                        if let Expression::Ident(enum_name) = object.as_ref() {
                            if self.enum_ast_defs.contains_key(enum_name) {
                                self.variable_enum_names
                                    .insert(name.clone(), enum_name.clone());
                                Type::Named(enum_name.clone())
                            } else {
                                var_type.clone()
                            }
                        } else {
                            var_type.clone()
                        }
                    } else if let Expression::Ident(func_name) = func.as_ref() {
                        eprintln!("  → Function call: {}, checking return type...", func_name);
                        if let Some(func_def) = self.function_defs.get(func_name) {
                            eprintln!(
                                "  → Found func_def, return_type: {:?}",
                                func_def.return_type
                            );
                            if let Some(Type::Named(type_name)) = &func_def.return_type {
                                eprintln!("  → Named type: {}", type_name);
                                if self.struct_defs.contains_key(type_name) {
                                    eprintln!("  ✅ Tracking as struct: {}", type_name);
                                    self.variable_struct_names
                                        .insert(name.clone(), type_name.clone());
                                    Type::Named(type_name.clone())
                                } else if self.enum_ast_defs.contains_key(type_name) {
                                    eprintln!("  ✅ Tracking as enum: {}", type_name);
                                    self.variable_enum_names
                                        .insert(name.clone(), type_name.clone());
                                    Type::Named(type_name.clone())
                                } else {
                                    eprintln!(
                                        "  ❌ Type {} not found in struct_defs or enum_ast_defs",
                                        type_name
                                    );
                                    var_type.clone()
                                }
                            } else {
                                var_type.clone()
                            }
                        } else {
                            var_type.clone()
                        }
                    } else {
                        var_type.clone()
                    }
                }
                Expression::FieldAccess { object, field } => {
                    let object_struct_name = self.get_expression_struct_name(object)?;

                    if let Some(struct_name) = object_struct_name {
                        let field_type_opt =
                            self.struct_defs.get(&struct_name).and_then(|struct_def| {
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
                                            .insert(name.clone(), field_struct_name.clone());
                                        Type::Named(field_struct_name)
                                    } else {
                                        var_type.clone()
                                    }
                                }
                                Type::Generic {
                                    name: field_struct_name,
                                    type_args,
                                } => {
                                    match self
                                        .instantiate_generic_struct(&field_struct_name, &type_args)
                                    {
                                        Ok(mangled_name) => {
                                            self.variable_struct_names
                                                .insert(name.clone(), mangled_name.clone());
                                            Type::Generic {
                                                name: field_struct_name,
                                                type_args,
                                            }
                                        }
                                        Err(_) => var_type.clone(),
                                    }
                                }
                                _ => var_type.clone(),
                            }
                        } else {
                            var_type.clone()
                        }
                    } else {
                        var_type.clone()
                    }
                }
                _ => var_type.clone(),
            }
        } else {
            var_type.clone()
        };

        // Determine final LLVM type selection for the variable slot
        let final_llvm_type = if let Type::Named(type_name) = &final_var_type {
            if type_name == "Tuple" {
                llvm_type
            } else if self.enum_ast_defs.contains_key(type_name) {
                llvm_type
            } else if self.struct_defs.contains_key(type_name) {
                // value already a pointer to data (struct literal)
                llvm_type
            } else if type_name == "Range" || type_name == "RangeInclusive" {
                // Range types are stack values with struct type {i64, i64, i64}
                llvm_type
            } else {
                self.ast_type_to_llvm(&final_var_type)
            }
        } else if let Type::Generic { .. } = &final_var_type {
            llvm_type
        } else {
            llvm_type
        };

        // Detect tuple literal
        let is_tuple_literal = matches!(&adjusted_value, Expression::TupleLiteral(_));

        // Check if this is a builtin pointer type (Vec, Box, String, etc.)
        let is_builtin_pointer = matches!(&final_var_type, Type::Vec(_) | Type::Box(_))
            || (matches!(&final_var_type, Type::Named(name) if name.starts_with("Vec_") || name.starts_with("Box_")));

        let is_struct_or_tuple = if let Type::Named(type_name) = &final_var_type {
            type_name == "Tuple"
                || self.struct_defs.contains_key(type_name)
                || type_name == "Option"
                || type_name == "Result"
        } else {
            is_tuple_literal
        };

        eprintln!(
            "🔹 Let var={}, is_struct_or_tuple={}, is_builtin_pointer={}, val_type={:?}",
            name,
            is_struct_or_tuple,
            is_builtin_pointer,
            val.get_type()
        );

        if is_struct_or_tuple || is_builtin_pointer {
            // value is either:
            // 1. A pointer to stack-allocated data (from struct literal)
            // 2. A struct value (from method/function return)

            if let BasicValueEnum::PointerValue(data_ptr) = val {
                // Case 1: Direct pointer from struct literal
                self.variables.insert(name.clone(), data_ptr);

                if is_tuple_literal {
                    if let Some(struct_ty) = self.last_compiled_tuple_type {
                        self.variable_types.insert(name.clone(), struct_ty.into());
                        self.tuple_variable_types.insert(name.clone(), struct_ty);
                        self.variable_struct_names
                            .insert(name.clone(), "Tuple".to_string());
                        self.last_compiled_tuple_type = None;
                    } else {
                        return Err("Tuple literal didn't set last_compiled_tuple_type".to_string());
                    }
                } else {
                    self.variable_types.insert(name.clone(), final_llvm_type);
                }
            } else if let BasicValueEnum::StructValue(_struct_val) = val {
                // Case 2: Struct returned by value (from function/method)
                // Need to allocate storage and store the value
                eprintln!("📦 Allocating storage for struct value returned from function");
                eprintln!(
                    "📦 final_llvm_type: {:?}, val.get_type(): {:?}",
                    final_llvm_type,
                    val.get_type()
                );
                let alloca = self.create_entry_block_alloca(name, &final_var_type, is_mutable)?;
                self.build_store_aligned(alloca, val)?;
                self.variables.insert(name.clone(), alloca);
                // Use actual struct type from value, not inferred type (which might be wrong)
                self.variable_types.insert(name.clone(), val.get_type());
            } else {
                return Err(format!(
                    "Struct/Tuple literal should return pointer or struct value, got {:?}",
                    val
                ));
            }
        } else {
            // Regular variable (not struct/tuple)
            let alloca = self.create_entry_block_alloca(name, &final_var_type, is_mutable)?;
            // Use alignment-aware store to fix memory corruption bug
            self.build_store_aligned(alloca, val)?;
            self.variables.insert(name.clone(), alloca);
            self.variable_types.insert(name.clone(), final_llvm_type);
        }

        // Track closure variables
        if let BasicValueEnum::PointerValue(fn_ptr) = val {
            if let Some(env_ptr) = self.closure_envs.get(&fn_ptr) {
                eprintln!(
                    "📝 Tracking closure variable: {} -> fn={:?}, env={:?}",
                    name, fn_ptr, env_ptr
                );
                self.closure_variables
                    .insert(name.clone(), (fn_ptr, *env_ptr));
            }
        }

        // Register for automatic cleanup if Vec or Box
        if let Type::Named(type_name) = &final_var_type {
            if type_name == "Vec" || type_name == "Box" {
                self.register_for_cleanup(name.clone(), type_name.clone());
            }
        }

        Ok(())
    }

    /// Recursively inject type arguments into nested generic struct literals
    /// Handles Box<Box<Box<T>>> with nested StructLiteral { Box { value: StructLiteral { Box { ... } } } }
    pub(crate) fn inject_type_args_recursive(
        &self,
        expr: &Expression,
        target_type: &Type,
    ) -> Result<Expression, String> {
        match expr {
            Expression::StructLiteral {
                name: struct_name,
                type_args: ref literal_type_args,
                fields: ref literal_fields,
            } => {
                // If struct literal has empty type_args and target type is Generic, inject
                let new_type_args = if literal_type_args.is_empty() {
                    match target_type {
                        Type::Generic {
                            name: target_struct_name,
                            type_args: ref target_type_args,
                        } if struct_name == target_struct_name => {
                            eprintln!(
                                "  🔧 Injecting type args into {}: {:?}",
                                struct_name, target_type_args
                            );
                            target_type_args.clone()
                        }
                        Type::Box(inner_type) if struct_name == "Box" => {
                            vec![inner_type.as_ref().clone()]
                        }
                        Type::Vec(inner_type) if struct_name == "Vec" => {
                            vec![inner_type.as_ref().clone()]
                        }
                        Type::Option(inner_type) if struct_name == "Option" => {
                            vec![inner_type.as_ref().clone()]
                        }
                        Type::Result(ok_type, err_type) if struct_name == "Result" => {
                            vec![ok_type.as_ref().clone(), err_type.as_ref().clone()]
                        }
                        _ => literal_type_args.clone(),
                    }
                } else {
                    literal_type_args.clone()
                };

                // Recursively process field values
                let mut new_fields = Vec::new();
                for (field_name, field_expr) in literal_fields.iter() {
                    // Determine expected type for this field
                    let field_target_type = if field_name == "value" && !new_type_args.is_empty() {
                        Some(&new_type_args[0])
                    } else {
                        None
                    };

                    let new_field_expr = if let Some(ft) = field_target_type {
                        self.inject_type_args_recursive(field_expr, ft)?
                    } else {
                        field_expr.clone()
                    };

                    new_fields.push((field_name.clone(), new_field_expr));
                }

                Ok(Expression::StructLiteral {
                    name: struct_name.clone(),
                    type_args: new_type_args,
                    fields: new_fields,
                })
            }
            _ => Ok(expr.clone()),
        }
    }
}
