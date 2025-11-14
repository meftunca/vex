// Expression compilation - special expressions (channels, closures, casts, ranges, etc.)
use super::ASTCodeGen;
use inkwell::values::BasicValueEnum;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile channel receive expressions (<-ch)
    pub(crate) fn compile_channel_receive_dispatch(
        &mut self,
        channel_expr: &vex_ast::Expression,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Channel receive operator: <-ch
        // Desugar to method call: ch.recv()
        let recv_call = vex_ast::Expression::MethodCall {
            receiver: Box::new(channel_expr.clone()),
            method: "recv".to_string(),
            type_args: vec![],
            args: vec![],
            is_mutable_call: true,
        };
        self.compile_expression(&recv_call)
    }

    /// Compile closure expressions
    pub(crate) fn compile_closure_dispatch(
        &mut self,
        params: &[vex_ast::Param],
        return_type: &Option<vex_ast::Type>,
        body: &vex_ast::Expression,
        capture_mode: &vex_ast::CaptureMode,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        self.compile_closure(params, return_type, body, capture_mode)
    }

    /// Compile cast expressions
    pub(crate) fn compile_cast_dispatch(
        &mut self,
        expr: &vex_ast::Expression,
        target_type: &vex_ast::Type,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        self.compile_cast_expression(expr, target_type)
    }

    /// Compile range expressions
    pub(crate) fn compile_range_dispatch(
        &mut self,
        start: &Option<Box<vex_ast::Expression>>,
        end: &Option<Box<vex_ast::Expression>>,
        inclusive: bool,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        self.compile_range(start, end, inclusive)
    }

    /// Compile type constructor expressions
    pub(crate) fn compile_type_constructor_dispatch(
        &mut self,
        type_name: &str,
        type_args: &[vex_ast::Type],
        args: &[vex_ast::Expression],
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Type constructor: Vec<i32>(), Point(10, 20)
        // Desugar to static method call: Type<T>.new(args)

        // ⭐ Phase 3: Handle Vec() without type args - will be inferred from usage
        eprintln!(
            "🔧 Type constructor: {}() with {} type args",
            type_name,
            type_args.len()
        );

        // ⭐ CRITICAL: Preserve generic type arguments!
        // Vec<i32>() should become Vec<i32>.new(), not Vec.new()
        let method_call = vex_ast::Expression::MethodCall {
            receiver: Box::new(vex_ast::Expression::Ident(type_name.to_string())),
            method: "new".to_string(),
            type_args: type_args.to_vec(), // ✅ Pass through generic type args
            args: args.to_vec(),
            is_mutable_call: false,
        };

        // Compile as static method call (handled in compile_method_call)
        self.compile_expression(&method_call)
    }

    /// Compile typeof expressions
    pub(crate) fn compile_typeof_dispatch(
        &mut self,
        expr: &vex_ast::Expression,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // typeof(expr) - returns type name as string (compile-time)
        // For now, infer the type and return a constant string
        let inferred_type = self.infer_expression_type(expr)?;
        let type_name = self.type_to_string(&inferred_type);

        // Return type name as string constant
        let global_str = self
            .builder
            .build_global_string_ptr(&type_name, "typeof_str")
            .map_err(|e| format!("Failed to create typeof string: {}", e))?;
        Ok(global_str.as_pointer_value().into())
    }
}
