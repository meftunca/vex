//! Type classification - determining Copy vs Move types

use super::checker::MoveChecker;
use vex_ast::Type;

impl MoveChecker {
    /// Determine if a type is Copy or Move
    ///
    /// Copy types: primitive integers, floats, bools, references
    /// Move types: String, structs, enums, arrays (for now)
    pub(super) fn is_move_type(&self, ty: &Type) -> bool {
        match ty {
            Type::Unknown => false, // Unknown type - assume non-Copy for safety

            // ⭐ FIXED: Primitive types are COPY (not move!)
            Type::I8
            | Type::I16
            | Type::I32
            | Type::I64
            | Type::I128
            | Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::U128
            | Type::F16
            | Type::F32
            | Type::F64
            | Type::Bool
            | Type::Byte => false, // ← Changed from true to false

            // References are Copy (copying a pointer)
            Type::Reference(_, _) => false,

            // String is Move
            Type::String => true,

            // Any is Move (heap allocated, dynamic type)
            Type::Any => true,

            // Builtin types are Move (Phase 0)
            Type::Option(_) => true,    // Option<T> is Move (contains T)
            Type::Result(_, _) => true, // Result<T,E> is Move
            Type::Vec(_) => true,       // Vec<T> is Move (owns heap data)
            Type::Box(_) => false,
            Type::Channel(_) => false,
            Type::Future(_) => false,   // Future<T> is Copy (pointer to runtime handle)
            Type::Named(_) => true, // Assume move for now, will be refined with Copy trait
            Type::Generic { .. } => true,

            // Arrays and slices are Move
            Type::Array(_, _) | Type::Slice(_, _) | Type::ConstArray { .. } => true,

            // Tuples are Move if any element is Move
            Type::Tuple(types) => types.iter().any(|t| self.is_move_type(t)),

            // Function types are Copy (function pointers)
            Type::Function { .. } => false,

            // Complex types are Move
            Type::Union(_) | Type::Intersection(_) | Type::Conditional { .. } => true,

            // Unit type is Copy
            Type::Unit | Type::Nil | Type::Error => false,

            Type::Infer(_) => false, // Infer is only for type checking

            // Never type is Copy (never instantiated)
            Type::Never => false,

            // Raw pointers are Copy (just addresses)
            Type::RawPtr { .. } => false,

            // Typeof is compile-time only, treated as Copy
            Type::Typeof(_) => false,

            // Self type and associated types - resolve at compile time
            Type::SelfType => true, // Conservative: treat as Move until resolved
            Type::AssociatedType { .. } => true, // Conservative: treat as Move until resolved
        }
    }
}
