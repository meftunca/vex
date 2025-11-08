# VEX Runtime Central Macro Management

## Overview

The Vex runtime now uses a **centralized macro management system** via `vex_macros.h`. This eliminates macro duplication across source files and provides a single source of truth for all common macros.

## Problem Statement

Previously, macros like `VEX_SIMD_X86`, `VEX_LIKELY`, `VEX_UNLIKELY`, etc. were redefined in multiple files:
- `vex_string.c`
- `vex_url.c`
- `vex_swisstable.c`
- `vex_testing.c`
- And many others...

This caused:
- ❌ Code duplication
- ❌ Inconsistent definitions
- ❌ Difficult maintenance
- ❌ Macro redefinition warnings
- ❌ Hard to update platform support

## Solution: `vex_macros.h`

A single header file (`vex_macros.h`) now defines ALL common macros used across the Vex runtime. This file is automatically included via `vex.h`.

### Architecture

```
┌─────────────┐
│   vex.h     │  ← Main runtime header
└──────┬──────┘
       │ includes
       ▼
┌─────────────────┐
│  vex_macros.h   │  ← Central macro definitions
└─────────────────┘
       │ provides macros to
       ▼
┌──────────────────────────────────────┐
│  All runtime source files            │
│  (vex_string.c, vex_url.c, etc.)     │
└──────────────────────────────────────┘
```

## Macro Categories

### 1. Platform Detection

| Macro | Purpose | Values |
|-------|---------|--------|
| `VEX_OS_LINUX` | Linux detection | 0 or 1 |
| `VEX_OS_MACOS` | macOS detection | 0 or 1 |
| `VEX_OS_WINDOWS` | Windows detection | 0 or 1 |
| `VEX_OS_BSD` | BSD detection | 0 or 1 |
| `VEX_OS_POSIX` | POSIX systems | 0 or 1 |

| Macro | Purpose | Values |
|-------|---------|--------|
| `VEX_ARCH_X86_64` | x86-64 (AMD64) | 0 or 1 |
| `VEX_ARCH_X86_32` | x86 32-bit | 0 or 1 |
| `VEX_ARCH_ARM64` | ARM 64-bit (AArch64) | 0 or 1 |
| `VEX_ARCH_ARM32` | ARM 32-bit | 0 or 1 |
| `VEX_ARCH_X86` | Any x86 (32 or 64) | 0 or 1 |
| `VEX_ARCH_ARM` | Any ARM (32 or 64) | 0 or 1 |

| Macro | Purpose | Values |
|-------|---------|--------|
| `VEX_COMPILER_CLANG` | Clang compiler | 0 or 1 |
| `VEX_COMPILER_GCC` | GCC compiler | 0 or 1 |
| `VEX_COMPILER_MSVC` | MSVC compiler | 0 or 1 |

### 2. SIMD Detection

| Macro | Purpose | Includes |
|-------|---------|----------|
| `VEX_SIMD_X86` | x86/x86-64 SIMD support | `<immintrin.h>` or `<x86intrin.h>` |
| `VEX_SIMD_NEON` | ARM NEON SIMD support | `<arm_neon.h>` |
| `VEX_SIMD_AVAILABLE` | Any SIMD available | Combination of above |

**Note:** `VEX_SIMD_X86` and `VEX_SIMD_NEON` are compile-time detection macros (0 or 1).
The enum `VexSimdLevel` in `vex.h` is for runtime CPU feature detection with values like `VEX_SIMD_LEVEL_AVX2`, `VEX_SIMD_LEVEL_NEON`, etc.

### 3. Compiler Hints & Attributes

#### Branch Prediction
```c
VEX_LIKELY(condition)    // Hint: condition is likely true
VEX_UNLIKELY(condition)  // Hint: condition is likely false
```

**Example:**
```c
if (VEX_LIKELY(x > 0)) {
    // Hot path - optimized for this branch
}
if (VEX_UNLIKELY(error)) {
    // Cold path - error handling
}
```

#### Function Inlining
```c
VEX_INLINE              // Aggressive inline (always_inline)
VEX_FORCE_INLINE        // Force inline (always_inline)
VEX_NO_INLINE           // Prevent inlining
```

#### Code Layout Optimization
```c
VEX_HOT                 // Mark hot function (frequently called)
VEX_COLD                // Mark cold function (rarely called)
VEX_FLATTEN             // Inline all callees
```

#### Function Properties
```c
VEX_PURE                // No side effects, only depends on arguments
VEX_CONST               // No side effects, doesn't read memory
```

#### Memory & Performance
```c
VEX_RESTRICT            // Pointer aliasing hint (__restrict__)
VEX_PREFETCH(ptr, rw, locality)  // Prefetch memory
VEX_BARRIER()           // Compiler memory barrier
VEX_UNREACHABLE()       // Mark unreachable code
```

#### Attributes
```c
VEX_UNUSED              // Suppress unused warnings
VEX_ALIGN(n)            // Align to n bytes
VEX_PACKED              // Packed struct (no padding)
```

### 4. Utility Macros

#### Array & Math
```c
VEX_ARRAY_LEN(arr)      // Array length (compile-time)
VEX_MIN(a, b)           // Minimum of two values
VEX_MAX(a, b)           // Maximum of two values
VEX_CLAMP(x, min, max)  // Clamp value between min and max
```

#### Alignment
```c
VEX_ALIGN_UP(x, align)    // Round up to alignment
VEX_ALIGN_DOWN(x, align)  // Round down to alignment
VEX_IS_POWER_OF_2(x)      // Check if power of 2
```

#### Metaprogramming
```c
VEX_CAT(a, b)           // Token concatenation (a##b)
VEX_STRINGIFY(x)        // Convert to string (#x)
VEX_SWAP(a, b)          // Swap two values
```

#### Constants
```c
VEX_CACHE_LINE_SIZE     // 64 bytes (typical)
```

#### Debugging
```c
VEX_STATIC_ASSERT(cond, msg)  // Compile-time assertion
VEX_DEBUG_ONLY(code)          // Code only in debug builds
```

## Migration Guide

### Before (Old Code)

```c
// vex_string.c
#if defined(__x86_64__) || defined(_M_X64) || defined(__i386__)
  #include <immintrin.h>
  #define VEX_SIMD_X86 1
#else
  #define VEX_SIMD_X86 0
#endif

#if defined(__ARM_NEON) || defined(__ARM_NEON__) || defined(__aarch64__)
  #include <arm_neon.h>
  #define VEX_SIMD_NEON 1
#else
  #define VEX_SIMD_NEON 0
#endif
```

### After (New Code)

```c
// vex_string.c
#include "vex.h"

// vex.h already includes vex_macros.h which provides:
// - VEX_SIMD_X86, VEX_SIMD_NEON (with proper intrinsics)
// - VEX_LIKELY, VEX_UNLIKELY
// - VEX_RESTRICT, VEX_INLINE, etc.
```

### Updated Files

The following files have been updated to use `vex_macros.h`:

✅ **Core Runtime:**
- `vex.h` - Now includes `vex_macros.h`
- `vex_intrinsics.h` - Removed duplicate macros
- `vex_string.c` - Removed SIMD detection
- `vex_url.c` - Removed SIMD detection
- `vex_cpu.c` - Updated enum values (`VEX_SIMD_LEVEL_*`)

✅ **SwissTable:**
- `swisstable/vex_swisstable.c` - Removed duplicate macros

✅ **Tests:**
- All test files now benefit from centralized macros

## Usage Examples

### Example 1: SIMD-Accelerated Code

```c
#include "vex.h"

void process_data(uint8_t *data, size_t len) {
    #if VEX_SIMD_X86
        // Use x86 SIMD (SSE, AVX)
        __m128i zero = _mm_setzero_si128();
        // ... SIMD code ...
    #elif VEX_SIMD_NEON
        // Use ARM NEON
        uint8x16_t zero = vdupq_n_u8(0);
        // ... SIMD code ...
    #else
        // Scalar fallback
        for (size_t i = 0; i < len; i++) {
            // ... scalar code ...
        }
    #endif
}
```

### Example 2: Platform-Specific Code

```c
#include "vex.h"

void init_platform() {
    #if VEX_OS_LINUX
        // Linux-specific initialization
        setup_epoll();
    #elif VEX_OS_MACOS
        // macOS-specific initialization
        setup_kqueue();
    #elif VEX_OS_WINDOWS
        // Windows-specific initialization
        setup_iocp();
    #endif
}
```

### Example 3: Branch Prediction

```c
#include "vex.h"

int parse_token(const char *str) {
    if (VEX_LIKELY(str != NULL)) {
        // Hot path: string is usually valid
        return parse_impl(str);
    }
    
    if (VEX_UNLIKELY(global_error_flag)) {
        // Cold path: error recovery
        handle_error();
    }
    
    return -1;
}
```

### Example 4: Alignment & Performance

```c
#include "vex.h"

// Cache-aligned structure for performance
typedef struct VEX_ALIGN(VEX_CACHE_LINE_SIZE) {
    uint64_t counter;
    uint64_t timestamp;
    // ... more fields ...
} CacheAlignedCounter;

// Hot path function
VEX_HOT
void increment_counter(CacheAlignedCounter *c) {
    VEX_PREFETCH(c, 1, 3);  // Prefetch for write, high locality
    c->counter++;
}
```

## Benefits

### ✅ Single Source of Truth
- All macros defined in one place
- Easy to update and maintain
- No inconsistencies

### ✅ No Duplication
- Reduced code size
- Eliminated redundant definitions
- Cleaner source files

### ✅ Consistent API
- Same macro names everywhere
- Same behavior across files
- Predictable semantics

### ✅ Better Compiler Support
- Handles Clang, GCC, MSVC
- Graceful fallbacks
- Platform-agnostic code

### ✅ Easy to Extend
- Add new platforms easily
- Update SIMD support
- Centralized feature detection

## Technical Details

### Macro Naming Convention

| Prefix | Purpose | Example |
|--------|---------|---------|
| `VEX_OS_*` | Operating system | `VEX_OS_LINUX` |
| `VEX_ARCH_*` | Architecture | `VEX_ARCH_X86_64` |
| `VEX_COMPILER_*` | Compiler | `VEX_COMPILER_CLANG` |
| `VEX_SIMD_*` | SIMD detection (compile-time) | `VEX_SIMD_X86` |
| `VEX_SIMD_LEVEL_*` | SIMD level (runtime enum) | `VEX_SIMD_LEVEL_AVX2` |
| `VEX_*` | General utilities | `VEX_LIKELY`, `VEX_MIN` |

### Include Order

```c
// Correct order:
#include "vex.h"  // Includes vex_macros.h automatically

// Now you can use:
// - VEX_SIMD_X86, VEX_SIMD_NEON
// - VEX_LIKELY, VEX_UNLIKELY
// - VEX_MIN, VEX_MAX
// - All other VEX_* macros
```

### Compatibility

- **C Standard:** C11+ (some features work on C99)
- **Compilers:** Clang, GCC, MSVC
- **Platforms:** Linux, macOS, Windows, BSD
- **Architectures:** x86-64, x86, ARM64, ARM32

## Testing

All tests pass with the new macro system:

```bash
# Test string operations (uses VEX_SIMD_X86/NEON)
make test-unit

# Test SwissTable (uses VEX_LIKELY/UNLIKELY/RESTRICT)
make test-swisstable

# Test vex_time (uses platform detection)
make test-vextime
```

## Version

**vex_macros.h Version:** 1.0.0

## Future Enhancements

Potential future additions:
- RISC-V detection (`VEX_ARCH_RISCV`)
- WebAssembly detection (`VEX_ARCH_WASM`)
- More SIMD ISAs (AVX-512 variants, SVE2)
- Memory ordering hints
- TLS (Thread-Local Storage) macros

## Summary

The Vex runtime now has a **production-ready, centralized macro management system** that:
- ✅ Eliminates code duplication
- ✅ Provides consistent behavior
- ✅ Simplifies maintenance
- ✅ Supports all major platforms
- ✅ Makes SIMD code portable

**All existing code continues to work**, but now benefits from the centralized definitions in `vex_macros.h`.

🚀 **Vex runtime is now more maintainable and scalable!**

