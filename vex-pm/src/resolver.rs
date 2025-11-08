// Dependency resolver with Minimum Version Selection (MVS)

use crate::manifest::{Dependency, Manifest};
use anyhow::{bail, Context, Result};
use std::collections::{HashMap, HashSet};

/// Package version information
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageVersion {
    pub name: String,
    pub version: String,
}

/// Resolved package with dependencies
#[derive(Debug, Clone)]
pub struct ResolvedPackage {
    pub name: String,
    pub version: String,
    pub dependencies: Vec<PackageVersion>,
}

/// Dependency graph
pub struct DependencyGraph {
    packages: HashMap<String, ResolvedPackage>,
    resolved: HashMap<String, String>, // package name -> resolved version
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            packages: HashMap::new(),
            resolved: HashMap::new(),
        }
    }

    /// Add a package to the graph
    pub fn add_package(&mut self, package: ResolvedPackage) {
        self.packages.insert(package.name.clone(), package);
    }

    /// Resolve all dependencies using Minimum Version Selection (MVS)
    pub fn resolve(&mut self, root_manifest: &Manifest) -> Result<Vec<PackageVersion>> {
        // Build dependency tree
        let mut to_visit: Vec<(String, String)> = root_manifest
            .dependencies
            .iter()
            .map(|(name, dep)| {
                let version = match dep {
                    Dependency::Simple(v) => v.clone(),
                    Dependency::Detailed { version, .. } => version.clone(),
                };
                (name.clone(), version)
            })
            .collect();

        let mut visited = HashSet::new();

        while let Some((name, requested_version)) = to_visit.pop() {
            if visited.contains(&name) {
                // Check version compatibility
                if let Some(existing_version) = self.resolved.get(&name) {
                    let selected = select_higher_version(existing_version, &requested_version)?;
                    self.resolved.insert(name.clone(), selected);
                }
                continue;
            }

            visited.insert(name.clone());
            self.resolved
                .insert(name.clone(), requested_version.clone());

            // Add transitive dependencies (if package info available)
            if let Some(package) = self.packages.get(&name) {
                for dep in &package.dependencies {
                    to_visit.push((dep.name.clone(), dep.version.clone()));
                }
            }
        }

        // Convert to sorted list
        let mut result: Vec<PackageVersion> = self
            .resolved
            .iter()
            .map(|(name, version)| PackageVersion {
                name: name.clone(),
                version: version.clone(),
            })
            .collect();

        result.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(result)
    }

    /// Detect version conflicts
    pub fn detect_conflicts(&self) -> Vec<String> {
        let mut conflicts = Vec::new();

        for (name, package) in &self.packages {
            if let Some(resolved_version) = self.resolved.get(name) {
                for dep in &package.dependencies {
                    if let Some(dep_resolved) = self.resolved.get(&dep.name) {
                        if !versions_compatible(&dep.version, dep_resolved) {
                            conflicts.push(format!(
                                "Version conflict for {}: {} requires {}, but {} is resolved",
                                dep.name, name, dep.version, dep_resolved
                            ));
                        }
                    }
                }
            }
        }

        conflicts
    }

    /// Get resolved version for a package
    pub fn get_resolved(&self, name: &str) -> Option<&String> {
        self.resolved.get(name)
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Select higher version between two semver versions
fn select_higher_version(v1: &str, v2: &str) -> Result<String> {
    let ver1 = parse_semver(v1)?;
    let ver2 = parse_semver(v2)?;

    if ver1.0 != ver2.0 {
        // Major version mismatch - incompatible
        bail!(
            "Incompatible versions: {} and {} (major version mismatch)",
            v1,
            v2
        );
    }

    // Return higher version
    if ver1 > ver2 {
        Ok(v1.to_string())
    } else {
        Ok(v2.to_string())
    }
}

/// Check if two versions are compatible (same major version)
fn versions_compatible(requested: &str, resolved: &str) -> bool {
    if requested == "latest" {
        return true;
    }

    if let (Ok(req), Ok(res)) = (parse_semver(requested), parse_semver(resolved)) {
        // Compatible if same major version and resolved >= requested
        req.0 == res.0 && res >= req
    } else {
        false
    }
}

/// Parse semver version (v1.2.3 -> (1, 2, 3))
fn parse_semver(version: &str) -> Result<(u32, u32, u32)> {
    let version = version.strip_prefix('v').unwrap_or(version);
    let parts: Vec<&str> = version.split('.').collect();

    if parts.len() != 3 {
        bail!("Invalid semver: {}", version);
    }

    let major = parts[0].parse::<u32>().context("Invalid major version")?;
    let minor = parts[1].parse::<u32>().context("Invalid minor version")?;
    let patch = parts[2].parse::<u32>().context("Invalid patch version")?;

    Ok((major, minor, patch))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_semver() {
        assert_eq!(parse_semver("v1.2.3").unwrap(), (1, 2, 3));
        assert_eq!(parse_semver("1.2.3").unwrap(), (1, 2, 3));
        assert_eq!(parse_semver("v2.0.0").unwrap(), (2, 0, 0));
    }

    #[test]
    fn test_select_higher_version() {
        assert_eq!(select_higher_version("v1.2.0", "v1.3.0").unwrap(), "v1.3.0");
        assert_eq!(select_higher_version("v1.5.0", "v1.3.0").unwrap(), "v1.5.0");

        // Major version conflict
        assert!(select_higher_version("v1.0.0", "v2.0.0").is_err());
    }

    #[test]
    fn test_versions_compatible() {
        assert!(versions_compatible("v1.2.0", "v1.3.0"));
        assert!(versions_compatible("v1.2.0", "v1.2.5"));
        assert!(!versions_compatible("v1.0.0", "v2.0.0"));
        assert!(!versions_compatible("v1.5.0", "v1.3.0"));
    }
}
