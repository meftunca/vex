# Builtin Types Implementation Guide - Quick Start

**Date:** November 5, 2025  
**Target:** Phase 0 - Tier 0 Core Types (Part A)  
**Duration:** 3-4 days  
**Types:** Option, Result, Vec, Box, Tuple

---

## 🎯 Phase 0 Goals

Implement the **5 most critical builtin types** that form the foundation:

1. **Option<T>** - Null safety (`Some(value)` or `None`)
2. **Result<T,E>** - Error handling (`Ok(value)` or `Err(error)`)
3. **Vec<T>** - Dynamic arrays (growable, heap-allocated)
4. **Box<T>** - Heap allocation (enables recursive types)
5. **Tuple<T,U,V>** - Multi-value grouping (`(42, "hello", true)`)

---

## 📋 Day 1: C Runtime Foundation (6-8 hours)

### Step 1.1: Vec Implementation (2-3 hours)

**File:** `vex-runtime/c/vex_vec.c`

```c
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "vex.h"

// Generic vector (type-erased with elem_size)
typedef struct {
    void *data;          // Pointer to heap-allocated array
    size_t len;          // Current number of elements
    size_t capacity;     // Allocated capacity
    size_t elem_size;    // Size of each element in bytes
} vex_vec_t;

// Create new empty vector
vex_vec_t vex_vec_new(size_t elem_size) {
    return (vex_vec_t){
        .data = NULL,
        .len = 0,
        .capacity = 0,
        .elem_size = elem_size
    };
}

// Internal: Grow capacity (2x strategy)
static void vex_vec_grow(vex_vec_t *vec) {
    size_t new_cap = vec->capacity == 0 ? 8 : vec->capacity * 2;
    void *new_data = realloc(vec->data, new_cap * vec->elem_size);
    if (!new_data) {
        fprintf(stderr, "Vec allocation failed\n");
        abort();
    }
    vec->data = new_data;
    vec->capacity = new_cap;
}

// Push element to end
void vex_vec_push(vex_vec_t *vec, const void *elem) {
    if (vec->len >= vec->capacity) {
        vex_vec_grow(vec);
    }
    void *dest = (uint8_t*)vec->data + (vec->len * vec->elem_size);
    memcpy(dest, elem, vec->elem_size);
    vec->len++;
}

// Get element at index (returns pointer for zero-copy)
void *vex_vec_get(vex_vec_t *vec, size_t index) {
    if (index >= vec->len) {
        fprintf(stderr, "Vec index out of bounds: %zu >= %zu\n", index, vec->len);
        abort();
    }
    return (uint8_t*)vec->data + (index * vec->elem_size);
}

// Pop last element (writes to out)
bool vex_vec_pop(vex_vec_t *vec, void *out) {
    if (vec->len == 0) {
        return false;
    }
    vec->len--;
    void *src = (uint8_t*)vec->data + (vec->len * vec->elem_size);
    memcpy(out, src, vec->elem_size);
    return true;
}

// Reserve additional capacity
void vex_vec_reserve(vex_vec_t *vec, size_t additional) {
    size_t required = vec->len + additional;
    if (required <= vec->capacity) {
        return;
    }
    size_t new_cap = vec->capacity == 0 ? 8 : vec->capacity;
    while (new_cap < required) {
        new_cap *= 2;
    }
    void *new_data = realloc(vec->data, new_cap * vec->elem_size);
    if (!new_data) {
        fprintf(stderr, "Vec reserve failed\n");
        abort();
    }
    vec->data = new_data;
    vec->capacity = new_cap;
}

// Get length
size_t vex_vec_len(vex_vec_t *vec) {
    return vec->len;
}

// Free vector
void vex_vec_free(vex_vec_t *vec) {
    if (vec->data) {
        free(vec->data);
        vec->data = NULL;
    }
    vec->len = 0;
    vec->capacity = 0;
}
```

**Header:** Add to `vex-runtime/c/vex.h`:

```c
// Vec<T> - dynamic array
typedef struct vex_vec_s vex_vec_t;

vex_vec_t vex_vec_new(size_t elem_size);
void vex_vec_push(vex_vec_t *vec, const void *elem);
void *vex_vec_get(vex_vec_t *vec, size_t index);
bool vex_vec_pop(vex_vec_t *vec, void *out);
void vex_vec_reserve(vex_vec_t *vec, size_t additional);
size_t vex_vec_len(vex_vec_t *vec);
void vex_vec_free(vex_vec_t *vec);
```

### Step 1.2: Option Implementation (1 hour)

**File:** `vex-runtime/c/vex_option.c`

```c
#include <stdio.h>
#include <stdlib.h>
#include "vex.h"

// Option is compile-time struct: { u8 tag, T value }
// Runtime only provides unwrap helper

void *vex_option_unwrap(void *opt_ptr, size_t type_size, const char *file, int line) {
    uint8_t tag = *(uint8_t*)opt_ptr;
    if (tag == 0) {  // None
        fprintf(stderr, "Unwrap failed at %s:%d - Option is None\n", file, line);
        abort();
    }
    // Return pointer to value (skip 1-byte tag)
    return (uint8_t*)opt_ptr + 1;
}

// Helper for is_some check
bool vex_option_is_some(void *opt_ptr) {
    return *(uint8_t*)opt_ptr == 1;
}

// Helper for is_none check
bool vex_option_is_none(void *opt_ptr) {
    return *(uint8_t*)opt_ptr == 0;
}
```

**Header:** Add to `vex.h`:

```c
// Option<T> - nullable type (compile-time struct)
void *vex_option_unwrap(void *opt_ptr, size_t type_size, const char *file, int line);
bool vex_option_is_some(void *opt_ptr);
bool vex_option_is_none(void *opt_ptr);
```

### Step 1.3: Result Implementation (1 hour)

**File:** `vex-runtime/c/vex_result.c`

```c
#include <stdio.h>
#include <stdlib.h>
#include "vex.h"

// Result is compile-time struct: { u8 tag, union { T ok, E err } }

void *vex_result_unwrap(void *result_ptr, size_t type_size, const char *file, int line) {
    uint8_t tag = *(uint8_t*)result_ptr;
    if (tag == 0) {  // Err
        fprintf(stderr, "Unwrap failed at %s:%d - Result is Err\n", file, line);
        abort();
    }
    // Return pointer to ok value (skip 1-byte tag)
    return (uint8_t*)result_ptr + 1;
}

void *vex_result_expect(void *result_ptr, size_t type_size, const char *msg, const char *file, int line) {
    uint8_t tag = *(uint8_t*)result_ptr;
    if (tag == 0) {  // Err
        fprintf(stderr, "Expect failed at %s:%d - %s\n", file, line, msg);
        abort();
    }
    return (uint8_t*)result_ptr + 1;
}

bool vex_result_is_ok(void *result_ptr) {
    return *(uint8_t*)result_ptr == 1;
}

bool vex_result_is_err(void *result_ptr) {
    return *(uint8_t*)result_ptr == 0;
}
```

**Header:** Add to `vex.h`:

```c
// Result<T,E> - error handling (compile-time struct)
void *vex_result_unwrap(void *result_ptr, size_t type_size, const char *file, int line);
void *vex_result_expect(void *result_ptr, size_t type_size, const char *msg, const char *file, int line);
bool vex_result_is_ok(void *result_ptr);
bool vex_result_is_err(void *result_ptr);
```

### Step 1.4: Box Implementation (1 hour)

**File:** `vex-runtime/c/vex_box.c`

```c
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "vex.h"

// Box<T> - heap-allocated value with ownership
typedef struct {
    void *ptr;
    size_t size;
} vex_box_t;

// Create new Box with copied value
vex_box_t vex_box_new(const void *value, size_t size) {
    void *ptr = malloc(size);
    if (!ptr) {
        fprintf(stderr, "Box allocation failed\n");
        abort();
    }
    memcpy(ptr, value, size);
    return (vex_box_t){ .ptr = ptr, .size = size };
}

// Borrow pointer (read-only)
void *vex_box_get(vex_box_t *box) {
    return box->ptr;
}

// Borrow mutable pointer
void *vex_box_get_mut(vex_box_t *box) {
    return box->ptr;
}

// Move out inner value (caller takes ownership)
void *vex_box_into_inner(vex_box_t box) {
    return box.ptr;  // Don't free, caller owns
}

// Free Box
void vex_box_free(vex_box_t box) {
    if (box.ptr) {
        free(box.ptr);
    }
}
```

**Header:** Add to `vex.h`:

```c
// Box<T> - heap allocation
typedef struct {
    void *ptr;
    size_t size;
} vex_box_t;

vex_box_t vex_box_new(const void *value, size_t size);
void *vex_box_get(vex_box_t *box);
void *vex_box_get_mut(vex_box_t *box);
void *vex_box_into_inner(vex_box_t box);
void vex_box_free(vex_box_t box);
```

### Step 1.5: Tuple Documentation (15 minutes)

**File:** `vex-runtime/c/vex_tuple.c`

```c
// Tuple<T, U, V> is COMPILE-TIME ONLY
// No runtime implementation needed
//
// Compiler generates struct layout:
// (i32, String, bool) → struct { i32 _0; vex_string_t _1; bool _2; }
//
// Field access:
// tuple.0 → access struct field _0
// tuple.1 → access struct field _1
// tuple.2 → access struct field _2
//
// Destructuring:
// let (x, y, z) = tuple; → let x = tuple._0; let y = tuple._1; let z = tuple._2;
```

### Step 1.6: Build & Test (30 minutes)

```bash
cd vex-runtime/c
./build.sh

# Test Vec
gcc -o test_vec test_vec.c vex_vec.c -I.
./test_vec

# Test Option/Result
gcc -o test_option test_option.c vex_option.c vex_result.c -I.
./test_option

# Test Box
gcc -o test_box test_box.c vex_box.c -I.
./test_box
```

---

## 📋 Day 2: AST & Parser (6-8 hours)

### Step 2.1: Add AST Types (1 hour)

**File:** `vex-ast/src/lib.rs`

Find the `Type` enum and add:

```rust
pub enum Type {
    // ... existing types ...

    // Tier 0: Core builtin types (no imports needed)
    Option(Box<Type>),                    // Option<T>
    Result(Box<Type>, Box<Type>),         // Result<T, E>
    Vec(Box<Type>),                       // Vec<T>
    Box(Box<Type>),                       // Box<T>
    Tuple(Vec<Type>),                     // (T, U, V, ...)
}
```

### Step 2.2: Parser - Type Parsing (2-3 hours)

**File:** `vex-parser/src/parser/types.rs`

Add to `parse_type()` method:

```rust
fn parse_type(&mut self) -> Result<Type, ParseError> {
    match &self.current.kind {
        // Existing type parsing...

        // Builtin generic types
        TokenKind::Ident(name) if name == "Option" => {
            self.advance();
            self.expect(TokenKind::Lt)?;
            let inner = self.parse_type()?;
            self.expect(TokenKind::Gt)?;
            Ok(Type::Option(Box::new(inner)))
        }

        TokenKind::Ident(name) if name == "Result" => {
            self.advance();
            self.expect(TokenKind::Lt)?;
            let ok_type = self.parse_type()?;
            self.expect(TokenKind::Comma)?;
            let err_type = self.parse_type()?;
            self.expect(TokenKind::Gt)?;
            Ok(Type::Result(Box::new(ok_type), Box::new(err_type)))
        }

        TokenKind::Ident(name) if name == "Vec" => {
            self.advance();
            self.expect(TokenKind::Lt)?;
            let inner = self.parse_type()?;
            self.expect(TokenKind::Gt)?;
            Ok(Type::Vec(Box::new(inner)))
        }

        TokenKind::Ident(name) if name == "Box" => {
            self.advance();
            self.expect(TokenKind::Lt)?;
            let inner = self.parse_type()?;
            self.expect(TokenKind::Gt)?;
            Ok(Type::Box(Box::new(inner)))
        }

        TokenKind::LParen => {
            // Tuple type: (T, U, V)
            self.advance();
            let mut types = Vec::new();

            if !self.check(&TokenKind::RParen) {
                types.push(self.parse_type()?);
                while self.check(&TokenKind::Comma) {
                    self.advance();
                    if self.check(&TokenKind::RParen) {
                        break;  // Trailing comma
                    }
                    types.push(self.parse_type()?);
                }
            }

            self.expect(TokenKind::RParen)?;
            Ok(Type::Tuple(types))
        }

        _ => {
            // Existing type parsing fallback...
        }
    }
}
```

### Step 2.3: Parser - Enum Constructors (1-2 hours)

**File:** `vex-parser/src/parser/expressions.rs`

Add enum variant parsing for `Some()`, `None`, `Ok()`, `Err()`:

```rust
fn parse_primary(&mut self) -> Result<Expr, ParseError> {
    match &self.current.kind {
        // Existing primary expressions...

        TokenKind::Ident(name) if matches!(name.as_str(), "Some" | "None" | "Ok" | "Err") => {
            let variant_name = name.clone();
            self.advance();

            // None has no arguments
            if variant_name == "None" {
                return Ok(Expr::EnumVariant {
                    enum_name: "Option".to_string(),
                    variant_name,
                    value: None,
                });
            }

            // Some, Ok, Err have single argument
            self.expect(TokenKind::LParen)?;
            let value = self.parse_expression()?;
            self.expect(TokenKind::RParen)?;

            let enum_name = match variant_name.as_str() {
                "Some" => "Option",
                "Ok" | "Err" => "Result",
                _ => unreachable!(),
            };

            Ok(Expr::EnumVariant {
                enum_name: enum_name.to_string(),
                variant_name,
                value: Some(Box::new(value)),
            })
        }

        _ => {
            // Existing primary expression fallback...
        }
    }
}
```

### Step 2.4: Parser - Box.new() Constructor (1 hour)

Add method call parsing for `Box.new()`:

```rust
// In parse_postfix or similar:
TokenKind::Ident(name) if name == "Box" => {
    self.advance();
    if self.check(&TokenKind::Dot) {
        self.advance();
        if let TokenKind::Ident(method) = &self.current.kind {
            if method == "new" {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let value = self.parse_expression()?;
                self.expect(TokenKind::RParen)?;

                return Ok(Expr::BoxNew {
                    value: Box::new(value),
                });
            }
        }
    }
    // ...
}
```

---

## 📋 Day 3: LLVM Codegen (6-8 hours)

### Step 3.1: Type Compilation (2 hours)

**File:** `vex-compiler/src/codegen_ast/types.rs`

Add to `compile_type()`:

```rust
pub fn compile_type(&mut self, ty: &Type) -> Result<BasicTypeEnum<'ctx>, String> {
    match ty {
        // Existing type compilation...

        Type::Option(inner) => {
            let inner_ty = self.compile_type(inner)?;
            // Struct: { u8 tag, T value }
            let tag_ty = self.context.i8_type();
            let struct_ty = self.context.struct_type(&[tag_ty.into(), inner_ty], false);
            Ok(struct_ty.into())
        }

        Type::Result(ok_ty, err_ty) => {
            let ok_llvm = self.compile_type(ok_ty)?;
            let err_llvm = self.compile_type(err_ty)?;
            // Struct: { u8 tag, union { T ok, E err } }
            // Use largest size for union
            let tag_ty = self.context.i8_type();
            let union_size = ok_llvm.size_of().unwrap().max(err_llvm.size_of().unwrap());
            let union_ty = self.context.i8_type().array_type(union_size as u32);
            let struct_ty = self.context.struct_type(&[tag_ty.into(), union_ty.into()], false);
            Ok(struct_ty.into())
        }

        Type::Vec(elem_ty) => {
            // Struct: { ptr, len, capacity, elem_size }
            let ptr_ty = self.context.i8_type().ptr_type(AddressSpace::default());
            let size_ty = self.context.i64_type();
            let struct_ty = self.context.struct_type(
                &[ptr_ty.into(), size_ty.into(), size_ty.into(), size_ty.into()],
                false,
            );
            Ok(struct_ty.into())
        }

        Type::Box(inner) => {
            // Struct: { ptr, size }
            let ptr_ty = self.context.i8_type().ptr_type(AddressSpace::default());
            let size_ty = self.context.i64_type();
            let struct_ty = self.context.struct_type(&[ptr_ty.into(), size_ty.into()], false);
            Ok(struct_ty.into())
        }

        Type::Tuple(types) => {
            let mut field_types = Vec::new();
            for ty in types {
                field_types.push(self.compile_type(ty)?);
            }
            let struct_ty = self.context.struct_type(&field_types, false);
            Ok(struct_ty.into())
        }

        _ => {
            // Existing type fallback...
        }
    }
}
```

### Step 3.2: Builtin Types Module (1 hour)

**File:** `vex-compiler/src/codegen_ast/builtin_types/mod.rs` (NEW)

```rust
pub mod option;
pub mod result;
pub mod vec;
pub mod box_type;
pub mod tuple;
```

### Step 3.3: Vec Codegen (2 hours)

**File:** `vex-compiler/src/codegen_ast/builtin_types/vec.rs` (NEW)

```rust
use inkwell::values::{FunctionValue, PointerValue, BasicValueEnum};
use inkwell::types::BasicTypeEnum;
use crate::codegen_ast::ASTCodeGen;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile Vec.new() constructor
    pub fn compile_vec_new(&mut self, elem_ty: &Type) -> Result<PointerValue<'ctx>, String> {
        let elem_llvm = self.compile_type(elem_ty)?;
        let elem_size = elem_llvm.size_of().unwrap();

        // Call vex_vec_new(elem_size)
        let vec_new_fn = self.get_builtin_function("vex_vec_new")?;
        let result = self.builder.build_call(
            vec_new_fn,
            &[elem_size.into()],
            "vec_new",
        );

        Ok(result.try_as_basic_value().left().unwrap().into_pointer_value())
    }

    /// Compile vec.push(elem)
    pub fn compile_vec_push(
        &mut self,
        vec_ptr: PointerValue<'ctx>,
        elem: BasicValueEnum<'ctx>,
    ) -> Result<(), String> {
        // Allocate temp for elem
        let elem_ptr = self.builder.build_alloca(elem.get_type(), "elem_temp");
        self.builder.build_store(elem_ptr, elem);

        // Call vex_vec_push(vec, elem_ptr)
        let vec_push_fn = self.get_builtin_function("vex_vec_push")?;
        self.builder.build_call(
            vec_push_fn,
            &[vec_ptr.into(), elem_ptr.into()],
            "",
        );

        Ok(())
    }

    /// Compile vec[index] indexing
    pub fn compile_vec_index(
        &mut self,
        vec_ptr: PointerValue<'ctx>,
        index: BasicValueEnum<'ctx>,
        elem_ty: &Type,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Call vex_vec_get(vec, index) → returns pointer
        let vec_get_fn = self.get_builtin_function("vex_vec_get")?;
        let result_ptr = self.builder.build_call(
            vec_get_fn,
            &[vec_ptr.into(), index],
            "vec_get",
        ).try_as_basic_value().left().unwrap().into_pointer_value();

        // Cast to correct type and load
        let elem_llvm = self.compile_type(elem_ty)?;
        let typed_ptr = self.builder.build_pointer_cast(
            result_ptr,
            elem_llvm.ptr_type(AddressSpace::default()),
            "typed_ptr",
        );

        Ok(self.builder.build_load(typed_ptr, "elem").into())
    }
}
```

Continue similar patterns for Option, Result, Box...

---

## 📋 Day 4: Tests & Integration (4-6 hours)

### Step 4.1: Example Tests

**File:** `examples/10_builtins/vec_basic.vx`

```vex
fn main(): i32 {
    let! vec = Vec.new();
    vec.push(10);
    vec.push(20);
    vec.push(30);

    let first = vec[0];
    let second = vec[1];

    return first + second;  // 10 + 20 = 30
}
```

**File:** `examples/10_builtins/option_basic.vx`

```vex
fn divide(x: i32, y: i32): Option<i32> {
    if y == 0 {
        return None;
    }
    return Some(x / y);
}

fn main(): i32 {
    let result = divide(10, 2);
    match result {
        Some(value) => return value,  // 5
        None => return -1,
    }
}
```

**File:** `examples/10_builtins/box_recursive.vx`

```vex
struct Node {
    value: i32,
    next: Option<Box<Node>>
}

fn main(): i32 {
    let node3 = Node { value: 3, next: None };
    let node2 = Node { value: 2, next: Some(Box.new(node3)) };
    let node1 = Node { value: 1, next: Some(Box.new(node2)) };

    return node1.value;  // 1
}
```

### Step 4.2: Run Tests

```bash
~/.cargo/target/debug/vex run examples/10_builtins/vec_basic.vx
~/.cargo/target/debug/vex run examples/10_builtins/option_basic.vx
~/.cargo/target/debug/vex run examples/10_builtins/box_recursive.vx
```

---

## ✅ Success Criteria for Phase 0

- [ ] All 5 C runtime files compile and link
- [ ] AST has all 5 new Type variants
- [ ] Parser accepts `Option<T>`, `Result<T,E>`, `Vec<T>`, `Box<T>`, `(T,U,V)` syntax
- [ ] Parser accepts `Some()`, `None`, `Ok()`, `Err()`, `Box.new()` constructors
- [ ] Codegen compiles all 5 types to LLVM structs
- [ ] Tests pass: `vec_basic.vx`, `option_basic.vx`, `box_recursive.vx`
- [ ] Memory leaks tested (valgrind)

---

## 🚀 Next Steps (Phase 1 - Day 5-7)

After Phase 0 completion, continue to:

1. **Range & RangeInclusive** - For iteration (`0..10`, `0..=10`)
2. **Array<T, N>** - Fixed-size stack arrays (`[i32; 5]`)
3. **String/str integration** - Verify existing implementation
4. **Pattern matching** - Match on Option/Result variants
5. **Methods** - `vec.push()`, `opt.unwrap()`, etc.

See `BUILTIN_TYPES_ARCHITECTURE.md` for complete roadmap.

---

**Ready to start! Begin with Day 1 - C Runtime Foundation** 🚀
