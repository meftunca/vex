// Build integration: resolve dependencies and link external packages

use crate::cache::Cache;
use crate::lockfile::LockFile;
use crate::manifest::Manifest;
use crate::platform::{select_platform_file, Platform};
use crate::resolver::DependencyGraph;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Resolved dependency paths for compilation
#[derive(Debug, Clone)]
pub struct DependencyPaths {
    /// Package name -> source directory
    pub packages: HashMap<String, PathBuf>,

    /// Platform-specific file mappings
    pub platform_files: HashMap<String, String>,
}

impl DependencyPaths {
    pub fn new() -> Self {
        Self {
            packages: HashMap::new(),
            platform_files: HashMap::new(),
        }
    }

    /// Get all source directories (for compiler module search paths)
    pub fn source_directories(&self) -> Vec<PathBuf> {
        self.packages.values().cloned().collect()
    }

    /// Get platform-specific file for a module
    pub fn get_platform_file(&self, module_name: &str) -> Option<String> {
        self.platform_files.get(module_name).cloned()
    }
}

/// Resolve dependencies for build (respects lock file if exists)
pub fn resolve_dependencies_for_build(locked: bool) -> Result<DependencyPaths> {
    let manifest_path = Path::new("vex.json");
    if !manifest_path.exists() {
        // No manifest = no dependencies
        return Ok(DependencyPaths::new());
    }

    let manifest = Manifest::from_file(manifest_path)?;
    let cache = Cache::new()?;

    // Check for lock file
    let lock_path = Path::new("vex.lock");
    if locked {
        // CI mode: lock file MUST exist and be valid
        if !lock_path.exists() {
            anyhow::bail!("vex.lock not found. Run 'vex build' without --locked to generate it.");
        }

        let lockfile = LockFile::from_file(lock_path)?;

        // Validate lock file integrity
        let errors = lockfile.validate(&cache)?;
        if !errors.is_empty() {
            anyhow::bail!("Lock file validation failed:\n{}", errors.join("\n"));
        }

        // Use locked versions
        link_locked_packages(&lockfile, &cache)
    } else {
        // Normal mode: resolve dependencies (use lock if exists, regenerate if not)
        if lock_path.exists() {
            let lockfile = LockFile::from_file(lock_path)?;

            // Validate lock file
            let errors = lockfile.validate(&cache)?;
            if errors.is_empty() {
                // Lock file is valid, use it
                link_locked_packages(&lockfile, &cache)
            } else {
                // Lock file is invalid, re-resolve
                eprintln!("⚠️  Lock file is outdated, re-resolving dependencies...");
                resolve_and_link(&manifest, &cache)
            }
        } else {
            // No lock file, resolve and generate
            resolve_and_link(&manifest, &cache)
        }
    }
}

/// Link packages from lock file
fn link_locked_packages(lockfile: &LockFile, cache: &Cache) -> Result<DependencyPaths> {
    let mut paths = DependencyPaths::new();
    let platform = Platform::detect();

    for (name, locked) in &lockfile.dependencies {
        let git_cache_path = cache.git_dir().join(name);

        if !git_cache_path.exists() {
            anyhow::bail!(
                "Package '{}' not in cache. Run 'vex add {}' or 'vex update'.",
                name,
                name
            );
        }

        // Add source directory
        let src_dir = git_cache_path.join("src");
        if src_dir.exists() {
            paths.packages.insert(name.clone(), src_dir);
        } else {
            // Fallback to root if src/ doesn't exist
            paths.packages.insert(name.clone(), git_cache_path.clone());
        }

        // Platform-specific file selection
        if let Some(platform_map) = &locked.platform {
            for (module, base_file) in platform_map {
                let selected_path = select_platform_file(git_cache_path.join(base_file), &platform);
                paths
                    .platform_files
                    .insert(module.clone(), selected_path.to_string_lossy().to_string());
            }
        }
    }

    Ok(paths)
}

/// Resolve dependencies and link packages
fn resolve_and_link(manifest: &Manifest, cache: &Cache) -> Result<DependencyPaths> {
    // Build dependency graph
    let mut graph = DependencyGraph::new();

    // Resolve dependencies from manifest
    let resolved = graph.resolve(manifest)?;

    // Link resolved packages
    let mut paths = DependencyPaths::new();
    let platform = Platform::detect();

    for pkg in resolved {
        let git_cache_path = cache.git_dir().join(&pkg.name);

        if !git_cache_path.exists() {
            anyhow::bail!(
                "Package '{}' not in cache. Run 'vex add {}'.",
                pkg.name,
                pkg.name
            );
        }

        // Add source directory
        let src_dir = git_cache_path.join("src");
        let src_path = if src_dir.exists() {
            paths.packages.insert(pkg.name.clone(), src_dir.clone());
            src_dir
        } else {
            paths
                .packages
                .insert(pkg.name.clone(), git_cache_path.clone());
            git_cache_path.clone()
        };

        // Scan for platform-specific files in src/
        if let Ok(entries) = std::fs::read_dir(&src_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(file_name) = path.file_name() {
                    let name_str = file_name.to_string_lossy();
                    if name_str.ends_with(".vx") {
                        // Check for platform-specific variants
                        let base_name = name_str.trim_end_matches(".vx");
                        let selected_path =
                            select_platform_file(src_path.join(base_name), &platform);
                        paths.platform_files.insert(
                            base_name.to_string(),
                            selected_path.to_string_lossy().to_string(),
                        );
                    }
                }
            }
        }
    }

    Ok(paths)
}

/// Get dependency source directories for compiler
pub fn get_dependency_source_dirs() -> Result<Vec<PathBuf>> {
    let paths = resolve_dependencies_for_build(false)?;
    Ok(paths.source_directories())
}

/// Check if lock file is up to date
pub fn is_lockfile_valid() -> Result<bool> {
    let manifest_path = Path::new("vex.json");
    let lock_path = Path::new("vex.lock");

    if !manifest_path.exists() {
        return Ok(true); // No manifest = no dependencies
    }

    if !lock_path.exists() {
        return Ok(false); // Manifest exists but no lock file
    }

    let cache = Cache::new()?;
    let lockfile = LockFile::from_file(lock_path)?;

    // Validate integrity
    let errors = lockfile.validate(&cache)?;
    Ok(errors.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_paths_creation() {
        let paths = DependencyPaths::new();
        assert!(paths.packages.is_empty());
        assert!(paths.platform_files.is_empty());
    }

    #[test]
    fn test_source_directories() {
        let mut paths = DependencyPaths::new();
        paths
            .packages
            .insert("pkg1".to_string(), PathBuf::from("/cache/pkg1/src"));
        paths
            .packages
            .insert("pkg2".to_string(), PathBuf::from("/cache/pkg2/src"));

        let dirs = paths.source_directories();
        assert_eq!(dirs.len(), 2);
    }
}
