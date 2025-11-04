// Variable statements: let, assignment, return

use super::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile a let statement
    pub(crate) fn compile_let_statement(
        &mut self,
        is_mutable: bool,
        name: &str,
        ty: &Option<Type>,
        value: &Expression,
    ) -> Result<(), String> {
        // v0.9: is_mutable determines if variable is mutable (let vs let!)
        // FIRST: Determine struct name from expression BEFORE compiling
        // (because after compilation, we lose the expression structure)
        eprintln!(
            "🔷 Let statement: var={}, has_type_annotation={}",
            name,
            ty.is_some()
        );
        let struct_name_from_expr = if ty.is_none() {
            eprintln!("  → Type inference needed, analyzing expression...");
            match value {
                Expression::StructLiteral {
                    name: s_name,
                    type_args,
                    ..
                } => {
                    eprintln!("  → StructLiteral: {}", s_name);

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
                Expression::Call { func, .. } => {
                    eprintln!("  → Call expression");
                    if let Expression::Ident(func_name) = func.as_ref() {
                        eprintln!("    → Function: {}", func_name);
                        if let Some(func_def) = self.function_defs.get(func_name) {
                            eprintln!(
                                "    → Found func_def, return_type: {:?}",
                                func_def.return_type
                            );
                            if let Some(Type::Named(s_name)) = &func_def.return_type {
                                eprintln!("    → Named return type: {}", s_name);
                                if self.struct_defs.contains_key(s_name) {
                                    eprintln!("    ✅ Struct: {}", s_name);
                                    Some(s_name.clone())
                                } else if self.enum_ast_defs.contains_key(s_name) {
                                    eprintln!("    ✅ Enum: {}", s_name);
                                    // Function returns enum - track it!
                                    Some(s_name.clone())
                                } else {
                                    eprintln!("    ❌ Type not found: {}", s_name);
                                    None
                                }
                            } else {
                                eprintln!("    → Return type is not Named");
                                None
                            }
                        } else {
                            eprintln!("    ❌ func_def not found");
                            None
                        }
                    } else {
                        eprintln!("    → Not an Ident");
                        None
                    }
                }
                _ => {
                    eprintln!("  → Other expression type");
                    None
                }
            }
        } else {
            eprintln!("  → Type annotation present, skipping inference");
            None
        };

        eprintln!("  → struct_name_from_expr: {:?}", struct_name_from_expr);

        // Special handling for tuple literals: pre-compute struct type before compilation
        let tuple_struct_type = if let Expression::TupleLiteral(elements) = value {
            // Pre-compute element types to build the struct type
            let mut element_types = Vec::new();
            for elem_expr in elements.iter() {
                let elem_val = self.compile_expression(elem_expr)?;
                element_types.push(elem_val.get_type());
            }
            let struct_ty = self.context.struct_type(&element_types, false);
            // Save it for pattern matching
            self.tuple_variable_types.insert(name.to_string(), struct_ty);
            Some(struct_ty)
        } else {
            None
        };

        let mut val = self.compile_expression(value)?;

        // For tuple literals, load the struct value (tuple literal returns pointer)
        if let Some(struct_ty) = tuple_struct_type {
            if val.is_pointer_value() {
                val = self
                    .builder
                    .build_load(struct_ty, val.into_pointer_value(), "tuple_val_loaded")
                    .map_err(|e| format!("Failed to load tuple value: {}", e))?;
            }
        }

        // Determine type from value or explicit type
        let (var_type, llvm_type) = if let Some(t) = ty {
            (t.clone(), self.ast_type_to_llvm(t))
        } else if let Some(struct_ty) = tuple_struct_type {
            // Use the pre-computed tuple struct type
            // For tuples, use a placeholder AST type and directly use the struct LLVM type
            eprintln!("[DEBUG LET] Using tuple struct_ty: {:?}", struct_ty);
            (Type::Named("Tuple".to_string()), struct_ty.into())
        } else {
            // Infer type from LLVM value
            let inferred_llvm_type = val.get_type();
            let inferred_ast_type = self.infer_ast_type_from_llvm(inferred_llvm_type)?;
            (inferred_ast_type, inferred_llvm_type)
        };

        // Track struct type name if this is a named struct type or inferred from expression
        let final_var_type = if let Type::Named(struct_name) = &var_type {
            // Check if this is actually an enum (inferred as AnonymousStruct)
            if struct_name == "AnonymousStruct" {
                // Check if we have type info from expression analysis
                if let Some(type_name) = struct_name_from_expr.as_ref() {
                    eprintln!("  → AnonymousStruct resolved to: {}", type_name);
                    if self.struct_defs.contains_key(type_name) {
                        eprintln!("  ✅ Tracking as struct (from expr): {}", type_name);
                        self.variable_struct_names
                            .insert(name.to_string(), type_name.clone());
                        Type::Named(type_name.clone())
                    } else if self.enum_ast_defs.contains_key(type_name) {
                        eprintln!("  ✅ Tracking as enum (from expr): {}", type_name);
                        self.variable_enum_names
                            .insert(name.to_string(), type_name.clone());
                        Type::Named(type_name.clone())
                    } else {
                        var_type.clone()
                    }
                } else if let Expression::EnumLiteral { enum_name, .. } = value {
                    // Fallback: Check if value is an enum literal
                    self.variable_enum_names
                        .insert(name.to_string(), enum_name.clone());
                    Type::Named(enum_name.clone())
                } else {
                    var_type.clone()
                }
            } else if self.struct_defs.contains_key(struct_name) {
                self.variable_struct_names
                    .insert(name.to_string(), struct_name.clone());
                var_type.clone()
            } else if self.enum_ast_defs.contains_key(struct_name) {
                // This is an enum type annotation
                self.variable_enum_names
                    .insert(name.to_string(), struct_name.clone());
                var_type.clone()
            } else {
                var_type.clone()
            }
        } else if let Type::Generic {
            name: struct_name,
            type_args,
        } = &var_type
        {
            // Generic type annotation: Box<i32>, Pair<T, U>
            // Instantiate to get mangled name and track it
            match self.instantiate_generic_struct(struct_name, type_args) {
                Ok(mangled_name) => {
                    self.variable_struct_names
                        .insert(name.to_string(), mangled_name.clone());
                    Type::Generic {
                        name: struct_name.clone(),
                        type_args: type_args.clone(),
                    }
                }
                Err(_) => var_type.clone(),
            }
        } else if let Some(type_name) = struct_name_from_expr {
            // We found type name from expression analysis (could be struct or enum)
            eprintln!("  → Checking type_name: {}", type_name);
            eprintln!(
                "    → Is struct: {}",
                self.struct_defs.contains_key(&type_name)
            );
            eprintln!(
                "    → Is enum: {}",
                self.enum_ast_defs.contains_key(&type_name)
            );
            if self.struct_defs.contains_key(&type_name) {
                eprintln!("  ✅ Tracking as struct: {}", type_name);
                self.variable_struct_names
                    .insert(name.to_string(), type_name.clone());
                Type::Named(type_name.clone())
            } else if self.enum_ast_defs.contains_key(&type_name) {
                eprintln!("  ✅ Tracking as enum: {}", type_name);
                self.variable_enum_names
                    .insert(name.to_string(), type_name.clone());
                Type::Named(type_name.clone())
            } else {
                var_type.clone()
            }
        } else {
            var_type.clone()
        };

        // Determine final LLVM type
        let final_llvm_type = if let Some(struct_ty) = tuple_struct_type {
            struct_ty.into()
        } else {
            llvm_type
        };

        // Allocate space for the variable
        // v0.9: Use is_mutable to determine if variable is mutable
        let alloca = self.create_entry_block_alloca(name, &final_var_type, is_mutable)?;

        // Store the value
        self.builder
            .build_store(alloca, val)
            .map_err(|e| format!("Failed to store variable: {}", e))?;
        self.variables.insert(name.to_string(), alloca);
        self.variable_types.insert(name.to_string(), final_llvm_type);

        Ok(())
    }

    /// Compile an assignment statement
    pub(crate) fn compile_assignment_statement(
        &mut self,
        target: &Expression,
        value: &Expression,
    ) -> Result<(), String> {
        let val = self.compile_expression(value)?;

        // Get target pointer
        if let Expression::Ident(name) = target {
            let ptr = self
                .variables
                .get(name)
                .ok_or_else(|| format!("Variable {} not found", name))?;
            self.builder
                .build_store(*ptr, val)
                .map_err(|e| format!("Failed to assign: {}", e))?;
        } else {
            return Err(
                "Complex assignment targets not yet supported (field access, etc.)"
                    .to_string(),
            );
        }

        Ok(())
    }

    /// Compile a compound assignment statement (x += y, x -= y, etc.)
    pub(crate) fn compile_compound_assignment_statement(
        &mut self,
        target: &Expression,
        op: &CompoundOp,
        value: &Expression,
    ) -> Result<(), String> {
        // Compound assignment: x += expr is equivalent to x = x + expr
        // First load the current value
        let current_val = self.compile_expression(target)?;
        let rhs_val = self.compile_expression(value)?;

        // Perform the operation based on type
        let result: BasicValueEnum = if current_val.is_int_value() {
            let lhs = current_val.into_int_value();
            let rhs = rhs_val.into_int_value();
            match op {
                CompoundOp::Add => self
                    .builder
                    .build_int_add(lhs, rhs, "add_tmp")
                    .map_err(|e| format!("Failed to build add: {}", e))?
                    .into(),
                CompoundOp::Sub => self
                    .builder
                    .build_int_sub(lhs, rhs, "sub_tmp")
                    .map_err(|e| format!("Failed to build sub: {}", e))?
                    .into(),
                CompoundOp::Mul => self
                    .builder
                    .build_int_mul(lhs, rhs, "mul_tmp")
                    .map_err(|e| format!("Failed to build mul: {}", e))?
                    .into(),
                CompoundOp::Div => self
                    .builder
                    .build_int_signed_div(lhs, rhs, "div_tmp")
                    .map_err(|e| format!("Failed to build div: {}", e))?
                    .into(),
            }
        } else if current_val.is_float_value() {
            let lhs = current_val.into_float_value();
            let rhs = rhs_val.into_float_value();
            match op {
                CompoundOp::Add => self
                    .builder
                    .build_float_add(lhs, rhs, "add_tmp")
                    .map_err(|e| format!("Failed to build add: {}", e))?
                    .into(),
                CompoundOp::Sub => self
                    .builder
                    .build_float_sub(lhs, rhs, "sub_tmp")
                    .map_err(|e| format!("Failed to build sub: {}", e))?
                    .into(),
                CompoundOp::Mul => self
                    .builder
                    .build_float_mul(lhs, rhs, "mul_tmp")
                    .map_err(|e| format!("Failed to build mul: {}", e))?
                    .into(),
                CompoundOp::Div => self
                    .builder
                    .build_float_div(lhs, rhs, "div_tmp")
                    .map_err(|e| format!("Failed to build div: {}", e))?
                    .into(),
            }
        } else {
            return Err("Invalid type for compound assignment".to_string());
        };

        // Store the result back
        match target {
            Expression::Ident(name) => {
                // Simple variable assignment
                let ptr = self
                    .variables
                    .get(name)
                    .ok_or_else(|| format!("Variable {} not found", name))?;
                self.builder
                    .build_store(*ptr, result)
                    .map_err(|e| format!("Failed to store compound assignment: {}", e))?;
            }
            Expression::FieldAccess { object, field } => {
                // Field access assignment: obj.field += value
                let field_ptr = self.get_field_pointer(object, field)?;
                self.builder.build_store(field_ptr, result).map_err(|e| {
                    format!("Failed to store field compound assignment: {}", e)
                })?;
            }
            Expression::Index { object, index } => {
                // Array index assignment: arr[i] += value
                let elem_ptr = self.get_index_pointer(object, index)?;
                self.builder.build_store(elem_ptr, result).map_err(|e| {
                    format!("Failed to store array compound assignment: {}", e)
                })?;
            }
            _ => {
                return Err(
                    "This compound assignment target is not yet supported".to_string()
                );
            }
        }

        Ok(())
    }

    /// Compile a return statement
    pub(crate) fn compile_return_statement(
        &mut self,
        expr: &Option<Expression>,
    ) -> Result<(), String> {
        // Execute deferred statements in reverse order before returning
        self.execute_deferred_statements()?;

        if let Some(e) = expr {
            let val = self.compile_expression(e)?;
            self.builder
                .build_return(Some(&val))
                .map_err(|e| format!("Failed to build return: {}", e))?;
        } else {
            let zero = self.context.i32_type().const_int(0, false);
            self.builder
                .build_return(Some(&zero))
                .map_err(|e| format!("Failed to build return: {}", e))?;
        }

        Ok(())
    }
}

