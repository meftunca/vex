// Module resolution system for Vex compiler
// Loads and resolves imports from vex-libs/std/

use crate::resolver::{ResolveError, StdlibResolver};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use vex_ast::{Import, Program};
use vex_parser::Parser;

/// Module resolver - loads and caches parsed modules
pub struct ModuleResolver {
    /// Base path for standard library (vex-libs/std/)
    std_lib_path: PathBuf,

    /// Cached parsed modules (module_path -> Program)
    module_cache: HashMap<String, Program>,

    /// Stdlib resolver for platform-specific file selection
    stdlib_resolver: StdlibResolver,
}

impl ModuleResolver {
    pub fn new(std_lib_path: impl AsRef<Path>) -> Self {
        let path = std_lib_path.as_ref().to_path_buf();
        Self {
            std_lib_path: path.clone(),
            module_cache: HashMap::new(),
            stdlib_resolver: StdlibResolver::new(path),
        }
    }

    /// Resolve all imports in a program
    pub fn resolve_imports(&mut self, program: &mut Program) -> Result<(), String> {
        for import in &program.imports {
            self.load_module(&import.module)?;
        }
        Ok(())
    }

    /// Load a module from disk (with caching)
    pub fn load_module(&mut self, module_path: &str) -> Result<&Program, String> {
        // Check cache first
        if self.module_cache.contains_key(module_path) {
            return Ok(self.module_cache.get(module_path).unwrap());
        }

        // Use StdlibResolver if it's a stdlib module
        let file_path = if self.stdlib_resolver.is_stdlib_module(module_path) {
            self.stdlib_resolver
                .resolve_module(module_path)
                .map_err(|e| format!("Failed to resolve stdlib module {}: {}", module_path, e))?
        } else {
            // Fallback to old resolution for non-stdlib modules
            self.module_path_to_file_path(module_path)?
        };

        // Read source file
        let source = fs::read_to_string(&file_path)
            .map_err(|e| format!("Failed to read module {}: {}", module_path, e))?;

        // Parse module
        let mut parser = Parser::new(&source)
            .map_err(|e| format!("Failed to lex module {}: {}", module_path, e))?;
        let parsed = parser
            .parse_file()
            .map_err(|e| format!("Failed to parse module {}: {}", module_path, e))?;

        // Cache and return
        self.module_cache.insert(module_path.to_string(), parsed);
        Ok(self.module_cache.get(module_path).unwrap())
    }

    /// Convert module path to filesystem path
    fn module_path_to_file_path(&self, module_path: &str) -> Result<PathBuf, String> {
        // Handle different module path formats:
        // "std" -> vex-libs/std/mod.vx
        // "std::io" or "std/io" -> vex-libs/std/io/mod.vx
        // "std::http::client" or "std/http/client" -> vex-libs/std/http/client/mod.vx

        // Normalize path: convert both :: and / separators to a common format
        let normalized_path = module_path.replace("::", "/");

        // Start with base path (vex-libs/std)
        let mut file_path = self.std_lib_path.clone();

        // For "std" module specifically, just add mod.vx
        if normalized_path == "std" {
            file_path.push("mod.vx");
        } else {
            // For submodules like "std/io", split and add components
            let path_parts: Vec<&str> = normalized_path.split('/').collect();

            // Skip "std" prefix if present
            let parts_to_add: Vec<&str> = if !path_parts.is_empty() && path_parts[0] == "std" {
                path_parts[1..].to_vec()
            } else {
                path_parts
            };

            // Add path components
            for part in parts_to_add.iter() {
                file_path.push(part);
            }

            // Add mod.vx at the end
            file_path.push("mod.vx");
        }

        // Check if file exists
        if !file_path.exists() {
            return Err(format!("Module file not found: {:?}", file_path));
        }

        Ok(file_path)
    }

    /// Get all exported functions from a module
    pub fn get_module_exports(&self, module_path: &str) -> Result<Vec<String>, String> {
        let program = self
            .module_cache
            .get(module_path)
            .ok_or_else(|| format!("Module {} not loaded", module_path))?;

        let mut exports = Vec::new();

        for item in &program.items {
            if let vex_ast::Item::Function(func) = item {
                // In real implementation, check for 'export' keyword
                // For now, assume all functions are exported
                exports.push(func.name.clone());
            }
        }

        Ok(exports)
    }

    /// Check if a module is already loaded
    pub fn is_loaded(&self, module_path: &str) -> bool {
        self.module_cache.contains_key(module_path)
    }

    /// Get a loaded module
    pub fn get_module(&self, module_path: &str) -> Option<&Program> {
        self.module_cache.get(module_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_path_conversion() {
        let resolver = ModuleResolver::new("vex-libs/std");

        // Test basic module path
        let path = resolver.module_path_to_file_path("std").unwrap();
        assert!(path.to_string_lossy().contains("std/mod.vx"));

        // Test nested module path
        let path = resolver.module_path_to_file_path("std::io").unwrap();
        assert!(path.to_string_lossy().contains("std/io/mod.vx"));
    }
}
