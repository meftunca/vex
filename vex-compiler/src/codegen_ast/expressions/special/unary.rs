// Unary and postfix operations

use crate::codegen_ast::ASTCodeGen;
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
            
            // If this statement terminated the block (e.g., return), stop processing
            if let Some(current_bb) = self.builder.get_insert_block() {
                if current_bb.get_terminator().is_some() {
                    // Block is terminated, return dummy value
                    return Ok(self.context.i32_type().const_int(0, false).into());
                }
            }
        }

        // If there's a return expression, compile and return it
        if let Some(expr) = return_expr {
            self.compile_expression(expr)
        } else {
            // No return value, return unit (i32 0)
            Ok(self.context.i32_type().const_int(0, false).into())
        }
    }
}
