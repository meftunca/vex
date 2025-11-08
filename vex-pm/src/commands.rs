// Package manager commands (add, remove, list)

use crate::cache::Cache;
use crate::git::{
    checkout_tag, clone_repository, fetch_tags, get_latest_tag, package_url_to_git_url,
};
use crate::lockfile::{LockFile, LockedPackage};
use crate::manifest::{Dependency, Manifest};
use crate::resolver::{DependencyGraph, PackageVersion};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Add a dependency to vex.json
pub fn add_dependency(package: &str, version: Option<&str>) -> Result<()> {
    // Load vex.json
    let manifest_path = Path::new("vex.json");
    if !manifest_path.exists() {
        anyhow::bail!("vex.json not found. Run 'vex init' first.");
    }

    let mut manifest = Manifest::from_file(manifest_path)?;

    // Parse package URL and version
    let (package_url, package_version) = parse_package_spec(package, version)?;

    // Download package to cache
    let cache = Cache::new()?;
    download_package(&package_url, &package_version, &cache)?;

    // Add to dependencies
    manifest.dependencies.insert(
        package_url.clone(),
        Dependency::Simple(package_version.clone()),
    );

    // Save manifest
    manifest.to_file(manifest_path)?;

    // Generate/update lock file
    update_lockfile(&manifest)?;

    println!("✅ Added dependency: {} @ {}", package_url, package_version);
    println!("   Saved to vex.json");
    println!("   Updated vex.lock");

    Ok(())
}

/// Remove a dependency from vex.json
pub fn remove_dependency(package: &str) -> Result<()> {
    // Load vex.json
    let manifest_path = Path::new("vex.json");
    if !manifest_path.exists() {
        anyhow::bail!("vex.json not found.");
    }

    let mut manifest = Manifest::from_file(manifest_path)?;

    // Find and remove dependency
    let removed = manifest.dependencies.remove(package);

    if removed.is_none() {
        anyhow::bail!("Dependency not found: {}", package);
    }

    // Save manifest
    manifest.to_file(manifest_path)?;

    // Update lock file
    update_lockfile(&manifest)?;

    println!("✅ Removed dependency: {}", package);
    println!("   Updated vex.lock");

    Ok(())
}

/// List all dependencies
pub fn list_dependencies() -> Result<()> {
    // Load vex.json
    let manifest_path = Path::new("vex.json");
    if !manifest_path.exists() {
        anyhow::bail!("vex.json not found.");
    }

    let manifest = Manifest::from_file(manifest_path)?;

    if manifest.dependencies.is_empty() {
        println!("No dependencies found.");
        return Ok(());
    }

    println!("📦 Dependencies:");
    println!();

    for (name, dep) in &manifest.dependencies {
        let version = match dep {
            Dependency::Simple(v) => v.clone(),
            Dependency::Detailed { version, .. } => version.clone(),
        };

        println!("  {} @ {}", name, version);
    }

    println!();
    println!("Total: {} dependencies", manifest.dependencies.len());

    Ok(())
}

/// Parse package specification (package[@version])
fn parse_package_spec(package: &str, explicit_version: Option<&str>) -> Result<(String, String)> {
    // Check for explicit version flag
    if let Some(ver) = explicit_version {
        return Ok((package.to_string(), ver.to_string()));
    }

    // Parse package@version format
    if let Some(at_pos) = package.rfind('@') {
        let pkg = package[..at_pos].to_string();
        let ver = package[at_pos + 1..].to_string();
        return Ok((pkg, ver));
    }

    // No version specified - use "latest"
    Ok((package.to_string(), "latest".to_string()))
}

/// Download package to cache
fn download_package(package_url: &str, version: &str, cache: &Cache) -> Result<()> {
    // Convert package URL to git URL
    let git_url = package_url_to_git_url(package_url);

    println!("📥 Downloading {} @ {}...", package_url, version);
    println!("   Git URL: {}", git_url);

    // Clone repository to cache
    let git_cache = cache.git_dir();
    fs::create_dir_all(&git_cache)?;

    let repo_path = clone_repository(&git_url, &git_cache)?;
    println!("   ✅ Cloned to cache");

    // Fetch latest tags
    fetch_tags(&repo_path)?;

    // Resolve version
    let resolved_version = if version == "latest" {
        let latest = get_latest_tag(&repo_path)?;
        println!("   Latest version: {}", latest);
        latest
    } else {
        version.to_string()
    };

    // Checkout tag
    checkout_tag(&repo_path, &resolved_version)?;
    println!("   ✅ Checked out {}", resolved_version);

    Ok(())
}

/// Update lock file based on manifest
fn update_lockfile(manifest: &Manifest) -> Result<()> {
    use crate::lockfile::LockFile;
    use crate::resolver::PackageVersion;

    let cache = Cache::new()?;

    // Build list of resolved packages
    let mut resolved = Vec::new();
    for (name, dep) in &manifest.dependencies {
        let version = match dep {
            Dependency::Simple(v) => v.clone(),
            Dependency::Detailed { version, .. } => version.clone(),
        };

        resolved.push(PackageVersion {
            name: name.clone(),
            version,
        });
    }

    // Generate lock file
    let lockfile = LockFile::generate(manifest, &resolved, &cache)?;

    // Save to disk
    lockfile.to_file("vex.lock")?;

    Ok(())
}

/// Update all dependencies to latest versions
pub fn update_dependencies() -> Result<()> {
    // Load vex.json
    let manifest_path = Path::new("vex.json");
    if !manifest_path.exists() {
        anyhow::bail!("vex.json not found.");
    }

    let mut manifest = Manifest::from_file(manifest_path)?;
    let cache = Cache::new()?;

    println!("🔄 Updating dependencies...");
    println!();

    let mut updated_count = 0;

    // Update each dependency to latest version
    for (package_url, dep) in &mut manifest.dependencies {
        let current_version = match dep {
            Dependency::Simple(v) => v.clone(),
            Dependency::Detailed { version, .. } => version.clone(),
        };

        // Fetch latest version
        let git_url = package_url_to_git_url(package_url);
        let git_cache = cache.git_dir();
        fs::create_dir_all(&git_cache)?;

        let repo_path = clone_repository(&git_url, &git_cache)?;
        fetch_tags(&repo_path)?;

        let latest_version = match get_latest_tag(&repo_path) {
            Ok(v) => v,
            Err(_) => {
                println!("   ⚠️  {} - Cannot determine latest version", package_url);
                continue;
            }
        };

        if latest_version != current_version {
            println!(
                "   ✅ {} {} → {}",
                package_url, current_version, latest_version
            );

            // Update manifest
            match dep {
                Dependency::Simple(v) => *v = latest_version.clone(),
                Dependency::Detailed { version, .. } => *version = latest_version.clone(),
            }

            // Checkout new version
            checkout_tag(&repo_path, &latest_version)?;
            updated_count += 1;
        } else {
            println!("   ✓  {} (already at {})", package_url, current_version);
        }
    }

    // Save manifest
    manifest.to_file(manifest_path)?;

    // Regenerate lock file
    update_lockfile(&manifest)?;

    println!();
    if updated_count > 0 {
        println!("✅ Updated {} dependencies", updated_count);
        println!("   Saved to vex.json and vex.lock");
    } else {
        println!("✅ All dependencies are up to date");
    }

    Ok(())
}

/// Clean cache and build artifacts
pub fn clean_cache() -> Result<()> {
    use anyhow::Context;

    let cache = Cache::new()?;

    // Get cache size before cleaning
    let size_before = cache.size()?;

    println!("🧹 Cleaning cache...");
    println!("   Cache size: {}", crate::cache::format_size(size_before));

    // Clean cache
    cache.clean()?;

    // Clean build directory
    let build_dir = Path::new("vex-builds");
    if build_dir.exists() {
        fs::remove_dir_all(build_dir).context("Failed to remove vex-builds/")?;
        println!("   ✅ Removed vex-builds/");
    }

    println!("✅ Cache cleaned successfully");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_package_spec() {
        let (pkg, ver) = parse_package_spec("github.com/user/repo@v1.0.0", None).unwrap();
        assert_eq!(pkg, "github.com/user/repo");
        assert_eq!(ver, "v1.0.0");

        let (pkg, ver) = parse_package_spec("github.com/user/repo", Some("v2.0.0")).unwrap();
        assert_eq!(pkg, "github.com/user/repo");
        assert_eq!(ver, "v2.0.0");

        let (pkg, ver) = parse_package_spec("github.com/user/repo", None).unwrap();
        assert_eq!(pkg, "github.com/user/repo");
        assert_eq!(ver, "latest");
    }
}
