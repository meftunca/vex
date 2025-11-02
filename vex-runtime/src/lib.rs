//! Vex Runtime - Async runtime with tokio backend and UTF-8 support
//!
//! This module provides:
//! - Tokio-based async runtime (multi-threaded, io_uring/kqueue/IOCP)
//! - Ultra-fast UTF-8 validation and conversion (simdutf, 20GB/s)
//! - C FFI bindings for Vex language integration
//! - Cross-platform support (Linux, macOS, Windows)

use thiserror::Error;

pub mod native_runtime;

// Tokio async runtime FFI
#[cfg(feature = "tokio-runtime")]
pub mod tokio_ffi;

// simdutf UTF-8/UTF-16 FFI (requires libsimdutf)
#[cfg(feature = "simdutf")]
pub mod simdutf_ffi;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("Failed to create runtime: {0}")]
    CreationError(String),

    #[error("Task execution failed: {0}")]
    ExecutionError(String),
}

// Re-export native runtime
pub use native_runtime::{Scheduler, Task, TaskState};

#[cfg(feature = "io-uring-backend")]
pub mod io_uring_eventloop {
    //! Linux-specific io_uring event loop
    //! This module provides high-performance async I/O using io_uring
    //! TODO: Implement io_uring integration
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduler_creation() {
        let mut scheduler = Scheduler::new();
        assert_eq!(scheduler.spawn(), 0);
        assert_eq!(scheduler.spawn(), 1);
    }

    #[test]
    fn test_task_lifecycle() {
        native_runtime::vex_native_runtime_init();
        let task1 = native_runtime::vex_native_runtime_spawn();
        let task2 = native_runtime::vex_native_runtime_spawn();
        assert_ne!(task1, task2);
    }
}
