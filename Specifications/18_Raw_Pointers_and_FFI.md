# Raw Pointers and FFI

**Version:** 0.9.2
**Last Updated:** November 2025

This document describes Vex's support for raw pointers and Foreign Function Interface (FFI) capabilities, including unsafe code blocks, extern declarations, and interoperability with C libraries.

---

## Table of Contents

1. [Overview](#overview)
2. [Raw Pointer Types](#raw-pointer-types)
3. [Unsafe Code Blocks](#unsafe-code-blocks)
4. [Foreign Function Interface](#foreign-function-interface)
5. [Memory Management](#memory-management)
6. [Best Practices](#best-practices)
7. [Examples](#examples)

---

## Overview

Vex provides low-level memory access and FFI capabilities through:

- **Raw pointers**: `*T` (mutable) and `*const T` (immutable)
- **Unsafe blocks**: `unsafe { ... }` for operations requiring special care
- **Extern declarations**: Interface with C and other languages
- **Memory intrinsics**: Direct memory operations

These features are isolated in `unsafe` blocks to maintain memory safety guarantees elsewhere in the codebase.

---

## Raw Pointer Types

Vex supports two types of raw pointers:

```vex
*T        // Mutable raw pointer
*const T  // Immutable raw pointer
```

### Basic Usage

```vex
// Mutable pointer
let mut value = 42;
let ptr: *i32 = &value as *i32;

// Immutable pointer
let const_ptr: *const i32 = &value as *const i32;

// Null pointer
let null_ptr: *i32 = 0 as *i32;
```

### Pointer Operations

All pointer operations must occur within `unsafe` blocks:

```vex
unsafe {
    // Dereference
    let value = *ptr;

    // Modify through pointer
    *ptr = 100;

    // Pointer arithmetic
    let next_ptr = ptr.offset(1);

    // Check null
    if !ptr.is_null() {
        *ptr = 42;
    }
}
```

### Common Patterns

```vex
// Array access through pointers
fn sum_array(arr: *const i32, len: usize): i32 {
    let mut sum = 0;
    unsafe {
        for i in 0..len {
            sum += *arr.offset(i as isize);
        }
    }
    sum
}

// String manipulation
fn strlen(s: *const u8): usize {
    unsafe {
        let mut len = 0;
        while *s.offset(len as isize) != 0 {
            len += 1;
        }
        len
    }
}
```

---

## Unsafe Code Blocks

Unsafe blocks allow operations that bypass Vex's safety guarantees:

```vex
unsafe {
    // Raw pointer operations
    let ptr = 0x1000 as *mut i32;
    *ptr = 42;

    // Call unsafe functions
    libc::memset(ptr as *mut u8, 0, 4);

    // Access union fields
    let value = my_union.unsafe_field;
}
```

### Unsafe Functions

Mark functions as unsafe to indicate they require special care:

```vex
unsafe fn dangerous_operation(ptr: *mut i32) {
    *ptr = 999;
}

fn safe_wrapper(value: &mut i32) {
    unsafe {
        dangerous_operation(value as *mut i32);
    }
}
```

### Safety Contracts

```vex
/// # Safety
/// - `ptr` must be valid for `len` elements
/// - Memory must not be accessed concurrently
/// - `ptr` must be properly aligned
unsafe fn process_buffer(ptr: *mut u8, len: usize) {
    // Implementation with safety assumptions
}
```

---

## Foreign Function Interface

Vex can interface with C and other languages using `extern` blocks:

### Basic Extern Declaration

```vex
extern "C" {
    fn printf(format: *const u8, ...): i32;
    fn malloc(size: usize): *mut u8;
    fn free(ptr: *mut u8);
}

fn main(): i32 {
    unsafe {
        let msg = "Hello, World!\n" as *const u8;
        printf(msg, 0);
    }
    0
}
```

### Function Pointers

```vex
extern "C" {
    fn qsort(
        base: *mut u8,
        nmemb: usize,
        size: usize,
        compar: fn(*const u8, *const u8): i32
    );
}

fn compare_ints(a: *const u8, b: *const u8): i32 {
    unsafe {
        let x = *(a as *const i32);
        let y = *(b as *const i32);
        if x < y { -1 } else if x > y { 1 } else { 0 }
    }
}

fn sort_array(arr: &mut [i32]) {
    unsafe {
        qsort(
            arr.as_mut_ptr() as *mut u8,
            arr.len(),
            4, // sizeof(i32)
            compare_ints
        );
    }
}
```

### Variadic Functions

```vex
extern "C" {
    fn sprintf(dest: *mut u8, format: *const u8, ...): i32;
}

fn format_number(dest: &mut [u8], value: i32): usize {
    unsafe {
        sprintf(
            dest.as_mut_ptr(),
            "%d\n" as *const u8,
            value
        ) as usize
    }
}
```

### Different ABIs

```vex
// C ABI (default)
extern "C" {
    fn c_function(): i32;
}

// System ABI
extern "system" {
    fn system_call(id: i32, ...): i32;
}
```

---

## Memory Management

### Manual Allocation

```vex
fn manual_allocation() {
    unsafe {
        // Allocate memory
        let ptr = malloc(100) as *mut i32;

        if ptr.is_null() {
            panic("Allocation failed");
        }

        // Use memory
        for i in 0..25 {
            *ptr.offset(i) = i * 2;
        }

        // Deallocate
        free(ptr as *mut u8);
    }
}
```

### Stack Allocation

```vex
fn stack_buffer() {
    // Fixed-size stack allocation
    let mut buffer: [u8; 1024] = [0; 1024];

    unsafe {
        // Get pointer to buffer
        let ptr = buffer.as_mut_ptr();

        // Fill buffer
        libc::memset(ptr, 65, 1024); // Fill with 'A'
    }

    // Buffer automatically cleaned up
}
```

### Memory Mapping

```vex
extern "C" {
    fn mmap(
        addr: *mut u8,
        len: usize,
        prot: i32,
        flags: i32,
        fd: i32,
        offset: i64
    ): *mut u8;
    fn munmap(addr: *mut u8, len: usize): i32;
}

const PROT_READ: i32 = 1;
const PROT_WRITE: i32 = 2;
const MAP_PRIVATE: i32 = 2;
const MAP_ANONYMOUS: i32 = 32;

fn allocate_huge_page(size: usize): *mut u8 {
    unsafe {
        mmap(
            0 as *mut u8,
            size,
            PROT_READ | PROT_WRITE,
            MAP_PRIVATE | MAP_ANONYMOUS,
            -1,
            0
        )
    }
}
```

---

## Best Practices

### 1. Minimize Unsafe Code

```vex
// Good: Isolate unsafe code
fn safe_abstraction(data: &mut [i32]) {
    unsafe {
        // Minimal unsafe operations
        libc::memset(data.as_mut_ptr() as *mut u8, 0, data.len() * 4);
    }
}

// Bad: Large unsafe blocks
fn bad_example(data: &mut [i32]) {
    unsafe {
        // Lots of unsafe code mixed with safe operations
        for i in 0..data.len() {
            *data.as_mut_ptr().offset(i as isize) = i as i32;
        }
        libc::qsort(data.as_mut_ptr() as *mut u8, data.len(), 4, compare);
        validate_data(data);
    }
}
```

### 2. Validate Pointers

```vex
fn safe_dereference(ptr: *const i32): Option<i32> {
    if ptr.is_null() {
        return None;
    }

    unsafe {
        Some(*ptr)
    }
}
```

### 3. Use Safe Abstractions

```vex
// Good: Provide safe interface over unsafe operations
struct SafeBuffer {
    ptr: *mut u8,
    len: usize,
}

impl SafeBuffer {
    fn new(size: usize): Option<SafeBuffer> {
        unsafe {
            let ptr = malloc(size);
            if ptr.is_null() {
                None
            } else {
                Some(SafeBuffer { ptr, len: size })
            }
        }
    }

    fn as_slice(&self): &[u8] {
        unsafe {
            std::slice::from_raw_parts(self.ptr, self.len)
        }
    }
}

impl Drop for SafeBuffer {
    fn drop(&mut self) {
        unsafe {
            free(self.ptr);
        }
    }
}
```

### 4. Document Safety Requirements

```vex
/// Reads exactly `len` bytes from `src` into `dest`.
///
/// # Safety
/// - `src` must be valid for `len` bytes
/// - `dest` must be valid for `len` bytes
/// - Memory regions must not overlap
/// - Both pointers must be properly aligned
unsafe fn memcpy(dest: *mut u8, src: *const u8, len: usize) {
    // Implementation
}
```

### 5. Avoid Common Pitfalls

```vex
// Bad: Use after free
fn use_after_free() {
    unsafe {
        let ptr = malloc(4) as *mut i32;
        *ptr = 42;
        free(ptr as *mut u8);
        let value = *ptr; // Undefined behavior!
    }
}

// Bad: Double free
fn double_free() {
    unsafe {
        let ptr = malloc(4);
        free(ptr);
        free(ptr); // Undefined behavior!
    }
}

// Bad: Buffer overflow
fn buffer_overflow() {
    let buffer: [i32; 10] = [0; 10];
    unsafe {
        for i in 0..20 { // Overflow!
            *buffer.as_ptr().offset(i) = i;
        }
    }
}
```

---

## Examples

### C Library Integration

```vex
extern "C" {
    fn fopen(filename: *const u8, mode: *const u8): *mut File;
    fn fread(ptr: *mut u8, size: usize, count: usize, stream: *mut File): usize;
    fn fclose(stream: *mut File): i32;
}

struct File;

fn read_file_contents(filename: String): Result<Vec<u8>, String> {
    unsafe {
        let file = fopen(filename.as_ptr(), "rb" as *const u8);
        if file.is_null() {
            return Err("Failed to open file".to_string());
        }

        let mut buffer = Vec::with_capacity(1024);
        buffer.resize(1024, 0);

        let bytes_read = fread(
            buffer.as_mut_ptr(),
            1,
            buffer.len(),
            file
        );

        fclose(file);

        buffer.truncate(bytes_read);
        Ok(buffer)
    }
}
```

### SIMD Operations

```vex
extern "C" {
    // SIMD intrinsics
    fn _mm_add_ps(a: __m128, b: __m128): __m128;
    fn _mm_load_ps(ptr: *const f32): __m128;
    fn _mm_store_ps(ptr: *mut f32, val: __m128);
}

type __m128 = [f32; 4]; // 128-bit SIMD register

fn vector_add(a: &[f32], b: &[f32], result: &mut [f32]) {
    unsafe {
        for i in (0..a.len()).step_by(4) {
            let va = _mm_load_ps(a.as_ptr().offset(i as isize));
            let vb = _mm_load_ps(b.as_ptr().offset(i as isize));
            let sum = _mm_add_ps(va, vb);
            _mm_store_ps(result.as_mut_ptr().offset(i as isize), sum);
        }
    }
}
```

### System Calls

```vex
extern "C" {
    fn syscall(number: i64, ...) -> i64;
}

const SYS_write: i64 = 1;
const SYS_exit: i64 = 60;

fn write_to_stdout(data: &[u8]) {
    unsafe {
        syscall(SYS_write, 1, data.as_ptr(), data.len());
    }
}

fn exit(code: i32) {
    unsafe {
        syscall(SYS_exit, code);
    }
}
```

---

**Previous**: [17_Error_Handling.md](./17_Error_Handling.md)  
**Next**: [19_Package_Manager.md](./19_Package_Manager.md)

**Maintained by**: Vex Language Team</content>
<parameter name="filePath">/Users/mapletechnologies/Desktop/big_projects/vex_lang/Specifications/18_Raw_Pointers_and_FFI.md
