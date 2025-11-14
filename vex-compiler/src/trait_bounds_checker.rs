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
        // Collect contract definitions
        for item in &program.items {
            if let vex_ast::Item::Contract(contract_def) = item {
                self.traits
                    .insert(contract_def.name.clone(), contract_def.clone());
            }
        }

        // Collect trait implementations from inline struct declarations
        for item in &program.items {
            if let vex_ast::Item::Struct(struct_def) = item {
                if !struct_def.impl_traits.is_empty() {
                    let trait_names: Vec<String> = struct_def.impl_traits.iter()
                        .map(|t| t.name.clone())
                        .collect();
                    self.type_impls
                        .insert(struct_def.name.clone(), trait_names);
                }
            }
        }

        // Collect trait implementations from external impl blocks (ExternalTraitImpl)
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
    /// Note: type_args may be shorter than type_params if defaults are used
    pub fn check_function_bounds(
        &mut self,
        func: &Function,
        type_args: &[Type],
    ) -> Result<(), String> {
        // Allow type_args to be shorter if remaining params have defaults
        if type_args.len() > func.type_params.len() {
            return Err(format!(
                "Too many type arguments for function '{}': expected at most {} type parameters, got {}",
                func.name,
                func.type_params.len(),
                type_args.len()
            ));
        }

        // Check each provided type parameter's bounds
        for (type_param, concrete_type) in func.type_params.iter().zip(type_args.iter()) {
            self.check_type_bounds(&type_param, concrete_type)?;
        }

        // Check that any unprovided params have defaults
        for (i, type_param) in func.type_params.iter().enumerate() {
            if i >= type_args.len() && type_param.default_type.is_none() {
                return Err(format!(
                    "Missing type argument for parameter '{}' in function '{}' (no default provided)",
                    type_param.name, func.name
                ));
            }
        }

        Ok(())
    }

    /// Check if a struct instantiation with generic type arguments satisfies trait bounds
    /// Example: Box<Point> where Box<T: Clone>
    /// Note: type_args may be shorter than type_params if defaults are used
    pub fn check_struct_bounds(
        &mut self,
        struct_def: &Struct,
        type_args: &[Type],
    ) -> Result<(), String> {
        // Allow type_args to be shorter if remaining params have defaults
        if type_args.len() > struct_def.type_params.len() {
            return Err(format!(
                "Too many type arguments for struct '{}': expected at most {} type parameters, got {}",
                struct_def.name,
                struct_def.type_params.len(),
                type_args.len()
            ));
        }

        // Check each provided type parameter's bounds
        for (type_param, concrete_type) in struct_def.type_params.iter().zip(type_args.iter()) {
            self.check_type_bounds(&type_param, concrete_type)?;
        }

        // Check that any unprovided params have defaults
        for (i, type_param) in struct_def.type_params.iter().enumerate() {
            if i >= type_args.len() && type_param.default_type.is_none() {
                return Err(format!(
                    "Missing type argument for parameter '{}' in struct '{}' (no default provided)",
                    type_param.name, struct_def.name
                ));
            }
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
            Type::F16 => "f16".to_string(),
            Type::F32 => "f32".to_string(),
            Type::F64 => "f64".to_string(),
            Type::Bool => "bool".to_string(),
            Type::String => "string".to_string(),
            Type::Any => "any".to_string(),
            Type::Byte => "byte".to_string(),
            Type::Array(_, _) => "Array".to_string(),
            Type::ConstArray { .. } => "ConstArray".to_string(),
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
            Type::SelfType => "Self".to_string(),
            Type::AssociatedType { name, .. } => name.clone(), // Return associated type name
        }
    }

    /// Validate const generic parameters
    /// Ensures const param types are valid compile-time integer types
    pub fn validate_const_params(&mut self, const_params: &[(String, Type)]) -> Result<(), String> {
        for (name, ty) in const_params {
            if !self.is_valid_const_type(ty) {
                return Err(format!(
                    "Const parameter '{}' has invalid type '{:?}'. Const parameters must be integer types (i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, usize, isize)",
                    name, ty
                ));
            }
        }
        Ok(())
    }

    /// Check if a type is valid for const generic parameters
    /// Only integer types are allowed (no floats, strings, etc.)
    fn is_valid_const_type(&self, ty: &Type) -> bool {
        match ty {
            Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::I128 |
            Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::U128 => true,
            Type::Named(name) if name == "usize" || name == "isize" => true,
            _ => false,
        }
    }

    /// Check where clause predicates for a function
    /// Validates that concrete type arguments satisfy where clause constraints
    /// Example: where T: Display, T.Item: Clone
    pub fn check_where_clause(
        &mut self,
        where_clause: &[vex_ast::WhereClausePredicate],
        type_substitutions: &HashMap<String, Type>,
    ) -> Result<(), String> {
        use vex_ast::WhereClausePredicate;

        for predicate in where_clause {
            match predicate {
                WhereClausePredicate::TypeBound { type_param, bounds } => {
                    // Get the concrete type for this type parameter
                    let concrete_type = type_substitutions
                        .get(type_param)
                        .ok_or_else(|| {
                            format!("Type parameter '{}' not found in substitutions", type_param)
                        })?;

                    let type_name = self.extract_type_name(concrete_type);

                    // Check each trait bound
                    for bound in bounds {
                        match bound {
                            TraitBound::Simple(trait_name) => {
                                if !self.type_implements_trait(&type_name, trait_name) {
                                    return Err(format!(
                                        "Where clause not satisfied: type `{}` does not implement trait `{}` (required for type parameter `{}`)",
                                        type_name, trait_name, type_param
                                    ));
                                }
                            }
                            TraitBound::Callable { trait_name, .. } => {
                                match concrete_type {
                                    Type::Function { .. } => {
                                        // Function types satisfy closure traits
                                    }
                                    _ => {
                                        return Err(format!(
                                            "Where clause not satisfied: type `{}` does not implement closure trait `{}` (required for type parameter `{}`)",
                                            type_name, trait_name, type_param
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
                WhereClausePredicate::AssociatedTypeBound {
                    type_param,
                    assoc_type,
                    bounds,
                } => {
                    // For associated type bounds like T.Item: Display
                    // We need to:
                    // 1. Get the concrete type for T
                    // 2. Find what T.Item resolves to
                    // 3. Check if that type implements the required traits

                    let concrete_type = type_substitutions
                        .get(type_param)
                        .ok_or_else(|| {
                            format!("Type parameter '{}' not found in substitutions", type_param)
                        })?;

                    // For now, we'll do basic validation
                    // Full implementation would require resolving associated types
                    let type_name = self.extract_type_name(concrete_type);

                    // TODO: Resolve associated type T.Item to concrete type
                    // For now, we'll just validate that T implements the trait that defines Item
                    // This is incomplete but better than nothing
                    eprintln!(
                        "⚠️  Associated type bound validation incomplete: {}.{}: {:?}",
                        type_name, assoc_type, bounds
                    );

                    // At minimum, ensure the type is known
                    if type_name == "Unknown" || type_name == "error" {
                        return Err(format!(
                            "Cannot validate associated type bound {}.{} for unknown type",
                            type_param, assoc_type
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    /// Build type substitution map for generic function call
    /// Maps type parameter names to concrete types
    pub fn build_type_substitutions(
        type_params: &[TypeParam],
        type_args: &[Type],
    ) -> HashMap<String, Type> {
        let mut substitutions = HashMap::new();

        for (param, arg) in type_params.iter().zip(type_args.iter()) {
            substitutions.insert(param.name.clone(), arg.clone());
        }

        // Fill in defaults for missing arguments
        for (i, param) in type_params.iter().enumerate() {
            if i >= type_args.len() {
                if let Some(default_ty) = &param.default_type {
                    substitutions.insert(param.name.clone(), default_ty.clone());
                }
            }
        }

        substitutions
    }
}
