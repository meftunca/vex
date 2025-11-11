# Mutability and Pointers

**Version:** 0.1.2
**Last Updated:** November 2025

This document provides a comprehensive guide to Vex's mutability system and pointer types, including raw pointers, references, and memory safety guarantees.

---

## Table of Contents

1. \1
2. \1
3. \1
4. \1
5. \1
6. \1
7. \1

---

## Mutability System

### Variable Mutability

Vex uses explicit mutability markers:

``````vex
let x = 42;        // Immutable (default)
let! y = 42;       // Mutable (explicit ! suffix)

x = 100;           // ERROR: Cannot assign to immutable
y = 100;           // OK: y is mutable
```

### Field Mutability

Struct fields inherit mutability from their containing variable:

[10 lines code: ```vex]

### Method Mutability

Vex uses a hybrid model for method mutability.

### 1. Inline Methods (in `struct` or `trait`)

- **Declaration**: `fn method_name()!`
- **Behavior**: The method can mutate `self`.
- **Call**: `object.method_name()` (no `!` at call site). The compiler ensures this is only called on a mutable (`let!`) variable.

[9 lines code: ```vex]

### 2. External Methods (Golang-style)

- **Declaration**: `fn (self: &MyType!) method_name()`
- **Behavior**: The method can mutate `self`.
- **Call**: `object.method_name()` (no `!` at call site).

[13 lines code: ```vex]

---

## References

### Immutable References

``````vex
&T                    // Immutable reference to T
let x = 42;
let ref_x: &i32 = &x;  // Reference to x
println("{}", ref_x);  // Dereference with *
```

### Mutable References

``````vex
&T!                   // Mutable reference to T
let! x = 42;
let ref_x: &i32! = &x; // Mutable reference
*ref_x = 100;         // Modify through reference
```

### Reference Rules

1. **Single Writer**: Only one mutable reference at a time
2. **No Aliasing**: Mutable references cannot coexist with other references
3. **Lifetime Bounds**: References cannot outlive their referent

``````vex
let! x = 42;
let r1: &i32! = &x;
// let r2: &i32! = &x;  // ERROR: Multiple mutable borrows
// let r3: &i32 = &x;   // ERROR: Mutable and immutable borrow conflict
```

### Deref Operator

``````vex
let x = 42;
let r: &i32 = &x;
println("{}", *r);     // Dereference: prints 42

let! y = 42;
let mr: &i32! = &y;
*mr = 100;             // Modify through mutable reference
```

---

## Raw Pointers

### Raw Pointer Types

``````vex
*T                    // Raw immutable pointer
*T!                   // Raw mutable pointer
```

### Creation

``````vex
let x = 42;
let ptr: *i32 = &x as *i32;        // Cast reference to raw pointer
let mut_ptr: *i32! = &x as *i32!;  // Cast to mutable raw pointer
```

### Dereferencing

``````vex
unsafe {
    let value = *ptr;           // Dereference immutable pointer
    *mut_ptr = 100;             // Modify through mutable pointer
}
```

### Null Pointers

``````vex
let null_ptr: *i32 = 0 as *i32;     // Null pointer
let is_null = ptr == 0 as *i32;     // Check for null
```

---

## Pointer Arithmetic

### Basic Arithmetic

[10 lines code: ```vex]

### Array Iteration

[13 lines code: ```vex]

### Pointer Subtraction

``````vex
let arr = [10, 20, 30, 40];
let start: *i32 = &arr[0] as *i32;
let end: *i32 = &arr[3] as *i32;

let distance = end - start;  // distance = 3
```

---

## Memory Safety

### Borrow Checker Integration

Raw pointers bypass borrow checker but require `unsafe` blocks:

[11 lines code: ```vex]

### Lifetime Safety

References have lifetime bounds, raw pointers do not:

[9 lines code: ```vex]

### Common Unsafe Patterns

[22 lines code: ```vex]

---

## FFI Integration

### C Interoperability

Raw pointers are essential for C FFI:

[17 lines code: ```vex]

### Struct Layout Compatibility

C-compatible struct layout is automatic in Vex (no attributes needed):

[18 lines code: ```vex]

```````

### Struct Layout Compatibility

```
#[repr(C)]
struct CPoint {
 x: f32,
 y: f32,
}

extern "C" {
 fn create_point(x: f32, y: f32): *CPoint;
 fn get_x(point: *CPoint): f32;
}

fn use_c_library() {
 unsafe {
 let point = create_point(1.0, 2.0);
 let x = get_x(point);
 println("x: {}", x);
 // Remember to deallocate if required by C library
 }
}
```````

---

## Common Patterns

### Safe Wrapper Types

```
struct SafePtr<T> {
 ptr: *T,
 valid: bool,
}

impl<T> SafePtr<T> {
 fn new(value: T): SafePtr<T> {
 unsafe {
 let ptr = malloc(sizeof<T>()) as *T;
 *ptr = value;
 return SafePtr { ptr: ptr, valid: true };
 }
 }

 fn get(self: &SafePtr<T>): &T {
 assert(self.valid, "Pointer is invalid");
 unsafe {
 return &*self.ptr;
 }
 }

 fn drop(self: &SafePtr!) {
 if self.valid {
 unsafe {
 free(self.ptr as *u8);
 }
 self.valid = false;
 }
 }
}
```

### Iterator Implementation

```
struct ArrayIter<T> {
 ptr: *T,
 end: *T,
}

impl<T> ArrayIter<T> {
 fn new(arr: &Vec<T>): ArrayIter<T> {
 unsafe {
 let start = &arr[0] as *T;
 let end = start + arr.len();
 return ArrayIter { ptr: start, end: end };
 }
 }

 fn next(self: &ArrayIter!): Option<&T> {
 if self.ptr >= self.end {
 return Option.None;
 }

 unsafe {
 let result = &*self.ptr;
 self.ptr = self.ptr + 1;
 return Option.Some(result);
 }
 }
}
```

### Manual Memory Management

```
fn manual_vec_demo() {
 unsafe {
 // Allocate space for 10 i32s
 let ptr = malloc(10 * sizeof<i32>()) as *i32!;

 // Initialize
 for i in 0..10 {
 *(ptr + i) = i as i32 * 2;
 }

 // Use
 for i in 0..10 {
 println("{}", *(ptr + i));
 }

 // Deallocate
 free(ptr as *u8);
 }
}
```

### Performance-Critical Code

```
fn fast_memcpy(dest: *u8!, src: *u8, n: usize) {
 unsafe {
 let mut d = dest;
 let mut s = src;

 // Copy in chunks of 8 bytes when possible
 while n >= 8 {
 *(d as *u64!) = *(s as *u64);
 d = d + 8;
 s = s + 8;
 n -= 8;
 }

 // Copy remaining bytes
 while n > 0 {
 *d = *s;
 d = d + 1;
 s = s + 1;
 n -= 1;
 }
 }
}
[28 lines code: (unknown)]
// ❌ Dangling pointer
fn bad() {
 let ptr: *i32;
 {
 let x = 42;
 ptr = &x as *i32;
 }
 // ptr now dangles!
}

// ✅ Safe alternative
fn good() -> i32 {
 let x = 42;
 return x; // Value moved out
}
