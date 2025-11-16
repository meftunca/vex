# Vex Builtin Functions (Rust-Implemented)

**Generated:** 2025-11-16  
**Source:** `vex-compiler/src/codegen_ast/builtins/mod.rs`

This document lists all builtin functions that are implemented in Rust and available in the Vex compiler. These functions are compiled directly into the binary and provide core functionality for memory management, intrinsics, reflection, async runtime, and standard library operations.

---

## 1. Core Builtins

Essential functions for basic program operation.

- `print` - Print to stdout
- `println` - Print to stdout with newline
- `panic` - Panic and abort execution
- `assert` - Runtime assertion check
- `unreachable` - Mark unreachable code paths

---

## 2. Memory Management

Low-level memory allocation and deallocation primitives.

- `alloc` - Allocate memory
- `free` - Free allocated memory
- `realloc` - Reallocate memory block
- `sizeof` - Get size of type in bytes
- `alignof` - Get alignment requirement of type

---

## 3. LLVM Intrinsics

### Bit Manipulation

High-performance bitwise operations using LLVM intrinsics.

- `ctlz` - Count leading zeros
- `cttz` - Count trailing zeros
- `ctpop` - Population count (count set bits)
- `bswap` - Byte swap (endianness conversion)
- `bitreverse` - Reverse bits in integer

### Overflow Checking

Safe arithmetic operations with overflow detection.

- `sadd_overflow` - Signed addition with overflow check
- `ssub_overflow` - Signed subtraction with overflow check
- `smul_overflow` - Signed multiplication with overflow check

---

## 4. Compiler Hints

Optimization hints for the compiler and LLVM backend.

- `assume` - Assume condition is true (optimizer hint)
- `likely` - Mark branch as likely to be taken
- `unlikely` - Mark branch as unlikely to be taken
- `prefetch` - Prefetch memory for cache optimization

---

## 5. String Functions

C-style string operations and Vex String utilities.

- `strlen` - Get C-string length
- `strcmp` - Compare C-strings
- `strcpy` - Copy C-string
- `strcat` - Concatenate C-strings
- `strdup` - Duplicate C-string
- `vex_string_as_cstr` - Convert Vex String to C-string
- `vex_string_len` - Get Vex String length

---

## 6. Formatting Functions

Type-to-string conversion utilities.

- `i32_to_string` - Convert i32 to string
- `f64_to_string` - Convert f64 to string
- `bool_to_string` - Convert bool to string

---

## 7. Memory Operations

Raw memory manipulation functions.

- `memcpy` - Copy memory block (non-overlapping)
- `memset` - Fill memory with constant byte
- `memcmp` - Compare memory blocks
- `memmove` - Copy memory block (overlapping-safe)

---

## 8. UTF-8 Functions

Unicode string validation and manipulation.

- `utf8_valid` - Validate UTF-8 string encoding
- `utf8_char_count` - Count UTF-8 characters (not bytes)
- `utf8_char_at` - Get UTF-8 character at index

---

## 9. Array Functions

Fixed-size array operations.

- `array_len` - Get array length
- `array_get` - Get array element at index
- `array_set` - Set array element at index
- `array_append` - Append element to array

---

## 10. Reflection Functions

Runtime type information and introspection.

- `typeof` - Get type name as string
- `type_id` - Get unique type identifier
- `type_size` - Get type size in bytes
- `type_align` - Get type alignment requirement
- `is_int_type` - Check if type is integer
- `is_float_type` - Check if type is floating-point
- `is_pointer_type` - Check if type is pointer
- `field_metadata` - Get struct field metadata

---

## 11. HashMap/Map Functions

Hash table operations for key-value storage.

- `hashmap_new` / `map_new` - Create new HashMap
- `hashmap_insert` / `map_insert` - Insert key-value pair
- `hashmap_get` / `map_get` - Get value by key
- `hashmap_len` / `map_len` - Get number of entries
- `hashmap_free` / `map_free` - Free HashMap memory
- `hashmap_contains` - Check if key exists
- `hashmap_remove` - Remove key-value pair
- `hashmap_clear` - Clear all entries

---

## 12. Slice Functions

Dynamic view into arrays and vectors.

- `slice_from_vec` - Create slice from Vec
- `slice_new` - Create new slice
- `slice_get` - Get element at index
- `slice_len` - Get slice length

---

## 13. Set Functions

Hash set operations for unique value storage.

- `set_new` - Create new Set
- `set_with_capacity` - Create Set with initial capacity
- `set_insert` - Insert element
- `set_contains` - Check if element exists
- `set_remove` - Remove element
- `set_len` - Get number of elements
- `set_clear` - Clear all elements

---

## 14. Builtin Type Constructors

### Vec<T> - Dynamic Array

- `vec_new` / `Vec.new` - Create empty Vec
- `vec_with_capacity` - Create Vec with initial capacity
- `vec_free` - Free Vec memory

### Box<T> - Heap Allocation

- `box_new` / `Box.new` - Allocate value on heap
- `box_free` - Free Box memory

### String - Dynamic String

- `string_new` / `String.new` - Create empty String
- `string_from` - Create String from C-string
- `string_free` - Free String memory

### Channel - Async Communication

- `channel_new` / `Channel.new` - Create new channel
- `Channel.send` - Send value to channel
- `Channel.recv` - Receive value from channel

### Option<T> - Optional Values

- `Some` - Create Some variant (has value)
- `None` - Create None variant (no value)

### Result<T, E> - Error Handling

- `Ok` - Create Ok variant (success)
- `Err` - Create Err variant (error)

### Primitive to String Conversions

Type-safe conversions from primitive types to String.

- `vex_i32_to_string` - Convert i32 to String
- `vex_i64_to_string` - Convert i64 to String
- `vex_u32_to_string` - Convert u32 to String
- `vex_u64_to_string` - Convert u64 to String
- `vex_f32_to_string` - Convert f32 to String
- `vex_f64_to_string` - Convert f64 to String
- `vex_bool_to_string` - Convert bool to String
- `vex_string_to_string` - Clone String to String

---

## 15. Async Runtime Functions

Asynchronous execution and concurrency primitives.

- `runtime_create` - Create async runtime instance
- `runtime_destroy` - Destroy async runtime
- `runtime_run` - Run async runtime event loop
- `runtime_shutdown` - Gracefully shutdown runtime
- `async_sleep` - Async sleep for duration
- `spawn_async` - Spawn async task

---

## 16. Stdlib Module Functions

Standard library modules with namespaced functions.

### Logger Module

Structured logging with severity levels.

- `logger::debug` - Debug-level log message
- `logger::info` - Info-level log message
- `logger::warn` - Warning-level log message
- `logger::error` - Error-level log message

### Time Module

Time and duration utilities.

- `time::now` - Get current timestamp (Unix epoch)
- `time::high_res` - Get high-resolution time counter
- `time::sleep_ms` - Sleep for milliseconds

### Testing Module

Unit testing assertions.

- `testing::assert` - Assert condition is true
- `testing::assert_eq` - Assert two values are equal
- `testing::assert_ne` - Assert two values are not equal

---

## Runtime Function Declarations

These are low-level C runtime functions declared by the compiler.

### Memory Runtime

- `vex_malloc(size: i64) -> *i8` - Allocate memory block
- `vex_free(ptr: *i8)` - Free memory block
- `vex_realloc(ptr: *i8, size: i64) -> *i8` - Reallocate memory block

### C Standard Library

- `abort()` - Abort program execution immediately

### Format Library

Type-safe formatting functions from vex-runtime.

- `vex_fmt_buffer_new(capacity: i64) -> *FormatBuffer` - Create format buffer
- `vex_fmt_buffer_free(buf: *FormatBuffer)` - Free format buffer
- `vex_fmt_buffer_append_str(buf: *FormatBuffer, str: *u8, len: i64)` - Append string
- `vex_fmt_buffer_to_string(buf: *FormatBuffer) -> *u8` - Convert to string
- `vex_fmt_i32(value: i32, spec: *FormatSpec) -> *u8` - Format i32
- `vex_fmt_i64(value: i64, spec: *FormatSpec) -> *u8` - Format i64
- `vex_fmt_u32(value: u32, spec: *FormatSpec) -> *u8` - Format u32
- `vex_fmt_u64(value: u64, spec: *FormatSpec) -> *u8` - Format u64
- `vex_fmt_f32(value: f32, spec: *FormatSpec) -> *u8` - Format f32
- `vex_fmt_f64(value: f64, spec: *FormatSpec) -> *u8` - Format f64
- `vex_fmt_bool(value: bool, spec: *FormatSpec) -> *u8` - Format bool
- `vex_fmt_string(str: *u8, len: i64, spec: *FormatSpec) -> *u8` - Format string

### LLVM Intrinsics

- `llvm.memset.p0.i64(ptr: *i8, val: i8, len: i64, volatile: i1)` - LLVM memset intrinsic

---

## Summary

**Total Categories:** 16  
**Total Builtin Functions:** ~140+  
**Implementation Language:** Rust  
**Location:** `vex-compiler/src/codegen_ast/builtins/`

All builtin functions are:

- **Zero-cost abstractions** - Compiled to efficient LLVM IR
- **Type-safe** - Checked at compile time
- **Memory-safe** - Integrated with borrow checker
- **Platform-optimized** - Using LLVM target-specific optimizations

For implementation details, see `vex-compiler/src/codegen_ast/builtins/mod.rs` and related submodules.
