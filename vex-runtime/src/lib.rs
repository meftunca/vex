//! Vex Runtime - Native M:N async I/O runtime
//!
//! This module provides:
//! - C-based M:N async runtime (kqueue/epoll/io_uring/IOCP)
//! - Work-stealing scheduler with lock-free queues
//! - Ultra-fast UTF-8 validation and conversion (simdutf, 20GB/s)
//! - FFI bindings for Vex language integration
//! - Cross-platform support (Linux, macOS, Windows)

use thiserror::Error;

pub mod async_runtime;

// simdutf UTF-8/UTF-16 FFI (requires libsimdutf)
#[cfg(feature = "simdutf")]
pub mod simdutf_ffi;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("Failed to create runtime: {0}")]
    CreationError(String),

    #[error("Task execution failed: {0}")]
    ExecutionError(String),

    #[error("I/O error: {0}")]
    IoError(String),
}

// Re-export async runtime
pub use async_runtime::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_creation() {
        let rt = AsyncRuntime::new(2);
        let stats = rt.stats();
        assert_eq!(stats.tasks_spawned, 0);
    }
}
