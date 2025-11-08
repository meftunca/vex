# Mutability and Pointers

**Version:** 0.9.2
**Last Updated:** November 2025

This document provides a comprehensive guide to Vex's mutability system and pointer types, including raw pointers, references, and memory safety guarantees.

---

## Table of Contents

1. [Mutability System](#mutability-system)
2. [References](#references)
3. [Raw Pointers](#raw-pointers)
4. [Pointer Arithmetic](#pointer-arithmetic)
5. [Memory Safety](#memory-safety)
6. [FFI Integration](#ffi-integration)
7. [Common Patterns](#common-patterns)

---

## Mutability System

### Variable Mutability

Vex uses explicit mutability markers:

```vex
let x = 42;        // Immutable (default)
let! y = 42;       // Mutable (explicit ! suffix)

x = 100;           // ERROR: Cannot assign to immutable
y = 100;           // OK: y is mutable
```

### Field Mutability

Struct fields inherit mutability from their containing variable:

```vex
struct Point {
    x: i32,
    y: i32,
}

let p = Point { x: 1, y: 2 };
// p.x = 10;        // ERROR: p is immutable

let! p2 = Point { x: 1, y: 2 };
p2.x = 10;         // OK: p2 is mutable
```

### Method Mutability

Methods can be called on mutable or immutable receivers:

```vex
impl Point {
    fn get_x(self: &Point): i32 {
        return self.x;    // Immutable access
    }

    fn set_x(self: &Point!, x: i32) {
        self.x = x;       // Mutable access
    }
}

let! p = Point { x: 1, y: 2 };
let x = p.get_x();     // OK: immutable method
p.set_x(42);           // OK: mutable method
```

---

## References

### Immutable References

```vex
&T                    // Immutable reference to T
let x = 42;
let ref_x: &i32 = &x;  // Reference to x
println("{}", ref_x);  // Dereference with *
```

### Mutable References

```vex
&T!                   // Mutable reference to T
let! x = 42;
let ref_x: &i32! = &x; // Mutable reference
*ref_x = 100;         // Modify through reference
```

### Reference Rules

1. **Single Writer**: Only one mutable reference at a time
2. **No Aliasing**: Mutable references cannot coexist with other references
3. **Lifetime Bounds**: References cannot outlive their referent

```vex
let! x = 42;
let r1: &i32! = &x;
// let r2: &i32! = &x;  // ERROR: Multiple mutable borrows
// let r3: &i32 = &x;   // ERROR: Mutable and immutable borrow conflict
```

### Deref Operator

```vex
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

```vex
*T                    // Raw immutable pointer
*T!                   // Raw mutable pointer
```

### Creation

```vex
let x = 42;
let ptr: *i32 = &x as *i32;        // Cast reference to raw pointer
let mut_ptr: *i32! = &x as *i32!;  // Cast to mutable raw pointer
```

### Dereferencing

```vex
unsafe {
    let value = *ptr;           // Dereference immutable pointer
    *mut_ptr = 100;             // Modify through mutable pointer
}
```

### Null Pointers

```vex
let null_ptr: *i32 = 0 as *i32;     // Null pointer
let is_null = ptr == 0 as *i32;     // Check for null
```

---

## Pointer Arithmetic

### Basic Arithmetic

```vex
let arr = [1, 2, 3, 4, 5];
let ptr: *i32 = &arr[0] as *i32;

unsafe {
    let second = ptr + 1;       // Points to arr[1]
    let third = ptr + 2;        // Points to arr[2]

    println("{}", *second);     // Prints 2
    println("{}", *third);      // Prints 3
}
```

### Array Iteration

```vex
fn sum_array(arr: *i32, len: usize): i32 {
    let mut sum = 0;
    let mut ptr = arr;

    for i in 0..len {
        unsafe {
            sum += *ptr;
            ptr = ptr + 1;
        }
    }

    return sum;
}
```

### Pointer Subtraction

```vex
let arr = [10, 20, 30, 40];
let start: *i32 = &arr[0] as *i32;
let end: *i32 = &arr[3] as *i32;

let distance = end - start;  // distance = 3
```

---

## Memory Safety

### Borrow Checker Integration

Raw pointers bypass borrow checker but require `unsafe` blocks:

```vex
let! x = 42;
let ref_x: &i32! = &x;

// Safe: borrow checker enforced
*ref_x = 100;

// Unsafe: raw pointer bypasses checks
let raw: *i32! = ref_x as *i32!;
unsafe {
    *raw = 200;  // No borrow checker validation
}
```

### Lifetime Safety

References have lifetime bounds, raw pointers do not:

```vex
fn safe_ref<'a>(data: &'a Vec<i32>): &'a i32 {
    return &data[0];  // Lifetime 'a enforced
}

fn unsafe_ptr(data: *Vec<i32>): *i32 {
    unsafe {
        return &(*data)[0] as *i32;  // No lifetime tracking
    }
}
```

### Common Unsafe Patterns

```vex
// Iterator invalidation
unsafe {
    let mut vec = Vec.new<i32>();
    vec.push(1);
    vec.push(2);

    let ptr = &vec[0] as *i32;
    vec.push(3);  // May reallocate, invalidating ptr

    // *ptr now dangling! Undefined behavior
}

// Use-after-free
unsafe {
    let ptr: *i32;
    {
        let x = 42;
        ptr = &x as *i32;
    }  // x dropped here

    // *ptr is now dangling! Undefined behavior
}
```

---

## FFI Integration

### C Interoperability

Raw pointers are essential for C FFI:

```vex
extern "C" {
    fn malloc(size: usize): *u8;
    fn free(ptr: *u8);
    fn memcpy(dest: *u8, src: *u8, n: usize);
}

fn allocate_buffer(size: usize): *u8 {
    unsafe {
        return malloc(size);
    }
}

fn deallocate_buffer(ptr: *u8) {
    unsafe {
        free(ptr);
    }
}
```

### Struct Layout Compatibility

```vex
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
```

---

## Common Patterns

### Safe Wrapper Types

```vex
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

```vex
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

```vex
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

```vex
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
```

---

## Best Practices

### When to Use References

- **Default choice** for function parameters
- **Safe** and enforced by borrow checker
- **Zero-cost** abstractions

### When to Use Raw Pointers

- **FFI boundaries** with C libraries
- **Performance-critical** code requiring manual control
- **Unsafe operations** that bypass borrow checker
- **Low-level system programming**

### Safety Guidelines

1. **Minimize unsafe blocks** - Keep them as small as possible
2. **Validate pointers** - Check for null before dereferencing
3. **Respect lifetimes** - Don't create dangling pointers
4. **Use safe abstractions** - Wrap unsafe code in safe interfaces
5. **Test thoroughly** - Unsafe code needs extensive testing

### Common Pitfalls

```vex
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
    return x;  // Value moved out
}
```

---

**Previous**: [20_Policy_System.md](./20_Policy_System.md)  
**Next**: [22_Advanced_Topics.md](./22_Advanced_Topics.md) (planned)

**Maintained by**: Vex Language Team
