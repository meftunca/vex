// vex-runtime: Tokio async runtime FFI bindings
// Exposes C-compatible functions for Vex to use tokio runtime

use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_void};
use tokio::runtime::{Builder, Runtime};
use tokio::task::JoinHandle;

// Safety wrapper for function pointers (they are Send)
unsafe impl Send for CallbackWrapper {}
struct CallbackWrapper(extern "C" fn(c_int, *mut c_void), usize);
struct TcpCallback(extern "C" fn(c_int, *mut c_void), usize);
unsafe impl Send for TcpCallback {}
struct FileReadCallback(extern "C" fn(*const u8, usize, *mut c_void), usize);
unsafe impl Send for FileReadCallback {}

// Opaque pointer for Vex
pub struct VexRuntime {
    runtime: Runtime,
}

pub struct VexTask {
    #[allow(dead_code)]
    handle: Option<JoinHandle<()>>,
}

// ============================================================================
// Runtime Management
// ============================================================================

/// Create a new multi-threaded tokio runtime
/// Returns: Opaque pointer to VexRuntime
#[unsafe(no_mangle)]
pub extern "C" fn vex_runtime_new() -> *mut VexRuntime {
    let runtime = Builder::new_multi_thread()
        .worker_threads(num_cpus::get())
        .enable_all()
        .build()
        .expect("Failed to create tokio runtime");

    Box::into_raw(Box::new(VexRuntime { runtime }))
}

/// Create a new single-threaded tokio runtime (for testing)
#[unsafe(no_mangle)]
pub extern "C" fn vex_runtime_new_current_thread() -> *mut VexRuntime {
    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to create tokio runtime");

    Box::into_raw(Box::new(VexRuntime { runtime }))
}

/// Destroy runtime and free memory
#[unsafe(no_mangle)]
pub extern "C" fn vex_runtime_destroy(runtime: *mut VexRuntime) {
    if !runtime.is_null() {
        unsafe {
            let _ = Box::from_raw(runtime);
        }
    }
}

// ============================================================================
// Task Spawning
// ============================================================================

/// Spawn a task on the runtime
/// task_fn: Function pointer to async task
/// user_data: Opaque pointer passed to task
#[unsafe(no_mangle)]
pub extern "C" fn vex_runtime_spawn(
    runtime: *mut VexRuntime,
    task_fn: extern "C" fn(*mut c_void),
    user_data: *mut c_void,
) -> *mut VexTask {
    if runtime.is_null() {
        return std::ptr::null_mut();
    }

    let runtime = unsafe { &*runtime };

    // Wrap raw pointer to make it Send
    let user_data_usize = user_data as usize;

    let handle = runtime.runtime.spawn(async move {
        let user_data_ptr = user_data_usize as *mut c_void;
        task_fn(user_data_ptr);
    });

    Box::into_raw(Box::new(VexTask {
        handle: Some(handle),
    }))
}

/// Block on a future (runs async task synchronously)
#[unsafe(no_mangle)]
pub extern "C" fn vex_runtime_block_on(
    runtime: *mut VexRuntime,
    task_fn: extern "C" fn(*mut c_void),
    user_data: *mut c_void,
) {
    if runtime.is_null() {
        return;
    }

    let runtime = unsafe { &*runtime };

    runtime.runtime.block_on(async move {
        task_fn(user_data);
    });
}

// ============================================================================
// Async Sleep
// ============================================================================

/// Async sleep for milliseconds (must be called from within tokio runtime)
/// Returns a task handle that can be awaited
#[unsafe(no_mangle)]
pub extern "C" fn vex_async_sleep_ms(runtime: *mut VexRuntime, millis: u64) -> *mut VexTask {
    if runtime.is_null() {
        return std::ptr::null_mut();
    }
    let runtime = unsafe { &*runtime };
    let handle = runtime.runtime.spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(millis)).await;
    });
    Box::into_raw(Box::new(VexTask {
        handle: Some(handle),
    }))
}

/// Async sleep for seconds (must be called from within tokio runtime)
/// Returns a task handle that can be awaited
#[unsafe(no_mangle)]
pub extern "C" fn vex_async_sleep_secs(runtime: *mut VexRuntime, secs: u64) -> *mut VexTask {
    if runtime.is_null() {
        return std::ptr::null_mut();
    }
    let runtime = unsafe { &*runtime };
    let handle = runtime.runtime.spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(secs)).await;
    });
    Box::into_raw(Box::new(VexTask {
        handle: Some(handle),
    }))
}

// ============================================================================
// Async TCP (high-level)
// ============================================================================

/// Async TCP connect
/// Returns: File descriptor (>= 0) on success, -1 on error
#[unsafe(no_mangle)]
pub extern "C" fn vex_tcp_connect_async(
    host: *const c_char,
    port: u16,
    callback: extern "C" fn(c_int, *mut c_void),
    user_data: *mut c_void,
) {
    if host.is_null() {
        callback(-1, user_data);
        return;
    }

    let host_str = unsafe { CStr::from_ptr(host) }.to_str().unwrap_or("");
    let addr = format!("{}:{}", host_str, port);

    let cb = TcpCallback(callback, user_data as usize);

    tokio::spawn(async move {
        match tokio::net::TcpStream::connect(&addr).await {
            Ok(stream) => {
                use std::os::unix::io::AsRawFd;
                let fd = stream.as_raw_fd();
                // Keep stream alive by leaking it (Vex will manage it)
                std::mem::forget(stream);
                (cb.0)(fd, cb.1 as *mut c_void);
            }
            Err(_) => {
                (cb.0)(-1, cb.1 as *mut c_void);
            }
        }
    });
}

// ============================================================================
// Async File I/O (tokio-fs)
// ============================================================================

/// Async file read
#[unsafe(no_mangle)]
pub extern "C" fn vex_file_read_async(
    path: *const c_char,
    callback: extern "C" fn(*const u8, usize, *mut c_void),
    user_data: *mut c_void,
) {
    if path.is_null() {
        callback(std::ptr::null(), 0, user_data);
        return;
    }

    let path_str = unsafe { CStr::from_ptr(path) }
        .to_string_lossy()
        .to_string();
    let cb = FileReadCallback(callback, user_data as usize);

    tokio::spawn(async move {
        match tokio::fs::read(&path_str).await {
            Ok(data) => {
                let ptr = data.as_ptr();
                let len = data.len();
                // Leak data so Vex can use it (Vex must free it later)
                std::mem::forget(data);
                (cb.0)(ptr, len, cb.1 as *mut c_void);
            }
            Err(_) => {
                (cb.0)(std::ptr::null(), 0, cb.1 as *mut c_void);
            }
        }
    });
}

/// Async file write
#[unsafe(no_mangle)]
pub extern "C" fn vex_file_write_async(
    path: *const c_char,
    data: *const u8,
    len: usize,
    callback: extern "C" fn(c_int, *mut c_void), // 0 = success, -1 = error
    user_data: *mut c_void,
) {
    if path.is_null() || data.is_null() {
        callback(-1, user_data);
        return;
    }

    let path_str = unsafe { CStr::from_ptr(path) }
        .to_string_lossy()
        .to_string();
    let data_vec = unsafe { std::slice::from_raw_parts(data, len) }.to_vec();
    let cb = CallbackWrapper(callback, user_data as usize);

    tokio::spawn(async move {
        match tokio::fs::write(&path_str, data_vec).await {
            Ok(_) => (cb.0)(0, cb.1 as *mut c_void),
            Err(_) => (cb.0)(-1, cb.1 as *mut c_void),
        }
    });
}

// ============================================================================
// Runtime Statistics
// ============================================================================

/// Get number of worker threads
#[unsafe(no_mangle)]
pub extern "C" fn vex_runtime_worker_threads() -> usize {
    num_cpus::get()
}

/// Check if running inside tokio runtime
#[unsafe(no_mangle)]
pub extern "C" fn vex_runtime_is_inside_runtime() -> bool {
    tokio::runtime::Handle::try_current().is_ok()
}
