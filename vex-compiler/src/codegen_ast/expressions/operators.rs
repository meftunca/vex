// Expression compilation - operators (binary, unary, postfix)
use super::ASTCodeGen;
use inkwell::values::BasicValueEnum;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile binary operations with optional expected type for overflow checking
    pub(crate) fn compile_binary_op_with_expected(
        &mut self,
        left: &vex_ast::Expression,
        op: &vex_ast::BinaryOp,
        right: &vex_ast::Expression,
        expected_type: Option<&vex_ast::Type>,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Check for operator overloading BEFORE compiling (needs Expression AST for type inference)
        if let Some(result) = self.check_operator_overloading(left, op, right, expected_type)? {
            return Ok(result);
        }

        // Smart literal compilation: If one operand is a literal and the other is typed,
        // use the typed operand's type for the literal
        let left_is_literal = matches!(
            left,
            vex_ast::Expression::IntLiteral(_)
                | vex_ast::Expression::FloatLiteral(_)
                | vex_ast::Expression::BigIntLiteral(_)
        );
        let right_is_literal = matches!(
            right,
            vex_ast::Expression::IntLiteral(_)
                | vex_ast::Expression::FloatLiteral(_)
                | vex_ast::Expression::BigIntLiteral(_)
        );

        // Infer types early to avoid lifetime issues
        let inferred_left_type = if right_is_literal && !left_is_literal {
            self.infer_expression_type(left).ok()
        } else {
            None
        };
        let inferred_right_type = if left_is_literal && !right_is_literal {
            self.infer_expression_type(right).ok()
        } else {
            None
        };

        let (left_expected, right_expected) = if expected_type.is_some() {
            (expected_type, expected_type)
        } else if left_is_literal && inferred_right_type.is_some() {
            (inferred_right_type.as_ref(), None)
        } else if right_is_literal && inferred_left_type.is_some() {
            (None, inferred_left_type.as_ref())
        } else {
            (None, None)
        };

        // Compile left and right operands with inferred/expected types
        let left_val = self.compile_expression_with_type(left, left_expected)?;
        let right_val = self.compile_expression_with_type(right, right_expected)?;

        // Load operands if they are pointers to structs/enums
        let (left_val, right_val) =
            self.load_operands_if_needed(left_val, right_val, left, right)?;

        // Dispatch to type-specific handlers
        match (&left_val, &right_val) {
            (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                // Infer AST types for coercion checks
                let left_ast_type = self.infer_expression_type(left)?;
                let right_ast_type = self.infer_expression_type(right)?;

                // Align integer widths with AST type awareness
                let (l, r) =
                    self.align_integer_widths_with_ast(*l, *r, &left_ast_type, &right_ast_type)?;
                match op {
                    vex_ast::BinaryOp::Pow => self.compile_int_power(l, r),
                    _ => self.compile_integer_binary_op_with_expected(l, r, op, expected_type),
                }
            }
            (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                // Align float widths (already uses coercion_rules internally)
                let (l, r) = self.align_float_widths(*l, *r)?;
                match op {
                    vex_ast::BinaryOp::Pow => self.compile_float_power(l, r),
                    _ => self.compile_float_binary_op(l, r, op),
                }
            }
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

    /// Compile binary operations (backward compat wrapper)
    pub(crate) fn compile_binary_op(
        &mut self,
        left: &vex_ast::Expression,
        op: &vex_ast::BinaryOp,
        right: &vex_ast::Expression,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        self.compile_binary_op_with_expected(left, op, right, None)
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
        expected_type: Option<&vex_ast::Type>,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        self.compile_unary_op(op, expr, expected_type)
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
