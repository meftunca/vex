// src/codegen/analysis.rs
use super::*;
use std::collections::{HashMap, HashSet};

impl<'ctx> ASTCodeGen<'ctx> {
    pub(crate) fn check_circular_struct_dependencies(&self, program: &Program) -> Result<(), String> {
        let mut dependencies: HashMap<String, Vec<String>> = HashMap::new();

        for item in &program.items {
            if let Item::Struct(struct_def) = item {
                let mut deps = Vec::new();
                for field in &struct_def.fields {
                    if let Some(dep_name) = self.extract_struct_dependency(&field.ty) {
                        deps.push(dep_name);
                    }
                }
                dependencies.insert(struct_def.name.clone(), deps);
            }
        }

        for struct_name in dependencies.keys() {
            let mut visited = HashSet::new();
            let mut path = Vec::new();
            if self.has_cycle(&dependencies, struct_name, &mut visited, &mut path) {
                return Err(format!(
                    "Circular dependency detected in struct definitions: {}",
                    path.join(" -> ")
                ));
            }
        }
        Ok(())
    }

    fn extract_struct_dependency(&self, ty: &Type) -> Option<String> {
        match ty {
            Type::Named(name) => {
                if self.struct_ast_defs.contains_key(name) { Some(name.clone()) } else { None }
            }
            Type::Generic { name, .. } => {
                if self.struct_ast_defs.contains_key(name) { Some(name.clone()) } else { None }
            }
            Type::Array(inner, _) => self.extract_struct_dependency(inner),
            Type::Reference(inner, _) => self.extract_struct_dependency(inner),
            _ => None,
        }
    }

    fn has_cycle(
        &self,
        dependencies: &HashMap<String, Vec<String>>,
        current: &str,
        visited: &mut HashSet<String>,
        path: &mut Vec<String>,
    ) -> bool {
        if path.contains(&current.to_string()) {
            path.push(current.to_string());
            return true;
        }
        if visited.contains(current) {
            return false;
        }
        visited.insert(current.to_string());
        path.push(current.to_string());
        if let Some(deps) = dependencies.get(current) {
            for dep in deps {
                if self.has_cycle(dependencies, dep, visited, path) {
                    return true;
                }
            }
        }
        path.pop();
        false
    }
}
