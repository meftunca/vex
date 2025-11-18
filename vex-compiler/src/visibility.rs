// Visibility and Contract Enforcement System
// Ensures exported methods are backed by contracts

use std::collections::{HashMap, HashSet};
use vex_ast::{Function, Item, Program, Struct, Trait};

pub struct VisibilityChecker {
    /// Maps contract name -> set of method names
    contract_methods: HashMap<String, HashSet<String>>,
    /// Maps struct name -> list of implemented contracts
    struct_to_contracts: HashMap<String, Vec<String>>,
}

impl VisibilityChecker {
    pub fn new() -> Self {
        Self {
            contract_methods: HashMap::new(),
            struct_to_contracts: HashMap::new(),
        }
    }

    /// Build contract method registry from program AST
    pub fn build_registry(&mut self, program: &Program) {
        // Register all contracts
        for item in &program.items {
            if let Item::Contract(trait_def) = item {
                self.register_contract(trait_def);
            }
        }
        
        // Map structs to their contracts
        for item in &program.items {
            if let Item::Struct(struct_def) = item {
                let contracts: Vec<String> = struct_def
                    .impl_traits
                    .iter()
                    .map(|t| t.name.clone())
                    .collect();
                self.struct_to_contracts
                    .insert(struct_def.name.clone(), contracts);
            }
        }
    }

    /// Register all methods from a contract
    fn register_contract(&mut self, trait_def: &Trait) {
        let mut methods = HashSet::new();
        for method in &trait_def.methods {
            methods.insert(method.name.clone());
        }
        self.contract_methods.insert(trait_def.name.clone(), methods);
    }

    /// Check if a method is declared in any contract
    fn is_method_in_contracts(&self, method_name: &str, contracts: &[String]) -> bool {
        for contract_name in contracts {
            if let Some(methods) = self.contract_methods.get(contract_name) {
                if methods.contains(method_name) {
                    return true;
                }
            }
        }
        false
    }

    /// Check if a method in a struct is properly backed by a contract
    pub fn check_method_contract(
        &self,
        struct_name: &str,
        method: &Function,
    ) -> Result<(), String> {
        // Private methods (start with _) don't need contracts
        if method.name.starts_with('_') {
            return Ok(());
        }
        
        // Constructor (op) is allowed without explicit contract declaration
        if method.name == "op" {
            return Ok(());
        }

        // Get contracts for this struct
        let contracts = self
            .struct_to_contracts
            .get(struct_name)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);

        // If method has receiver (is a method), check if it's in a contract
        if method.receiver.is_some() || !contracts.is_empty() {
            if self.is_method_in_contracts(&method.name, contracts) {
                return Ok(());
            }

            // Method not found in any contract
            return Err(format!(
                "Public method '{}' in struct '{}' must be declared in a contract.\n\
                 \n\
                 Help: Consider implementing a standard contract from stdlib/contracts:\n\
                 \n\
                 import {{ Collection, Stack }} from \"std/contracts\";\n\
                 \n\
                 struct {} impl Stack<T> {{\n\
                     fn {}(...) {{ ... }}\n\
                 }}\n\
                 \n\
                 Or define a custom contract:\n\
                 \n\
                 contract {}Ops {{\n\
                     fn {}(...);\n\
                 }}\n\
                 \n\
                 struct {} impl {}Ops {{\n\
                     fn {}(...) {{ ... }}\n\
                 }}",
                method.name,
                struct_name,
                struct_name,
                method.name,
                struct_name,
                method.name,
                struct_name,
                struct_name,
                method.name
            ));
        }

        Ok(())
    }

    /// Check all methods in a struct
    pub fn check_struct(&self, struct_def: &Struct) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        for method in &struct_def.methods {
            if let Err(e) = self.check_method_contract(&struct_def.name, method) {
                errors.push(e);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// Check external methods (Go-style: fn (receiver) method_name)
    pub fn check_external_methods(&self, program: &Program) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        for item in &program.items {
            if let Item::Function(func) = item {
                // External method has receiver
                if let Some(receiver) = &func.receiver {
                    // Extract struct name from receiver type
                    let struct_name = self.extract_struct_name(&receiver.ty);
                    
                    if let Some(name) = struct_name {
                        if let Err(e) = self.check_method_contract(&name, func) {
                            errors.push(e);
                        }
                    }
                }
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// Extract struct name from type (simplified)
    fn extract_struct_name(&self, ty: &vex_ast::Type) -> Option<String> {
        match ty {
            vex_ast::Type::Named(name) => Some(name.clone()),
            vex_ast::Type::Reference(inner, _) => self.extract_struct_name(inner),
            vex_ast::Type::Generic { name, .. } => Some(name.clone()),
            _ => None,
        }
    }

    /// Check entire program for contract violations
    pub fn check_program(&mut self, program: &Program) -> Result<(), Vec<String>> {
        // First build registry of all contracts
        self.build_registry(program);

        let mut all_errors = Vec::new();

        // Check all structs (inline methods)
        for item in &program.items {
            if let Item::Struct(struct_def) = item {
                if let Err(errors) = self.check_struct(struct_def) {
                    all_errors.extend(errors);
                }
            }
        }
        
        // Check external methods (Go-style)
        if let Err(errors) = self.check_external_methods(program) {
            all_errors.extend(errors);
        }

        if all_errors.is_empty() {
            Ok(())
        } else {
            Err(all_errors)
        }
    }
}
