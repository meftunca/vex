// Closure utilities and helper functions

use crate::codegen_ast::ASTCodeGen;
use inkwell::types::BasicType;
use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue};
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Find free variables in an expression (variables used but not defined in params)
    pub(super) fn find_free_variables(&self, expr: &Expression, params: &[Param]) -> Vec<String> {
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
            Expression::Binary {
                span_id: _,
                left,
                right,
                ..
            } => {
                self.collect_variables(left, params, free_vars, visited);
                self.collect_variables(right, params, free_vars, visited);
            }
            Expression::Unary {
                span_id: _, expr, ..
            } => {
                self.collect_variables(expr, params, free_vars, visited);
            }
            Expression::Call { func, args, .. } => {
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

    /// Generate closure struct with trait implementation
    /// Creates: struct __Closure_1 impl Callable(i32): i32 { fn_ptr, env_ptr }
    pub(super) fn generate_closure_struct(
        &mut self,
        closure_name: &str,
        params: &[Param],
        return_type: &Option<Type>,
        capture_mode: &CaptureMode,
        _closure_fn: FunctionValue<'ctx>,
        _env_ptr: Option<PointerValue<'ctx>>,
    ) -> Result<(), String> {
        // Note: Field, Function, Receiver, Struct, Type are already imported via vex_ast::*

        // Determine trait name based on capture mode
        let trait_name = match capture_mode {
            CaptureMode::Immutable | CaptureMode::Infer => "Callable",
            CaptureMode::Mutable => "CallableMut",
            CaptureMode::Once => "CallableOnce",
        };

        eprintln!(
            "🏗️  Generating closure struct: {} impl {}",
            closure_name, trait_name
        );

        // Create struct type name from closure name
        let struct_name = format!(
            "{}{}",
            &closure_name[0..1].to_uppercase(),
            &closure_name[1..]
        );

        // Create trait method: call/call_mut/call_once
        let method_name = match capture_mode {
            CaptureMode::Immutable | CaptureMode::Infer => "call",
            CaptureMode::Mutable => "call_mut",
            CaptureMode::Once => "call_once",
        };

        // Build method parameters from closure parameters (just copy the vector)
        let method_params: Vec<Param> = params.to_vec();

        // Determine receiver mutability
        let is_mutable = matches!(capture_mode, CaptureMode::Mutable);

        // Create method with receiver
        let method = Function {
            is_async: false,
            is_gpu: false,
            is_mutable,         // ⭐ NEW: Method mutability matches closure capture mode
            is_operator: false, // Closures are not operators
            receiver: Some(Receiver {
                name: "self".to_string(), // Generated closure struct methods use 'self'
                is_mutable,
                ty: Type::Reference(Box::new(Type::Named(struct_name.clone())), is_mutable),
            }),
            name: method_name.to_string(),
            type_params: vec![],
            const_params: vec![],
            where_clause: vec![],
            params: method_params,
            return_type: return_type.clone(),
            body: Block {
                statements: vec![], // Empty body - will be generated in codegen
            },
            is_variadic: false,
            variadic_type: None,
        };

        // Create struct definition with trait impl (no fields - managed by LLVM)
        // The actual closure struct layout (fn_ptr + env_ptr) is internal to LLVM
        let struct_def = Struct {
            name: struct_name.clone(),
            type_params: vec![],
            const_params: vec![],
            where_clause: vec![],
            policies: vec![], // No policies for generated closure structs
            impl_traits: vec![TraitImpl {
                name: trait_name.to_string(),
                type_args: vec![],
            }],
            associated_type_bindings: vec![], // No associated types for closures
            fields: vec![],                   // Internal LLVM representation
            methods: vec![method],
        };

        // Register struct in AST definitions
        self.struct_ast_defs.insert(struct_name.clone(), struct_def);

        eprintln!(
            "✅ Generated closure struct: {} impl {}",
            struct_name, trait_name
        );

        Ok(())
    }

    /// Infer parameter type from usage in closure body
    pub fn infer_param_type_from_body(&self, param_name: &str, body: &Expression) -> Option<Type> {
        match body {
            Expression::Binary {
                left, right, op, ..
            } => {
                // Check if parameter is used in binary expression
                if let Expression::Ident(name) = &**left {
                    if name == param_name {
                        // Infer from right side
                        return self.infer_expr_type(right);
                    }
                }
                if let Expression::Ident(name) = &**right {
                    if name == param_name {
                        // Infer from left side
                        return self.infer_expr_type(left);
                    }
                }
                // Recurse into both sides
                self.infer_param_type_from_body(param_name, left)
                    .or_else(|| self.infer_param_type_from_body(param_name, right))
            }
            Expression::Call { func, args, .. } => {
                // Check arguments for parameter usage
                for arg in args {
                    if let Some(ty) = self.infer_param_type_from_body(param_name, arg) {
                        return Some(ty);
                    }
                }
                self.infer_param_type_from_body(param_name, func)
            }
            Expression::Ident(name) if name == param_name => {
                // Direct usage, can't infer from this alone
                None
            }
            Expression::Block { statements, .. } => {
                // Check last expression in block
                if let Some(Statement::Expression(expr)) = statements.last() {
                    return self.infer_param_type_from_body(param_name, expr);
                }
                None
            }
            _ => None,
        }
    }

    /// Infer return type from closure body
    pub fn infer_return_type_from_body(&self, body: &Expression) -> Option<Type> {
        match body {
            Expression::Block { statements, .. } => {
                // Get type of last expression
                if let Some(Statement::Expression(expr)) = statements.last() {
                    return self.infer_expr_type(expr);
                }
                None
            }
            _ => self.infer_expr_type(body),
        }
    }

    /// Infer type from expression structure
    fn infer_expr_type(&self, expr: &Expression) -> Option<Type> {
        match expr {
            Expression::IntLiteral(_) => Some(Type::I32),
            Expression::FloatLiteral(_) => Some(Type::F64),
            Expression::BoolLiteral(_) => Some(Type::Bool),
            Expression::StringLiteral(_) => Some(Type::String),
            Expression::Binary {
                left, right, op, ..
            } => {
                use vex_ast::BinaryOp::*;
                match op {
                    Add | Sub | Mul | Div | Mod | Pow => {
                        // Arithmetic - propagate operand types
                        self.infer_expr_type(left)
                            .or_else(|| self.infer_expr_type(right))
                    }
                    Eq | NotEq | Lt | LtEq | Gt | GtEq | And | Or => Some(Type::Bool),
                    _ => None,
                }
            }
            Expression::Ident(_name) => {
                // Variable type lookup disabled - LLVM pointer types don't map back to Type enum easily
                // This is fine for closure inference since we can infer from literals/operators
                None
            }
            Expression::Call { func, .. } => {
                // Try to infer from function signature
                if let Expression::Ident(fn_name) = &**func {
                    // Check builtin functions
                    match fn_name.as_str() {
                        "i32_to_string" | "f64_to_string" | "bool_to_string" => Some(Type::String),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            Expression::Block { statements, .. } => {
                if let Some(Statement::Expression(expr)) = statements.last() {
                    return self.infer_expr_type(expr);
                }
                None
            }
            _ => None,
        }
    }
}
