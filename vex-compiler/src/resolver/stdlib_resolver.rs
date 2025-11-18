/**
 * Standard Library Resolver
 * Resolves import statements to stdlib file paths with platform-specific selection
 */
use super::platform::Target;
use std::path::{Path, PathBuf};

/// Standard library modules (built-in)
const STDLIB_MODULES: &[&str] = &[
    // "io",
    "core",
    "collections",
    // "string",
    // "memory",
    // "sync",
    // "time",
    // "net",
    // "encoding",
    // "crypto",
    // "db",
    // "strconv",
    // "path",
    // "http",
    // "json",
    // "fmt",
    "testing",
    "math", // Mathematical functions
            // "fs",      // File system operations
            // "env",     // Environment variables
            // "process", // Process management
];

/// Errors that can occur during module resolution
#[derive(Debug, Clone)]
pub enum ResolveError {
    ModuleNotFound(String),
    InvalidPath(String),
    AmbiguousModule(String),
}

impl std::fmt::Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolveError::ModuleNotFound(name) => {
                write!(f, "Module '{}' not found", name)
            }
            ResolveError::InvalidPath(path) => {
                write!(f, "Invalid module path: {}", path)
            }
            ResolveError::AmbiguousModule(name) => {
                write!(f, "Ambiguous module reference: {}", name)
            }
        }
    }
}

impl std::error::Error for ResolveError {}

/// Standard library resolver
pub struct StdlibResolver {
    stdlib_root: PathBuf,
    target: Target,
}

impl StdlibResolver {
    /// Create a new stdlib resolver
    ///
    /// # Arguments
    /// * `stdlib_root` - Root directory of the standard library (e.g., "vex-libs/std")
    pub fn new<P: AsRef<Path>>(stdlib_root: P) -> Self {
        Self {
            stdlib_root: stdlib_root.as_ref().to_path_buf(),
            target: Target::current(),
        }
    }

    /// Create a resolver with a custom target (for cross-compilation)
    pub fn with_target<P: AsRef<Path>>(stdlib_root: P, target: Target) -> Self {
        Self {
            stdlib_root: stdlib_root.as_ref().to_path_buf(),
            target,
        }
    }

    /// Check if a module name is a stdlib module
    ///
    /// # Example
    /// ```
    /// # use vex_compiler::resolver::stdlib_resolver::StdlibResolver;
    /// let resolver = StdlibResolver::new("vex-libs/std");
    /// assert!(resolver.is_stdlib_module("io"));
    /// assert!(resolver.is_stdlib_module("core/vec")); // Submodule
    /// assert!(!resolver.is_stdlib_module("my_custom_lib"));
    /// ```
    pub fn is_stdlib_module(&self, module_name: &str) -> bool {
        // Check if module_name or its root is in STDLIB_MODULES
        // e.g., "core" → true, "core/vec" → true (root is "core")
        let root_module = module_name.split('/').next().unwrap_or(module_name);
        STDLIB_MODULES.contains(&root_module)
    }

    /// Get all stdlib module names
    pub fn all_modules() -> &'static [&'static str] {
        STDLIB_MODULES
    }

    /// Resolve a module name to a file path
    ///
    /// Supports both top-level modules and submodules:
    /// - "io" → stdlib/io/src/lib.vx
    /// - "core/vec" → stdlib/core/src/vec.vx
    ///
    /// Priority chain (first match wins):
    /// 1. `lib.{os}.{arch}.vx` - Platform + architecture specific
    /// 2. `lib.{arch}.vx` - Architecture specific
    /// 3. `lib.{os}.vx` - Platform specific
    /// 4. `lib.vx` - Generic fallback
    ///
    /// For submodules (e.g., "core/vec"):
    /// 1. `{submodule}.vx` - Direct file
    /// 2. `{submodule}/lib.vx` - Submodule directory
    ///
    /// # Example
    /// On Linux x64:
    /// - First tries: `io/src/lib.linux.x64.vx`
    /// - Then tries: `io/src/lib.x64.vx`
    /// - Then tries: `io/src/lib.linux.vx`
    /// - Finally: `io/src/lib.vx`
    ///
    /// # Errors
    /// Returns `ResolveError::ModuleNotFound` if no suitable file exists
    pub fn resolve_module(&self, module_name: &str) -> Result<PathBuf, ResolveError> {
        if !self.is_stdlib_module(module_name) {
            return Err(ResolveError::ModuleNotFound(module_name.to_string()));
        }

        // Split module path: "core/vec" → ("core", Some("vec"))
        let parts: Vec<&str> = module_name.split('/').collect();
        let root_module = parts[0];
        let submodule_path = if parts.len() > 1 {
            Some(parts[1..].join("/"))
        } else {
            None
        };

        let module_dir = self.stdlib_root.join(root_module);
        if !module_dir.exists() {
            return Err(ResolveError::ModuleNotFound(module_name.to_string()));
        }

        // If submodule path explicitly includes .vx or .vxc extension, treat as direct file path
        if let Some(ref subpath) = submodule_path {
            if subpath.ends_with(".vx") || subpath.ends_with(".vxc") {
                let direct_file = module_dir.join(subpath);
                if direct_file.exists() {
                    return Ok(direct_file);
                }
                return Err(ResolveError::ModuleNotFound(format!(
                    "File '{}' not found in module '{}'",
                    subpath, root_module
                )));
            }
        }

        let src_dir = module_dir.join("src");
        if !src_dir.exists() {
            return Err(ResolveError::InvalidPath(format!(
                "Module '{}' has no src/ directory",
                root_module
            )));
        }

        // If we have a submodule path, try to find it directly
        if let Some(subpath) = submodule_path {
            // Try: core/src/vec.vx
            let direct_file = src_dir.join(format!("{}.vx", subpath));
            if direct_file.exists() {
                return Ok(direct_file);
            }

            // Try: core/src/vec/lib.vx
            let submodule_lib = src_dir.join(&subpath).join("lib.vx");
            if submodule_lib.exists() {
                return Ok(submodule_lib);
            }

            // Try: core/src/vec/lib.vxc (C-interop)
            let submodule_lib_vxc = src_dir.join(&subpath).join("lib.vxc");
            if submodule_lib_vxc.exists() {
                return Ok(submodule_lib_vxc);
            }

            return Err(ResolveError::ModuleNotFound(format!(
                "Submodule '{}' not found in '{}'",
                subpath, root_module
            )));
        }

        // Priority chain: platform.arch > arch > platform > generic
        // Check both .vx and .vxc extensions
        let candidates = vec![
            format!(
                "lib.{}.{}.vx",
                self.target.platform.as_str(),
                self.target.arch.as_str()
            ),
            format!(
                "lib.{}.{}.vxc",
                self.target.platform.as_str(),
                self.target.arch.as_str()
            ),
            format!("lib.{}.vx", self.target.arch.as_str()),
            format!("lib.{}.vxc", self.target.arch.as_str()),
            format!("lib.{}.vx", self.target.platform.as_str()),
            format!("lib.{}.vxc", self.target.platform.as_str()),
            "lib.vx".to_string(),
            "lib.vxc".to_string(),
        ];

        for candidate in candidates {
            let candidate_path = src_dir.join(&candidate);
            if candidate_path.exists() {
                return Ok(candidate_path);
            }
        }

        Err(ResolveError::ModuleNotFound(format!(
            "No suitable file found for module '{}'",
            module_name
        )))
    }

    /// Resolve multiple modules at once
    /// Returns a map of module names to file paths
    pub fn resolve_modules(
        &self,
        module_names: &[&str],
    ) -> Result<Vec<(String, PathBuf)>, ResolveError> {
        let mut results = Vec::new();
        for &name in module_names {
            let path = self.resolve_module(name)?;
            results.push((name.to_string(), path));
        }
        Ok(results)
    }

    /// Get the current target
    pub fn target(&self) -> Target {
        self.target
    }

    /// Get the stdlib root directory
    pub fn stdlib_root(&self) -> &Path {
        &self.stdlib_root
    }
}

#[cfg(test)]
mod tests {
    use crate::{Arch, Platform, Target};

    use super::*;

    #[test]
    fn test_is_stdlib_module() {
        let resolver = StdlibResolver::new("vex-libs/std");
        assert!(resolver.is_stdlib_module("io"));
        assert!(resolver.is_stdlib_module("collections"));
        assert!(resolver.is_stdlib_module("string"));
        assert!(!resolver.is_stdlib_module("my_custom_lib"));
        assert!(!resolver.is_stdlib_module("nonexistent"));
    }

    #[test]
    fn test_all_modules() {
        let modules = StdlibResolver::all_modules();
        // Updated: stdlib now has 21 modules (was 17)
        assert_eq!(modules.len(), 21);
        assert!(modules.contains(&"io"));
        assert!(modules.contains(&"collections"));
        assert!(modules.contains(&"testing"));
    }

    #[test]
    fn test_resolve_generic() {
        // Use absolute path from project root
        let project_root = std::env::current_dir().expect("Failed to get current directory");
        let stdlib_path = project_root.join("vex-libs/std");

        if !stdlib_path.exists() {
            // Skip test if not in project root
            return;
        }

        let resolver = StdlibResolver::new(&stdlib_path);

        // Test io module (has platform-specific files)
        let io_path = resolver
            .resolve_module("io")
            .expect("Failed to resolve io module");
        assert!(io_path.exists());
        assert!(io_path
            .to_str()
            .expect("Invalid UTF-8 in path")
            .contains("io/src/lib"));
        assert!(io_path
            .to_str()
            .expect("Invalid UTF-8 in path")
            .ends_with(".vx"));
    }

    #[test]
    fn test_resolve_nonexistent() {
        let resolver = StdlibResolver::new("vex-libs/std");
        let result = resolver.resolve_module("nonexistent");
        assert!(matches!(result, Err(ResolveError::ModuleNotFound(_))));
    }

    #[test]
    fn test_resolve_multiple() {
        let project_root = std::env::current_dir().expect("Failed to get current directory");
        let stdlib_path = project_root.join("vex-libs/std");

        if !stdlib_path.exists() {
            return;
        }

        let resolver = StdlibResolver::new(&stdlib_path);
        let modules = vec!["io", "collections", "string"];
        let results = resolver
            .resolve_modules(&modules)
            .expect("Failed to resolve modules");

        assert_eq!(results.len(), 3);
        for (name, path) in results {
            assert!(modules.contains(&name.as_str()));
            assert!(path.exists());
        }
    }

    #[test]
    fn test_target() {
        let resolver = StdlibResolver::new("vex-libs/std");
        let target = resolver.target();
        assert_eq!(target.platform, Platform::current());
        assert_eq!(target.arch, Arch::current());
    }

    #[test]
    fn test_custom_target() {
        let target = Target::new(Platform::Linux, Arch::Arm64);
        let resolver = StdlibResolver::with_target("vex-libs/std", target);
        assert_eq!(resolver.target().platform, Platform::Linux);
        assert_eq!(resolver.target().arch, Arch::Arm64);
    }

    #[test]
    fn test_stdlib_root() {
        let resolver = StdlibResolver::new("vex-libs/std");
        assert_eq!(resolver.stdlib_root(), Path::new("vex-libs/std"));
    }

    #[test]
    fn test_priority_chain() {
        let project_root = std::env::current_dir().expect("Failed to get current directory");
        let stdlib_path = project_root.join("vex-libs/std");

        if !stdlib_path.exists() {
            return;
        }

        // This test verifies the priority chain logic
        let resolver = StdlibResolver::new(&stdlib_path);

        // io module has platform-specific variants
        let io_path = resolver
            .resolve_module("io")
            .expect("Failed to resolve io module");
        let path_str = io_path.to_str().expect("Invalid UTF-8 in path");

        // Should select platform-specific or generic file
        assert!(
            path_str.contains("lib.linux.vx")
                || path_str.contains("lib.macos.vx")
                || path_str.contains("lib.windows.vx")
                || path_str.contains("lib.vx")
        );
    }
}
