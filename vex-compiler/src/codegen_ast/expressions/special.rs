// Special operations (unary, postfix)

use super::super::ASTCodeGen;
use inkwell::types::BasicType;
use inkwell::values::BasicValueEnum;
use inkwell::IntPredicate;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile unary operation
    pub(crate) fn compile_unary_op(
        &mut self,
        op: &UnaryOp,
        expr: &Expression,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        let val = self.compile_expression(expr)?;

        match op {
            UnaryOp::Neg => {
                if let BasicValueEnum::IntValue(iv) = val {
                    Ok(self
                        .builder
                        .build_int_neg(iv, "neg")
                        .map_err(|e| format!("Failed to negate: {}", e))?
                        .into())
                } else if let BasicValueEnum::FloatValue(fv) = val {
                    Ok(self
                        .builder
                        .build_float_neg(fv, "fneg")
                        .map_err(|e| format!("Failed to negate: {}", e))?
                        .into())
                } else {
                    Err("Cannot negate non-numeric value".to_string())
                }
            }
            UnaryOp::Not => {
                if let BasicValueEnum::IntValue(iv) = val {
                    let zero = iv.get_type().const_int(0, false);
                    Ok(self
                        .builder
                        .build_int_compare(IntPredicate::EQ, iv, zero, "not")
                        .map_err(|e| format!("Failed to compare: {}", e))?
                        .into())
                } else {
                    Err("Cannot apply ! to non-integer value".to_string())
                }
            }
            _ => Err(format!("Unary operation not yet implemented: {:?}", op)),
        }
    }

    /// Compile postfix operation (++ or --)
    pub(crate) fn compile_postfix_op(
        &mut self,
        expr: &Expression,
        op: &PostfixOp,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Get variable
        if let Expression::Ident(name) = expr {
            let ptr = *self
                .variables
                .get(name)
                .ok_or_else(|| format!("Variable {} not found", name))?;
            let var_type = *self
                .variable_types
                .get(name)
                .ok_or_else(|| format!("Type for variable {} not found", name))?;

            // Load current value
            let current = self
                .builder
                .build_load(var_type, ptr, name)
                .map_err(|e| format!("Failed to load: {}", e))?;

            if let BasicValueEnum::IntValue(iv) = current {
                let one = iv.get_type().const_int(1, false);
                let new_val = match op {
                    PostfixOp::Increment => self.builder.build_int_add(iv, one, "inc"),
                    PostfixOp::Decrement => self.builder.build_int_sub(iv, one, "dec"),
                }
                .map_err(|e| format!("Failed to build operation: {}", e))?;

                // Store back
                self.builder
                    .build_store(ptr, new_val)
                    .map_err(|e| format!("Failed to store: {}", e))?;

                // Return old value
                Ok(current)
            } else {
                Err("Can only increment/decrement integers".to_string())
            }
        } else {
            Err("Can only increment/decrement variables".to_string())
        }
    }

    /// Compile block expression: { stmt1; stmt2; expr }
    /// Last expression without semicolon becomes the return value
    pub(crate) fn compile_block_expression(
        &mut self,
        statements: &[Statement],
        return_expr: &Option<Box<Expression>>,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Compile all statements
        for stmt in statements {
            self.compile_statement(stmt)?;
        }

        // If there's a return expression, compile and return it
        if let Some(expr) = return_expr {
            self.compile_expression(expr)
        } else {
            // No return value, return unit (i32 0)
            Ok(self.context.i32_type().const_int(0, false).into())
        }
    }

    /// Find free variables in an expression (variables used but not defined in params)
    fn find_free_variables(&self, expr: &Expression, params: &[Param]) -> Vec<String> {
        use std::collections::HashSet;

        let param_names: HashSet<String> = params.iter().map(|p| p.name.clone()).collect();
        let mut free_vars = Vec::new();
        let mut visited = HashSet::new();

        self.collect_variables(expr, &param_names, &mut free_vars, &mut visited);
        free_vars
    }

    /// Recursively collect variable references
    fn collect_variables(
        &self,
        expr: &Expression,
        params: &std::collections::HashSet<String>,
        free_vars: &mut Vec<String>,
        visited: &mut std::collections::HashSet<String>,
    ) {
        match expr {
            Expression::Ident(name) => {
                // If it's not a parameter and not already visited
                if !params.contains(name) && !visited.contains(name) {
                    // Check if it's a local variable (not a function name)
                    if self.variables.contains_key(name) {
                        visited.insert(name.clone());
                        free_vars.push(name.clone());
                    }
                }
            }
            Expression::Binary { left, right, .. } => {
                self.collect_variables(left, params, free_vars, visited);
                self.collect_variables(right, params, free_vars, visited);
            }
            Expression::Unary { expr, .. } => {
                self.collect_variables(expr, params, free_vars, visited);
            }
            Expression::Call { func, args } => {
                self.collect_variables(func, params, free_vars, visited);
                for arg in args {
                    self.collect_variables(arg, params, free_vars, visited);
                }
            }
            Expression::MethodCall { receiver, args, .. } => {
                self.collect_variables(receiver, params, free_vars, visited);
                for arg in args {
                    self.collect_variables(arg, params, free_vars, visited);
                }
            }
            Expression::FieldAccess { object, .. } => {
                self.collect_variables(object, params, free_vars, visited);
            }
            Expression::Index { object, index } => {
                self.collect_variables(object, params, free_vars, visited);
                self.collect_variables(index, params, free_vars, visited);
            }
            Expression::Array(elements) => {
                for elem in elements {
                    self.collect_variables(elem, params, free_vars, visited);
                }
            }
            Expression::TupleLiteral(elements) => {
                for elem in elements {
                    self.collect_variables(elem, params, free_vars, visited);
                }
            }
            Expression::StructLiteral { fields, .. } => {
                for (_, expr) in fields {
                    self.collect_variables(expr, params, free_vars, visited);
                }
            }
            Expression::Match { value, arms } => {
                self.collect_variables(value, params, free_vars, visited);
                for arm in arms {
                    if let Some(guard) = &arm.guard {
                        self.collect_variables(guard, params, free_vars, visited);
                    }
                    self.collect_variables(&arm.body, params, free_vars, visited);
                }
            }
            Expression::Block { return_expr, .. } => {
                // For blocks, we mainly care about the return expression
                // TODO: Handle statement expressions more thoroughly
                if let Some(ret) = return_expr {
                    self.collect_variables(ret, params, free_vars, visited);
                }
            }
            _ => {} // Literals, other expressions
        }
    }

    /// Compile closure/lambda expression: |x: i32| x * 2
    /// Returns a closure struct containing function pointer and captured environment
    pub(crate) fn compile_closure(
        &mut self,
        params: &[Param],
        return_type: &Option<Type>,
        body: &Expression,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Generate unique closure name
        static mut CLOSURE_COUNTER: usize = 0;
        let closure_name = unsafe {
            CLOSURE_COUNTER += 1;
            format!("__closure_{}", CLOSURE_COUNTER)
        };

        // Step 1: Detect free variables (captured from environment)
        let free_vars = self.find_free_variables(body, params);

        eprintln!(
            "🔍 Closure {}: Found {} free variables: {:?}",
            closure_name,
            free_vars.len(),
            free_vars
        );

        // Step 2: Create environment struct type if we have captures
        let env_struct_type = if !free_vars.is_empty() {
            let mut field_types = Vec::new();
            for var_name in &free_vars {
                if let Some(var_type) = self.variable_types.get(var_name) {
                    field_types.push(*var_type);
                } else {
                    return Err(format!(
                        "Cannot find type for captured variable: {}",
                        var_name
                    ));
                }
            }
            Some(self.context.struct_type(&field_types, false))
        } else {
            None
        };

        // Step 3: Build parameter types for closure function
        // If we have captures, add environment pointer as first parameter
        let mut param_basic_types = Vec::new();
        if env_struct_type.is_some() {
            // Add environment pointer as hidden first parameter
            param_basic_types.push(
                self.context
                    .ptr_type(inkwell::AddressSpace::default())
                    .into(),
            );
        }

        // Add user-defined parameters
        for param in params {
            let param_ty = self.ast_type_to_llvm(&param.ty);
            param_basic_types.push(param_ty.into());
        }

        // Determine return type
        let ret_type = if let Some(ty) = return_type {
            self.ast_type_to_llvm(ty)
        } else {
            // Try to infer from body expression
            // For now, default to i32
            self.context.i32_type().into()
        };

        // Create function type
        let fn_type = ret_type.fn_type(&param_basic_types, false);

        // Create the closure function
        let closure_fn = self.module.add_function(&closure_name, fn_type, None);

        // Save current function and builder state
        let saved_fn = self.current_function;
        let saved_variables = self.variables.clone();

        // Set current function to closure
        self.current_function = Some(closure_fn);

        // Create entry block for closure
        let entry = self.context.append_basic_block(closure_fn, "entry");
        self.builder.position_at_end(entry);

        // Step 4: Load captured variables from environment struct
        let mut param_offset = 0;
        if let Some(env_type) = env_struct_type {
            // Get environment pointer (first parameter)
            let env_ptr = closure_fn
                .get_nth_param(0)
                .ok_or("Failed to get environment pointer")?
                .into_pointer_value();
            env_ptr.set_name("env");

            eprintln!(
                "📦 Loading {} captured variables from environment",
                free_vars.len()
            );

            // Load each captured variable from struct
            for (idx, var_name) in free_vars.iter().enumerate() {
                let var_type = self
                    .variable_types
                    .get(var_name)
                    .ok_or_else(|| format!("Type not found for captured variable: {}", var_name))?;

                // GEP to get pointer to field
                let field_ptr = unsafe {
                    self.builder
                        .build_in_bounds_gep(
                            env_type,
                            env_ptr,
                            &[
                                self.context.i32_type().const_int(0, false),
                                self.context.i32_type().const_int(idx as u64, false),
                            ],
                            &format!("{}_ptr", var_name),
                        )
                        .map_err(|e| format!("Failed to GEP for {}: {}", var_name, e))?
                };

                // Load the value
                let loaded_value = self
                    .builder
                    .build_load(*var_type, field_ptr, var_name)
                    .map_err(|e| format!("Failed to load {}: {}", var_name, e))?;

                // Allocate local stack space and store
                let alloca = self
                    .builder
                    .build_alloca(*var_type, &format!("{}_local", var_name))
                    .map_err(|e| format!("Failed to allocate {}: {}", var_name, e))?;

                self.builder
                    .build_store(alloca, loaded_value)
                    .map_err(|e| format!("Failed to store {}: {}", var_name, e))?;

                // Register in variables map
                self.variables.insert(var_name.clone(), alloca.into());

                eprintln!("  ✓ Loaded captured variable: {}", var_name);
            }

            param_offset = 1; // Skip environment pointer when processing user params
        }

        // Register closure parameters in scope
        for (i, param) in params.iter().enumerate() {
            let llvm_param = closure_fn
                .get_nth_param((i + param_offset) as u32)
                .ok_or_else(|| format!("Failed to get parameter {} for closure", i))?;

            // Name the parameter
            if let BasicValueEnum::IntValue(iv) = llvm_param {
                iv.set_name(&param.name);
            } else if let BasicValueEnum::FloatValue(fv) = llvm_param {
                fv.set_name(&param.name);
            } else if let BasicValueEnum::PointerValue(pv) = llvm_param {
                pv.set_name(&param.name);
            }

            // Allocate stack space and store parameter
            let param_ty = self.ast_type_to_llvm(&param.ty);
            let alloca = self
                .builder
                .build_alloca(param_ty, &param.name)
                .map_err(|e| format!("Failed to allocate parameter {}: {}", param.name, e))?;

            self.builder
                .build_store(alloca, llvm_param)
                .map_err(|e| format!("Failed to store parameter {}: {}", param.name, e))?;

            // Store in variables map (as pointer)
            self.variables.insert(param.name.clone(), alloca.into());
            self.variable_types.insert(param.name.clone(), param_ty);
        }

        // Compile closure body
        let body_value = self.compile_expression(body)?;

        // Build return
        self.builder
            .build_return(Some(&body_value))
            .map_err(|e| format!("Failed to build return in closure: {}", e))?;

        // Restore previous function and scope
        self.current_function = saved_fn;
        self.variables = saved_variables;

        // If there's a current function, position builder at its end
        if let Some(current_fn) = self.current_function {
            if let Some(bb) = current_fn.get_last_basic_block() {
                self.builder.position_at_end(bb);
            }
        }

        // Step 5: Create environment struct instance and populate with captured values
        if let Some(env_type) = env_struct_type {
            eprintln!(
                "🏗️  Creating environment struct with {} captures",
                free_vars.len()
            );

            // Allocate space for environment struct
            let env_alloca = self
                .builder
                .build_alloca(env_type, "closure_env")
                .map_err(|e| format!("Failed to allocate environment: {}", e))?;

            // Store each captured variable into the struct
            for (idx, var_name) in free_vars.iter().enumerate() {
                let var_ptr = self
                    .variables
                    .get(var_name)
                    .ok_or_else(|| format!("Captured variable not found: {}", var_name))?;

                let var_type = self
                    .variable_types
                    .get(var_name)
                    .ok_or_else(|| format!("Type not found for captured variable: {}", var_name))?;

                // Load current value
                let var_value = self
                    .builder
                    .build_load(*var_type, *var_ptr, var_name)
                    .map_err(|e| format!("Failed to load {}: {}", var_name, e))?;

                // GEP to field in struct
                let field_ptr = unsafe {
                    self.builder
                        .build_in_bounds_gep(
                            env_type,
                            env_alloca,
                            &[
                                self.context.i32_type().const_int(0, false),
                                self.context.i32_type().const_int(idx as u64, false),
                            ],
                            &format!("env_{}_ptr", var_name),
                        )
                        .map_err(|e| format!("Failed to GEP for {}: {}", var_name, e))?
                };

                // Store into struct field
                self.builder
                    .build_store(field_ptr, var_value)
                    .map_err(|e| format!("Failed to store {} to env: {}", var_name, e))?;

                eprintln!("  ✓ Captured variable: {} = {:?}", var_name, var_value);
            }

            // Step 6: Create fat pointer struct { fn_ptr, env_ptr }
            // This allows us to pass both function and environment together
            let fn_ptr = closure_fn.as_global_value().as_pointer_value();
            let env_ptr = env_alloca; // Environment struct pointer

            // Store mapping for later use during function calls
            self.closure_envs.insert(fn_ptr, env_ptr);

            eprintln!("✅ Created closure with environment binding");
            eprintln!("   Function: {:?}, Environment: {:?}", fn_ptr, env_ptr);

            // For now, return function pointer
            // When this closure is called, we'll look up its environment in closure_envs
            Ok(fn_ptr.into())
        } else {
            // No captures, just return function pointer
            eprintln!("✅ Created closure without captures (pure function)");
            Ok(closure_fn.as_global_value().as_pointer_value().into())
        }
    }
}
