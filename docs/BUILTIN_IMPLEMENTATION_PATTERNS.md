# Vex Builtin Implementation Patterns

**Last Updated:** November 9, 2025  
**Status:** Complete Architecture Documentation

---

## 📋 Overview

This document describes how Vex implements builtin types (Vec, Box, String, Map, Set, Range, Slice, Channel) using a **thin Rust wrapper pattern** around C runtime functions.

**Pattern:** Rust compiler code provides type-safe wrappers that call production-ready C runtime functions.

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Vex Source Code                          │
│               let v = Vec.new()                             │
│               v.push(42)                                     │
│               let len = v.len()                              │
└───────────────────────────┬─────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              Vex Parser (vex-parser/)                        │
│         Parses method calls: v.push(42)                      │
└───────────────────────────┬─────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│           Vex Compiler (vex-compiler/)                       │
│                                                              │
│  1. Check: Is "v" a builtin type? (Vec, Box, String, etc.)  │
│  2. Dispatch to specialized builtin method compiler          │
│  3. Generate LLVM IR for C runtime function call             │
│  4. Type safety: Cast void* pointers to concrete types       │
└───────────────────────────┬─────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              C Runtime (vex-runtime/)                        │
│          vex_vec_push(vec_ptr, value_ptr)                    │
│          vex_vec_len(vec_ptr) -> i64                         │
│          vex_string_len(string_ptr) -> size_t                │
│          vex_map_insert(map_ptr, key_ptr, val_ptr) -> bool   │
│                                                              │
│   ✅ Production-ready: 13,258 lines, 240+ functions          │
└─────────────────────────────────────────────────────────────┘
```

---

## 📂 File Organization

### Compiler Builtin Method Files

| File                             | Types                 | Methods Implemented                                              |
| -------------------------------- | --------------------- | ---------------------------------------------------------------- |
| `builtins/mod.rs`                | Dispatcher            | `try_compile_builtin_method()` - Routes to specialized compilers |
| `builtins/vec_box.rs`            | Vec, Box              | `push`, `len`, `get`, `set`, `as_slice`, `unwrap`                |
| `builtins/string_collections.rs` | String, Map, Set      | `len`, `is_empty`, `char_count`, `insert`, `get`, `contains`     |
| `builtins/ranges_arrays.rs`      | Range, Slice, Channel | `next`, `len`, `get`, `send`, `recv`                             |
| `builtins/builtin_contracts.rs`  | Contracts             | `Display`, `Clone`, `Debug`, `Eq` implementations                |

**Location:** `vex-compiler/src/codegen_ast/expressions/calls/builtins/`

### C Runtime Files

| File            | Functions             | Purpose                                       |
| --------------- | --------------------- | --------------------------------------------- |
| `vex_runtime.c` | 240+ functions        | Core runtime (strings, vectors, maps, memory) |
| `vex_runtime.h` | Function declarations | C API interface                               |

**Location:** `vex-runtime/src/`

---

## 🔧 Implementation Pattern: Vec Methods

### Example: `Vec.push(value)`

#### 1. Vex Source Code

```vex
let v = Vec.new()
v.push(42)
```

#### 2. Rust Compiler Code

**File:** `vex-compiler/src/codegen_ast/expressions/calls/builtins/vec_box.rs`

```rust
pub(super) fn compile_vec_method(
    &mut self,
    var_name: &str,
    method: &str,
    args: &[Expression],
) -> Result<Option<BasicValueEnum<'ctx>>, String> {
    match method {
        "push" => {
            // 1. Validate arguments
            if args.len() != 1 {
                return Err("Vec.push() requires exactly 1 argument".to_string());
            }

            // 2. Get Vec pointer from variable
            let vec_ptr_alloca = *self.variables.get(var_name)
                .ok_or_else(|| format!("Vec variable {} not found", var_name))?;

            let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
            let vec_ptr = self.builder
                .build_load(ptr_type, vec_ptr_alloca, "vec_ptr_load")?;

            // 3. Compile argument expression
            let value = self.compile_expression(&args[0])?;

            // 4. Allocate stack space for value (C expects pointer)
            let value_ptr = self.builder
                .build_alloca(value.get_type(), "vec_push_value")?;
            self.builder.build_store(value_ptr, value)?;

            // 5. Declare C runtime function: vex_vec_push(void* vec, void* value)
            let void_fn_type = self.context.void_type()
                .fn_type(&[ptr_type.into(), ptr_type.into()], false);
            let vex_vec_push_fn = self.module
                .add_function("vex_vec_push", void_fn_type, None);

            // 6. Call C runtime function
            self.builder.build_call(
                vex_vec_push_fn,
                &[vec_ptr.into(), value_ptr.into()],
                ""
            )?;

            // 7. Return dummy value (void)
            Ok(Some(self.context.i8_type().const_zero().into()))
        }
        // ... other methods
    }
}
```

#### 3. C Runtime Implementation

**File:** `vex-runtime/src/vex_runtime.c`

```c
void vex_vec_push(vex_vec_t* vec, void* value) {
    if (!vec || !value) return;

    // Check capacity, resize if needed
    if (vec->len >= vec->capacity) {
        size_t new_capacity = vec->capacity == 0 ? 8 : vec->capacity * 2;
        void* new_data = realloc(vec->data, new_capacity * vec->elem_size);
        if (!new_data) return;  // OOM
        vec->data = new_data;
        vec->capacity = new_capacity;
    }

    // Copy value into vec->data
    memcpy((char*)vec->data + vec->len * vec->elem_size, value, vec->elem_size);
    vec->len++;
}
```

#### 4. Generated LLVM IR (Simplified)

```llvm
; Load Vec pointer
%vec_ptr_load = load ptr, ptr %v, align 8

; Allocate stack space for value
%vec_push_value = alloca i32, align 4
store i32 42, ptr %vec_push_value, align 4

; Call C runtime function
call void @vex_vec_push(ptr %vec_ptr_load, ptr %vec_push_value)
```

---

## 🔍 Pattern Breakdown

### Step-by-Step Builtin Method Compilation

1. **Dispatch Check** (`builtins/mod.rs`)

   - Check if receiver type is a builtin type
   - Check if user-defined struct shadows builtin type name
   - Route to specialized compiler (vec_box, string_collections, ranges_arrays)

2. **Type Validation** (Specialized compiler)

   - Verify argument count and types
   - Get variable from symbol table
   - Load pointer value from stack/heap

3. **Argument Compilation**

   - Compile each argument expression
   - Allocate stack space for value arguments (C expects pointers)
   - Store values into allocated memory

4. **C Function Declaration**

   - Declare C runtime function with correct signature
   - Use opaque pointer types (`ptr` in LLVM)
   - Map Vex types to C types (i64, bool, void\*)

5. **Function Call**

   - Build LLVM `call` instruction
   - Pass pointers to C runtime
   - Handle return values (void, i64, bool, ptr)

6. **Type Safety**
   - Cast `void*` return pointers to concrete types
   - Validate pointer alignment
   - Return typed LLVM values to caller

---

## 📚 Builtin Type Reference

### Vec<T>

| Method        | Signature        | C Runtime Function                | Return Type           |
| ------------- | ---------------- | --------------------------------- | --------------------- |
| `new()`       | `() -> Vec<T>`   | `vex_vec_new(elem_size)`          | `Vec<T>`              |
| `push(value)` | `(T) -> void`    | `vex_vec_push(vec, &value)`       | `void`                |
| `len()`       | `() -> i64`      | `vex_vec_len(vec)`                | `i64`                 |
| `get(index)`  | `(i64) -> T`     | `vex_vec_get(vec, index)`         | `void*` (cast to `T`) |
| `as_slice()`  | `() -> Slice<T>` | `vex_slice_from_vec(&slice, vec)` | `Slice<T>` (sret)     |

**Implementation:** `builtins/vec_box.rs`

### String

| Method         | Signature        | C Runtime Function            | Return Type         |
| -------------- | ---------------- | ----------------------------- | ------------------- |
| `len()`        | `() -> i64`      | `vex_string_len(str)`         | `i64` (byte length) |
| `is_empty()`   | `() -> bool`     | `vex_string_is_empty(str)`    | `bool`              |
| `char_count()` | `() -> i64`      | `vex_string_char_count(str)`  | `i64` (UTF-8 chars) |
| `push_str(s)`  | `(&str) -> void` | `vex_string_push_str(str, s)` | `void`              |

**Implementation:** `builtins/string_collections.rs`

### Map<K, V>

| Method         | Signature        | C Runtime Function            | Return Type            |
| -------------- | ---------------- | ----------------------------- | ---------------------- |
| `insert(k, v)` | `(K, V) -> bool` | `vex_map_insert(map, &k, &v)` | `bool`                 |
| `get(k)`       | `(K) -> V?`      | `vex_map_get(map, &k)`        | `void*` (cast to `V*`) |
| `len()`        | `() -> i64`      | `vex_map_len(map)`            | `i64`                  |

**Implementation:** `builtins/string_collections.rs`

### Set<T>

| Method            | Signature     | C Runtime Function              | Return Type |
| ----------------- | ------------- | ------------------------------- | ----------- |
| `insert(value)`   | `(T) -> bool` | `vex_set_insert(set, &value)`   | `bool`      |
| `contains(value)` | `(T) -> bool` | `vex_set_contains(set, &value)` | `bool`      |
| `len()`           | `() -> i64`   | `vex_set_len(set)`              | `i64`       |

**Implementation:** `builtins/string_collections.rs`

### Range / RangeInclusive

| Method       | Signature      | C Runtime Function            | Return Type |
| ------------ | -------------- | ----------------------------- | ----------- |
| `next(&out)` | `(&T) -> bool` | `vex_range_next(range, &out)` | `bool`      |
| `len()`      | `() -> i64`    | `vex_range_len(range)`        | `i64`       |

**Implementation:** `builtins/ranges_arrays.rs`

### Slice<T>

| Method       | Signature    | C Runtime Function             | Return Type           |
| ------------ | ------------ | ------------------------------ | --------------------- |
| `len()`      | `() -> i64`  | `vex_slice_len(&slice)`        | `i64`                 |
| `get(index)` | `(i64) -> T` | `vex_slice_get(&slice, index)` | `void*` (cast to `T`) |

**Implementation:** `builtins/ranges_arrays.rs`

---

## 🎯 Key Patterns

### Pattern 1: Pointer Arguments

**C runtime expects pointers for all arguments**

```rust
// ❌ Wrong: Pass value directly
call_c_fn(&[value.into()]);

// ✅ Correct: Allocate + store + pass pointer
let value_ptr = self.builder.build_alloca(value.get_type(), "arg")?;
self.builder.build_store(value_ptr, value)?;
call_c_fn(&[value_ptr.into()]);
```

### Pattern 2: Return Value Casting

**C returns `void*`, Rust casts to concrete type**

```rust
// C function: void* vex_vec_get(vex_vec_t* vec, int64_t index)

let result_ptr = self.builder.build_call(get_fn, &[vec_ptr, index], "vec_get")?
    .try_as_basic_value().left()?;

// Cast void* to i32* (assuming Vec<i32>)
let i32_ptr = self.builder.build_pointer_cast(
    result_ptr.into_pointer_value(),
    self.context.i32_type().ptr_type(AddressSpace::default()),
    "cast_to_i32_ptr"
)?;

// Load i32 value
let value = self.builder.build_load(self.context.i32_type(), i32_ptr, "load_value")?;
```

### Pattern 3: SRET (Struct Return)

**For large structs, C writes to caller-allocated memory**

```rust
// C function: void vex_slice_from_vec(vex_slice_t* out_slice, vex_vec_t* vec)

// Allocate Slice struct on stack
let slice_alloca = self.builder.build_alloca(slice_struct_type, "slice_ret")?;

// Call C function with sret attribute
let sret_fn = self.declare_sret_fn(
    "vex_slice_from_vec",
    slice_struct_type,  // Return type
    &[ptr_type.into()]  // Arg types
);

self.builder.build_call(sret_fn, &[slice_alloca.into(), vec_ptr.into()], "")?;

// Slice is now in slice_alloca memory
```

---

## 🚀 How to Add a New Builtin Method

### Example: Adding `Vec.pop() -> Option<T>`

#### Step 1: Add C Runtime Function

**File:** `vex-runtime/src/vex_runtime.c`

```c
bool vex_vec_pop(vex_vec_t* vec, void* out_value) {
    if (!vec || vec->len == 0) return false;

    vec->len--;
    memcpy(out_value, (char*)vec->data + vec->len * vec->elem_size, vec->elem_size);
    return true;
}
```

**File:** `vex-runtime/src/vex_runtime.h`

```c
bool vex_vec_pop(vex_vec_t* vec, void* out_value);
```

#### Step 2: Add Rust Compiler Wrapper

**File:** `vex-compiler/src/codegen_ast/expressions/calls/builtins/vec_box.rs`

```rust
pub(super) fn compile_vec_method(
    &mut self,
    var_name: &str,
    method: &str,
    args: &[Expression],
) -> Result<Option<BasicValueEnum<'ctx>>, String> {
    match method {
        // ... existing methods ...

        "pop" => {
            // Vec.pop(&out) -> bool
            if args.len() != 1 {
                return Err("Vec.pop() requires 1 argument (output pointer)".to_string());
            }

            // Get Vec pointer
            let vec_ptr_alloca = *self.variables.get(var_name)?;
            let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
            let vec_ptr = self.builder.build_load(ptr_type, vec_ptr_alloca, "vec_ptr")?;

            // Compile output pointer argument
            let out_ptr = self.compile_expression(&args[0])?;

            // Declare C function: bool vex_vec_pop(void* vec, void* out)
            let pop_fn = self.declare_runtime_fn(
                "vex_vec_pop",
                &[ptr_type.into(), ptr_type.into()],
                self.context.bool_type().into(),
            );

            // Call C runtime
            let call_site = self.builder.build_call(
                pop_fn,
                &[vec_ptr.into(), out_ptr.into()],
                "vec_pop"
            )?;

            // Return bool (true if popped, false if empty)
            Ok(call_site.try_as_basic_value().left())
        }
    }
}
```

#### Step 3: Write Tests

**File:** `stdlib-tests/test_vec_pop.vx`

```vex
contract Test {
    fn test_vec_pop_some() {
        let v = Vec.new()
        v.push(10)
        v.push(20)

        let out: i32
        let success = v.pop(&out)

        assert(success == true)
        assert(out == 20)
        assert(v.len() == 1)
    }

    fn test_vec_pop_empty() {
        let v = Vec.new()
        let out: i32
        let success = v.pop(&out)

        assert(success == false)
        assert(v.len() == 0)
    }
}
```

---

## 🔍 Dispatch Logic

### Builtin Type Check

**File:** `builtins/mod.rs`

```rust
pub fn try_compile_builtin_method(
    &mut self,
    receiver: &str,
    method_name: &str,
    args: &[Expression],
) -> Result<Option<BasicValueEnum<'ctx>>, String> {
    // 1. Check if receiver is a builtin type
    let receiver_type = self.variables.get(receiver)
        .and_then(|ptr| self.get_variable_type(ptr));

    // 2. Check for user-defined struct shadowing
    if self.struct_types.contains_key(receiver_type) {
        return Ok(None);  // User struct takes precedence
    }

    // 3. Dispatch to specialized compiler
    match receiver_type {
        Some("Vec") => self.compile_vec_method(receiver, method_name, args),
        Some("Box") => self.compile_box_method(receiver, method_name, args),
        Some("String") => self.compile_string_method(receiver, method_name, args),
        Some("Map") => self.compile_map_method(receiver, method_name, args),
        Some("Set") => self.compile_set_method(receiver, method_name, args),
        Some("Range") => self.compile_range_method(receiver, method_name, args, false),
        Some("RangeInclusive") => self.compile_range_method(receiver, method_name, args, true),
        Some("Slice") => self.compile_slice_method(receiver, method_name, args),
        Some("Channel") => self.compile_channel_method(receiver, method_name, args),
        _ => Ok(None),  // Not a builtin type
    }
}
```

---

## 📊 Comparison: Current vs. Needed

### ✅ Already Implemented (Builtin)

| Type           | Methods                                     | Status        |
| -------------- | ------------------------------------------- | ------------- |
| **Vec<T>**     | `new`, `push`, `len`, `get`, `as_slice`     | ✅ Production |
| **Box<T>**     | `new`, `get`, `set`, `unwrap`               | ✅ Production |
| **String**     | `len`, `is_empty`, `char_count`, `push_str` | ✅ Production |
| **Map<K,V>**   | `insert`, `get`, `len`                      | ✅ Production |
| **Set<T>**     | `insert`, `contains`, `len`                 | ✅ Production |
| **Range**      | `next`, `len`                               | ✅ Production |
| **Slice<T>**   | `len`, `get`                                | ✅ Production |
| **Channel<T>** | `send`, `recv`                              | ✅ Production |

### 🟡 Needs Vex Stdlib Wrapper

These C functions exist but lack Vex API:

| Category       | C Functions                                                  | Needed Vex API                                         |
| -------------- | ------------------------------------------------------------ | ------------------------------------------------------ |
| **String**     | `vex_string_concat`, `vex_string_substr`, `vex_string_split` | `String.concat()`, `String.substr()`, `String.split()` |
| **Vec**        | `vex_vec_pop`, `vex_vec_clear`, `vex_vec_resize`             | `Vec.pop()`, `Vec.clear()`, `Vec.resize()`             |
| **Map**        | `vex_map_remove`, `vex_map_contains_key`, `vex_map_clear`    | `Map.remove()`, `Map.contains_key()`, `Map.clear()`    |
| **Iterators**  | None                                                         | `Iterator` trait, `for x in vec {}` syntax             |
| **Formatting** | `vex_format_int`, `vex_format_float`                         | `fmt.printf()`, `Display` trait                        |

---

## 🎓 Key Takeaways

1. **Rust Wrapper Pattern**: All builtin types use thin Rust wrappers around C runtime
2. **C Runtime is Complete**: 240+ functions, production-ready, well-tested
3. **Type Safety**: Rust compiler handles void\* → T casting
4. **Pointer Arguments**: C expects pointers, Rust allocates stack + passes pointer
5. **SRET for Structs**: Large return values use caller-allocated memory
6. **User Shadowing**: User-defined structs take precedence over builtins
7. **Dispatch Hierarchy**: mod.rs → specialized file → C runtime

---

## 🚀 Next Steps for Stdlib Development

Based on this architecture:

1. **Phase 1: String API** (Use existing `vex_string_*` functions)

   - Add `String.concat()`, `String.substr()`, `String.split()` wrappers
   - Follow `string_collections.rs` pattern

2. **Phase 2: Vec Extensions** (Use existing `vex_vec_*` functions)

   - Add `Vec.pop()`, `Vec.clear()`, `Vec.resize()` wrappers
   - Follow `vec_box.rs` pattern

3. **Phase 3: Iterator Trait** (Requires new C runtime support)

   - Design `Iterator` contract in Vex
   - Implement C runtime iterator protocol
   - Add compiler support for `for x in vec {}` syntax

4. **Phase 4: Formatting** (Use existing `vex_format_*` functions)
   - Add `fmt.printf()` wrapper
   - Implement `Display` contract
   - Follow `builtin_contracts.rs` pattern

**Pattern:** Always check C runtime first → Add Rust wrapper → Write Vex tests → Document API

---

_This architecture enables rapid stdlib development by reusing battle-tested C runtime._
