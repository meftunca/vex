# Intrinsics and Low-Level Operations

**Version:** 0.9.2
**Last Updated:** November 9, 2025

This document describes Vex's support for low-level operations, compiler intrinsics, and platform-specific features.

---

## Table of Contents

1. [Bit Manipulation Intrinsics](#bit-manipulation-intrinsics)
2. [CPU Feature Detection](#cpu-feature-detection)
3. [Memory Intrinsics](#memory-intrinsics)
4. [SIMD Intrinsics](#simd-intrinsics)
5. [Platform-Specific Features](#platform-specific-features)

---

## Bit Manipulation Intrinsics

Vex provides LLVM bit manipulation intrinsics for high-performance low-level operations.

### Available Intrinsics

#### Count Leading Zeros: `ctlz(x)`

Counts the number of leading zero bits in an integer.

```vex
let x: u32 = 0b00001111_00000000_00000000_00000000; // 0x0F000000
let zeros = ctlz(x); // Returns 4 (leading zeros before first 1)
```

**Supported Types**: `u8`, `u16`, `u32`, `u64`, `u128`

#### Count Trailing Zeros: `cttz(x)`

Counts the number of trailing zero bits in an integer.

```vex
let x: u32 = 0b11110000_00000000_00000000_00000000; // 0xF0000000
let zeros = cttz(x); // Returns 24 (trailing zeros)
```

**Supported Types**: `u8`, `u16`, `u32`, `u64`, `u128`

#### Population Count: `popcnt(x)`

Counts the number of set bits (1s) in an integer.

```vex
let x: u32 = 0b10110100_11100011_00001111_00000000; // 0xB4E30F00
let count = popcnt(x); // Returns 12 (number of 1 bits)
```

**Supported Types**: `u8`, `u16`, `u32`, `u64`, `u128`

#### Bit Reverse: `bitreverse(x)`

Reverses the bits in an integer.

```vex
let x: u8 = 0b11010011; // 0xD3
let reversed = bitreverse(x); // Returns 0b11001011 (0xCB)
```

**Supported Types**: `u8`, `u16`, `u32`, `u64`, `u128`

### Usage Examples

```vex
fn is_power_of_two(x: u32): bool {
    return popcnt(x) == 1;
}

fn highest_set_bit(x: u32): u32 {
    return 31 - clz(x);
}

fn lowest_set_bit(x: u32): u32 {
    return cttz(x);
}
```

---

## CPU Feature Detection

Vex provides runtime CPU feature detection for SIMD and other processor capabilities.

### Architecture Support

#### x86/x64 Features

```vex
// Check for SIMD support
if cpu_has_sse() {
    // SSE instructions available
}

if cpu_has_avx2() {
    // AVX2 instructions available
}

if cpu_has_avx512f() {
    // AVX-512 foundation available
}
```

**Available Checks**:
- `cpu_has_sse()`, `cpu_has_sse2()`, `cpu_has_sse3()`
- `cpu_has_ssse3()`, `cpu_has_sse4_1()`, `cpu_has_sse4_2()`
- `cpu_has_avx()`, `cpu_has_avx2()`
- `cpu_has_avx512f()`, `cpu_has_avx512bw()`, `cpu_has_avx512vl()`
- `cpu_has_fma()`, `cpu_has_bmi1()`, `cpu_has_bmi2()`
- `cpu_has_popcnt()`, `cpu_has_aes()`

#### ARM/NEON Features

```vex
if cpu_has_neon() {
    // ARM NEON SIMD available
}
```

### Usage in Code

```vex
fn optimized_sum(a: [f32; 8], b: [f32; 8]): [f32; 8] {
    if cpu_has_avx() {
        // Use AVX-optimized path
        return a + b; // Compiler generates AVX instructions
    } else {
        // Fallback to scalar operations
        let mut result: [f32; 8] = [0.0; 8];
        for i in 0..8 {
            result[i] = a[i] + b[i];
        }
        return result;
    }
}
```

---

## Memory Intrinsics

Low-level memory operations with guaranteed semantics.

### Memory Copy: `memcpy(dst, src, count)`

Copies `count` bytes from `src` to `dst`.

```vex
unsafe {
    let src: *u8 = get_source_ptr();
    let dst: *u8 = get_dest_ptr();
    memcpy(dst, src, 1024); // Copy 1KB
}
```

### Memory Set: `memset(ptr, value, count)`

Sets `count` bytes starting at `ptr` to `value`.

```vex
unsafe {
    let buffer: *u8 = alloc_buffer(1024);
    memset(buffer, 0, 1024); // Zero 1KB buffer
}
```

### Memory Compare: `memcmp(a, b, count)`

Compares `count` bytes at `a` and `b`.

```vex
unsafe {
    let result = memcmp(ptr1, ptr2, 64);
    if result == 0 {
        // Memory regions are identical
    }
}
```

---

## SIMD Intrinsics

Direct access to SIMD operations when automatic vectorization is insufficient.

### Vector Types

```vex
// 128-bit vectors (4 x f32 or 16 x u8)
type vec4f32 = [f32; 4];
type vec16u8 = [u8; 16];

// 256-bit vectors (8 x f32 or 32 x u8)
type vec8f32 = [f32; 8];
type vec32u8 = [u8; 32];

// 512-bit vectors (16 x f32 or 64 x u8)
type vec16f32 = [f32; 16];
type vec64u8 = [u8; 64];
```

### SIMD Operations

```vex
fn manual_simd_add(a: vec8f32, b: vec8f32): vec8f32 {
    unsafe {
        // Direct SIMD addition
        return simd_add_f32x8(a, b);
    }
}

fn manual_simd_shuffle(v: vec8f32): vec8f32 {
    unsafe {
        // Shuffle elements: [0,2,4,6,1,3,5,7]
        return simd_shuffle_f32x8(v, [0,2,4,6,1,3,5,7]);
    }
}
```

### Available SIMD Intrinsics

**Arithmetic**: `simd_add_*`, `simd_sub_*`, `simd_mul_*`, `simd_div_*`
**Comparison**: `simd_eq_*`, `simd_lt_*`, `simd_gt_*`
**Shuffle**: `simd_shuffle_*`
**Load/Store**: `simd_load_*`, `simd_store_*`

---

## Platform-Specific Features

### Compiler Target Detection

```vex
// Compile-time target detection
#[cfg(target_arch = "x86_64")]
fn x86_optimized() { /* ... */ }

#[cfg(target_arch = "aarch64")]
fn arm_optimized() { /* ... */ }
```

### Endianness

```vex
if is_little_endian() {
    // Little-endian system
} else {
    // Big-endian system
}
```

### Atomic Operations

```vex
// Atomic load/store
let value = atomic_load(ptr);
atomic_store(ptr, new_value);

// Atomic arithmetic
let old = atomic_add(ptr, 1);
let old = atomic_sub(ptr, 1);
```

---

## Implementation Status

| Feature | Status | Location |
|---------|--------|----------|
| Bit Manipulation | ✅ Complete | `vex-compiler/src/codegen_ast/builtins/intrinsics.rs` |
| CPU Detection | ✅ Complete | `vex-runtime/c/vex_cpu.c` |
| Memory Intrinsics | ✅ Complete | `vex-runtime/c/vex_memory.c` |
| SIMD Intrinsics | 🚧 Partial | `vex-runtime/c/` (basic support) |
| Atomic Operations | ✅ Complete | `vex-runtime/c/vex_sync.c` |

---

## Examples

### Bit Manipulation

```vex
fn bit_operations_demo(): i32 {
    let x: u32 = 0b11010110_00000000_00000000_00000000;

    let leading = ctlz(x);      // Count leading zeros
    let trailing = cttz(x);     // Count trailing zeros
    let population = popcnt(x); // Count set bits
    let reversed = bitreverse(x); // Reverse bits

    return leading + trailing + population;
}
```

### CPU Feature Detection

```vex
fn vectorize_if_possible(data: [f32; 1000]): [f32; 1000] {
    if cpu_has_avx2() {
        // Use AVX2 SIMD operations
        return data * 2.0; // Compiler generates AVX2
    } else if cpu_has_sse() {
        // Fallback to SSE
        return data * 2.0; // Compiler generates SSE
    } else {
        // Scalar fallback
        let mut result: [f32; 1000] = [0.0; 1000];
        for i in 0..1000 {
            result[i] = data[i] * 2.0;
        }
        return result;
    }
}
```

### Memory Operations

```vex
unsafe fn buffer_operations() {
    // Allocate buffer
    let buffer: *u8 = malloc(1024);

    // Zero it
    memset(buffer, 0, 1024);

    // Copy data
    let source: *u8 = get_source_data();
    memcpy(buffer, source, 512);

    // Compare regions
    if memcmp(buffer, source, 512) == 0 {
        print("Copy successful");
    }

    free(buffer);
}
```

---

## Safety Considerations

All intrinsics and low-level operations must be used within `unsafe` blocks:

```vex
// ❌ Compile error - intrinsics require unsafe
let zeros = ctlz(x);

// ✅ OK - in unsafe block
unsafe {
    let zeros = ctlz(x);
}
```

**Rationale**: These operations bypass normal safety guarantees and require careful usage.

---

## Performance Notes

- **Intrinsics**: Direct LLVM intrinsic calls - zero overhead
- **CPU Detection**: Cached at startup - negligible runtime cost
- **SIMD**: Automatic vectorization preferred over manual SIMD
- **Memory**: Use high-level operations when possible

---

*This document covers low-level operations available in Vex. Use with caution and only when necessary.*