// Automatic destructor compilation for RAII
// Implements Drop trait and scope-based cleanup

use super::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use vex_ast::Type;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Register a variable for automatic Drop call at scope exit
    /// Called from register_variable() when variable implements Drop
    pub(crate) fn register_drop_variable(&mut self, name: &str, value_type: &Type) {
        if !self.type_has_drop(value_type) {
            return;
        }

        // Get mangled type name for generic types
        let type_name = match value_type {
            Type::Named(n) => n.clone(),
            Type::Generic { name, type_args } => {
                let suffix = type_args
                    .iter()
                    .map(|t| self.type_to_string(t))
                    .collect::<Vec<_>>()
                    .join("_");
                format!("{}_{}", name, suffix)
            }
            Type::Vec(inner) => format!("Vec_{}", self.type_to_string(inner)),
            Type::Box(inner) => format!("Box_{}", self.type_to_string(inner)),
            _ => return,
        };

        // Add to current scope
        if let Some(scope) = self.scope_stack.last_mut() {
            scope.push((name.to_string(), type_name));
            eprintln!("📌 Registered '{}' for automatic drop at scope exit", name);
        }
    }

    /// Call drop on a value when it goes out of scope
    /// Returns true if drop was called, false if type has no drop
    pub(crate) fn call_drop_if_needed(
        &mut self,
        value: BasicValueEnum<'ctx>,
        value_type: &Type,
        var_name: &str,
    ) -> Result<bool, String> {
        // Check if type implements Drop trait
        if !self.type_has_drop(value_type) {
            return Ok(false);
        }

        eprintln!(
            "🧹 Auto-calling drop for variable: {} (type: {:?})",
            var_name, value_type
        );

        // Call the drop method
        match value_type {
            Type::Vec(inner) => {
                self.call_vec_drop(value, inner)?;
            }
            Type::Box(inner) => {
                self.call_box_drop(value, inner)?;
            }
            Type::Generic { name, type_args } if name == "Vec" && !type_args.is_empty() => {
                self.call_vec_drop(value, &type_args[0])?;
            }
            Type::Generic { name, type_args } if name == "Box" && !type_args.is_empty() => {
                self.call_box_drop(value, &type_args[0])?;
            }
            Type::Named(name) => {
                self.call_struct_drop(value, name)?;
            }
            _ => {
                // Type has drop trait but no implementation yet
                eprintln!("⚠️  Type {:?} has Drop but no implementation", value_type);
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Check if a type implements Drop trait
    fn type_has_drop(&self, ty: &Type) -> bool {
        match ty {
            Type::Vec(_) => true,
            Type::Box(_) => true,
            Type::String => true,
            Type::Generic { name, .. } => matches!(name.as_str(), "Vec" | "Box"),
            Type::Named(name) => {
                // Check if struct has drop method
                let drop_method = format!("{}_drop", name);
                self.functions.contains_key(&drop_method)
            }
            _ => false,
        }
    }

    /// Call Vec drop
    fn call_vec_drop(&mut self, value: BasicValueEnum<'ctx>, _inner: &Type) -> Result<(), String> {
        // Vec drop signature: fn drop(self: &Vec<T>)
        let drop_fn = self.get_or_declare_vec_drop()?;

        self.builder
            .build_call(drop_fn, &[value.into()], "vec_drop")
            .map_err(|e| format!("Failed to call Vec drop: {}", e))?;

        Ok(())
    }

    /// Call Box drop
    fn call_box_drop(&mut self, value: BasicValueEnum<'ctx>, _inner: &Type) -> Result<(), String> {
        // Box drop signature: fn drop(self: &Box<T>)
        let drop_fn = self.get_or_declare_box_drop()?;

        self.builder
            .build_call(drop_fn, &[value.into()], "box_drop")
            .map_err(|e| format!("Failed to call Box drop: {}", e))?;

        Ok(())
    }

    /// Call struct drop method
    fn call_struct_drop(
        &mut self,
        value: BasicValueEnum<'ctx>,
        struct_name: &str,
    ) -> Result<(), String> {
        let drop_method = format!("{}_drop", struct_name);

        if let Some(drop_fn) = self.functions.get(&drop_method) {
            self.builder
                .build_call(*drop_fn, &[value.into()], "struct_drop")
                .map_err(|e| format!("Failed to call {} drop: {}", struct_name, e))?;
        }

        Ok(())
    }

    /// Get or declare Vec drop function
    fn get_or_declare_vec_drop(&mut self) -> Result<inkwell::values::FunctionValue<'ctx>, String> {
        if let Some(func) = self.functions.get("Vec_drop") {
            return Ok(*func);
        }

        // Declare generic Vec drop - will be instantiated on demand
        let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
        let fn_type = self.context.void_type().fn_type(&[ptr_type.into()], false);
        let func = self.module.add_function("Vec_drop", fn_type, None);

        self.functions.insert("Vec_drop".to_string(), func);
        Ok(func)
    }

    /// Get or declare Box drop function
    fn get_or_declare_box_drop(&mut self) -> Result<inkwell::values::FunctionValue<'ctx>, String> {
        if let Some(func) = self.functions.get("Box_drop") {
            return Ok(*func);
        }

        // Declare generic Box drop
        let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
        let fn_type = self.context.void_type().fn_type(&[ptr_type.into()], false);
        let func = self.module.add_function("Box_drop", fn_type, None);

        self.functions.insert("Box_drop".to_string(), func);
        Ok(func)
    }
}
