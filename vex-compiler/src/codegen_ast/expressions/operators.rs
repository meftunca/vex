// Expression compilation - operators (binary, unary, postfix)
use super::ASTCodeGen;
use inkwell::values::BasicValueEnum;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile binary operations
    pub(crate) fn compile_binary_op(
        &mut self,
        left: &vex_ast::Expression,
        op: &vex_ast::BinaryOp,
        right: &vex_ast::Expression,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Compile left and right operands first
        let left_val = self.compile_expression(left)?;
        let right_val = self.compile_expression(right)?;

        // Check for operator overloading first
        if let Some(result) = self.check_operator_overloading(left, op, right)? {
            return Ok(result);
        }

        // Load operands if they are pointers to structs/enums
        let (left_val, right_val) =
            self.load_operands_if_needed(left_val, right_val, left, right)?;

        // Dispatch to type-specific handlers
        match (&left_val, &right_val) {
            (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => match op {
                vex_ast::BinaryOp::Pow => self.compile_int_power(*l, *r),
                _ => self.compile_integer_binary_op(*l, *r, op),
            },
            (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => match op {
                vex_ast::BinaryOp::Pow => self.compile_float_power(*l, *r),
                _ => self.compile_float_binary_op(*l, *r, op),
            },
            (BasicValueEnum::PointerValue(l), BasicValueEnum::PointerValue(r)) => {
                self.compile_pointer_binary_op(*l, *r, op)
            }
            (BasicValueEnum::StructValue(l), BasicValueEnum::StructValue(r)) => {
                self.compile_struct_binary_op(*l, *r, op)
            }
            _ => Err(format!(
                "Unsupported binary operation: {:?} between types",
                op
            )),
        }
    }

    /// Compile binary operations dispatch
    pub(crate) fn compile_binary_op_dispatch(
        &mut self,
        left: &vex_ast::Expression,
        op: &vex_ast::BinaryOp,
        right: &vex_ast::Expression,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        self.compile_binary_op(left, op, right)
    }

    /// Compile unary operations
    pub(crate) fn compile_unary_op_dispatch(
        &mut self,
        op: &vex_ast::UnaryOp,
        expr: &vex_ast::Expression,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        self.compile_unary_op(op, expr)
    }

    /// Compile postfix operations
    pub(crate) fn compile_postfix_op_dispatch(
        &mut self,
        expr: &vex_ast::Expression,
        op: &vex_ast::PostfixOp,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        self.compile_postfix_op(expr, op)
    }
}
