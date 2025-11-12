// Borrow Checker Error Types

use crate::diagnostics::{error_codes, Diagnostic, ErrorLevel, Span};
use std::fmt;

/// Result type for borrow checker operations
pub type BorrowResult<T> = Result<T, BorrowError>;

/// Errors that can occur during borrow checking
#[derive(Debug, Clone, PartialEq)]
pub enum BorrowError {
    /// Attempted to assign to an immutable variable
    AssignToImmutable {
        variable: String,
        location: Option<String>,
    },

    /// ⭐ NEW: Attempted to assign to field of immutable variable
    AssignToImmutableField {
        variable: String,
        field: String,
        location: Option<String>,
    },

    /// Use of moved value (Phase 2)
    UseAfterMove {
        variable: String,
        moved_at: Option<String>,
        used_at: Option<String>,
    },

    /// Mutable borrow while already borrowed (Phase 3)
    MutableBorrowWhileBorrowed {
        variable: String,
        existing_borrow: Option<String>,
        new_borrow: Option<String>,
    },

    /// Immutable borrow while mutably borrowed (Phase 3)
    ImmutableBorrowWhileMutableBorrowed {
        variable: String,
        mutable_borrow: Option<String>,
        new_borrow: Option<String>,
    },

    /// Mutation while borrowed (Phase 3)
    MutationWhileBorrowed {
        variable: String,
        borrowed_at: Option<String>,
    },

    /// Move while borrowed (Phase 3)
    MoveWhileBorrowed {
        variable: String,
        borrow_location: Option<String>,
    },

    /// Returns reference to local variable (Phase 4)
    ReturnLocalReference { variable: String },

    /// Dangling reference - referenced variable out of scope (Phase 4)
    DanglingReference { reference: String, referent: String },

    /// Use after scope end (Phase 4)
    UseAfterScopeEnd {
        variable: String,
        available_names: Vec<String>,
    },

    /// Return dangling reference to local variable (Phase 4)
    ReturnDanglingReference { variable: String },

    /// Unsafe operation outside unsafe block (Phase 3)
    UnsafeOperationOutsideUnsafeBlock {
        operation: String,
        location: Option<String>,
    },
}

impl fmt::Display for BorrowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BorrowError::AssignToImmutable { variable, location } => {
                write!(f, "cannot assign to immutable variable `{}`", variable)?;
                if let Some(loc) = location {
                    write!(f, " at {}", loc)?;
                }
                write!(
                    f,
                    "\nhelp: consider making this binding mutable: `let! {}`",
                    variable
                )
            }

            BorrowError::AssignToImmutableField {
                variable,
                field,
                location,
            } => {
                write!(
                    f,
                    "cannot assign to field `{}` of immutable variable `{}`",
                    field, variable
                )?;
                if let Some(loc) = location {
                    write!(f, " at {}", loc)?;
                }
                write!(
                    f,
                    "\nhelp: consider making this binding mutable: `let! {}`",
                    variable
                )?;
                write!(
                    f,
                    "\nhelp: or if this is a method, add `!` to make it mutable: `fn method()!`"
                )
            }

            BorrowError::UseAfterMove {
                variable,
                moved_at,
                used_at,
            } => {
                write!(f, "use of moved value: `{}`", variable)?;
                if let Some(moved) = moved_at {
                    write!(f, "\nnote: value moved here: {}", moved)?;
                }
                if let Some(used) = used_at {
                    write!(f, "\nnote: used here: {}", used)?;
                }
                Ok(())
            }

            BorrowError::MutableBorrowWhileBorrowed {
                variable,
                existing_borrow,
                new_borrow,
            } => {
                write!(
                    f,
                    "cannot borrow `{}` as mutable because it is also borrowed as immutable",
                    variable
                )?;
                if let Some(existing) = existing_borrow {
                    write!(f, "\nnote: immutable borrow occurs here: {}", existing)?;
                }
                if let Some(new) = new_borrow {
                    write!(f, "\nnote: mutable borrow occurs here: {}", new)?;
                }
                Ok(())
            }

            BorrowError::ImmutableBorrowWhileMutableBorrowed {
                variable,
                mutable_borrow,
                new_borrow,
            } => {
                write!(
                    f,
                    "cannot borrow `{}` as immutable because it is also borrowed as mutable",
                    variable
                )?;
                if let Some(mutable) = mutable_borrow {
                    write!(f, "\nnote: mutable borrow occurs here: {}", mutable)?;
                }
                if let Some(new) = new_borrow {
                    write!(f, "\nnote: immutable borrow occurs here: {}", new)?;
                }
                Ok(())
            }

            BorrowError::MutationWhileBorrowed {
                variable,
                borrowed_at,
            } => {
                write!(f, "cannot assign to `{}` because it is borrowed", variable)?;
                if let Some(borrowed) = borrowed_at {
                    write!(f, "\nnote: borrow occurs here: {}", borrowed)?;
                }
                Ok(())
            }

            BorrowError::MoveWhileBorrowed {
                variable,
                borrow_location,
            } => {
                write!(f, "cannot move `{}` because it is borrowed", variable)?;
                if let Some(location) = borrow_location {
                    write!(f, "\nnote: {}", location)?;
                }
                Ok(())
            }

            BorrowError::ReturnLocalReference { variable } => {
                write!(
                    f,
                    "cannot return reference to local variable `{}`",
                    variable
                )?;
                write!(f, "\nhelp: consider returning an owned value instead")
            }

            BorrowError::DanglingReference {
                reference,
                referent,
            } => {
                write!(
                    f,
                    "dangling reference: `{}` references `{}` which is out of scope",
                    reference, referent
                )?;
                write!(
                    f,
                    "\nhelp: ensure the referenced value outlives the reference"
                )
            }

            BorrowError::UseAfterScopeEnd {
                variable,
                available_names,
            } => {
                write!(
                    f,
                    "use of variable `{}` after it has gone out of scope",
                    variable
                )?;
                write!(f, "\nhelp: ensure the variable is in scope before using it")?;
                if !available_names.is_empty() {
                    write!(f, "\navailable names: {}", available_names.join(", "))?;
                }
                Ok(())
            }

            BorrowError::ReturnDanglingReference { variable } => {
                write!(
                    f,
                    "cannot return reference to local variable `{}`\nthe variable will be dropped at the end of the function",
                    variable
                )?;
                write!(
                    f,
                    "\nhelp: consider returning an owned value or accepting a reference parameter"
                )
            }

            BorrowError::UnsafeOperationOutsideUnsafeBlock {
                operation,
                location,
            } => {
                write!(
                    f,
                    "unsafe operation `{}` requires unsafe block",
                    operation
                )?;
                if let Some(loc) = location {
                    write!(f, " at {}", loc)?;
                }
                write!(
                    f,
                    "\nhelp: wrap this operation in an `unsafe {{ }}` block"
                )
            }
        }
    }
}

impl std::error::Error for BorrowError {}

impl BorrowError {
    /// Convert borrow error to diagnostic for pretty printing
    pub fn to_diagnostic(&self) -> Diagnostic {
        match self {
            BorrowError::AssignToImmutable { variable, location } => {
                let mut notes = vec![format!("variable `{}` is immutable", variable)];
                if let Some(loc) = location {
                    notes.push(format!("assignment at {}", loc));
                }

                Diagnostic {
                    level: ErrorLevel::Error,
                    code: error_codes::IMMUTABLE_ASSIGN.to_string(),
                    message: format!("cannot assign to immutable variable `{}`", variable),
                    span: Span::unknown(),
                    notes,
                    help: Some(format!(
                        "consider making this binding mutable: `let! {}`",
                        variable
                    )),
                    suggestion: None,
                }
            }

            BorrowError::AssignToImmutableField {
                variable,
                field,
                location,
            } => {
                let mut notes = vec![
                    format!("variable `{}` is immutable", variable),
                    format!("attempting to assign to field `{}`", field),
                ];
                if let Some(loc) = location {
                    notes.push(format!("assignment at {}", loc));
                }

                Diagnostic {
                    level: ErrorLevel::Error,
                    code: error_codes::IMMUTABLE_ASSIGN.to_string(),
                    message: format!("cannot assign to field `{}` of immutable variable `{}`", field, variable),
                    span: Span::unknown(),
                    notes,
                    help: Some(format!("consider making this binding mutable: `let! {}`, or if this is a method, add `!` to make it mutable: `fn method()!`", variable)),
                    suggestion: None,
                }
            }

            BorrowError::UseAfterMove {
                variable,
                moved_at,
                used_at,
            } => {
                let mut notes = vec![];
                if let Some(moved) = moved_at {
                    notes.push(format!("value moved here: {}", moved));
                }
                if let Some(used) = used_at {
                    notes.push(format!("value used here: {}", used));
                }

                Diagnostic {
                    level: ErrorLevel::Error,
                    code: error_codes::USE_AFTER_MOVE.to_string(),
                    message: format!("use of moved value: `{}`", variable),
                    span: Span::unknown(),
                    notes,
                    help: Some("consider cloning the value if it needs to be used after move, or use references".to_string()),
                    suggestion: None,
                }
            }

            BorrowError::MutableBorrowWhileBorrowed {
                variable,
                existing_borrow,
                new_borrow,
            } => {
                let mut notes = vec![];
                if let Some(existing) = existing_borrow {
                    notes.push(format!("immutable borrow occurs here: {}", existing));
                }
                if let Some(new) = new_borrow {
                    notes.push(format!("mutable borrow occurs here: {}", new));
                }

                Diagnostic {
                    level: ErrorLevel::Error,
                    code: error_codes::MULTIPLE_MUTABLE_BORROW.to_string(),
                    message: format!(
                        "cannot borrow `{}` as mutable because it is also borrowed as immutable",
                        variable
                    ),
                    span: Span::unknown(),
                    notes,
                    help: Some("mutable references require exclusive access".to_string()),
                    suggestion: None,
                }
            }

            BorrowError::ImmutableBorrowWhileMutableBorrowed {
                variable,
                mutable_borrow,
                new_borrow,
            } => {
                let mut notes = vec![];
                if let Some(mutable) = mutable_borrow {
                    notes.push(format!("mutable borrow occurs here: {}", mutable));
                }
                if let Some(new) = new_borrow {
                    notes.push(format!("immutable borrow occurs here: {}", new));
                }

                Diagnostic {
                    level: ErrorLevel::Error,
                    code: error_codes::BORROW_ERROR.to_string(),
                    message: format!(
                        "cannot borrow `{}` as immutable because it is also borrowed as mutable",
                        variable
                    ),
                    span: Span::unknown(),
                    notes,
                    help: Some("mutable borrows prevent other borrows while active".to_string()),
                    suggestion: None,
                }
            }

            BorrowError::MutationWhileBorrowed {
                variable,
                borrowed_at,
            } => {
                let mut notes = vec![];
                if let Some(borrowed) = borrowed_at {
                    notes.push(format!("borrow occurs here: {}", borrowed));
                }

                Diagnostic {
                    level: ErrorLevel::Error,
                    code: error_codes::BORROW_ERROR.to_string(),
                    message: format!("cannot assign to `{}` because it is borrowed", variable),
                    span: Span::unknown(),
                    notes,
                    help: Some(
                        "borrowed values cannot be mutated while the borrow is active".to_string(),
                    ),
                    suggestion: None,
                }
            }

            BorrowError::MoveWhileBorrowed {
                variable,
                borrow_location,
            } => {
                let mut notes = vec![];
                if let Some(location) = borrow_location {
                    notes.push(location.clone());
                }

                Diagnostic {
                    level: ErrorLevel::Error,
                    code: error_codes::MOVE_ERROR.to_string(),
                    message: format!("cannot move `{}` because it is borrowed", variable),
                    span: Span::unknown(),
                    notes,
                    help: Some("cannot move out of a borrowed value".to_string()),
                    suggestion: None,
                }
            }

            BorrowError::ReturnLocalReference { variable } => Diagnostic {
                level: ErrorLevel::Error,
                code: error_codes::RETURN_LOCAL_REF.to_string(),
                message: format!("cannot return reference to local variable `{}`", variable),
                span: Span::unknown(),
                notes: vec!["local variable will be dropped at the end of the function".to_string()],
                help: Some("consider returning an owned value instead".to_string()),
                suggestion: None,
            },

            BorrowError::DanglingReference {
                reference,
                referent,
            } => Diagnostic {
                level: ErrorLevel::Error,
                code: error_codes::DANGLING_REFERENCE.to_string(),
                message: format!(
                    "dangling reference: `{}` references `{}` which is out of scope",
                    reference, referent
                ),
                span: Span::unknown(),
                notes: vec!["the referenced value has gone out of scope".to_string()],
                help: Some("ensure the referenced value outlives the reference".to_string()),
                suggestion: None,
            },

            BorrowError::UseAfterScopeEnd {
                variable,
                available_names,
            } => {
                // Use fuzzy matching to find similar names
                let suggestions = crate::diagnostics::fuzzy::find_similar_names(
                    &variable,
                    available_names,
                    0.7, // similarity threshold
                    3,   // max suggestions
                );

                let help_msg = if suggestions.is_empty() {
                    "ensure the variable is in scope before using it".to_string()
                } else {
                    format!("did you mean `{}`?", suggestions.join("`, `"))
                };

                Diagnostic {
                    level: ErrorLevel::Error,
                    code: error_codes::LIFETIME_ERROR.to_string(),
                    message: format!(
                        "use of variable `{}` after it has gone out of scope",
                        variable
                    ),
                    span: Span::unknown(),
                    notes: vec!["the variable is no longer accessible".to_string()],
                    help: Some(help_msg),
                    suggestion: None,
                }
            }

            BorrowError::ReturnDanglingReference { variable } => Diagnostic {
                level: ErrorLevel::Error,
                code: error_codes::RETURN_LOCAL_REF.to_string(),
                message: format!("cannot return reference to local variable `{}`", variable),
                span: Span::unknown(),
                notes: vec!["the variable will be dropped at the end of the function".to_string()],
                help: Some(
                    "consider returning an owned value or accepting a reference parameter"
                        .to_string(),
                ),
                suggestion: None,
            },

            BorrowError::UnsafeOperationOutsideUnsafeBlock {
                operation,
                location,
            } => {
                let mut notes = vec![format!("operation `{}` is unsafe", operation)];
                if let Some(loc) = location {
                    notes.push(format!("occurs at {}", loc));
                }

                Diagnostic {
                    level: ErrorLevel::Error,
                    code: error_codes::UNSAFE_REQUIRED.to_string(),
                    message: format!("unsafe operation `{}` requires unsafe block", operation),
                    span: Span::unknown(),
                    notes,
                    help: Some("wrap this operation in an `unsafe { }` block".to_string()),
                    suggestion: None,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = BorrowError::AssignToImmutable {
            variable: "x".to_string(),
            location: None,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("cannot assign to immutable variable"));
        assert!(msg.contains("let! x"));
    }

    #[test]
    fn test_use_after_move_display() {
        let err = BorrowError::UseAfterMove {
            variable: "vec".to_string(),
            moved_at: Some("line 5".to_string()),
            used_at: Some("line 7".to_string()),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("use of moved value"));
        assert!(msg.contains("line 5"));
    }
}
