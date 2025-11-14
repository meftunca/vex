//! Pattern variable declaration helpers

use super::checker::MoveChecker;
use vex_ast::Pattern;

impl MoveChecker {
    pub(super) fn declare_pattern_variables(&mut self, pattern: &Pattern) {
        match pattern {
            Pattern::Ident(name) => {
                self.moved_vars.remove(name);
                self.valid_vars.insert(name.clone());
            }
            Pattern::Tuple(patterns) => {
                for p in patterns {
                    self.declare_pattern_variables(p);
                }
            }
            Pattern::Struct { fields, .. } => {
                for (_, p) in fields {
                    self.declare_pattern_variables(p);
                }
            }
            Pattern::Enum { data, .. } => {
                for p in data {
                    self.declare_pattern_variables(p);
                }
            }
            Pattern::Array { elements, .. } => {
                for p in elements {
                    self.declare_pattern_variables(p);
                }
            }
            Pattern::Wildcard | Pattern::Literal(_) | Pattern::Or(_) => {}
        }
    }
}
