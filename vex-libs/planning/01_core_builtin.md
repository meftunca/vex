# Vex Stdlib Planning - 01: Core/Builtin

**Priority:** 1 (Highest)
**Status:** Partial (builtin exists, others missing)
**Dependencies:** None

## 📦 Packages in This Category

### 1.1 builtin
**Status:** ✅ Exists (core types implemented)
**Description:** Built-in types, functions, and constants

#### Current Implementation
- Box<T>, Vec<T>, Option<T>, Result<T, E> implemented in core module
- Basic print/println functions in io module
- Extern "C" integration working
- Tests: vex-libs/std/core/tests/test_stdlib*.vx
- Tests: vex-libs/std/core/tests/test_stdlib*.vx

#### Required Functions
```vex
// Type conversion
fn int(x: any): i64
fn float(x: any): f64
fn string(x: any): str
fn bool(x: any): bool

// Collection operations
fn len<T>(collection: T): usize  // Arrays, strings, maps, etc.
fn cap<T>(collection: T): usize  // Capacity for growable collections

// Memory operations
fn size_of<T>(): usize           // Size of type in bytes
fn align_of<T>(): usize          // Alignment of type

// Panic and assertion
fn panic(msg: str) -> !
fn assert(condition: bool, msg: str)

// Type information
fn typeof<T>(): TypeInfo

// Constants
const true: bool = true
const false: bool = false
```

#### Dependencies
- None (built-in)

#### Notes
- `any` type needed for generic conversions
- `TypeInfo` struct for reflection basics

### 1.2 unsafe
**Status:** Partial (used in modules, no dedicated package)
**Description:** Unsafe memory operations and pointer manipulation

#### Current Usage
- Used in db module for raw pointers
- No dedicated unsafe package
- Raw pointer operations available via extern "C"

#### Required Functions
```vex
// Raw pointers
fn ptr_offset<T>(ptr: *T, offset: isize): *T
fn ptr_read<T>(ptr: *T): T
fn ptr_write<T>(ptr: *T, value: T)

// Memory operations
fn memcpy(dst: *u8, src: *u8, len: usize)
fn memmove(dst: *u8, src: *u8, len: usize)
fn memset(ptr: *u8, value: u8, len: usize)

// Volatile operations
fn volatile_read<T>(ptr: *T): T
fn volatile_write<T>(ptr: *T, value: T)

// Architecture-specific
fn atomic_load<T>(ptr: *T): T
fn atomic_store<T>(ptr: *T, value: T)
fn atomic_add<T>(ptr: *T, value: T): T
```

#### Dependencies
- builtin

#### Notes
- **Language Issue:** No `unsafe` keyword in Vex syntax
- **Workaround:** Use `extern "C"` for unsafe operations
- **Security:** Need compiler warnings for unsafe usage

### 1.3 reflect
**Status:** ❌ Missing (major gap)
**Description:** Runtime type reflection and introspection

#### Required Types
```vex
struct Type {
    name: str,
    size: usize,
    align: usize,
    kind: TypeKind,
}

enum TypeKind {
    Bool, Int, Uint, Float, Complex,
    Array, Slice, String, Struct, Ptr,
    Func, Interface, Map, Chan,
}

struct Value {
    typ: Type,
    ptr: *u8,
    is_nil: bool,
}
```

#### Required Functions
```vex
// Type operations
fn typeof<T>(): Type
fn type_of(value: any): Type

// Value operations
fn value_of<T>(x: T): Value
fn interface_of<T>(x: T): any

// Reflection operations
fn field_by_name(v: Value, name: str): Value
fn method_by_name(v: Value, name: str): Value
fn call(v: Value, args: []Value): []Value

// Type assertions
fn is_nil(v: Value): bool
fn kind(v: Value): TypeKind
```

#### Dependencies
- builtin
- unsafe

#### Notes
- **Language Issue:** No runtime type information in current compiler
- **Implementation:** Requires compiler changes for type metadata
- **Complexity:** High - needs integration with borrow checker

### 1.4 errors
**Status:** Partial (Result/Option exist, dedicated errors package missing)
**Description:** Structured error handling utilities

#### Current Implementation
- Result<T, E> and Option<T> enums exist in core
- No dedicated errors package with utilities like errors.New(), errors.Join()

#### Required Types
```vex
trait Error {
    fn error(self: &Self): str;
}

struct SimpleError {
    msg: str,
}

struct JoinError {
    errs: []Error,
}
```

#### Required Functions
```vex
// Error constructors
fn new(msg: str): Error
fn join(errs: []Error): Error

// Error operations
fn is<T: Error>(err: Error, target: T): bool
fn as<T: Error>(err: Error): Option<T>

// Unwrap operations
fn unwrap<T>(result: Result<T, Error>): T
fn expect<T>(result: Result<T, Error>, msg: str): T
```

#### Dependencies
- builtin

#### Notes
- **Design Decision:** Integrate with existing Result/Option or provide wrapper?
- **Consistency:** Match Go's error handling patterns
- **Performance:** Zero-cost when not used

## 🎯 Implementation Priority

1. **builtin** - Extend existing builtin functions
2. **errors** - Basic error handling utilities
3. **unsafe** - Memory operations (with compiler warnings)
4. **reflect** - Runtime reflection (requires compiler work)

## ⚠️ Language Feature Issues

- **Unsafe Operations:** No `unsafe` keyword - need syntax addition or attribute
- **Runtime Reflection:** Compiler doesn't emit type metadata
- **Any Type:** `any` interface not fully implemented
- **Type Metadata:** No way to get type information at runtime

## 📋 Missing Critical Dependencies

- **Type System Extensions:** `any` type for dynamic typing
- **Compiler Intrinsics:** Built-in functions for reflection
- **Unsafe Markers:** Syntax for marking unsafe code blocks

## 🚀 Next Steps

1. Extend builtin with missing functions
2. Implement errors package
3. Add unsafe operations (with warnings)
4. Plan compiler changes for reflection