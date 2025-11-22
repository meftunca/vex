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
        // Infer types for later use
        let inferred_left_type = if !left_is_literal {
            self.infer_expression_type(left).ok()
        } else {
            None
        };
        let inferred_right_type = if !right_is_literal {
            self.infer_expression_type(right).ok()
        } else {
            None
        };

        // ⭐ CRITICAL: Literal coercion strategy:
        // 1. If expected_type exists, use it for ALL expressions (literals AND variables)
        // 2. If one side is literal and other is typed, use typed side's type for literal
        // 3. This ensures `0 - x` where x:i8 compiles 0 as i8, not i32
        let (left_expected, right_expected) = if let Some(exp_ty) = expected_type {
            // If we have an expected type, use it for both sides
            // This handles cases like: let result: i8 = 0 - x;
            (Some(exp_ty), Some(exp_ty))
        } else if left_is_literal && inferred_right_type.is_some() {
            // Left is literal, right is typed variable -> use right's type for left
            (inferred_right_type.as_ref(), None)
        } else if right_is_literal && inferred_left_type.is_some() {
            // Right is literal, left is typed variable -> use left's type for right
            (None, inferred_left_type.as_ref())
        } else {
            (None, None)
        };

        // Compile left and right operands with inferred/expected types
        eprintln!(
            "🔍 Binary op coercion: left_expected={:?}, right_expected={:?}",
            left_expected, right_expected
        );
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
