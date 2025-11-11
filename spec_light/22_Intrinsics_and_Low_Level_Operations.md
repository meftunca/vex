# Intrinsics and Low-Level Operations

Version: 0.1.2
Last Updated: November 9, 2025

This document describes Vex's support for low-level operations, compiler intrinsics, and platform-specific features.

---

## Bit Manipulation Intrinsics

Vex provides LLVM bit manipulation intrinsics for high-performance low-level operations.

### Available Intrinsics

### Count Leading Zeros: `ctlz(x)`

Counts the number of leading zero bits in an integer.

```vex
let x: u32 = 0b00001111_00000000_00000000_00000000; // 0x0F000000
let zeros = ctlz(x); // Returns 4 (leading zeros before first 1)
```

Supported Types: `u8`, `u16`, `u32`, `u64`, `u128`

### Count Trailing Zeros: `cttz(x)`

Counts the number of trailing zero bits in an integer.

```vex
let x: u32 = 0b11110000_00000000_00000000_00000000; // 0xF0000000
let zeros = cttz(x); // Returns 24 (trailing zeros)
```

Supported Types: `u8`, `u16`, `u32`, `u64`, `u128`

### Population Count: `popcnt(x)`

Counts the number of set bits (1s) in an integer.

```vex
let x: u32 = 0b10110100_11100011_00001111_00000000; // 0xB4E30F00
let count = popcnt(x); // Returns 12 (number of 1 bits)
```

Supported Types: `u8`, `u16`, `u32`, `u64`, `u128`

### Bit Reverse: `bitreverse(x)`

Reverses the bits in an integer.

```vex
let x: u8 = 0b11010011; // 0xD3
let reversed = bitreverse(x); // Returns 0b11001011 (0xCB)
```

Supported Types: `u8`, `u16`, `u32`, `u64`, `u128`

### Bit Swap: `bswap(x)`

Swaps the bytes in an integer (endianness conversion).

```vex
let x: u32 = 0x12345678;
let swapped = bswap(x); // Returns 0x78563412
```

Supported Types: `u16`, `u32`, `u64`, `u128`

### Overflow Checks

Signed Addition Overflow: `sadd_overflow(a, b)` → `(result, overflow)`

```vex
let (result, overflow) = sadd_overflow(i32::MAX, 1);
if overflow {
    // Handle overflow
}
```

Signed Subtraction Overflow: `ssub_overflow(a, b)` → `(result, overflow)`

Signed Multiplication Overflow: `smul_overflow(a, b)` → `(result, overflow)`

Supported Types: `i8`, `i16`, `i32`, `i64`, `i128`

### Memory Operations

### Memory Allocation: `alloc(size)` → `*u8`

Allocates `size` bytes of memory.

```vex
unsafe {
    let ptr: *u8 = alloc(1024); // Allocate 1KB
    // Use ptr...
    free(ptr);
}
```

### Memory Free: `free(ptr)`

Frees previously allocated memory.

### Memory Realloc: `realloc(ptr, new_size)` → `*u8`

Resizes allocated memory block.

### Memory Move: `memmove(dst, src, count)`

Copies `count` bytes from `src` to `dst`, handling overlapping regions.

```vex
unsafe {
    let buffer: *u8 = alloc(1024);
    // Move overlapping regions
    memmove(buffer.add(10), buffer, 100);
}
```

### Type Information

### Size Of: `sizeof<T>()` → `usize`

Returns the size in bytes of type `T`.

```vex
let size = sizeof<i32>();     // Returns 4
let arr_size = sizeof<[i32; 10]>(); // Returns 40
```

### Align Of: `alignof<T>()` → `usize`

Returns the alignment in bytes of type `T`.

```vex
let align = alignof<i64>();   // Returns 8
```

### Array Operations

### Array Length: `array_len(arr)` → `usize`

Returns the length of an array.

```vex
let arr: [i32; 5] = [1, 2, 3, 4, 5];
let len = array_len(arr); // Returns 5
```

### UTF-8 Operations

### UTF-8 Validation: `utf8_valid(str)` → `bool`

Checks if a string contains valid UTF-8.

```vex
let valid = utf8_valid("Hello, 世界!"); // Returns true
let invalid = utf8_valid(&[0xFF, 0xFF]); // Returns false
```

### UTF-8 Character Count: `utf8charcount(str)` → `usize`

Returns the number of Unicode characters in a UTF-8 string.

```vex
let count = utf8_char_count("Hello, 世界!"); // Returns 9 (not byte count)
```

### UTF-8 Character At: `utf8charat(str, index)` → `u32`

Returns the Unicode codepoint at the specified character index.

```vex
let codepoint = utf8_char_at("Hello, 世界!", 7); // Returns '世' (19990)
```

### Launch Expression (Future)

Syntax: `launch funcgrid_dims`

Launches a function on parallel compute units (GPU, HPC clusters).

```vex
// Launch kernel on GPU
launch vector_add[1024, 1](a, b, result);

// Multi-dimensional grid
launch matrix_mul[32, 32](matrix_a, matrix_b, output);
```

Status: AST support exists, implementation pending.

Note: This is a planned feature for high-performance computing integration.