// Lifetimes module - split into submodules for better organization

pub mod core;
pub mod expressions;
pub mod statements;

// Re-export the main LifetimeChecker from core
pub use core::LifetimeChecker;