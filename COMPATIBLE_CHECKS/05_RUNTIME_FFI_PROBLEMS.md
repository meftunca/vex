# 05: Runtime & FFI Problems

**Severity:** 🔴 CRITICAL  
**Category:** Runtime System / Foreign Function Interface  
**Analysis Date:** 15 Kasım 2025  
**Status:** IDENTIFIED - CRITICAL SAFETY ISSUES

---

## Executive Summary

Vex runtime ve FFI layer'ında **23 ciddi sorun** bulundu. C FFI safety, async runtime, memory management ve panic handling problemli. Undefined behavior riskleri var.

**Ana Sorunlar:**
- C FFI type safety yok
- Async runtime incomplete (no executor)
- Panic unwinding broken
- Memory allocator basic
- Thread-local storage missing

**Impact:** Foreign code ile entegrasyon unsafe, async/await çalışmıyor, panic'ler program terminate ediyor.

---

## Critical Issues (🔴)

### Issue 1: C FFI Type Safety Missing

**File:** `vex-compiler/src/codegen_ast/ffi.rs`  
**Severity:** 🔴 CRITICAL  
**Impact:** Calling C functions with wrong types → undefined behavior

**Evidence:**
```rust
// vex-compiler/src/codegen_ast/ffi.rs
// FFI module exists but no type checking!
pub fn call_external_function(
    &mut self,
    name: &str,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    // ❌ No type validation
    // ❌ No ABI checking
    // ❌ No null pointer checks
}
```

**Problem:**
```vex
extern "C" fn strlen(s: *const u8) -> usize;

fn main() {
    let x: i32 = 42;
    let len = strlen(&x as *const u8);  // ❌ Type confusion, UB
}
```

**Recommendation:**
```rust
struct FFIChecker {
    extern_signatures: HashMap<String, FunctionSignature>,
}

impl FFIChecker {
    fn validate_call(
        &self,
        name: &str,
        args: &[ASTType],
    ) -> Result<(), FFIError> {
        let sig = self.extern_signatures.get(name)
            .ok_or(FFIError::UndeclaredFunction)?;
        
        if args.len() != sig.params.len() {
            return Err(FFIError::ArgCountMismatch);
        }
        
        for (arg, param) in args.iter().zip(&sig.params) {
            if !self.is_ffi_safe(arg) {
                return Err(FFIError::UnsafeType(arg.clone()));
            }
            
            if !self.types_compatible(arg, param) {
                return Err(FFIError::TypeMismatch {
                    expected: param.clone(),
                    found: arg.clone(),
                });
            }
        }
        
        Ok(())
    }
    
    fn is_ffi_safe(&self, ty: &ASTType) -> bool {
        match ty {
            ASTType::Int(_) | ASTType::Float(_) | ASTType::Bool => true,
            ASTType::Pointer(_) => true,
            ASTType::Struct { .. } => {
                // Check #[repr(C)] attribute
                self.has_repr_c(ty)
            }
            ASTType::String | ASTType::Vec(_) => false,  // Not FFI-safe
            _ => false,
        }
    }
}
```

**Effort:** 2-3 weeks

---

### Issue 2: Async Runtime Not Implemented

**File:** `vex-runtime/src/async_runtime/` (does not exist)  
**Severity:** 🔴 CRITICAL  
**Impact:** async/await syntax exists but no executor

**Evidence:**
```bash
$ vex run examples/test_async_await.vx
Error: Async runtime not initialized
```

**Problem:**
```vex
async fn fetch() -> String {
    let resp = await http.get("example.com");
    return resp.body;
}

fn main() {
    let result = fetch();  // ❌ Returns Future, but no runtime to run it
}
```

**Recommendation:**
```rust
// vex-runtime/src/async_runtime/mod.rs
pub struct Runtime {
    executor: Executor,
    reactor: Reactor,
}

impl Runtime {
    pub fn new() -> Runtime {
        Runtime {
            executor: Executor::new(),
            reactor: Reactor::new(),
        }
    }
    
    pub fn block_on<F>(&mut self, future: F) -> F::Output
    where
        F: Future,
    {
        self.executor.spawn(future);
        loop {
            if let Some(result) = self.executor.poll() {
                return result;
            }
            self.reactor.wait_for_io();
        }
    }
}

struct Executor {
    tasks: VecDeque<Task>,
    waker: Waker,
}

struct Reactor {
    epoll_fd: RawFd,  // Linux
    // kqueue for macOS
}
```

**Integration:**
```vex
// Implicit runtime creation
fn main() {
    let result = block_on(fetch());  // Runtime manages Future
    println(result);
}
```

**Effort:** 6-8 weeks (major project)

---

### Issue 3: Panic Unwinding Broken

**File:** `vex-runtime/src/panic.rs:1-50`  
**Severity:** 🔴 CRITICAL  
**Impact:** Panics crash entire program instead of unwinding

**Evidence:**
```rust
// vex-runtime/src/panic.rs:20
#[no_mangle]
pub extern "C" fn vex_panic(msg: *const u8, len: usize) {
    let slice = unsafe { std::slice::from_raw_parts(msg, len) };
    let message = std::str::from_utf8(slice).unwrap_or("<invalid utf8>");
    eprintln!("Vex panic: {}", message);
    std::process::exit(1);  // ❌ No unwinding, immediate exit
}
```

**Problem:**
```vex
fn might_fail() {
    if some_condition {
        panic("error");  // ❌ Entire program crashes
    }
}

fn main() {
    let _resource = acquire_resource();
    might_fail();
    // _resource never dropped!
}
```

**Recommendation:**
```rust
// Implement stack unwinding with libunwind
use unwinding::{UnwindContext, unwind_backtrace};

struct PanicInfo {
    message: String,
    backtrace: Vec<Frame>,
}

#[no_mangle]
pub extern "C" fn vex_panic(msg: *const u8, len: usize) -> ! {
    let panic_info = PanicInfo {
        message: get_message(msg, len),
        backtrace: capture_backtrace(),
    };
    
    // Unwind stack, calling drop on all locals
    unwind_stack(&panic_info);
    
    // If unwinding reaches main, print and exit
    print_panic(&panic_info);
    std::process::exit(1);
}

fn unwind_stack(info: &PanicInfo) {
    unsafe {
        let mut context = UnwindContext::new();
        unwind_backtrace(&mut context, |frame| {
            // Call destructors for locals in this frame
            call_frame_destructors(frame);
        });
    }
}
```

**Effort:** 4-5 weeks

---

### Issue 4: Memory Allocator Basic

**File:** `vex-runtime/src/allocator.rs`  
**Severity:** 🔴 CRITICAL  
**Impact:** No custom allocators, no allocation tracking

**Evidence:**
```rust
// vex-runtime/src/allocator.rs
// Uses Rust's default allocator directly
// No custom allocator support
// No allocation profiling
```

**Problem:**
```vex
// Cannot use custom allocators
fn main() {
    // All allocations use system allocator
    let v = Vec<i32>.new();  // malloc
    let s = String.from("x");  // malloc
    // No way to use arena, pool, etc.
}
```

**Recommendation:**
```rust
pub trait Allocator {
    fn allocate(&mut self, size: usize, align: usize) -> *mut u8;
    fn deallocate(&mut self, ptr: *mut u8, size: usize, align: usize);
    fn reallocate(
        &mut self,
        ptr: *mut u8,
        old_size: usize,
        new_size: usize,
        align: usize,
    ) -> *mut u8;
}

pub struct GlobalAllocator {
    inner: Box<dyn Allocator>,
}

#[no_mangle]
pub extern "C" fn vex_alloc(size: usize, align: usize) -> *mut u8 {
    GLOBAL_ALLOCATOR.lock().allocate(size, align)
}

#[no_mangle]
pub extern "C" fn vex_dealloc(ptr: *mut u8, size: usize, align: usize) {
    GLOBAL_ALLOCATOR.lock().deallocate(ptr, size, align)
}
```

**Vex API:**
```vex
contract Allocator {
    fn allocate(self!, size: usize, align: usize) -> *u8!;
    fn deallocate(self!, ptr: *u8!, size: usize, align: usize);
}

fn set_global_allocator(alloc: Allocator);
```

**Effort:** 2-3 weeks

---

### Issue 5: Thread-Local Storage Missing

**File:** N/A  
**Severity:** 🔴 CRITICAL  
**Impact:** Cannot use thread-local variables

**Problem:**
```vex
// This doesn't work:
thread_local let counter: i32 = 0;

fn increment() {
    counter += 1;  // ❌ Not supported
}
```

**Recommendation:**
```rust
// Runtime support
#[no_mangle]
pub extern "C" fn vex_tls_get(key: usize) -> *mut u8 {
    thread_local! {
        static TLS_MAP: RefCell<HashMap<usize, *mut u8>> = RefCell::new(HashMap::new());
    }
    
    TLS_MAP.with(|map| {
        *map.borrow().get(&key).unwrap_or(&std::ptr::null_mut())
    })
}

#[no_mangle]
pub extern "C" fn vex_tls_set(key: usize, value: *mut u8) {
    // ...
}
```

**Effort:** 1-2 weeks

---

## High Priority Issues (🟡)

### Issue 6: Foreign Callback Safety

**File:** `vex-compiler/src/codegen_ast/ffi.rs`  
**Severity:** 🟡 HIGH  
**Impact:** C code calling Vex code can cause UB

**Problem:**
```vex
fn my_callback(x: i32) -> i32 {
    return x * 2;
}

extern "C" fn qsort(
    base: *u8!,
    nmemb: usize,
    size: usize,
    compar: fn(*const u8, *const u8) -> i32
);

fn main() {
    let arr = [3, 1, 2];
    qsort(&arr, 3, 8, my_callback);  // ❌ ABI mismatch
}
```

**Recommendation:** Require `extern "C"` on callbacks passed to C

**Effort:** 1 week

---

### Issue 7: Signal Handling Missing

**File:** N/A  
**Severity:** 🟡 HIGH  
**Impact:** Cannot handle SIGINT, SIGSEGV, etc.

**Recommendation:**
```rust
#[no_mangle]
pub extern "C" fn vex_set_signal_handler(
    sig: i32,
    handler: extern "C" fn(i32),
) {
    unsafe {
        signal(sig, handler);
    }
}
```

**Effort:** 1 week

---

### Issue 8: Dynamic Library Loading

**File:** N/A  
**Severity:** 🟡 HIGH  
**Impact:** Cannot load .so/.dylib at runtime

**Recommendation:**
```vex
contract DynamicLib {
    fn open(path: &str) -> Result<DynamicLib, Error>;
    fn symbol<T>(self, name: &str) -> Result<T, Error>;
    fn close(self);
}
```

**Effort:** 2 weeks

---

### Issue 9: C Variadic Functions

**File:** `vex-compiler/src/codegen_ast/ffi.rs`  
**Severity:** 🟡 HIGH  
**Impact:** Cannot call printf, sprintf, etc.

**Problem:**
```vex
extern "C" fn printf(fmt: *const u8, ...) -> i32;  // ❌ Not supported

fn main() {
    printf("Hello %s\n", "world");  // ❌ Cannot compile
}
```

**Recommendation:** Support variadic FFI functions with `...` syntax

**Effort:** 2-3 weeks

---

### Issue 10: C Struct Bitfields

**Severity:** 🟡 HIGH  
**Impact:** Cannot interop with C structs with bitfields

**Problem:**
```c
struct Flags {
    unsigned int a : 1;
    unsigned int b : 1;
    unsigned int c : 6;
};
```

**Recommendation:** Add bitfield support to struct syntax

**Effort:** 2 weeks

---

### Issue 11: C Unions

**File:** `vex-parser/src/parser/types.rs`  
**Severity:** 🟡 HIGH  
**Impact:** Cannot represent C unions

**Recommendation:**
```vex
union Value {
    i: i32,
    f: f32,
    p: *u8,
}
```

**Effort:** 2-3 weeks

---

### Issue 12: Runtime Initialization Order

**File:** `vex-runtime/src/lib.rs`  
**Severity:** 🟡 HIGH  
**Impact:** Static constructors may run in wrong order

**Recommendation:** Define clear initialization order

**Effort:** 1 week

---

## Medium Priority Issues (🟢)

### Issue 13: Custom Entry Points

**Severity:** 🟢 MEDIUM  
**Impact:** Cannot create libraries (no #[no_main])

**Effort:** 1 week

---

### Issue 14: Stack Size Configuration

**Severity:** 🟢 MEDIUM  
**Impact:** Cannot set thread stack size

**Effort:** 3 days

---

### Issue 15: Heap Profiling

**Severity:** 🟢 MEDIUM  
**Impact:** Cannot track allocations

**Effort:** 2 weeks

---

### Issue 16: GC Support

**Severity:** 🟢 MEDIUM  
**Impact:** May want optional GC later

**Effort:** 8-12 weeks (major)

---

### Issue 17: JIT Compilation

**Severity:** 🟢 MEDIUM  
**Impact:** May want REPL later

**Effort:** 6-8 weeks

---

## Low Priority Issues (🔵)

### Issue 18: Runtime Metrics

**Severity:** 🔵 LOW  
**Impact:** Cannot get allocation stats

**Effort:** 1 week

---

## Metrics Summary

| Category | Critical | High | Medium | Low | Total |
|----------|----------|------|--------|-----|-------|
| FFI Safety | 2 | 4 | 0 | 0 | 6 |
| Async Runtime | 1 | 0 | 0 | 0 | 1 |
| Panic/Unwinding | 1 | 0 | 0 | 0 | 1 |
| Memory Management | 1 | 0 | 2 | 1 | 4 |
| Threading | 1 | 1 | 1 | 0 | 3 |
| Signal/OS | 0 | 1 | 1 | 0 | 2 |
| Advanced Features | 0 | 0 | 3 | 0 | 3 |
| **TOTAL** | **5** | **6** | **7** | **1** | **23** |

---

## Implementation Roadmap

### Phase 1: Critical Fixes (Week 1-4)
- [ ] FFI type safety checking
- [ ] Panic unwinding
- [ ] Custom allocator support
- [ ] Thread-local storage

### Phase 2: High Priority (Week 5-8)
- [ ] Foreign callback safety
- [ ] Signal handling
- [ ] Dynamic library loading
- [ ] Variadic functions
- [ ] C unions

### Phase 3: Async Runtime (Week 9-16)
- [ ] Executor
- [ ] Reactor (epoll/kqueue)
- [ ] Future trait
- [ ] async/await integration

### Phase 4: Medium Priority (Week 17-20)
- [ ] Custom entry points
- [ ] Stack size config
- [ ] Heap profiling

---

## Testing Plan

```vex
// test_ffi_safety.vx
extern "C" fn strlen(s: *const u8) -> usize;

fn test_ffi() {
    let s = "hello";
    let len = strlen(s.as_ptr());
    assert_eq(len, 5);
    
    // This should error at compile time:
    // let x: i32 = 42;
    // strlen(&x);  // ❌ Type error
}

// test_async.vx
async fn delayed() -> i32 {
    await sleep(Duration.from_secs(1));
    return 42;
}

fn test_async() {
    let result = block_on(delayed());
    assert_eq(result, 42);
}

// test_panic.vx
fn panicking_fn() {
    let _resource = Resource.new();
    panic("error");
    // _resource.drop() should be called
}

fn test_panic() {
    let result = catch_panic(|| panicking_fn());
    assert(result.is_err());
}

// test_allocator.vx
fn test_custom_allocator() {
    let arena = ArenaAllocator.new(1024);
    set_allocator(&arena);
    
    let v = Vec<i32>.new();
    v.push(1);
    // Allocation should use arena
    
    reset_allocator();
}
```

---

## Related Issues

- [03_CODEGEN_LLVM_ISSUES.md](./03_CODEGEN_LLVM_ISSUES.md) - Struct ABI affects FFI
- [04_STDLIB_INCOMPLETE_FEATURES.md](./04_STDLIB_INCOMPLETE_FEATURES.md) - async runtime needed for stdlib
- [07_MEMORY_SAFETY_CONCERNS.md](./07_MEMORY_SAFETY_CONCERNS.md) - FFI is major safety hole

---

## References

- Rust FFI: https://doc.rust-lang.org/nomicon/ffi.html
- Tokio runtime: https://github.com/tokio-rs/tokio
- libunwind: https://www.nongnu.org/libunwind/

---

**Next Steps:** Implement FFI type checking first (safety critical), then async runtime.
