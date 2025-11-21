// Closure Trait Analysis
// Phase 5: Analyze closure capture modes and determine Callable traits

use crate::borrow_checker::closure_traits::analyze_closure_body;
use crate::borrow_checker::errors::BorrowResult;
use vex_ast::{Expression, Item, Program, Statement};

impl super::BorrowChecker {
    /// Phase 5: Analyze all closures and set their capture modes
    /// This determines which trait each closure implements:
    /// - Callable: Immutable capture (can call multiple times)
    /// - CallableMut: Mutable capture (can call multiple times, mutates environment)
    /// - CallableOnce: Move capture (can only call once, takes ownership)
    pub(super) fn analyze_closure_traits(&mut self, program: &mut Program) -> BorrowResult<()> {
        for item in &mut program.items {
            self.analyze_item_closures(item)?;
        }
        Ok(())
    }

    /// Recursively analyze closures in an item
    fn analyze_item_closures(&mut self, item: &mut Item) -> BorrowResult<()> {
        match item {
            Item::Function(func) => {
                for stmt in &mut func.body.statements {
                    self.analyze_statement_closures(stmt)?;
                }
                Ok(())
            }
            Item::TraitImpl(trait_impl) => {
                for func in &mut trait_impl.methods {
                    for stmt in &mut func.body.statements {
                        self.analyze_statement_closures(stmt)?;
                    }
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Recursively analyze closures in a statement
    fn analyze_statement_closures(&mut self, stmt: &mut Statement) -> BorrowResult<()> {
        match stmt {
            Statement::Let { value, .. } => {
                self.analyze_expression_closures(value)?;
            }
            Statement::LetPattern { value, .. } => {
                self.analyze_expression_closures(value)?;
            }
            Statement::Assign {
                span_id: _,
                target,
                value,
            } => {
                self.analyze_expression_closures(target)?;
                self.analyze_expression_closures(value)?;
            }
            Statement::Return {
                span_id: _,
                value: Some(expr),
            } => {
                self.analyze_expression_closures(expr)?;
            }
            Statement::Expression(expr) => {
                self.analyze_expression_closures(expr)?;
            }
            Statement::If {
                span_id: _,
                condition,
                then_block,
                elif_branches,
                else_block,
            } => {
                self.analyze_expression_closures(condition)?;
                for stmt in &mut then_block.statements {
                    self.analyze_statement_closures(stmt)?;
                }
                for (cond, block) in elif_branches {
                    self.analyze_expression_closures(cond)?;
                    for stmt in &mut block.statements {
                        self.analyze_statement_closures(stmt)?;
                    }
                }
                if let Some(block) = else_block {
                    for stmt in &mut block.statements {
                        self.analyze_statement_closures(stmt)?;
                    }
                }
            }
            Statement::While {
                span_id: _,
                condition,
                body,
            } => {
                self.analyze_expression_closures(condition)?;
                for stmt in &mut body.statements {
                    self.analyze_statement_closures(stmt)?;
                }
            }
            Statement::Loop { span_id: _, body } => {
                for stmt in &mut body.statements {
                    self.analyze_statement_closures(stmt)?;
                }
            }
            Statement::For {
                span_id: _,
                init,
                condition,
                post,
                body,
            } => {
                if let Some(init_stmt) = init {
                    self.analyze_statement_closures(init_stmt)?;
                }
                if let Some(cond) = condition {
                    self.analyze_expression_closures(cond)?;
                }
                if let Some(post_stmt) = post {
                    self.analyze_statement_closures(post_stmt)?;
                }
                for stmt in &mut body.statements {
                    self.analyze_statement_closures(stmt)?;
                }
            }
            Statement::ForIn { iterable, body, .. } => {
                self.analyze_expression_closures(iterable)?;
                for stmt in &mut body.statements {
                    self.analyze_statement_closures(stmt)?;
                }
            }
            Statement::Switch {
                span_id: _,
                value,
                cases,
                default_case,
            } => {
                if let Some(val) = value {
                    self.analyze_expression_closures(val)?;
                }
                for case in cases {
                    for stmt in &mut case.body.statements {
                        self.analyze_statement_closures(stmt)?;
                    }
                }
                if let Some(default_block) = default_case {
                    for stmt in &mut default_block.statements {
                        self.analyze_statement_closures(stmt)?;
                    }
                }
            }
            Statement::Select { .. }
            | Statement::Unsafe {
                span_id: _,
                block: _,
            } => {
                // These don't typically contain user closures
            }
            Statement::Defer(stmt) => {
                self.analyze_statement_closures(stmt)?;
            }
            Statement::Go { span_id: _, expr } => {
                self.analyze_expression_closures(expr)?;
            }
            Statement::CompoundAssign {
                span_id: _,
                target,
                op: _,
                value,
            } => {
                self.analyze_expression_closures(target)?;
                self.analyze_expression_closures(value)?;
            }
            Statement::Break { span_id: _ }
            | Statement::Continue { span_id: _ }
            | Statement::Return {
                span_id: _,
                value: None,
            } => {
                // No expressions to analyze
            }
        }
        Ok(())
    }

    /// Recursively analyze closures in an expression
    fn analyze_expression_closures(&mut self, expr: &mut Expression) -> BorrowResult<()> {
        match expr {
            Expression::Closure {
                params,
                body,
                capture_mode,
                ..
            } => {
                // First, recursively analyze nested closures in the body
                self.analyze_expression_closures(body)?;

                // Then analyze this closure and determine its capture mode
                let analyzed_mode = analyze_closure_body(params, body);

                // Update the capture mode from Infer to the analyzed result
                *capture_mode = analyzed_mode;

                Ok(())
            }
            Expression::Binary {
                span_id: _,
                left,
                right,
                ..
            } => {
                self.analyze_expression_closures(left)?;
                self.analyze_expression_closures(right)?;
                Ok(())
            }
            Expression::Unary {
                span_id: _, expr, ..
            } => {
                self.analyze_expression_closures(expr)?;
                Ok(())
            }
            Expression::Call { func, args, .. } => {
                self.analyze_expression_closures(func)?;
                for arg in args {
                    self.analyze_expression_closures(arg)?;
                }
                Ok(())
            }
            Expression::MethodCall { receiver, args, .. } => {
                self.analyze_expression_closures(receiver)?;
                for arg in args {
                    self.analyze_expression_closures(arg)?;
                }
                Ok(())
            }
            Expression::FieldAccess { object, .. } => {
                self.analyze_expression_closures(object)?;
                Ok(())
            }
            Expression::Index { object, index } => {
                self.analyze_expression_closures(object)?;
                self.analyze_expression_closures(index)?;
                Ok(())
            }
            Expression::Array(elements) => {
                for elem in elements {
                    self.analyze_expression_closures(elem)?;
                }
                Ok(())
            }
            Expression::ArrayRepeat(value, count) => {
                self.analyze_expression_closures(value)?;
                self.analyze_expression_closures(count)?;
                Ok(())
            }
            Expression::TupleLiteral(elements) => {
                for elem in elements {
                    self.analyze_expression_closures(elem)?;
                }
                Ok(())
            }
            Expression::StructLiteral { fields, .. } => {
                for (_, value) in fields {
                    self.analyze_expression_closures(value)?;
                }
                Ok(())
            }
            Expression::MapLiteral(entries) => {
                for (key, value) in entries {
                    self.analyze_expression_closures(key)?;
                    self.analyze_expression_closures(value)?;
                }
                Ok(())
            }
            Expression::EnumLiteral { data, .. } => {
                for data_expr in data {
                    self.analyze_expression_closures(data_expr)?;
                }
                Ok(())
            }
            Expression::Block {
                statements,
                return_expr,
            } => {
                for stmt in statements {
                    self.analyze_statement_closures(stmt)?;
                }
                if let Some(expr) = return_expr {
                    self.analyze_expression_closures(expr)?;
                }
                Ok(())
            }
            Expression::AsyncBlock {
                statements,
                return_expr,
            } => {
                for stmt in statements {
                    self.analyze_statement_closures(stmt)?;
                }
                if let Some(expr) = return_expr {
                    self.analyze_expression_closures(expr)?;
                }
                Ok(())
            }
            Expression::Match { value, arms } => {
                self.analyze_expression_closures(value)?;
                for arm in arms {
                    if let Some(guard) = &mut arm.guard {
                        self.analyze_expression_closures(guard)?;
                    }
                    self.analyze_expression_closures(&mut arm.body)?;
                }
                Ok(())
            }
            Expression::Reference { expr, .. } => {
                self.analyze_expression_closures(expr)?;
                Ok(())
            }
            Expression::Range { start, end } | Expression::RangeInclusive { start, end } => {
                if let Some(s) = start {
                    self.analyze_expression_closures(s)?;
                }
                if let Some(e) = end {
                    self.analyze_expression_closures(e)?;
                }
                Ok(())
            }
            Expression::Await(expr)
            | Expression::TryOp { expr }
            | Expression::New(expr)
            | Expression::Deref(expr)
            | Expression::ChannelReceive(expr)
            | Expression::ErrorNew(expr) => {
                self.analyze_expression_closures(expr)?;
                Ok(())
            }
            Expression::Cast { expr, .. } => {
                self.analyze_expression_closures(expr)?;
                Ok(())
            }
            Expression::PostfixOp { expr, .. } => {
                self.analyze_expression_closures(expr)?;
                Ok(())
            }
            Expression::Make { size, .. } => {
                self.analyze_expression_closures(size)?;
                Ok(())
            }
            Expression::Launch { grid, args, .. } => {
                for expr in grid {
                    self.analyze_expression_closures(expr)?;
                }
                for arg in args {
                    self.analyze_expression_closures(arg)?;
                }
                Ok(())
            }
            // Literals and identifiers have no nested closures
            Expression::IntLiteral(_)
            | Expression::TypedIntLiteral { .. }
            | Expression::BigIntLiteral(_)
            | Expression::TypedBigIntLiteral { .. }
            | Expression::FloatLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::FStringLiteral(_)
            | Expression::BoolLiteral(_)
            | Expression::Nil
            | Expression::Ident(_) => Ok(()),

            Expression::Typeof(expr) => {
                // Check nested closures in typeof expression
                self.analyze_expression_closures(expr)
            }

            Expression::TypeConstructor { args, .. } => {
                // Check nested closures in constructor arguments
                for arg in args {
                    self.analyze_expression_closures(arg)?;
                }
                Ok(())
            }
        }
    }
}
