// Borrow Checker Error Types

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
    UseAfterScopeEnd { variable: String },

    /// Return dangling reference to local variable (Phase 4)
    ReturnDanglingReference { variable: String },
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

            BorrowError::UseAfterScopeEnd { variable } => {
                write!(
                    f,
                    "use of variable `{}` after it has gone out of scope",
                    variable
                )?;
                write!(f, "\nhelp: ensure the variable is in scope before using it")
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
        }
    }
}

impl std::error::Error for BorrowError {}

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
