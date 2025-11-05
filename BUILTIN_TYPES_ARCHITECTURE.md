# Builtin Types Architecture - Core Language Integration

**Date:** November 5, 2025  
**Version:** Vex v0.2.0  
**Status:** Planning → Implementation  
**Total Types:** 17 builtin types (10 Tier 0 + 4 Tier 1 + 3 Tier 2)  
**Timeline:** 19-27 days (4-5.5 weeks)

---

## 📊 Executive Summary

This document defines the complete builtin types system for Vex, expanding from the original 10 types to **17 comprehensive builtin types** organized in 3 tiers:

### **Tier 0: Core Types (10 types)** - Critical foundation

`Option<T>`, `Result<T,E>`, `Vec<T>`, `Slice<T>`, `String`, `str`, `Range`, `RangeInclusive`, `Box<T>`, `Tuple<T, U, ...>`

**Why critical:** Enable null safety, error handling, dynamic arrays, iteration, heap allocation, recursive types, and multi-value returns. Required for basic language functionality.

### **Tier 1: Collections & Concurrency (4 types)** - Essential features

`Map<K,V>`, `Set<T>`, `Channel<T>`, `Iterator<T>`

**Why essential:** Hash-based collections, message passing, and iteration protocol. Required for practical applications.

### **Tier 2: Advanced Types (3 types)** - FFI & safety

`Array<T, N>`, `Never (!)`, `RawPtr<T>`

**Why advanced:** Fixed-size arrays, diverging functions, and raw pointers for FFI. Required for low-level operations and C interop.

### **Key Design Decisions:**

1. **Zero imports** - All 17 types available everywhere without `import std.*`
2. **Zero cost** - Monomorphized generics, inlined methods, compile-time optimization
3. **Compiler integration** - Special syntax support (`.`, `?`, `[]`, `..`, `for-in`)
4. **Borrow checker aware** - Ownership semantics enforced (moves, borrows, lifetimes)
5. **C runtime** - Type-erased, zero-copy implementations in vex-runtime/c/
6. **LLVM codegen** - Type-safe, monomorphized code generation

### **Implementation Strategy:**

- **Phase 0-1** (5-7 days): Foundation - all Tier 0 core types
- **Phase 2-3** (4-6 days): Ergonomics - pattern matching, methods, operators
- **Phase 4-6** (7-11 days): Advanced - Tier 1 collections + Tier 2 FFI
- **Phase 7** (2-3 days): Polish - optimization, benchmarks, docs

**Total: 19-27 days** to complete all 17 builtin types with full integration.

---

## 🎯 Philosophy: Builtin vs Library

### ✅ **Builtin Types** (Language-level, no imports)

These types are **fundamental to the language** and available everywhere:

#### **Tier 0: Core Types** (Critical - syntax support required)

- `Option<T>` - Nullable types (replaces null)
- `Result<T, E>` - Error handling (replaces exceptions)
- `Vec<T>` - Dynamic arrays (growable)
- `Slice<T>` - Borrowed views (&[T], &[T]!)
- `String` - UTF-8 strings (owned)
- `str` - String slices (borrowed)
- `Range` - Exclusive range (0..10) for iteration
- `RangeInclusive` - Inclusive range (0..=10) for iteration
- `Box<T>` - Heap allocation, enables recursive types
- `Tuple<T, U, ...>` - Multi-value grouping (T, U, V)

#### **Tier 1: Collections & Concurrency**

- `Map<K, V>` - Hash tables (SwissTable implementation)
- `Set<T>` - Hash sets (wrapper over Map<T, ()>)
- `Channel<T>` - Message passing (mpsc)
- `Iterator<T>` - Iteration protocol (trait + for-in syntax)

#### **Tier 2: Advanced Types**

- `Array<T, N>` - Fixed-size stack arrays [T; N]
- `Never (!)` - Diverging functions (never returns)
- `RawPtr<T>` - Raw pointers (*T, *T!) for FFI

**Total: 17 builtin types**

**Why builtin?**

1. **Zero imports**: Available everywhere without `import std.collections`
2. **Compiler integration**: Special syntax support (e.g., `arr[i]` for Vec, `?` for Option, `0..10` for Range)
3. **Borrow checker aware**: Compiler understands ownership semantics (Box moves, Range borrows)
4. **Zero-cost**: Monomorphized generics, inlined methods, compile-time Range optimization
5. **Ergonomic**: Pattern matching, operators, automatic derefs, for-in loops

### 📚 **Standard Library** (Explicit imports)

These are **convenience types** built on builtins:

- `TreeMap<K, V>` - Sorted map (B-Tree based)
- `TreeSet<T>` - Sorted set
- `LinkedList<T>` - Doubly-linked list
- `Deque<T>` - Double-ended queue
- `Arc<T>`, `Rc<T>` - Reference counting
- `Mutex<T>`, `RwLock<T>` - Thread synchronization

**Why library?**

- Less commonly used
- Can be implemented in Vex using builtins
- No special compiler support needed

---

## 📐 Architecture Design

### Layer 1: Compiler Integration (vex-ast, vex-parser)

**Location:** `vex-ast/src/lib.rs`

```rust
// Add to Type enum
pub enum Type {
    // ... existing types ...

    /// Tier 0: Core builtin types (no imports needed)
    Option(Box<Type>),                    // Option<T>
    Result(Box<Type>, Box<Type>),         // Result<T, E>
    Vec(Box<Type>),                       // Vec<T>
    Slice(Box<Type>, bool),               // &[T] or &[T]! (already exists!)
    String,                               // String (already exists!)
    Range(Box<Type>),                     // Range (0..10) - exclusive end
    RangeInclusive(Box<Type>),            // RangeInclusive (0..=10) - inclusive end
    Box(Box<Type>),                       // Box<T> - heap allocation
    Tuple(Vec<Type>),                     // (T, U, V) - multi-value

    /// Tier 1: Collections & concurrency
    Map(Box<Type>, Box<Type>),            // Map<K, V> (hash-based)
    Set(Box<Type>),                       // Set<T> (hash-based)
    Channel(Box<Type>),                   // Channel<T>
    Iterator(Box<Type>),                  // Iterator<T> (trait-based)

    /// Tier 2: Advanced types
    Array(Box<Type>, usize),              // [T; N] - fixed-size stack array
    Never,                                // ! - diverging function type
    RawPtr(Box<Type>, bool),              // *T or *T! - raw pointers for FFI
}
```

**Parser Support:**

```rust
// vex-parser/src/parser/types.rs

fn parse_generic_builtin(&mut self) -> Result<Type, ParseError> {
    match self.current.kind {
        // Tier 0: Core types
        TokenKind::Ident("Option") => {
            self.expect(TokenKind::Lt)?;
            let inner = self.parse_type()?;
            self.expect(TokenKind::Gt)?;
            Ok(Type::Option(Box::new(inner)))
        }
        TokenKind::Ident("Result") => { /* Result<T, E> */ }
        TokenKind::Ident("Vec") => { /* Vec<T> */ }
        TokenKind::Ident("Box") => { /* Box<T> */ }
        TokenKind::Ident("Range") => { /* Range (from expr syntax) */ }
        TokenKind::Ident("RangeInclusive") => { /* RangeInclusive */ }

        // Tier 1: Collections & concurrency
        TokenKind::Ident("Map") => { /* Map<K, V> */ }
        TokenKind::Ident("Set") => { /* Set<T> */ }
        TokenKind::Ident("Channel") => { /* Channel<T> */ }
        TokenKind::Ident("Iterator") => { /* Iterator<T> */ }

        // Tier 2: Advanced types
        TokenKind::Bang if prev_is_fn_return => Ok(Type::Never),  // fn(): !
        TokenKind::Star => self.parse_raw_pointer(),  // *T or *T!
        TokenKind::LBracket => self.parse_array_or_slice(),  // [T; N] or [T]

        _ => Err(ParseError::ExpectedBuiltinType)
    }
}

// Special syntax handling
fn parse_range_expr(&mut self) -> Result<Expr, ParseError> {
    // 0..10 → Range { start: 0, end: 10 }
    // 0..=10 → RangeInclusive { start: 0, end: 10 }
}
```

---

### Layer 2: C Runtime (vex-runtime/c/)

**Zero-copy, type-erased implementations for all 17 builtin types:**

#### **Tier 0: Core Types**

##### `vex-runtime/c/vex_vec.c`

```c
// Generic vector (type-erased with elem_size)
typedef struct {
    void *data;
    size_t len;
    size_t capacity;
    size_t elem_size;
} vex_vec_t;

vex_vec_t vex_vec_new(size_t elem_size);
void vex_vec_push(vex_vec_t *vec, const void *elem);
void *vex_vec_get(vex_vec_t *vec, size_t index);  // Returns pointer (zero-copy)
void vex_vec_pop(vex_vec_t *vec, void *out);
void vex_vec_reserve(vex_vec_t *vec, size_t additional);
void vex_vec_free(vex_vec_t *vec);

// Growth strategy: 2x when capacity reached
static void vex_vec_grow(vex_vec_t *vec) {
    size_t new_cap = vec->capacity == 0 ? 8 : vec->capacity * 2;
    void *new_data = realloc(vec->data, new_cap * vec->elem_size);
    if (!new_data) abort();
    vec->data = new_data;
    vec->capacity = new_cap;
}
```

#### `vex-runtime/c/vex_option.c`

```c
// Option is a struct with tag + value
// Layout: { u8 tag, T value } (optimized by compiler)
// No runtime overhead - compiler can optimize away tag for non-null pointers

// Helper for compiler codegen (not called by user code)
void *vex_option_unwrap(void *opt_ptr, size_t type_size, const char *file, int line) {
    uint8_t tag = *(uint8_t*)opt_ptr;
    if (tag == 0) {  // None
        fprintf(stderr, "Unwrap failed at %s:%d - Option is None\n", file, line);
        abort();
    }
    return (uint8_t*)opt_ptr + 1;  // Skip tag byte
}
```

#### `vex-runtime/c/vex_result.c`

```c
// Result is similar to Option but with error value
// Layout: { u8 tag, union { T ok, E err } }

void *vex_result_unwrap(void *result_ptr, size_t type_size, const char *file, int line) {
    uint8_t tag = *(uint8_t*)result_ptr;
    if (tag == 0) {  // Err
        fprintf(stderr, "Unwrap failed at %s:%d - Result is Err\n", file, line);
        abort();
    }
    return (uint8_t*)result_ptr + 1;
}
```

#### `vex-runtime/c/vex_hashmap.c`

```c
// Already implemented in vex_swisstable.c!
// Google SwissTable - SIMD-optimized, zero-copy

// Just need generic wrapper:
typedef struct {
    void *table;  // Opaque SwissTable pointer
    size_t key_size;
    size_t val_size;
} vex_hashmap_t;

vex_hashmap_t vex_hashmap_new(size_t key_size, size_t val_size);
void vex_hashmap_insert(vex_hashmap_t *map, const void *key, const void *val);
void *vex_hashmap_get(vex_hashmap_t *map, const void *key);  // Returns pointer or NULL
bool vex_hashmap_remove(vex_hashmap_t *map, const void *key);
void vex_hashmap_free(vex_hashmap_t *map);
```

#### `vex-runtime/c/vex_map.c` (renamed from hashmap)

```c
// Generic hash map (wrapper over SwissTable)
typedef struct {
    void *table;  // Opaque SwissTable pointer
    size_t key_size;
    size_t val_size;
} vex_map_t;

vex_map_t vex_map_new(size_t key_size, size_t val_size);
void vex_map_insert(vex_map_t *map, const void *key, const void *val);
void *vex_map_get(vex_map_t *map, const void *key);  // Returns pointer or NULL
bool vex_map_remove(vex_map_t *map, const void *key);
size_t vex_map_len(vex_map_t *map);
void vex_map_free(vex_map_t *map);
```

#### `vex-runtime/c/vex_set.c`

```c
// Generic hash set (wrapper over Map<T, ()>)
typedef struct {
    vex_map_t inner;  // Map<T, ()>
} vex_set_t;

vex_set_t vex_set_new(size_t elem_size);
void vex_set_insert(vex_set_t *set, const void *elem);
bool vex_set_contains(vex_set_t *set, const void *elem);
bool vex_set_remove(vex_set_t *set, const void *elem);
size_t vex_set_len(vex_set_t *set);
void vex_set_free(vex_set_t *set);
```

##### `vex-runtime/c/vex_range.c` **(NEW)**

```c
// Range iterator (0..10) - exclusive end
typedef struct {
    int64_t start;
    int64_t end;
    int64_t current;
} vex_range_t;

vex_range_t vex_range_new(int64_t start, int64_t end);
bool vex_range_next(vex_range_t *range, int64_t *out);
size_t vex_range_len(vex_range_t *range);  // end - start

// RangeInclusive (0..=10) - inclusive end
typedef struct {
    int64_t start;
    int64_t end;
    int64_t current;
    bool exhausted;  // For end == i64::MAX case
} vex_range_inclusive_t;

vex_range_inclusive_t vex_range_inclusive_new(int64_t start, int64_t end);
bool vex_range_inclusive_next(vex_range_inclusive_t *range, int64_t *out);
size_t vex_range_inclusive_len(vex_range_inclusive_t *range);  // end - start + 1
```

##### `vex-runtime/c/vex_box.c` **(NEW)**

```c
// Box<T> - heap-allocated value with ownership
typedef struct {
    void *ptr;
    size_t size;
} vex_box_t;

vex_box_t vex_box_new(const void *value, size_t size);
void *vex_box_get(vex_box_t *box);  // Borrow pointer
void *vex_box_into_inner(vex_box_t box);  // Move out (caller owns)
void vex_box_free(vex_box_t box);

// Example usage:
// Box<Node> node = Box.new(Node { value: 42, next: None });
// → vex_box_t box = vex_box_new(&node_value, sizeof(Node));
```

##### `vex-runtime/c/vex_tuple.c` **(NEW)**

```c
// Tuple is compile-time only - no runtime struct!
// Layout: { T0, T1, T2, ... } (struct with sequential fields)
// Compiler generates struct layout at compile time
// No runtime functions needed (just struct ops)

// Example:
// (i32, String, bool) → struct { i32 _0; vex_string_t _1; bool _2; }
```

##### `vex-runtime/c/vex_array.c` **(ENHANCED)**

```c
// Fixed-size stack array [T; N]
// Already have vex_array.c, but enhance with:

// Bounds checking (debug mode)
void vex_array_check_bounds(size_t index, size_t len, const char *file, int line);

// Stack allocation (compiler handles, no runtime needed)
// [i32; 5] → i32 arr[5] (native C array)

// SIMD operations for numeric arrays
void vex_array_fill_i32(int32_t *arr, size_t len, int32_t value);
void vex_array_copy_i32(int32_t *dst, const int32_t *src, size_t len);
```

#### **Tier 1: Collections & Concurrency**

##### `vex-runtime/c/vex_map.c` (already shown above)

##### `vex-runtime/c/vex_set.c` (already shown above)

##### `vex-runtime/c/vex_channel.c` **(EXISTING - enhance)**

```c
// Channel<T> - message passing (already exists)
// Enhance with:
typedef struct {
    void *buffer;
    size_t capacity;
    size_t elem_size;
    size_t head;
    size_t tail;
    pthread_mutex_t lock;
    pthread_cond_t not_empty;
    pthread_cond_t not_full;
    bool closed;
} vex_channel_t;

vex_channel_t vex_channel_new(size_t elem_size, size_t capacity);
bool vex_channel_send(vex_channel_t *ch, const void *elem);  // Blocks if full
bool vex_channel_recv(vex_channel_t *ch, void *elem);        // Blocks if empty
bool vex_channel_try_send(vex_channel_t *ch, const void *elem);  // Non-blocking
bool vex_channel_try_recv(vex_channel_t *ch, void *elem);        // Non-blocking
void vex_channel_close(vex_channel_t *ch);
void vex_channel_free(vex_channel_t *ch);
```

##### `vex-runtime/c/vex_iterator.c`

```c
// Generic iterator state
typedef struct {
    void *collection;     // Pointer to Vec, Map, Set, etc.
    size_t index;         // Current position
    size_t elem_size;     // Element size
    void *(*next)(void*); // Function pointer to next element
    void (*free)(void*);  // Cleanup function
} vex_iterator_t;

// Iterator for Vec<T>
vex_iterator_t vex_vec_iter(vex_vec_t *vec);
void *vex_vec_iter_next(vex_iterator_t *iter);

// Iterator for Map<K,V> - returns (K, V) pairs
vex_iterator_t vex_map_iter(vex_map_t *map);
void *vex_map_iter_next(vex_iterator_t *iter);

// Iterator for Set<T>
vex_iterator_t vex_set_iter(vex_set_t *set);
void *vex_set_iter_next(vex_iterator_t *iter);

// Iterator for Range/RangeInclusive
vex_iterator_t vex_range_iter(vex_range_t *range);
vex_iterator_t vex_range_inclusive_iter(vex_range_inclusive_t *range);

// Common iterator operations
bool vex_iterator_has_next(vex_iterator_t *iter);
void vex_iterator_free(vex_iterator_t *iter);
```

#### **Tier 2: Advanced Types**

##### `vex-runtime/c/vex_never.c` **(NEW)**

```c
// Never type (!) - compile-time only
// No runtime representation needed
// Functions returning ! never return (abort, exit, infinite loop)

// Helper for unreachable code paths
_Noreturn void vex_unreachable(const char *msg, const char *file, int line) {
    fprintf(stderr, "Unreachable code reached at %s:%d: %s\n", file, line, msg);
    abort();
}

// Example usage:
// fn panic(msg: String): ! {
//     vex_unreachable(msg.data, __FILE__, __LINE__);
// }
```

##### `vex-runtime/c/vex_raw_ptr.c` **(NEW)**

```c
// Raw pointers (*T, *T!) - minimal runtime support
// Mostly compile-time, but helpers for safety checks

// Null check (debug mode only)
void *vex_raw_ptr_check_null(void *ptr, const char *file, int line) {
    if (ptr == NULL) {
        fprintf(stderr, "Null pointer dereference at %s:%d\n", file, line);
        abort();
    }
    return ptr;
}

// Pointer arithmetic (unsafe)
void *vex_raw_ptr_offset(void *ptr, ptrdiff_t offset, size_t elem_size) {
    return (uint8_t*)ptr + (offset * elem_size);
}

// Cast helpers (for FFI)
void *vex_raw_ptr_cast(void *ptr) {
    return ptr;  // No-op, but tracks casts for debugging
}
```

---

### Layer 3: LLVM Codegen (vex-compiler/src/codegen_ast/)

**Type-safe, monomorphized implementations:**

#### `vex-compiler/src/codegen_ast/builtin_types/mod.rs` (NEW)

```rust
// New module for builtin type codegen
pub mod option;
pub mod result;
pub mod vec;
pub mod hashmap;
pub mod channel;
```

#### `vex-compiler/src/codegen_ast/builtin_types/option.rs`

```rust
use super::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile Option<T> as struct { u8 tag, T value }
    pub fn compile_option_type(&mut self, inner_ty: &Type) -> StructType<'ctx> {
        let inner_llvm = self.compile_type(inner_ty)?;

        // Struct layout: { i8 tag, T value }
        let tag_ty = self.context.i8_type();
        let struct_ty = self.context.struct_type(&[tag_ty.into(), inner_llvm], false);

        struct_ty
    }

    /// Compile Option::Some(value) constructor
    pub fn compile_option_some(&mut self, value: BasicValueEnum<'ctx>, inner_ty: &Type) -> BasicValueEnum<'ctx> {
        let struct_ty = self.compile_option_type(inner_ty);

        // Create { 1, value }
        let mut opt = struct_ty.get_undef();
        let tag = self.context.i8_type().const_int(1, false);
        opt = self.builder.build_insert_value(opt, tag, 0, "tag")?;
        opt = self.builder.build_insert_value(opt, value, 1, "value")?;

        opt.into()
    }

    /// Compile Option::None constructor
    pub fn compile_option_none(&mut self, inner_ty: &Type) -> BasicValueEnum<'ctx> {
        let struct_ty = self.compile_option_type(inner_ty);

        // Create { 0, undef }
        let mut opt = struct_ty.get_undef();
        let tag = self.context.i8_type().const_int(0, false);
        opt = self.builder.build_insert_value(opt, tag, 0, "tag")?;

        opt.into()
    }

    /// Compile Option::unwrap() - with panic on None
    pub fn compile_option_unwrap(&mut self, opt_val: BasicValueEnum<'ctx>, inner_ty: &Type) -> BasicValueEnum<'ctx> {
        let tag = self.builder.build_extract_value(opt_val.into_struct_value(), 0, "tag")?;
        let value = self.builder.build_extract_value(opt_val.into_struct_value(), 1, "value")?;

        // if tag == 0 { panic("unwrap on None") }
        let is_none = self.builder.build_int_compare(
            IntPredicate::EQ,
            tag.into_int_value(),
            self.context.i8_type().const_int(0, false),
            "is_none"
        )?;

        let then_block = self.context.append_basic_block(self.current_function.unwrap(), "unwrap_panic");
        let cont_block = self.context.append_basic_block(self.current_function.unwrap(), "unwrap_cont");

        self.builder.build_conditional_branch(is_none, then_block, cont_block)?;

        // Panic path
        self.builder.position_at_end(then_block);
        self.builtin_panic(&[/* error message */])?;
        self.builder.build_unreachable()?;

        // Continue path
        self.builder.position_at_end(cont_block);
        value
    }
}
```

#### `vex-compiler/src/codegen_ast/builtin_types/vec.rs`

```rust
impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile Vec<T> as struct { *T, len, capacity }
    pub fn compile_vec_type(&mut self, elem_ty: &Type) -> StructType<'ctx> {
        let elem_llvm = self.compile_type(elem_ty)?;
        let ptr_ty = elem_llvm.ptr_type(AddressSpace::default());
        let size_ty = self.context.i64_type();

        // Struct: { *T, usize, usize }
        self.context.struct_type(&[
            ptr_ty.into(),
            size_ty.into(),
            size_ty.into(),
        ], false)
    }

    /// Compile Vec::new()
    pub fn compile_vec_new(&mut self, elem_ty: &Type) -> BasicValueEnum<'ctx> {
        let elem_llvm = self.compile_type(elem_ty)?;
        let elem_size = elem_llvm.size_of().unwrap();

        // Call vex_vec_new(elem_size)
        let vex_vec_new = self.get_or_declare_runtime_fn(
            "vex_vec_new",
            self.context.struct_type(&[/* ... */], false),
            &[self.context.i64_type().into()],
        );

        self.builder.build_call(vex_vec_new, &[elem_size.into()], "vec")?
            .try_as_basic_value()
            .left()
            .unwrap()
    }

    /// Compile Vec::push(&mut self, value)
    pub fn compile_vec_push(&mut self, vec_ptr: PointerValue<'ctx>, value: BasicValueEnum<'ctx>, elem_ty: &Type) {
        // Call vex_vec_push(vec_ptr, &value)
        let vex_vec_push = self.get_or_declare_runtime_fn(
            "vex_vec_push",
            self.context.void_type().into(),
            &[
                vec_ptr.get_type().into(),
                self.context.i8_type().ptr_type(AddressSpace::default()).into(),
            ],
        );

        // Store value to stack, pass pointer
        let value_ptr = self.builder.build_alloca(value.get_type(), "value_tmp")?;
        self.builder.build_store(value_ptr, value)?;
        let value_ptr_i8 = self.builder.build_pointer_cast(
            value_ptr,
            self.context.i8_type().ptr_type(AddressSpace::default()),
            "value_ptr_cast"
        )?;

        self.builder.build_call(vex_vec_push, &[vec_ptr.into(), value_ptr_i8.into()], "")?;
    }

    /// Compile Vec::get(&self, index) -> Option<&T>
    pub fn compile_vec_get(&mut self, vec_ptr: PointerValue<'ctx>, index: IntValue<'ctx>, elem_ty: &Type) -> BasicValueEnum<'ctx> {
        // Call vex_vec_get(vec_ptr, index) -> *T or NULL
        let vex_vec_get = self.get_or_declare_runtime_fn(
            "vex_vec_get",
            self.compile_type(elem_ty).ptr_type(AddressSpace::default()).into(),
            &[vec_ptr.get_type().into(), self.context.i64_type().into()],
        );

        let ptr_or_null = self.builder.build_call(vex_vec_get, &[vec_ptr.into(), index.into()], "elem_ptr")?
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();

        // Convert NULL to None, non-NULL to Some(&T)
        let is_null = self.builder.build_is_null(ptr_or_null, "is_null")?;

        // If NULL -> None, else Some(ptr)
        // Return Option<&T>
        // ...
    }
}
```

---

### Layer 4: Syntax Sugar (vex-parser)

**Make builtins feel native:**

```vex
// Option
let x: Option<i32> = Some(42);
let y: Option<i32> = None;

// Pattern matching
match x {
    Some(val) => println(val),
    None => println("empty"),
}

// ? operator (syntax sugar for early return)
fn get_config(): Result<Config, Error> {
    let file = open_file(path)?;  // Unwrap or return Err
    let data = read_file(file)?;
    Ok(parse_config(data)?)
}

// Vec
let! numbers = Vec.new();
numbers.push(1);
numbers.push(2);
let first = numbers[0];  // Indexing (with bounds check)

// Map (not HashMap - hashing is default!)
let! scores = Map.new();
scores.insert("Alice", 100);
scores.insert("Bob", 85);

if let Some(score) = scores.get("Alice") {
    println("Alice: ", score);
}

// Set
let! tags = Set.new();
tags.insert("rust");
tags.insert("vex");
tags.insert("llvm");

if tags.contains("rust") {
    println("Found Rust!");
}

// Iterator & for-in loop
let numbers = Vec.from([1, 2, 3, 4, 5]);
for num in numbers {
    println(num);
}

// Iterator methods (chaining)
let doubled = numbers.iter()
    .map(|x| x * 2)
    .filter(|x| x > 5)
    .collect();

// Manual iteration
let! iter = numbers.iter();
while let Some(num) = iter.next() {
    println(num);
}

// Channel
let chan = Channel.new(10);  // Capacity 10
spawn(|| {
    chan.send(42);
});
let val = chan.recv();  // Blocks until value available
```

---

## 🔄 Memory Management Strategy

### Ownership & Borrow Checker Integration

```vex
// Vec owns its data
let vec = Vec.new();  // vec is owner
vec.push(42);
// vec dropped here -> calls vex_vec_free()

// Borrowing
let vec = Vec::new();
vec.push(1);
let slice: &[i32] = vec.as_slice();  // Borrow as slice
println(slice[0]);  // OK
// vec still owns data

// Moving
let vec1 = Vec::new();
let vec2 = vec1;  // Move (vec1 invalid)
// vec1.push(1);  // ERROR: use after move
```

### Drop Trait (RAII)

```rust
// vex-compiler/src/codegen_ast/builtins/drop.rs

impl<'ctx> ASTCodeGen<'ctx> {
    /// Generate drop glue for builtin types
    pub fn generate_drop_impl(&mut self, ty: &Type) {
        match ty {
            Type::Vec(elem_ty) => {
                // 1. Drop all elements
                // 2. Call vex_vec_free()
            }
            Type::HashMap(k, v) => {
                // 1. Drop all keys and values
                // 2. Call vex_hashmap_free()
            }
            Type::String => {
                // Call vex_free(str.ptr)
            }
            _ => {}
        }
    }
}
```

**Generated drop code:**

```llvm
; Vec<i32> drop glue
define void @__drop_Vec_i32(%Vec.i32* %vec) {
    %len_ptr = getelementptr inbounds %Vec.i32, %Vec.i32* %vec, i32 0, i32 1
    %len = load i64, i64* %len_ptr

    ; Loop to drop each element (if T has Drop)
    ; ...

    ; Free memory
    %data_ptr = getelementptr inbounds %Vec.i32, %Vec.i32* %vec, i32 0, i32 0
    %data = load i32*, i32** %data_ptr
    %data_i8 = bitcast i32* %data to i8*
    call void @vex_free(i8* %data_i8)
    ret void
}
```

---

## 📊 Implementation Phases (Revised for 17 Types)

### Phase 0: Tier 0 Core Types - Part A (3-4 days)

**Goal:** Option, Result, Vec, Box, Tuple basics working

1. **C Runtime** (1-2 days)

   - [ ] `vex_vec.c` - Generic vector
   - [ ] `vex_option.c` - Option helpers
   - [ ] `vex_result.c` - Result helpers
   - [ ] `vex_box.c` - Heap allocation **NEW**
   - [ ] `vex_tuple.c` - Document compile-time only **NEW**
   - [ ] Tests in C

2. **AST & Parser** (1 day)

   - [ ] Add `Type::Option`, `Type::Result`, `Type::Vec` to AST
   - [ ] Add `Type::Box`, `Type::Tuple` **NEW**
   - [ ] Parse `Option<T>`, `Result<T,E>`, `Vec<T>` syntax
   - [ ] Parse `Box<T>`, `(T, U, V)` syntax **NEW**
   - [ ] Parse `Some()`, `None`, `Ok()`, `Err()` constructors
   - [ ] Parse `Box.new()` constructor **NEW**

3. **Codegen** (1-2 days)

   - [ ] `builtin_types/option.rs` - Option codegen
   - [ ] `builtin_types/result.rs` - Result codegen
   - [ ] `builtin_types/vec.rs` - Vec codegen
   - [ ] `builtin_types/box.rs` - Box codegen **NEW**
   - [ ] `builtin_types/tuple.rs` - Tuple codegen **NEW**
   - [ ] Drop trait integration (Vec, Box)

4. **Tests** (half day)
   - [ ] `examples/10_builtins/option_basic.vx`
   - [ ] `examples/10_builtins/result_basic.vx`
   - [ ] `examples/10_builtins/vec_basic.vx`
   - [ ] `examples/10_builtins/box_basic.vx` **NEW**
   - [ ] `examples/10_builtins/tuple_basic.vx` **NEW**

### Phase 1: Tier 0 Core Types - Part B (2-3 days)

**Goal:** Range, RangeInclusive, Array, String/str working

1. **C Runtime** (1 day)

   - [ ] `vex_range.c` - Range iterator **NEW**
   - [ ] `vex_array.c` - Enhance existing with bounds checking **NEW**
   - [ ] String/str already exist, verify integration

2. **AST & Parser** (1 day)

   - [ ] Add `Type::Range`, `Type::RangeInclusive`, `Type::Array` **NEW**
   - [ ] Parse `0..10`, `0..=10` expressions **NEW**
   - [ ] Parse `[T; N]` array syntax **NEW**
   - [ ] Parse array literals `[1, 2, 3, 4, 5]`

3. **Codegen** (1 day)

   - [ ] `builtin_types/range.rs` - Range codegen **NEW**
   - [ ] `builtin_types/array.rs` - Fixed-size array **NEW**
   - [ ] Verify String/str codegen

4. **Tests** (half day)
   - [ ] `examples/10_builtins/range_basic.vx` **NEW**
   - [ ] `examples/10_builtins/array_basic.vx` **NEW**
   - [ ] `examples/10_builtins/string_basic.vx`

### Phase 2: Pattern Matching (2-3 days)

**Goal:** Match on Option/Result

1. **Parser** (1 day)

   - [ ] Pattern matching on enum variants
   - [ ] Support `Some(x)`, `None`, `Ok(x)`, `Err(e)` patterns

2. **Codegen** (1-2 days)

   - [ ] Compile match arms with tag checking
   - [ ] Extract values from variants

3. **Tests** (half day)
   - [ ] `examples/10_builtins/option_match.vx`
   - [ ] `examples/10_builtins/result_match.vx`

### Phase 3: Methods & Operators (2-3 days)

**Goal:** Ergonomic API

1. **Method syntax** (1 day)

   - [ ] `opt.is_some()`, `opt.unwrap()`, `opt.unwrap_or(default)`
   - [ ] `vec.push()`, `vec.pop()`, `vec.len()`
   - [ ] `res.is_ok()`, `res.expect(msg)`

2. **Operator overloading** (1 day)

   - [ ] `vec[i]` indexing
   - [ ] `?` operator for Result/Option

3. **Tests** (1 day)
   - [ ] `examples/10_builtins/option_methods.vx`
   - [ ] `examples/10_builtins/vec_indexing.vx`
   - [ ] `examples/10_builtins/result_question.vx`

### Phase 4: Tier 1 Collections (Map, Set, Iterator) (3-4 days)

**Goal:** Hash collections + iteration protocol

1. **C Runtime** (1 day)

   - [ ] `vex_map.c` - Wrapper over SwissTable (already exists as vex_swisstable.c)
   - [ ] `vex_set.c` - Wrapper over Map<T, ()>
   - [ ] `vex_iterator.c` - Iterator state management
   - [ ] Enhance for Range iteration **NEW**

2. **AST & Parser** (half day)

   - [ ] Add `Type::Map`, `Type::Set`, `Type::Iterator`
   - [ ] Parse `Map<K, V>`, `Set<T>`, `Iterator<T>`
   - [ ] Parse `for x in collection` syntax

3. **Codegen** (1-2 days)

   - [ ] `builtin_types/map.rs` - Map<K,V> codegen
   - [ ] `builtin_types/set.rs` - Set<T> codegen
   - [ ] `builtin_types/iterator.rs` - Iterator trait implementation
   - [ ] For-in loop desugaring to iterator

4. **Tests** (1 day)
   - [ ] `examples/10_builtins/map.vx`
   - [ ] `examples/10_builtins/set.vx`
   - [ ] `examples/10_builtins/iterator.vx`
   - [ ] `examples/10_builtins/for_in_loop.vx`
   - [ ] `examples/10_builtins/range_iteration.vx` **NEW**

### Phase 5: Tier 1 Concurrency (Channel) (3-4 days)

**Goal:** Message passing with channels

1. **C Runtime** (2 days)

   - [ ] `vex_channel.c` - Enhanced lock-free MPSC (already partially exists)
   - [ ] Tests for thread safety
   - [ ] Blocking and non-blocking operations

2. **AST & Parser** (half day)

   - [ ] Add `Type::Channel`
   - [ ] Parse `Channel<T>` syntax

3. **Codegen** (1 day)

   - [ ] `builtin_types/channel.rs`
   - [ ] Thread spawning integration

4. **Tests** (1 day)
   - [ ] `examples/10_builtins/channel.vx`
   - [ ] Multi-threaded test

### Phase 6: Tier 2 Advanced Types (2-3 days)

**Goal:** Never type, raw pointers for FFI

1. **C Runtime** (half day)

   - [ ] `vex_never.c` - Unreachable helper **NEW**
   - [ ] `vex_raw_ptr.c` - Pointer safety helpers **NEW**

2. **AST & Parser** (1 day)

   - [ ] Add `Type::Never` for `!` **NEW**
   - [ ] Add `Type::RawPtr` for `*T` and `*T!` **NEW**
   - [ ] Parse diverging function signatures: `fn panic(): !` **NEW**
   - [ ] Parse raw pointer casts: `x as *i32` **NEW**

3. **Codegen** (1 day)

   - [ ] `builtin_types/never.rs` - Never type codegen **NEW**
   - [ ] `builtin_types/raw_ptr.rs` - Raw pointer operations **NEW**
   - [ ] Unsafe context enforcement

4. **Tests** (half day)
   - [ ] `examples/10_builtins/never_type.vx` **NEW**
   - [ ] `examples/10_builtins/raw_pointers.vx` **NEW**
   - [ ] `examples/10_builtins/ffi_interop.vx` **NEW**

### Phase 7: Optimization & Polish (2-3 days)

**Goal:** Zero-cost guarantees

1. **Inlining** (1 day)

   - [ ] Mark small methods as `alwaysinline`
   - [ ] Inline Vec::push, Option::is_some, etc.

2. **Monomorphization** (1 day)

   - [ ] Generate specialized code per type
   - [ ] Vec<i32> ≠ Vec<String>

3. **Benchmarks** (1 day)
   - [ ] Compare to Rust equivalent
   - [ ] Verify <5% overhead

---

## 🎯 Zero-Cost Verification

### Benchmark Suite

```vex
// examples/benchmarks/vec_push.vx
fn bench_vec_push(): i64 {
    let! vec = Vec::new();
    let start = time_now();
    for i in 0..1000000 {
        vec.push(i);
    }
    let end = time_now();
    return end - start;
}

// Compare to Rust:
// fn bench_vec_push() -> i64 {
//     let mut vec = Vec::new();
//     let start = Instant::now();
//     for i in 0..1000000 {
//         vec.push(i);
//     }
//     start.elapsed().as_nanos() as i64
// }
```

**Target:**

- Vec::push: <5ns per operation
- HashMap::insert: <20ns per operation
- Option::unwrap: 0ns (optimized away)
- Match on Option: 0ns (branch prediction)

---

## 📋 Implementation Checklist (17 Builtin Types)

### C Runtime (13 new/enhanced files)

**Tier 0: Core Types**

- [ ] `vex-runtime/c/vex_vec.c` (generic vector)
- [ ] `vex-runtime/c/vex_option.c` (unwrap helpers)
- [ ] `vex-runtime/c/vex_result.c` (unwrap helpers)
- [ ] `vex-runtime/c/vex_range.c` (Range, RangeInclusive) **NEW**
- [ ] `vex-runtime/c/vex_box.c` (heap allocation) **NEW**
- [ ] `vex-runtime/c/vex_tuple.c` (compile-time only doc) **NEW**
- [ ] `vex-runtime/c/vex_array.c` (enhance existing) **NEW**
- [ ] `vex-runtime/c/vex_string.c` (extend existing)

**Tier 1: Collections & Concurrency**

- [ ] `vex-runtime/c/vex_map.c` (generic Map - rename from hashmap)
- [ ] `vex-runtime/c/vex_set.c` (wrapper over Map<T, ()>)
- [ ] `vex-runtime/c/vex_channel.c` (lock-free MPSC)
- [ ] `vex-runtime/c/vex_iterator.c` (iterator state)

**Tier 2: Advanced Types**

- [ ] `vex-runtime/c/vex_never.c` (unreachable helper) **NEW**
- [ ] `vex-runtime/c/vex_raw_ptr.c` (pointer safety) **NEW**

### AST & Types (17 new enum variants)

**Tier 0: Core Types**

- [ ] `Type::Option(Box<Type>)` in AST
- [ ] `Type::Result(Box<Type>, Box<Type>)` in AST
- [ ] `Type::Vec(Box<Type>)` in AST
- [ ] `Type::Range(Box<Type>)` **NEW**
- [ ] `Type::RangeInclusive(Box<Type>)` **NEW**
- [ ] `Type::Box(Box<Type>)` **NEW**
- [ ] `Type::Tuple(Vec<Type>)` **NEW**
- [ ] `Type::Array(Box<Type>, usize)` **NEW**
- [ ] `Type::String` (already exists)
- [ ] `Type::Slice` (already exists)

**Tier 1: Collections & Concurrency**

- [ ] `Type::Map(Box<Type>, Box<Type>)` in AST
- [ ] `Type::Set(Box<Type>)` in AST
- [ ] `Type::Channel(Box<Type>)` in AST
- [ ] `Type::Iterator(Box<Type>)` in AST

**Tier 2: Advanced Types**

- [ ] `Type::Never` **NEW**
- [ ] `Type::RawPtr(Box<Type>, bool)` **NEW**

### Parser (17 types + special syntax)

**Tier 0: Core Types**

- [ ] Parse `Option<T>`, `Result<T,E>`, `Vec<T>`, `Box<T>`
- [ ] Parse `(T, U, V)` tuple types **NEW**
- [ ] Parse `[T; N]` fixed-size array syntax **NEW**
- [ ] Parse `0..10`, `0..=10` range expressions **NEW**
- [ ] Parse `Some(expr)`, `None`, `Ok(expr)`, `Err(expr)`
- [ ] Parse `Box.new()` constructor **NEW**

**Tier 1: Collections & Concurrency**

- [ ] Parse `Map<K,V>`, `Set<T>`, `Channel<T>`, `Iterator<T>`
- [ ] Parse `for x in collection` syntax

**Tier 2: Advanced Types**

- [ ] Parse `!` never type in function signatures **NEW**
- [ ] Parse `*T`, `*T!` raw pointer types **NEW**
- [ ] Parse `unsafe` blocks for raw pointers **NEW**

**Operators & Special Syntax**

- [ ] Parse method calls: `vec.push(x)`, `opt.unwrap()`
- [ ] Parse `?` operator
- [ ] Parse `vec[index]` indexing

### Codegen (17 builtin type modules)

**Module Structure**

- [ ] `vex-compiler/src/codegen_ast/builtin_types/mod.rs` (new module)

**Tier 0: Core Types**

- [ ] `builtin_types/option.rs` - Option codegen
- [ ] `builtin_types/result.rs` - Result codegen
- [ ] `builtin_types/vec.rs` - Vec codegen
- [ ] `builtin_types/range.rs` - Range iterator codegen **NEW**
- [ ] `builtin_types/box.rs` - Box heap allocation **NEW**
- [ ] `builtin_types/tuple.rs` - Tuple struct generation **NEW**
- [ ] `builtin_types/array.rs` - Fixed-size array **NEW**

**Tier 1: Collections & Concurrency**

- [ ] `builtin_types/map.rs` - Map codegen
- [ ] `builtin_types/set.rs` - Set codegen
- [ ] `builtin_types/channel.rs` - Channel codegen
- [ ] `builtin_types/iterator.rs` - Iterator trait impl

**Tier 2: Advanced Types**

- [ ] `builtin_types/never.rs` - Never type codegen **NEW**
- [ ] `builtin_types/raw_ptr.rs` - Raw pointer ops **NEW**

**Infrastructure**

- [ ] `builtin_types/drop.rs` - Drop trait for builtins
- [ ] Monomorphization support (all generics)

### Tests (30+ test files)

**Tier 0: Core Types**

- [ ] `examples/10_builtins/option_basic.vx`
- [ ] `examples/10_builtins/option_match.vx`
- [ ] `examples/10_builtins/option_methods.vx`
- [ ] `examples/10_builtins/result_basic.vx`
- [ ] `examples/10_builtins/result_match.vx`
- [ ] `examples/10_builtins/vec_basic.vx`
- [ ] `examples/10_builtins/vec_methods.vx`
- [ ] `examples/10_builtins/vec_indexing.vx`
- [ ] `examples/10_builtins/range_basic.vx` **NEW**
- [ ] `examples/10_builtins/range_iteration.vx` **NEW**
- [ ] `examples/10_builtins/box_basic.vx` **NEW**
- [ ] `examples/10_builtins/box_recursive.vx` **NEW**
- [ ] `examples/10_builtins/tuple_basic.vx` **NEW**
- [ ] `examples/10_builtins/tuple_destructure.vx` **NEW**
- [ ] `examples/10_builtins/array_basic.vx` **NEW**
- [ ] `examples/10_builtins/array_simd.vx` **NEW**

**Tier 1: Collections & Concurrency**

- [ ] `examples/10_builtins/map.vx`
- [ ] `examples/10_builtins/set.vx`
- [ ] `examples/10_builtins/iterator.vx`
- [ ] `examples/10_builtins/for_in_loop.vx`
- [ ] `examples/10_builtins/channel.vx`
- [ ] `examples/10_builtins/channel_threaded.vx`

**Tier 2: Advanced Types**

- [ ] `examples/10_builtins/never_type.vx` **NEW**
- [ ] `examples/10_builtins/raw_pointers.vx` **NEW**
- [ ] `examples/10_builtins/ffi_interop.vx` **NEW**

**Benchmarks**

- [ ] `examples/benchmarks/vec_push.vx`
- [ ] `examples/benchmarks/map_insert.vx`
- [ ] `examples/benchmarks/range_iteration.vx` **NEW**
- [ ] `examples/benchmarks/hashmap_insert.vx`

### Documentation

- [ ] Update SYNTAX_REFERENCE.md with builtin types
- [ ] Add examples to README.md
- [ ] Document memory management guarantees
- [ ] Performance comparison with Rust

---

## 🚀 Timeline Summary (Revised for 17 Types)

| Phase                              | Duration | Deliverable                                     |
| ---------------------------------- | -------- | ----------------------------------------------- |
| **Phase 0: Tier 0 Core - Part A**  | 3-4 days | Option, Result, Vec, Box, Tuple                 |
| **Phase 1: Tier 0 Core - Part B**  | 2-3 days | Range, RangeInclusive, Array, String/str        |
| **Phase 2: Pattern Matching**      | 2-3 days | Match on Option/Result, tuple destructuring     |
| **Phase 3: Methods & Operators**   | 2-3 days | Ergonomic API, `?` operator, indexing           |
| **Phase 4: Tier 1 Collections**    | 3-4 days | Map, Set, Iterator, for-in loops                |
| **Phase 5: Tier 1 Concurrency**    | 3-4 days | Channel (enhanced), thread safety               |
| **Phase 6: Tier 2 Advanced**       | 2-3 days | Never type (!), raw pointers (\*T), FFI support |
| **Phase 7: Optimization & Polish** | 2-3 days | Zero-cost verified, benchmarks, documentation   |

**Total:** 19-27 days (4-5.5 weeks)

**Breakdown by Tier:**

- **Tier 0 (Core Types)**: 5-7 days (10 types)
- **Tier 1 (Collections & Concurrency)**: 6-8 days (4 types)
- **Tier 2 (Advanced Types)**: 2-3 days (3 types)
- **Infrastructure**: 6-9 days (pattern matching, operators, optimization)

**Critical Path:**

1. Phase 0-1 must complete first (foundation)
2. Phase 2-3 can partially overlap (patterns + methods)
3. Phase 4-6 can be done in any order (independent tiers)
4. Phase 7 requires all others complete

---

## 🎯 Success Criteria

1. ✅ **No imports needed** - All 17 builtins available everywhere
2. ✅ **Pattern matching** - `match opt { Some(x) => ..., None => ... }`
3. ✅ **Method calls** - `vec.push(x)`, `opt.unwrap()`, `Box.new(value)`
4. ✅ **Range iteration** - `for i in 0..10 { }` works with zero overhead
5. ✅ **Recursive types** - `Box<T>` enables linked lists, trees
6. ✅ **FFI support** - Raw pointers (`*T`) for C interop
7. ✅ **Never type** - Diverging functions properly typed with `!`
8. ✅ **Operator support** - `vec[i]`, `result?`
9. ✅ **Zero-cost** - <5% overhead vs Rust
10. ✅ **Borrow checker aware** - Ownership, moves, borrows tracked
11. ✅ **RAII** - Automatic cleanup via Drop
12. ✅ **Thread-safe** - Channels, atomics

---

**Ready to start Phase 1?** 🚀

Önce `vex_vec.c` ve `Type::Vec` ile başlayalım - en sık kullanılan tip.
