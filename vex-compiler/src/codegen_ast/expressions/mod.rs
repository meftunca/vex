// Expression code generation
// This module dispatches expression compilation and coordinates submodules

use super::ASTCodeGen;
use crate::diagnostics::{error_codes, Diagnostic, ErrorLevel, Span};
use inkwell::types::BasicTypeEnum;
use inkwell::values::BasicValueEnum;
use vex_ast::*;

// Submodules
mod access;
mod binary_ops;
mod calls;
mod control;
mod control_flow;
mod identifiers;
mod literals;
mod literals_expressions;
mod operators;
mod references;
mod special_expressions;
mod structs_enums;
pub(crate) mod pattern_matching;
mod special;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Main expression compiler - dispatches to specialized methods
    pub(crate) fn compile_expression(
        &mut self,
        expr: &vex_ast::Expression,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        match expr {
            Expression::IntLiteral(_) | Expression::BigIntLiteral(_) | Expression::FloatLiteral(_) | Expression::BoolLiteral(_) | Expression::StringLiteral(_) | Expression::FStringLiteral(_) | Expression::Nil => {
                self.compile_literal(expr)
            }

            Expression::Ident(name) => {
                self.compile_identifier(name)
            }

            Expression::Binary {
                span_id: _,
                left,
                op,
                right,
            } => self.compile_binary_op_dispatch(left, op, right),

            Expression::Unary {
                span_id: _,
                op,
                expr,
            } => self.compile_unary_op_dispatch(op, expr),

            Expression::Call {
                func,
                type_args,
                args,
                ..
            } => self.compile_call(func, type_args, args),

            Expression::MethodCall {
                receiver,
                method,
                type_args,
                args,
                is_mutable_call,
            } => self.compile_method_call(receiver, method, type_args, args, *is_mutable_call),

            Expression::Index { object, index } => self.compile_index(object, index),

            Expression::Array(elements) => self.compile_array_dispatch(elements),

            Expression::ArrayRepeat(value, count) => {
                self.compile_array_repeat_dispatch(value, count)
            }

            Expression::MapLiteral(entries) => self.compile_map_dispatch(entries),

            Expression::TupleLiteral(elements) => self.compile_tuple_dispatch(elements),

            Expression::StructLiteral {
                name,
                type_args,
                fields,
            } => self.compile_struct_dispatch(name, type_args, fields),

            Expression::FieldAccess { object, field } => self.compile_field_access(object, field),

            Expression::PostfixOp { expr, op } => self.compile_postfix_op_dispatch(expr, op),

            Expression::Await(expr) => {
                // Await expression: suspend coroutine and yield to scheduler
                // 1. Compile the future expression
                // 2. Check if it's ready (for now, assume always ready - TODO: poll)
                // 3. Call worker_await_after to yield control
                // 4. Return CORO_STATUS_YIELDED

                let _future_val = self.compile_expression(expr)?;

                // Get current WorkerContext (first parameter of resume function)
                let current_fn = self
                    .current_function
                    .ok_or_else(|| "Await outside of function".to_string())?;

                // Check if we're in an async function (resume function has WorkerContext* param)
                let is_in_async = current_fn
                    .get_name()
                    .to_str()
                    .map(|n| n.ends_with("_resume"))
                    .unwrap_or(false);

                if !is_in_async {
                    return Err("Await can only be used inside async functions".to_string());
                }

                // Get WorkerContext parameter (first param)
                let ctx_param = current_fn
                    .get_nth_param(0)
                    .ok_or_else(|| "Missing WorkerContext parameter".to_string())?
                    .into_pointer_value();

                // Call worker_await_after(ctx, 0) to yield
                let worker_await_fn = self.get_or_declare_worker_await();
                self.builder
                    .build_call(
                        worker_await_fn,
                        &[
                            ctx_param.into(),
                            self.context.i64_type().const_int(0, false).into(),
                        ],
                        "await_yield",
                    )
                    .map_err(|e| format!("Failed to call worker_await_after: {}", e))?;

                // Return CORO_STATUS_YIELDED (1)
                let yielded_status = self.context.i32_type().const_int(1, false);
                self.builder
                    .build_return(Some(&yielded_status))
                    .map_err(|e| format!("Failed to build await return: {}", e))?;

                // For type system compatibility, return a dummy value
                // (this code is unreachable after return)
                Ok(self.context.i8_type().const_int(0, false).into())
            }

            Expression::Match { value, arms } => self.compile_match_dispatch(value, arms),

            Expression::Block {
                statements,
                return_expr,
            } => self.compile_block_dispatch(statements, return_expr),

            Expression::QuestionMark(expr) => self.compile_question_mark_dispatch(expr),

            Expression::Reference { is_mutable, expr } => {
                self.compile_reference_dispatch(*is_mutable, expr)
            }

            Expression::Deref(expr) => {
                self.compile_dereference_dispatch(expr)
            }

            Expression::ChannelReceive(channel_expr) => {
                self.compile_channel_receive_dispatch(channel_expr)
            }

            Expression::EnumLiteral {
                enum_name,
                variant,
                data,
            } => self.compile_enum_dispatch(enum_name, variant, data),

            Expression::Closure {
                params,
                return_type,
                body,
                capture_mode,
            } => self.compile_closure_dispatch(params, return_type, body, capture_mode),

            Expression::Cast { expr, target_type } => {
                self.compile_cast_dispatch(expr, target_type)
            }

            Expression::Range { start, end } => self.compile_range_dispatch(start, end, false),

            Expression::RangeInclusive { start, end } => self.compile_range_dispatch(start, end, true),

            Expression::TypeConstructor {
                type_name,
                type_args,
                args,
            } => self.compile_type_constructor_dispatch(type_name, type_args, args),

            Expression::Typeof(expr) => self.compile_typeof_dispatch(expr),

            _ => {
                let expr_str = format!("{:?}", expr);
                self.diagnostics.emit(Diagnostic {
                    level: ErrorLevel::Error,
                    code: error_codes::NOT_IMPLEMENTED.to_string(),
                    message: "This expression type is not yet implemented".to_string(),
                    span: Span::unknown(),
                    notes: vec![format!("Expression: {}", expr_str)],
                    help: Some("This feature is planned for a future release".to_string()),
                    suggestion: None,
                });
                Err(format!("Expression not yet implemented: {:?}", expr))
            }
        }
    }
    /// Compile Range or RangeInclusive expressions
    fn compile_range(
        &mut self,
        start: &Option<Box<Expression>>,
        end: &Option<Box<Expression>>,
        _inclusive: bool,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Default values: start=0, end=max_i64
        let zero_i64 = self.context.i64_type().const_int(0, false);
        let max_i64 = self.context.i64_type().const_int(i64::MAX as u64, false);

        // Compile start expression or use 0
        let start_i64 = if let Some(s) = start {
            let start_val = self.compile_expression(s)?;
            if start_val.is_int_value() {
                let int_val = start_val.into_int_value();
                if int_val.get_type().get_bit_width() < 64 {
                    self.builder
                        .build_int_s_extend(int_val, self.context.i64_type(), "start_ext")
                        .map_err(|e| format!("Failed to extend start: {}", e))?
                } else {
                    int_val
                }
            } else {
                return Err("Range start must be an integer".to_string());
            }
        } else {
            zero_i64
        };

        // Compile end expression or use max_i64
        let end_i64 = if let Some(e) = end {
            let end_val = self.compile_expression(e)?;
            if end_val.is_int_value() {
                let int_val = end_val.into_int_value();
                if int_val.get_type().get_bit_width() < 64 {
                    self.builder
                        .build_int_s_extend(int_val, self.context.i64_type(), "end_ext")
                        .map_err(|e| format!("Failed to extend end: {}", e))?
                } else {
                    int_val
                }
            } else {
                return Err("Range end must be an integer".to_string());
            }
        } else {
            max_i64
        };

        // Create Range struct: { start: i64, end: i64, current: i64 }
        let range_struct_type = self.context.struct_type(
            &[
                self.context.i64_type().into(),
                self.context.i64_type().into(),
                self.context.i64_type().into(),
            ],
            false,
        );

        // Build struct value
        let mut range_val = range_struct_type.get_undef();
        range_val = self
            .builder
            .build_insert_value(range_val, start_i64, 0, "range_start")
            .map_err(|e| format!("Failed to insert start: {}", e))?
            .into_struct_value();
        range_val = self
            .builder
            .build_insert_value(range_val, end_i64, 1, "range_end")
            .map_err(|e| format!("Failed to insert end: {}", e))?
            .into_struct_value();
        range_val = self
            .builder
            .build_insert_value(range_val, start_i64, 2, "range_current")
            .map_err(|e| format!("Failed to insert current: {}", e))?
            .into_struct_value();

        Ok(range_val.into())
    }
}
