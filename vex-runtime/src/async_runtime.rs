//! Vex Async Runtime - FFI bindings to C async runtime
//!
//! This module provides safe Rust wrappers around the C M:N async runtime
//! located in `vex-runtime/c/vex_async_io/`

use std::ffi::c_void;
use std::os::raw::c_int;

/// Opaque runtime handle
#[repr(C)]
pub struct Runtime {
    _private: [u8; 0],
}

/// Opaque worker context handle
#[repr(C)]
pub struct WorkerContext {
    _private: [u8; 0],
}

/// Opaque cancel token handle
#[repr(C)]
pub struct CancelToken {
    _private: [u8; 0],
}

/// Coroutine status
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoroStatus {
    Running = 0,
    Yielded = 1,
    Done = 2,
}

/// Event type for I/O operations
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    None = 0,
    Readable = 1,
    Writable = 2,
}

/// Runtime statistics
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct RuntimeStats {
    pub tasks_spawned: u64,
    pub tasks_done: u64,
    pub poller_events: u64,
    pub io_submitted: u64,
    pub steals: u64,
    pub parks: u64,
    pub unparks: u64,
}

/// Coroutine resume function type
pub type CoroResumeFn = extern "C" fn(*mut WorkerContext, *mut c_void) -> CoroStatus;

// External C functions from vex_async_io runtime
extern "C" {
    /// Create a new runtime with N worker threads (0 = auto-detect)
    pub fn runtime_create(num_workers: c_int) -> *mut Runtime;

    /// Destroy runtime
    pub fn runtime_destroy(runtime: *mut Runtime);

    /// Spawn task to global queue
    pub fn runtime_spawn_global(
        runtime: *mut Runtime,
        resume_fn: CoroResumeFn,
        coro_data: *mut c_void,
    );

    /// Run runtime (blocks until shutdown)
    pub fn runtime_run(runtime: *mut Runtime);

    /// Request shutdown
    pub fn runtime_shutdown(runtime: *mut Runtime);

    /// Enable/disable tracing
    pub fn runtime_set_tracing(runtime: *mut Runtime, enabled: bool);

    /// Enable/disable auto-shutdown
    pub fn runtime_enable_auto_shutdown(runtime: *mut Runtime, enabled: bool);

    /// Get runtime statistics
    pub fn runtime_get_stats(runtime: *mut Runtime, out_stats: *mut RuntimeStats);

    /// Await I/O event
    pub fn worker_await_io(context: *mut WorkerContext, fd: c_int, event_type: EventType);

    /// Await timer (milliseconds)
    pub fn worker_await_after(context: *mut WorkerContext, millis: u64);

    /// Await deadline (nanoseconds since epoch)
    pub fn worker_await_deadline(context: *mut WorkerContext, deadline_ns: u64);

    /// Spawn task on local worker queue
    pub fn worker_spawn_local(
        context: *mut WorkerContext,
        resume_fn: CoroResumeFn,
        coro_data: *mut c_void,
    );

    /// Get cancellation token
    pub fn worker_cancel_token(context: *mut WorkerContext) -> *mut CancelToken;

    /// Check if cancellation requested
    pub fn cancel_requested(token: *const CancelToken) -> bool;

    /// Request cancellation
    pub fn cancel_request(token: *mut CancelToken);
}

/// Safe Rust wrapper for Runtime
pub struct AsyncRuntime {
    inner: *mut Runtime,
}

impl AsyncRuntime {
    /// Create a new async runtime with N worker threads
    /// If `num_workers` is 0, auto-detects CPU count
    pub fn new(num_workers: usize) -> Result<Self, String> {
        let inner = unsafe { runtime_create(num_workers as c_int) };
        if inner.is_null() {
            return Err("Failed to create runtime".to_string());
        }
        Ok(Self { inner })
    }

    /// Get raw pointer (for FFI)
    pub fn as_ptr(&self) -> *mut Runtime {
        self.inner
    }

    /// Spawn a task to the global queue
    pub fn spawn<F>(&self, task: F)
    where
        F: FnOnce(*mut WorkerContext) -> CoroStatus + Send + 'static,
    {
        let boxed = Box::new(task);
        let data = Box::into_raw(boxed) as *mut c_void;

        extern "C" fn trampoline<F>(ctx: *mut WorkerContext, data: *mut c_void) -> CoroStatus
        where
            F: FnOnce(*mut WorkerContext) -> CoroStatus + Send + 'static,
        {
            let task = unsafe { Box::from_raw(data as *mut F) };
            task(ctx)
        }

        unsafe {
            runtime_spawn_global(self.inner, trampoline::<F>, data);
        }
    }

    /// Run the runtime (blocks until shutdown)
    pub fn run(&self) {
        unsafe {
            runtime_run(self.inner);
        }
    }

    /// Request shutdown
    pub fn shutdown(&self) {
        unsafe {
            runtime_shutdown(self.inner);
        }
    }

    /// Enable auto-shutdown when all tasks complete
    pub fn enable_auto_shutdown(&self, enabled: bool) {
        unsafe {
            runtime_enable_auto_shutdown(self.inner, enabled);
        }
    }

    /// Get runtime statistics
    pub fn stats(&self) -> RuntimeStats {
        let mut stats = RuntimeStats::default();
        unsafe {
            runtime_get_stats(self.inner, &mut stats as *mut RuntimeStats);
        }
        stats
    }
}

impl Drop for AsyncRuntime {
    fn drop(&mut self) {
        unsafe {
            runtime_destroy(self.inner);
        }
    }
}

unsafe impl Send for AsyncRuntime {}
unsafe impl Sync for AsyncRuntime {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_creation() {
        let rt = AsyncRuntime::new(2).expect("Failed to create runtime");
        let stats = rt.stats();
        assert_eq!(stats.tasks_spawned, 0);
    }
}
