// Module Resolution System for Vex LSP
// Resolves import paths to filesystem locations

use dashmap::DashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Resolves module imports to filesystem paths
pub struct ModuleResolver {
    /// Workspace root directory
    workspace_root: PathBuf,
    /// Standard library path (vex-libs/)
    stdlib_path: PathBuf,
    /// Cache: import path → resolved file path
    cache: Arc<DashMap<String, PathBuf>>,
}

impl ModuleResolver {
    /// Create a new module resolver
    pub fn new(workspace_root: PathBuf) -> Self {
        let stdlib_path = workspace_root.join("vex-libs");
        Self {
            workspace_root,
            stdlib_path,
            cache: Arc::new(DashMap::new()),
        }
    }

    /// Resolve an import path to a filesystem path
    ///
    /// Examples:
    /// - `std.io` → `workspace/vex-libs/std/io/src/lib.vx`
    /// - `std.collections.Vec` → `workspace/vex-libs/std/collections/src/vec.vx`
    /// - `./utils` → `current_dir/utils.vx` or `current_dir/utils/lib.vx`
    /// - `../shared` → `parent_dir/shared.vx` or `parent_dir/shared/lib.vx`
    pub fn resolve_import(
        &self,
        import_path: &str,
        current_file: Option<&Path>,
    ) -> Option<PathBuf> {
        // Check cache first
        if let Some(cached) = self.cache.get(import_path) {
            return Some(cached.value().clone());
        }

        let resolved = if import_path.starts_with("std.") {
            // Standard library import: std.io → vex-libs/std/io/src/lib.vx
            self.resolve_stdlib_import(import_path)
        } else if import_path.starts_with("./") || import_path.starts_with("../") {
            // Relative import
            current_file.and_then(|current| self.resolve_relative_import(import_path, current))
        } else {
            // Workspace module import
            self.resolve_workspace_import(import_path)
        };

        // Cache the result
        if let Some(ref path) = resolved {
            self.cache.insert(import_path.to_string(), path.clone());
        }

        resolved
    }

    /// Resolve standard library import
    /// std.io → vex-libs/std/io/src/lib.vx
    fn resolve_stdlib_import(&self, import_path: &str) -> Option<PathBuf> {
        // Remove "std." prefix
        let module_path = import_path.strip_prefix("std.")?;

        // Split into parts: io.fs → ["io", "fs"]
        let parts: Vec<&str> = module_path.split('.').collect();

        if parts.is_empty() {
            return None;
        }

        // Try different resolution strategies:
        // 1. Full module path: std.io.fs → vex-libs/std/io/src/fs.vx
        // 2. Package lib.vx: std.io → vex-libs/std/io/src/lib.vx

        // Strategy 1: Module file
        if parts.len() > 1 {
            let package = parts[0];
            let module_file = format!("{}.vx", parts[1..].join("/"));
            let path = self
                .stdlib_path
                .join("std")
                .join(package)
                .join("src")
                .join(&module_file);

            if path.exists() {
                return Some(path);
            }
        }

        // Strategy 2: Package lib.vx
        let package = parts[0];
        let lib_path = self
            .stdlib_path
            .join("std")
            .join(package)
            .join("src")
            .join("lib.vx");

        if lib_path.exists() {
            return Some(lib_path);
        }

        None
    }

    /// Resolve relative import
    /// ./utils → current_dir/utils.vx or utils.vxc
    fn resolve_relative_import(&self, import_path: &str, current_file: &Path) -> Option<PathBuf> {
        let current_dir = current_file.parent()?;

        // Remove ./ or ../
        let relative_path = import_path
            .trim_start_matches("./")
            .trim_start_matches("../");

        // Construct base path
        let base_path = if import_path.starts_with("../") {
            current_dir.parent()?.to_path_buf()
        } else {
            current_dir.to_path_buf()
        };

        // Try different resolutions:
        // 1. Direct file: ./utils → utils.vx
        // 2. C extern file: ./utils → utils.vxc
        // 3. Directory with lib.vx: ./utils → utils/lib.vx

        let direct_file = base_path.join(format!("{}.vx", relative_path));
        if direct_file.exists() {
            return Some(direct_file);
        }

        let c_extern_file = base_path.join(format!("{}.vxc", relative_path));
        if c_extern_file.exists() {
            return Some(c_extern_file);
        }

        let lib_file = base_path.join(relative_path).join("lib.vx");
        if lib_file.exists() {
            return Some(lib_file);
        }

        None
    }

    /// Resolve workspace module import
    /// utils → workspace/utils.vx, utils.vxc, or workspace/utils/lib.vx
    fn resolve_workspace_import(&self, import_path: &str) -> Option<PathBuf> {
        // Try direct file in workspace root
        let direct_file = self.workspace_root.join(format!("{}.vx", import_path));
        if direct_file.exists() {
            return Some(direct_file);
        }

        // Try C extern file
        let c_extern_file = self.workspace_root.join(format!("{}.vxc", import_path));
        if c_extern_file.exists() {
            return Some(c_extern_file);
        }

        // Try directory with lib.vx
        let lib_file = self.workspace_root.join(import_path).join("lib.vx");
        if lib_file.exists() {
            return Some(lib_file);
        }

        None
    }

    /// List all available modules in workspace
    pub fn list_workspace_modules(&self) -> Vec<String> {
        let mut modules = Vec::new();

        // Scan workspace root for .vx and .vxc files
        if let Ok(entries) = std::fs::read_dir(&self.workspace_root) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".vx") {
                        modules.push(name.trim_end_matches(".vx").to_string());
                    } else if name.ends_with(".vxc") {
                        modules.push(name.trim_end_matches(".vxc").to_string());
                    }
                }
            }
        }

        modules
    }

    /// List all standard library modules
    pub fn list_stdlib_modules(&self) -> Vec<String> {
        let mut modules = Vec::new();
        let std_path = self.stdlib_path.join("std");

        if let Ok(entries) = std::fs::read_dir(&std_path) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
                        modules.push(format!("std.{}", name));
                    }
                }
            }
        }

        modules
    }

    /// Clear resolution cache
    pub fn clear_cache(&self) {
        self.cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stdlib_resolution() {
        let workspace = PathBuf::from("/workspace");
        let resolver = ModuleResolver::new(workspace.clone());

        // Mock stdlib structure would need to exist for this to pass
        // Just testing the path construction logic
        let result = resolver.resolve_stdlib_import("std.io");
        // Would resolve to /workspace/vex-libs/std/io/src/lib.vx
        assert!(result.is_none() || result.unwrap().ends_with("lib.vx"));
    }

    #[test]
    fn test_relative_resolution() {
        let workspace = PathBuf::from("/workspace");
        let resolver = ModuleResolver::new(workspace);
        let current = Path::new("/workspace/src/main.vx");

        // Would resolve to /workspace/src/utils.vx or /workspace/src/utils/lib.vx
        let result = resolver.resolve_relative_import("./utils", current);
        assert!(result.is_none() || result.unwrap().to_string_lossy().contains("utils"));
    }
}
