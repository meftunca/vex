// AST walker for counting await expressions before code generation
// Required for pre-allocating state machine switch cases

use vex_ast::{Block, Expression, Statement};

/// Count total number of await expressions in a block (recursive)
pub(crate) fn count_await_points(block: &Block) -> usize {
    let mut count = 0;

    for statement in &block.statements {
        count += count_await_in_statement(statement);
    }

    count
}

/// Count await expressions in a single statement
fn count_await_in_statement(stmt: &Statement) -> usize {
    match stmt {
        Statement::Let { value, .. } => count_await_in_expression(value),
        Statement::LetPattern { value, .. } => count_await_in_expression(value),
        Statement::Assign {
            span_id: _,
            target,
            value,
        } => count_await_in_expression(target) + count_await_in_expression(value),
        Statement::CompoundAssign {
            span_id: _,
            target,
            op: _,
            value,
        } => count_await_in_expression(target) + count_await_in_expression(value),
        Statement::Return {
            span_id: _,
            value: Some(expr),
        } => count_await_in_expression(expr),
        Statement::Return {
            span_id: _,
            value: None,
        }
        | Statement::Break { span_id: _ }
        | Statement::Continue { span_id: _ } => 0,
        Statement::Expression(expr) => count_await_in_expression(expr),
        Statement::If {
            condition,
            then_block,
            elif_branches,
            else_block,
            span_id: _,
        } => {
            let mut count = count_await_in_expression(condition);
            count += count_await_points(then_block);
            for (elif_cond, elif_block) in elif_branches {
                count += count_await_in_expression(elif_cond);
                count += count_await_points(elif_block);
            }
            if let Some(else_b) = else_block {
                count += count_await_points(else_b);
            }
            count
        }
        Statement::While {
            condition,
            body,
            span_id: _,
        } => count_await_in_expression(condition) + count_await_points(body),
        Statement::For {
            init,
            condition,
            post,
            body,
            span_id: _,
        } => {
            let mut count = 0;
            if let Some(init_stmt) = init {
                count += count_await_in_statement(init_stmt);
            }
            if let Some(cond) = condition {
                count += count_await_in_expression(cond);
            }
            if let Some(post_stmt) = post {
                count += count_await_in_statement(post_stmt);
            }
            count += count_await_points(body);
            count
        }
        Statement::ForIn {
            iterable,
            body,
            span_id: _,
            variable,
        } => count_await_in_expression(iterable) + count_await_points(body),
        Statement::Defer(stmt) => count_await_in_statement(stmt),
        Statement::Loop { span_id: _, body } => count_await_points(body),
        Statement::Switch {
            span_id: _,
            value,
            cases,
            default_case,
        } => {
            let mut count = 0;
            if let Some(v) = value {
                count += count_await_in_expression(v);
            }
            for case in cases {
                for pattern in &case.patterns {
                    count += count_await_in_expression(pattern);
                }
                count += count_await_points(&case.body);
            }
            if let Some(default) = default_case {
                count += count_await_points(default);
            }
            count
        }
        Statement::Select { span_id: _, cases } => {
            cases.iter().map(|c| count_await_points(&c.body)).sum()
        }
        Statement::Go {
            span_id: _,
            expr,
        } => count_await_in_expression(expr),
        Statement::Unsafe {
            span_id: _,
            block,
        } => count_await_points(block),
    }
}

/// Count await expressions in an expression (recursive)
fn count_await_in_expression(expr: &Expression) -> usize {
    match expr {
        // ⭐ KEY: Await expression found!
        Expression::Await(inner) => 1 + count_await_in_expression(inner),

        // Literals - no await
        Expression::IntLiteral(_)
        | Expression::TypedIntLiteral { .. }
        | Expression::BigIntLiteral(_)
        | Expression::TypedBigIntLiteral { .. }
        | Expression::FloatLiteral(_)
        | Expression::StringLiteral(_)
        | Expression::FStringLiteral(_)
        | Expression::BoolLiteral(_)
        | Expression::Nil
        | Expression::Ident(_) => 0,

        // Binary operations
        Expression::Binary { left, right, .. } => {
            count_await_in_expression(left) + count_await_in_expression(right)
        }

        // Unary operations
        Expression::Unary { expr, .. } => count_await_in_expression(expr),
        Expression::Deref(expr) => count_await_in_expression(expr),
        Expression::Reference { expr, .. } => count_await_in_expression(expr),

        // Function/method calls
        Expression::Call { func, args, .. } => {
            let mut count = 0;
            
            // ⭐ CRITICAL: Count async_sleep as an await point
            if let Expression::Ident(name) = func.as_ref() {
                if name == "async_sleep" {
                    count += 1;
                }
            }
            
            count += count_await_in_expression(func);
            for arg in args {
                count += count_await_in_expression(arg);
            }
            count
        }
        Expression::MethodCall { receiver, args, .. } => {
            let mut count = count_await_in_expression(receiver);
            for arg in args {
                count += count_await_in_expression(arg);
            }
            count
        }

        // Field/index access
        Expression::FieldAccess { object, .. } => count_await_in_expression(object),
        Expression::Index { object, index } => {
            count_await_in_expression(object) + count_await_in_expression(index)
        }

        // Collections
        Expression::Array(elements) => elements.iter().map(|e| count_await_in_expression(e)).sum(),
        Expression::ArrayRepeat(value, count_expr) => {
            count_await_in_expression(value) + count_await_in_expression(count_expr)
        }
        Expression::MapLiteral(pairs) => pairs
            .iter()
            .map(|(k, v)| count_await_in_expression(k) + count_await_in_expression(v))
            .sum(),
        Expression::TupleLiteral(elements) => {
            elements.iter().map(|e| count_await_in_expression(e)).sum()
        }
        Expression::StructLiteral { fields, .. } => fields
            .iter()
            .map(|(_, expr)| count_await_in_expression(expr))
            .sum(),
        Expression::EnumLiteral { data, .. } => {
            data.iter().map(|e| count_await_in_expression(e)).sum()
        }

        // Ranges
        Expression::Range { start, end } | Expression::RangeInclusive { start, end } => {
            let mut count = 0;
            if let Some(s) = start {
                count += count_await_in_expression(s);
            }
            if let Some(e) = end {
                count += count_await_in_expression(e);
            }
            count
        }

        // Control flow
        Expression::Match { value, arms } => {
            let mut count = count_await_in_expression(value);
            for arm in arms {
                count += count_await_in_expression(&arm.body);
                if let Some(guard) = &arm.guard {
                    count += count_await_in_expression(guard);
                }
            }
            count
        }
        Expression::Block {
            statements,
            return_expr,
        } => {
            let mut count = 0;
            for stmt in statements {
                count += count_await_in_statement(stmt);
            }
            if let Some(expr) = return_expr {
                count += count_await_in_expression(expr);
            }
            count
        }
        Expression::AsyncBlock {
            statements,
            return_expr,
        } => {
            // Nested async block - count its await points too
            let mut count = 0;
            for stmt in statements {
                count += count_await_in_statement(stmt);
            }
            if let Some(expr) = return_expr {
                count += count_await_in_expression(expr);
            }
            count
        }

        // Other expressions
        Expression::Cast { expr, .. } => count_await_in_expression(expr),
        Expression::TryOp { expr } => count_await_in_expression(expr),
        Expression::Typeof(expr) => count_await_in_expression(expr),
        Expression::PostfixOp { expr, .. } => count_await_in_expression(expr),
        Expression::ErrorNew(expr) => count_await_in_expression(expr),
        Expression::New(expr) => count_await_in_expression(expr),
        Expression::Make { size, .. } => count_await_in_expression(size),

        // HPC - launch
        Expression::Launch { grid, args, .. } => {
            let mut count = 0;
            for g in grid {
                count += count_await_in_expression(g);
            }
            for arg in args {
                count += count_await_in_expression(arg);
            }
            count
        }

        // Closures
        Expression::Closure { body, .. } => count_await_in_expression(body),

        // Channel operations
        Expression::ChannelReceive(expr) => count_await_in_expression(expr),

        // Type constructors
        Expression::TypeConstructor { args, .. } => {
            args.iter().map(|e| count_await_in_expression(e)).sum()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_simple_await() {
        let expr = Expression::Await(Box::new(Expression::Ident("future".to_string())));
        assert_eq!(count_await_in_expression(&expr), 1);
    }

    #[test]
    fn test_count_nested_await() {
        let inner = Expression::Await(Box::new(Expression::Ident("f1".to_string())));
        let outer = Expression::Await(Box::new(inner));
        assert_eq!(count_await_in_expression(&outer), 2);
    }

    #[test]
    fn test_count_binary_with_await() {
        let left = Expression::Await(Box::new(Expression::Ident("f1".to_string())));
        let right = Expression::Await(Box::new(Expression::Ident("f2".to_string())));
        let binary = Expression::Binary {
            span_id: None,
            left: Box::new(left),
            op: vex_ast::BinaryOp::Add,
            right: Box::new(right),
        };
        assert_eq!(count_await_in_expression(&binary), 2);
    }
}
