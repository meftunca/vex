// Trait bounds checker for generic functions
// Verifies that type arguments satisfy trait bounds at compile time

use std::collections::HashMap;
use vex_ast::{Function, Program, Struct, Trait, TraitBound, Type, TypeParam};
use vex_diagnostics::DiagnosticEngine;

pub struct TraitBoundsChecker {
    // Maps struct/type names to their trait implementations
    // Example: "Point" -> ["Display", "Clone"]
    type_impls: HashMap<String, Vec<String>>,

    // Maps trait names to their definitions
    traits: HashMap<String, Trait>,

    // Diagnostic engine for error reporting
    diagnostics: DiagnosticEngine,
}

impl TraitBoundsChecker {
    pub fn new() -> Self {
        Self {
            type_impls: HashMap::new(),
            traits: HashMap::new(),
            diagnostics: DiagnosticEngine::new(),
        }
    }

    pub fn diagnostics(&self) -> &DiagnosticEngine {
        &self.diagnostics
    }

    pub fn diagnostics_mut(&mut self) -> &mut DiagnosticEngine {
        &mut self.diagnostics
    }

    /// Initialize checker with program information
    pub fn initialize(&mut self, program: &Program) {
        // Collect trait definitions
        for item in &program.items {
            if let vex_ast::Item::Trait(trait_def) = item {
                self.traits
                    .insert(trait_def.name.clone(), trait_def.clone());
            }
        }

        // Collect trait implementations from inline struct declarations
        for item in &program.items {
            if let vex_ast::Item::Struct(struct_def) = item {
                if !struct_def.impl_traits.is_empty() {
                    self.type_impls
                        .insert(struct_def.name.clone(), struct_def.impl_traits.clone());
                }
            }
        }

        // Collect trait implementations from impl blocks
        for item in &program.items {
            if let vex_ast::Item::TraitImpl(impl_block) = item {
                let type_name = self.extract_type_name(&impl_block.for_type);
                self.type_impls
                    .entry(type_name)
                    .or_insert_with(Vec::new)
                    .push(impl_block.trait_name.clone());
            }
        }
    }

    /// Check if a function call with generic type arguments satisfies trait bounds
    /// Example: print_value<Point>(...) where print_value<T: Display>
    pub fn check_function_bounds(
        &mut self,
        func: &Function,
        type_args: &[Type],
    ) -> Result<(), String> {
        if func.type_params.len() != type_args.len() {
            return Err(format!(
                "Generic argument count mismatch for function '{}': expected {} type parameters, got {}",
                func.name,
                func.type_params.len(),
                type_args.len()
            ));
        }

        // Check each type parameter's bounds
        for (type_param, concrete_type) in func.type_params.iter().zip(type_args.iter()) {
            self.check_type_bounds(&type_param, concrete_type)?;
        }

        Ok(())
    }

    /// Check if a struct instantiation with generic type arguments satisfies trait bounds
    /// Example: Box<Point> where Box<T: Clone>
    pub fn check_struct_bounds(
        &mut self,
        struct_def: &Struct,
        type_args: &[Type],
    ) -> Result<(), String> {
        if struct_def.type_params.len() != type_args.len() {
            return Err(format!(
                "Generic argument count mismatch for struct '{}': expected {} type parameters, got {}",
                struct_def.name,
                struct_def.type_params.len(),
                type_args.len()
            ));
        }

        // Check each type parameter's bounds
        for (type_param, concrete_type) in struct_def.type_params.iter().zip(type_args.iter()) {
            self.check_type_bounds(&type_param, concrete_type)?;
        }

        Ok(())
    }

    /// Check if a concrete type satisfies the trait bounds of a type parameter
    fn check_type_bounds(
        &mut self,
        type_param: &TypeParam,
        concrete_type: &Type,
    ) -> Result<(), String> {
        // If no bounds, any type is valid
        if type_param.bounds.is_empty() {
            return Ok(());
        }

        let type_name = self.extract_type_name(concrete_type);

        // Check each required trait bound
        for required_trait in &type_param.bounds {
            match required_trait {
                TraitBound::Simple(trait_name) => {
                    if !self.type_implements_trait(&type_name, trait_name) {
                        return Err(format!(
                            "Trait bound not satisfied: type `{}` does not implement trait `{}` (required by type parameter `{}`)",
                            type_name, trait_name, type_param.name
                        ));
                    }
                }
                TraitBound::Callable { trait_name, .. } => {
                    // For closure traits, check if type is a function/closure type
                    // For now, we accept function types as satisfying closure traits
                    match concrete_type {
                        Type::Function { .. } => {
                            // Function types satisfy all closure traits
                            // TODO: More precise checking based on capture mode
                        }
                        _ => {
                            return Err(format!(
                                "Trait bound not satisfied: type `{}` does not implement closure trait `{}` (required by type parameter `{}`)",
                                type_name, trait_name, type_param.name
                            ));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Check if a type implements a specific trait
    fn type_implements_trait(&self, type_name: &str, trait_name: &str) -> bool {
        if let Some(impls) = self.type_impls.get(type_name) {
            impls.contains(&trait_name.to_string())
        } else {
            false
        }
    }

    /// Extract type name from Type enum for lookup
    fn extract_type_name(&self, ty: &Type) -> String {
        match ty {
            Type::Named(name) => name.clone(),
            Type::Generic { name, type_args: _ } => name.clone(),
            Type::I8 => "i8".to_string(),
            Type::I16 => "i16".to_string(),
            Type::I32 => "i32".to_string(),
            Type::I64 => "i64".to_string(),
            Type::I128 => "i128".to_string(),
            Type::U8 => "u8".to_string(),
            Type::U16 => "u16".to_string(),
            Type::U32 => "u32".to_string(),
            Type::U64 => "u64".to_string(),
            Type::U128 => "u128".to_string(),
            Type::F32 => "f32".to_string(),
            Type::F64 => "f64".to_string(),
            Type::F128 => "f128".to_string(),
            Type::Bool => "bool".to_string(),
            Type::String => "string".to_string(),
            Type::Byte => "byte".to_string(),
            Type::Array(_, _) => "Array".to_string(),
            Type::Slice(_, _) => "Slice".to_string(),
            Type::Tuple(_) => "Tuple".to_string(),
            Type::Function { .. } => "Function".to_string(),
            Type::Reference(inner, _) => self.extract_type_name(inner),
            // Builtin types (Phase 0)
            Type::Option(_) => "Option".to_string(),
            Type::Result(_, _) => "Result".to_string(),
            Type::Vec(_) => "Vec".to_string(),
            Type::Box(_) => "Box".to_string(),
            Type::Union(_) => "Union".to_string(),
            Type::Intersection(_) => "Intersection".to_string(),
            Type::Conditional { .. } => "Conditional".to_string(),
            Type::Infer(_) => "infer".to_string(),
            Type::Error => "error".to_string(),
            Type::Nil => "nil".to_string(),
            Type::Unit => "unit".to_string(),
            Type::Never => "!".to_string(),
            Type::RawPtr { inner, is_const } => {
                if *is_const {
                    format!("*const {}", self.extract_type_name(inner))
                } else {
                    format!("*{}", self.extract_type_name(inner))
                }
            }
            Type::Channel(inner) => format!("Channel<{}>", self.extract_type_name(inner)),
            Type::Typeof(_) => "typeof".to_string(), // Compile-time evaluated
        }
    }
}
