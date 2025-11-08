// vex-pm - Vex Package Manager
// Phase 0.1-0.4: MVP Foundation + Build Integration

pub mod build;
pub mod cache;
pub mod cli;
pub mod commands;
pub mod git;
pub mod lockfile;
pub mod manifest;
pub mod platform;
pub mod resolver;

pub use build::{
    get_dependency_source_dirs, is_lockfile_valid, resolve_dependencies_for_build, DependencyPaths,
};
pub use cache::Cache;
pub use cli::{create_new_project, init_project};
pub use commands::{
    add_dependency, clean_cache, list_dependencies, remove_dependency, update_dependencies,
};
pub use git::{checkout_tag, clone_repository, package_url_to_git_url};
pub use lockfile::{LockFile, LockedPackage};
pub use manifest::{Dependency, Manifest, Profile, TargetConfig};
pub use platform::{select_platform_file, select_platform_file_for_test, Platform};
pub use resolver::{DependencyGraph, PackageVersion, ResolvedPackage};

/// Package manager version
pub const VERSION: &str = "0.1.0";
