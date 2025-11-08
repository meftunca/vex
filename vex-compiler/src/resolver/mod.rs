/**
 * Module Resolver
 * Resolves import paths to file locations, handles stdlib vs user modules
 */
pub mod platform;
pub mod stdlib_resolver;

pub use platform::{Arch, Platform, Target};
pub use stdlib_resolver::{ResolveError, StdlibResolver};
