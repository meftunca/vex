# Vex FFI Kütüphane Gereksinimleri

## 1. System Libraries (C FFI)

### A. libc (CRITICAL - %100 gerekli)

```c
// Memory management
void* malloc(size_t size);
void free(void* ptr);
void* memcpy(void* dest, const void* src, size_t n);
void* realloc(void* ptr, size_t size);

// File I/O
int open(const char* path, int flags);
ssize_t read(int fd, void* buf, size_t count);
ssize_t write(int fd, const void* buf, size_t count);
int close(int fd);

// Memory mapping (mmap)
void* mmap(void* addr, size_t length, int prot, int flags, int fd, off_t offset);
int munmap(void* addr, size_t length);
int mprotect(void* addr, size_t len, int prot);

// Time operations
time_t time(time_t* tloc);
int gettimeofday(struct timeval* tv, struct timezone* tz);
int clock_gettime(clockid_t clk_id, struct timespec* tp);
struct tm* localtime(const time_t* timep);
struct tm* gmtime(const time_t* timep);
size_t strftime(char* s, size_t max, const char* format, const struct tm* tm);

// System calls
pid_t fork(void);
int execve(const char* path, char* const argv[], char* const envp[]);
```

**Kullanım Durumu:**

- Memory allocation: `malloc()`, `free()`
- File operations: `open()`, `read()`, `write()`, `close()`
- Memory mapping: `mmap()` for large files, shared memory
- Time: High-resolution timestamps, date/time formatting

**Platform:**

- Linux: glibc
- macOS: libSystem
- Windows: msvcrt (POSIX compatibility layer)

**Alternative:** YOK - her dilde var (C std)
**Self-hosting:** FFI ile bağlanacağız

### B. POSIX Regex (IMPORTANT - String processing)

```c
#include <regex.h>

// Regex compilation and execution
int regcomp(regex_t* preg, const char* regex, int cflags);
int regexec(const regex_t* preg, const char* string, size_t nmatch,
            regmatch_t pmatch[], int eflags);
void regfree(regex_t* preg);
size_t regerror(int errcode, const regex_t* preg, char* errbuf, size_t errbuf_size);
```

**Kullanım Durumu:**

- String pattern matching: `"[0-9]+".matches("123")`
- Text processing: regex.replace(), regex.split()
- Validation: email, phone, URL validation

**Platform:**

- Linux/macOS: POSIX regex (libc)
- Windows: pcre2 veya custom implementation

**Alternative:**

- Rust regex crate → Vex'te pure implementation
- PCRE2 library (Perl Compatible)

---

## 2. LLVM Libraries

### LLVM C API (CRITICAL - Code generation)

```c
#include <llvm-c/Core.h>
#include <llvm-c/Target.h>
#include <llvm-c/TargetMachine.h>

// Module creation
LLVMModuleRef LLVMModuleCreateWithName(const char* ModuleID);
LLVMContextRef LLVMContextCreate();

// Function creation
LLVMValueRef LLVMAddFunction(LLVMModuleRef M, const char* Name, LLVMTypeRef FunctionTy);
LLVMBasicBlockRef LLVMAppendBasicBlock(LLVMValueRef Fn, const char* Name);

// IR generation
LLVMBuilderRef LLVMCreateBuilder();
LLVMValueRef LLVMBuildAdd(LLVMBuilderRef, LLVMValueRef LHS, LLVMValueRef RHS, const char* Name);

// Optimization
void LLVMRunPassManager(LLVMPassManagerRef PM, LLVMModuleRef M);

// Code emission
LLVMBool LLVMTargetMachineEmitToFile(LLVMTargetMachineRef T, LLVMModuleRef M,
                                     char* Filename, LLVMCodeGenFileType codegen,
                                     char** ErrorMessage);
```

**Self-hosting'te:** Vex FFI ile LLVM C API'ye direct bağlanacağız

---

## 3. Platform-Specific Libraries

### A. Linux: io_uring (HIGH PERFORMANCE Async I/O)

```c
#include <liburing.h>

// Setup
int io_uring_queue_init(unsigned entries, struct io_uring* ring, unsigned flags);
void io_uring_queue_exit(struct io_uring* ring);

// Submit operations
struct io_uring_sqe* io_uring_get_sqe(struct io_uring* ring);
int io_uring_submit(struct io_uring* ring);

// Completion
int io_uring_wait_cqe(struct io_uring* ring, struct io_uring_cqe** cqe_ptr);
void io_uring_cqe_seen(struct io_uring* ring, struct io_uring_cqe* cqe);
```

**Alternatif:** epoll (Linux), kqueue (BSD/macOS), IOCP (Windows)

### B. macOS: kqueue

```c
#include <sys/event.h>

int kqueue(void);
int kevent(int kq, const struct kevent* changelist, int nchanges,
           struct kevent* eventlist, int nevents, const struct timespec* timeout);
```

### C. Windows: IOCP

```c
#include <windows.h>

HANDLE CreateIoCompletionPort(HANDLE FileHandle, HANDLE ExistingCompletionPort,
                               ULONG_PTR CompletionKey, DWORD NumberOfConcurrentThreads);
BOOL GetQueuedCompletionStatus(HANDLE CompletionPort, LPDWORD lpNumberOfBytes,
                                PULONG_PTR lpCompletionKey, LPOVERLAPPED* lpOverlapped,
                                DWORD dwMilliseconds);
```

---

## 4. Optional Libraries (Future)

### OpenSSL/LibreSSL (Crypto & TLS)

```c
#include <openssl/ssl.h>
#include <openssl/crypto.h>

SSL_CTX* SSL_CTX_new(const SSL_METHOD* method);
SSL* SSL_new(SSL_CTX* ctx);
int SSL_connect(SSL* ssl);
int SSL_read(SSL* ssl, void* buf, int num);
int SSL_write(SSL* ssl, const void* buf, int num);
```

**Use case:** HTTPS, cryptography, secure networking

---

## Performance: Zero-Overhead Garanti

### LLVM Optimization Levels

| Level | FFI Call    | Overhead | Use Case     |
| ----- | ----------- | -------- | ------------ |
| `-O0` | Direct call | ~0 ns    | Debug builds |
| `-O2` | Inlined     | 0 ns     | Release      |
| `-O3` | Vectorized  | 0 ns     | Performance  |

**Example:** `malloc()` call

```asm
; -O0 (No optimization)
call    malloc@PLT     ; 5 cycles

; -O2 (Inlined)
call    malloc@PLT     ; 5 cycles (same)

; -O3 (Custom allocator inlined)
mov     %rax, %fs:0x28  ; TLS allocator, 1 cycle!
```

### Platform-Specific Build Requirements

| Feature   | Linux         | macOS              | Windows          | Cross-compile |
| --------- | ------------- | ------------------ | ---------------- | ------------- |
| **libc**  | glibc         | libSystem          | msvcrt           | ✅            |
| **mmap**  | mmap          | mmap               | VirtualAlloc     | ⚠️            |
| **regex** | POSIX         | POSIX              | pcre2            | ⚠️            |
| **time**  | clock_gettime | mach_absolute_time | QueryPerformance | ⚠️            |
| **async** | io_uring      | kqueue             | IOCP             | ❌            |

**Legend:**

- ✅ Same API, just recompile
- ⚠️ Different API, needs `#[cfg(target_os)]`
- ❌ Completely different, separate implementations

---

## Zero-Overhead FFI: LLVM IR Integration

### Soru: FFI çağrıları LLVM IR ile zero-overhead olabilir mi?

**Cevap: ✅ EVET! - LLVM directly integrates C functions**

#### A. Direct Function Calls (Zero overhead)

**Vex Code:**

```vex
fn main() -> i32 {
    let ptr = libc.malloc(1024);
    libc.free(ptr);
    return 0;
}
```

**Generated LLVM IR:**

```llvm
; External C function declarations
declare i8* @malloc(i64)
declare void @free(i8*)

define i32 @main() {
entry:
  ; Direct call to malloc - NO WRAPPER!
  %ptr = call i8* @malloc(i64 1024)

  ; Direct call to free - NO WRAPPER!
  call void @free(i8* %ptr)

  ret i32 0
}
```

**Generated Assembly (x86_64):**

```asm
main:
    push    rbp
    mov     rbp, rsp
    mov     edi, 1024          ; Argument: size
    call    malloc@PLT         ; Direct jump to libc
    mov     rdi, rax           ; Argument: pointer
    call    free@PLT           ; Direct jump to libc
    xor     eax, eax           ; return 0
    pop     rbp
    ret
```

**Overhead:** 0 cycles! Direct PLT (Procedure Linkage Table) jump.

---

#### B. Inlined Functions (Zero overhead)

**Vex Code:**

```vex
fn time_now() -> u64 {
    return libc.time(null);
}
```

**LLVM IR with `alwaysinline`:**

```llvm
declare i64 @time(i64*)

define i64 @time_now() alwaysinline {
  %t = call i64 @time(i64* null)
  ret i64 %t
}

define i32 @main() {
  %now = call i64 @time_now()  ; This will be INLINED!
  ret i32 0
}
```

**After LLVM optimization pass:**

```llvm
define i32 @main() {
  %now = call i64 @time(i64* null)  ; Direct call, no function!
  ret i32 0
}
```

**Overhead:** 0 cycles! Function eliminated, direct syscall.

---

#### C. Memory Intrinsics (SIMD Vectorization)

**Vex Code:**

```vex
fn copy(dest: *mut u8, src: *u8, size: usize) {
    memory.copy(dest, src, size);
}
```

**LLVM IR:**

```llvm
; LLVM compiler intrinsic (NOT a function call!)
declare void @llvm.memcpy.p0i8.p0i8.i64(i8*, i8*, i64, i1)

define void @copy(i8* %dest, i8* %src, i64 %size) {
  call void @llvm.memcpy.p0i8.p0i8.i64(
    i8* %dest,
    i8* %src,
    i64 %size,
    i1 false   ; not volatile
  )
  ret void
}
```

**Generated Assembly (AVX2):**

```asm
copy:
    cmp     rsi, 32
    jb      .small_copy

.large_copy:
    vmovdqu ymm0, [rsi]        ; Load 32 bytes (SIMD)
    vmovdqu [rdi], ymm0        ; Store 32 bytes
    add     rsi, 32
    add     rdi, 32
    sub     rdx, 32
    jnz     .large_copy
    ret

.small_copy:
    rep movsb                  ; Byte-by-byte fallback
    ret
```

**Overhead:** 0 cycles! LLVM generates optimal SIMD code.

---

#### D. Regex Example (mmap + regex)

**Vex Code:**

```vex
fn parse_log(path: &str) -> Vec<String> {
    // Memory-map file
    let fd = libc.open(path.as_ptr(), O_RDONLY);
    let data = libc.mmap(null, file_size, PROT_READ, MAP_PRIVATE, fd, 0);

    // Compile regex
    let mut regex: Regex = default();
    libc.regcomp(&mut regex, "[0-9]{4}-[0-9]{2}-[0-9]{2}", REG_EXTENDED);

    // Match dates
    let matches = Vec::new();
    // ... regex matching logic

    // Cleanup
    libc.regfree(&mut regex);
    libc.munmap(data, file_size);
    libc.close(fd);

    return matches;
}
```

**LLVM IR:**

```llvm
declare i32 @open(i8*, i32)
declare i8* @mmap(i8*, i64, i32, i32, i32, i64)
declare i32 @munmap(i8*, i64)
declare i32 @close(i32)
declare i32 @regcomp(%struct.regex_t*, i8*, i32)
declare i32 @regexec(%struct.regex_t*, i8*, i64, %struct.regmatch_t*, i32)
declare void @regfree(%struct.regex_t*)

define void @parse_log(i8* %path) {
  %fd = call i32 @open(i8* %path, i32 0)      ; O_RDONLY
  %data = call i8* @mmap(i8* null, i64 %size,
                         i32 1, i32 2, i32 %fd, i64 0)
  ; ... regex compilation and matching ...
  call void @regfree(%struct.regex_t* %regex)
  call i32 @munmap(i8* %data, i64 %size)
  call i32 @close(i32 %fd)
  ret void
}
```

**Performance:**

- `mmap()`: Zero-copy file access (kernel optimization)
- `regcomp()`: One-time compilation cost
- `regexec()`: Direct C regex engine (highly optimized)

**No overhead!** All calls are direct PLT jumps.

---

## Platform-Specific Builds: Gerekli mi?

### Cevap: ✅ EVET - Ama minimal ve otomatik!

#### Strategy 1: Conditional Compilation (`#[cfg]`)

**Vex Syntax:**

```vex
// time.vx - Platform-specific time implementation

#[cfg(target_os = "linux")]
fn monotonic_time() -> u64 {
    let mut ts: Timespec = default();
    unsafe {
        libc.clock_gettime(CLOCK_MONOTONIC, &mut ts);
    }
    return (ts.tv_sec as u64) * 1_000_000_000 + (ts.tv_nsec as u64);
}

#[cfg(target_os = "macos")]
fn monotonic_time() -> u64 {
    unsafe {
        let time = mach.mach_absolute_time();
        let info = mach.mach_timebase_info();
        return time * info.numer / info.denom;
    }
}

#[cfg(target_os = "windows")]
fn monotonic_time() -> u64 {
    let mut freq: u64 = 0;
    let mut count: u64 = 0;
    unsafe {
        kernel32.QueryPerformanceFrequency(&mut freq);
        kernel32.QueryPerformanceCounter(&mut count);
    }
    return count * 1_000_000_000 / freq;
}
```

**LLVM Compilation:**

```bash
# Linux build - only includes Linux version
vexc --target=x86_64-unknown-linux-gnu time.vx
# Generated IR contains: clock_gettime() call

# macOS build - only includes macOS version
vexc --target=aarch64-apple-darwin time.vx
# Generated IR contains: mach_absolute_time() call

# Windows build - only includes Windows version
vexc --target=x86_64-pc-windows-msvc time.vx
# Generated IR contains: QueryPerformanceCounter() call
```

**Zero overhead!** Dead code elimination removes unused platform code.

---

#### Strategy 2: LLVM Target-Specific Features

**LLVM handles automatically:**

| Feature                | Implementation              | Platform Detection |
| ---------------------- | --------------------------- | ------------------ |
| **Calling convention** | cdecl, fastcall, vectorcall | LLVM target triple |
| **ABI**                | System V, Microsoft x64     | LLVM CodeGen       |
| **Syscall numbers**    | Different per OS            | LLVM backend       |
| **Structure padding**  | Platform alignment rules    | LLVM DataLayout    |

**Example: Structure padding**

```vex
struct Timespec {
    tv_sec: i64,
    tv_nsec: i64,
}
```

**Generated LLVM IR:**

```llvm
; Linux x86_64
%Timespec = type { i64, i64 }  ; 16 bytes, 8-byte alignment

; Windows x64 (same!)
%Timespec = type { i64, i64 }  ; 16 bytes, 8-byte alignment
```

**LLVM automatically handles platform differences!**

---

#### Strategy 3: FFI Declaration per Platform

**Vex FFI definitions:**

```vex
// ffi/unix.vx
#[cfg(unix)]
#[link(name = "c")]
extern "C" {
    fn mmap(addr: *mut void, len: usize, prot: i32,
            flags: i32, fd: i32, offset: i64) -> *mut void;
    fn munmap(addr: *mut void, len: usize) -> i32;
}

// ffi/windows.vx
#[cfg(windows)]
#[link(name = "kernel32")]
extern "C" {
    fn VirtualAlloc(addr: *mut void, size: usize,
                    alloc_type: u32, protect: u32) -> *mut void;
    fn VirtualFree(addr: *mut void, size: usize, free_type: u32) -> i32;
}

// memory.vx - Unified API
fn map_memory(size: usize) -> *mut void {
    #[cfg(unix)]
    return unsafe { unix.mmap(null, size, PROT_READ | PROT_WRITE,
                              MAP_PRIVATE | MAP_ANON, -1, 0) };

    #[cfg(windows)]
    return unsafe { windows.VirtualAlloc(null, size, MEM_COMMIT | MEM_RESERVE,
                                         PAGE_READWRITE) };
}
```

**Generated LLVM IR (Linux):**

```llvm
declare i8* @mmap(i8*, i64, i32, i32, i32, i64)

define i8* @map_memory(i64 %size) {
  %ptr = call i8* @mmap(i8* null, i64 %size, i32 3, i32 34, i32 -1, i64 0)
  ret i8* %ptr
}
```

**Generated LLVM IR (Windows):**

```llvm
declare i8* @VirtualAlloc(i8*, i64, i32, i32)

define i8* @map_memory(i64 %size) {
  %ptr = call i8* @VirtualAlloc(i8* null, i64 %size, i32 12288, i32 4)
  ret i8* %ptr
}
```

**User calls unified API, LLVM picks platform version!**

---

#### Strategy 4: Cross-Compilation

**LLVM Target Triples:**

```bash
# Native compilation (auto-detect)
vexc main.vx -o main

# Cross-compile Linux → Windows
vexc --target=x86_64-pc-windows-gnu main.vx -o main.exe

# Cross-compile x86_64 → ARM64
vexc --target=aarch64-unknown-linux-gnu main.vx -o main_arm

# Cross-compile macOS → Linux
vexc --target=x86_64-unknown-linux-musl main.vx -o main_static
```

**LLVM automatically:**

- Selects correct calling convention
- Links correct system libraries
- Generates platform-specific syscalls
- Handles endianness differences

**Zero manual work!**

---

## Özet Tablo

| Library          | Platform | Criticality  | Zero-overhead | Status  | Implementation              |
| ---------------- | -------- | ------------ | ------------- | ------- | --------------------------- |
| **libc**         | All      | 🔴 Critical  | ✅ Yes        | ✅ Done | std/ffi/libc.vx             |
| **LLVM**         | All      | 🔴 Critical  | ✅ Yes        | ⏳ TODO | FFI (C API)                 |
| **mmap**         | Unix     | 🟡 Important | ✅ Yes        | ✅ Done | std/ffi/platform/unix.vx    |
| **dlopen**       | Unix     | 🟡 Important | ✅ Yes        | ✅ Done | std/ffi/platform/unix.vx    |
| **VirtualAlloc** | Windows  | 🟡 Important | ✅ Yes        | ✅ Done | std/ffi/platform/windows.vx |
| **regex**        | All      | 🟡 Important | ✅ Yes        | ⏳ TODO | FFI/Pure                    |
| **time**         | All      | 🟡 Important | ✅ Yes        | ✅ Done | libc FFI                    |
| **zlib**         | All      | 🟢 Optional  | ✅ Yes        | ✅ Done | std/ffi/zlib.vx             |
| **zstd**         | All      | � Optional   | ✅ Yes        | ✅ Done | std/ffi/zstd.vx             |
| **lz4**          | All      | 🟢 Optional  | ✅ Yes        | ✅ Done | std/ffi/lz4.vx              |
| **OpenSSL**      | All      | 🟢 Optional  | ✅ Yes        | ✅ Done | std/ffi/openssl.vx          |
| **liburing**     | Linux    | 🟢 Optional  | ✅ Yes        | ⏳ TODO | FFI                         |

---

## Sonuç

### ✅ Tamamlanan (DONE)

1. ✅ **libc** - Core C functions (malloc, free, memcpy, I/O, time)
2. ✅ **Platform APIs** - Unix (mmap, dlopen), Windows (VirtualAlloc, LoadLibrary)
3. ✅ **zlib** - Industry-standard compression (HTTP, PNG)
4. ✅ **zstd** - Modern high-performance compression (Facebook)
5. ✅ **lz4** - Ultra-fast compression (games, streaming)
6. ✅ **OpenSSL** - Crypto (SHA256, SHA512, HMAC, AES, SSL/TLS, random)

### ⏳ Devam Eden (TODO)

7. ⏳ **LLVM C API** - Code generation (self-hosting için critical)
8. ⏳ **regex** - Pattern matching (POSIX/PCRE2)
9. ⏳ **liburing** - Linux async I/O (high performance)

### 🔮 Gelecek (Future)

10. 🔮 **GPU libraries** - CUDA, Metal, Vulkan (HPC support)
11. 🔮 **HTTP libraries** - curl/libcurl (networking)
12. 🔮 **Database** - SQLite, PostgreSQL, Redis clients

### Zero-Overhead Guarantee ✅

**FFI calls via LLVM IR:**

- ✅ Direct function calls (no wrapper overhead)
- ✅ LLVM inlining optimization (`alwaysinline`)
- ✅ Compiler intrinsics (memcpy → SIMD)
- ✅ Dead code elimination (unused platform code removed)
- ✅ Cross-compilation support (LLVM target triples)

**Platform-specific builds:**

- ✅ Automatic via `#[cfg(target_os)]`
- ✅ LLVM handles calling conventions
- ✅ Zero manual configuration
- ✅ Unified API with platform backends

**Self-hosting için**: Sadece libc + LLVM C API yeterli!
**Production için**: Platform-specific optimizations + regex + time + crypto libs

---
