// Closure trait analysis for Vex borrow checker
// Determines which closure trait (Callable, CallableMut, CallableOnce) a closure implements

use std::collections::{HashMap, HashSet};
use vex_ast::*;

/// Analyze closure to determine which trait it should implement
pub fn analyze_closure_trait(closure: &Expression, params: &[Param]) -> CaptureMode {
    if let Expression::Closure { body, .. } = closure {
        let analysis = CaptureAnalyzer::new(params);
        analysis.analyze_body(body)
    } else {
        CaptureMode::Infer
    }
}

/// Analyze closure body directly to determine capture mode
/// Used when we already have the body and params separately
pub fn analyze_closure_body(params: &[Param], body: &Expression) -> CaptureMode {
    let analysis = CaptureAnalyzer::new(params);
    analysis.analyze_body(body)
}

struct CaptureAnalyzer<'a> {
    param_names: HashSet<String>,
    captured_vars: HashMap<String, CaptureInfo>,
    local_vars: HashSet<String>,
    _phantom: std::marker::PhantomData<&'a ()>,
}

#[derive(Debug, Clone)]
struct CaptureInfo {
    is_mutated: bool,
    is_moved: bool,
}

impl<'a> CaptureAnalyzer<'a> {
    fn new(params: &[Param]) -> Self {
        let param_names: HashSet<String> = params.iter().map(|p| p.name.clone()).collect();

        Self {
            param_names,
            captured_vars: HashMap::new(),
            local_vars: HashSet::new(),
            _phantom: std::marker::PhantomData,
        }
    }

    fn analyze_body(&self, body: &Expression) -> CaptureMode {
        // Clone self to make mutable
        let mut analyzer = CaptureAnalyzer {
            param_names: self.param_names.clone(),
            captured_vars: self.captured_vars.clone(),
            local_vars: self.local_vars.clone(),
            _phantom: std::marker::PhantomData,
        };

        analyzer.visit_expression(body);

        // Determine capture mode based on how variables are used
        analyzer.infer_capture_mode()
    }

    fn visit_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::Ident(name) => {
                // Check if this is a captured variable (not param, not local)
                if !self.param_names.contains(name) && !self.local_vars.contains(name) {
                    // This is a captured variable - mark as used
                    self.captured_vars
                        .entry(name.clone())
                        .or_insert(CaptureInfo {
                            is_mutated: false,
                            is_moved: false,
                        });
                }
            }

            // Note: Assignment is handled as Statement, not Binary expression in Vex
            Expression::Binary { span_id: _,  left, right, .. } => {
                self.visit_expression(left);
                self.visit_expression(right);
            }

            Expression::Unary { span_id: _,  expr, .. } => {
                self.visit_expression(expr);
            }

            Expression::Call { func, args, .. } => {
                self.visit_expression(func);
                for arg in args {
                    self.visit_expression(arg);
                }
            }

            Expression::MethodCall { receiver, args, .. } => {
                self.visit_expression(receiver);
                for arg in args {
                    self.visit_expression(arg);
                }
            }

            Expression::Block {
                statements,
                return_expr,
            } => {
                for stmt in statements {
                    self.visit_statement(stmt);
                }
                if let Some(ret) = return_expr {
                    self.visit_expression(ret);
                }
            }

            Expression::Match { value, arms } => {
                self.visit_expression(value);
                for arm in arms {
                    if let Some(guard) = &arm.guard {
                        self.visit_expression(guard);
                    }
                    self.visit_expression(&arm.body);
                }
            }

            Expression::Array(elements) => {
                for elem in elements {
                    self.visit_expression(elem);
                }
            }

            Expression::TupleLiteral(elements) => {
                for elem in elements {
                    self.visit_expression(elem);
                }
            }

            Expression::StructLiteral { fields, .. } => {
                for (_, expr) in fields {
                    self.visit_expression(expr);
                }
            }

            Expression::FieldAccess { object, .. } => {
                self.visit_expression(object);
            }

            Expression::Index { object, index } => {
                self.visit_expression(object);
                self.visit_expression(index);
            }

            Expression::Reference { expr, .. } => {
                self.visit_expression(expr);
            }

            Expression::Deref(expr) => {
                self.visit_expression(expr);
            }

            Expression::Closure { body, params, .. } => {
                // Nested closure - add its params to local vars temporarily
                let saved_locals = self.local_vars.clone();
                for param in params {
                    self.local_vars.insert(param.name.clone());
                }
                self.visit_expression(body);
                self.local_vars = saved_locals;
            }

            // Literals don't capture
            Expression::IntLiteral(_)
            | Expression::FloatLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::FStringLiteral(_)
            | Expression::BoolLiteral(_)
            | Expression::Nil => {}

            _ => {
                // Other expression types - conservatively mark as needing analysis
            }
        }
    }

    fn visit_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Let { name, value, .. } => {
                // New local variable in closure
                self.local_vars.insert(name.clone());
                self.visit_expression(value);
            }

            Statement::Assign { target, value } => {
                // Check if we're mutating a captured variable
                if let Expression::Ident(name) = target {
                    if !self.param_names.contains(name) && !self.local_vars.contains(name) {
                        // Mutating a captured variable
                        if let Some(info) = self.captured_vars.get_mut(name) {
                            info.is_mutated = true;
                        } else {
                            self.captured_vars.insert(
                                name.clone(),
                                CaptureInfo {
                                    is_mutated: true,
                                    is_moved: false,
                                },
                            );
                        }
                    }
                }
                self.visit_expression(value);
            }

            Statement::CompoundAssign { target, value, .. } => {
                // Compound assignment also mutates
                if let Expression::Ident(name) = target {
                    if !self.param_names.contains(name) && !self.local_vars.contains(name) {
                        if let Some(info) = self.captured_vars.get_mut(name) {
                            info.is_mutated = true;
                        } else {
                            self.captured_vars.insert(
                                name.clone(),
                                CaptureInfo {
                                    is_mutated: true,
                                    is_moved: false,
                                },
                            );
                        }
                    }
                }
                self.visit_expression(value);
            }

            Statement::Return(expr) => {
                if let Some(e) = expr {
                    self.visit_expression(e);
                }
            }

            Statement::If { span_id: _, 
                condition,
                then_block,
                elif_branches,
                else_block,
            } => {
                self.visit_expression(condition);
                self.visit_block(then_block);
                for (cond, block) in elif_branches {
                    self.visit_expression(cond);
                    self.visit_block(block);
                }
                if let Some(else_b) = else_block {
                    self.visit_block(else_b);
                }
            }

            Statement::While { span_id: _,  condition, body } => {
                self.visit_expression(condition);
                self.visit_block(body);
            }

            Statement::For { span_id: _, 
                init,
                condition,
                post,
                body,
            } => {
                if let Some(i) = init {
                    self.visit_statement(i);
                }
                if let Some(c) = condition {
                    self.visit_expression(c);
                }
                if let Some(p) = post {
                    self.visit_statement(p);
                }
                self.visit_block(body);
            }

            Statement::ForIn {
                iterable,
                body,
                variable,
            } => {
                self.local_vars.insert(variable.clone());
                self.visit_expression(iterable);
                self.visit_block(body);
            }

            // Match is an expression, not a statement in Vex
            Statement::Defer(stmt) => {
                self.visit_statement(stmt);
            }
            Statement::Go(expr) => {
                self.visit_expression(expr);
            }

            Statement::Break | Statement::Continue => {}

            Statement::Expression(expr) => {
                self.visit_expression(expr);
            }

            _ => {}
        }
    }

    fn visit_block(&mut self, block: &Block) {
        for stmt in &block.statements {
            self.visit_statement(stmt);
        }
    }

    fn infer_capture_mode(&self) -> CaptureMode {
        // Rule 1: If any captured variable is moved, use CallableOnce
        for info in self.captured_vars.values() {
            if info.is_moved {
                return CaptureMode::Once;
            }
        }

        // Rule 2: If any captured variable is mutated, use CallableMut
        for info in self.captured_vars.values() {
            if info.is_mutated {
                return CaptureMode::Mutable;
            }
        }

        // Rule 3: Otherwise, use Callable (immutable capture)
        if self.captured_vars.is_empty() {
            // No captures - pure function, implements all traits
            CaptureMode::Immutable
        } else {
            // Has captures but only reads them
            CaptureMode::Immutable
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_immutable_capture() {
        // let x = 5; let f = |y| x + y;
        // Should infer Callable (Immutable)
    }

    #[test]
    fn test_mutable_capture() {
        // let! counter = 0; let f = || { counter = counter + 1; };
        // Should infer CallableMut (Mutable)
    }

    #[test]
    fn test_move_capture() {
        // let data = vec![1,2,3]; let f = || process(data);
        // Should infer CallableOnce (Once)
    }
}
