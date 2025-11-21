# Vex SIMD Support

Vex Runtime automatically detects and utilizes SIMD instructions on supported platforms to accelerate critical operations like Hash Map probing, String processing, and Cryptography.

## Supported Architectures

### 1. x86 / x86-64 (Intel & AMD)

- **Instruction Set**: **SSE2** (Streaming SIMD Extensions 2)
- **Status**: ✅ Fully Supported
- **Implementation**: Uses `<immintrin.h>` intrinsics (`_mm_cmpeq_epi8`, `_mm_movemask_epi8`).
- **Why SSE2?**: Vex Swisstable uses **16-byte groups** for metadata control. SSE2 registers (XMM) are exactly 128-bit (16 bytes), making them the perfect fit.
- **Performance**: Hardware-accelerated group probing (16 slots at once).

### 2. ARM64 / AArch64 (Apple Silicon, AWS Graviton, etc.)

- **Instruction Set**: **NEON** (Advanced SIMD)
- **Status**: ✅ Fully Supported & Optimized
- **Implementation**: Uses `<arm_neon.h>` intrinsics.
- **Optimization**: Includes a custom `neon_movemask_u8` implementation to emulate the x86 `movemask` instruction, which is missing on ARM.

## Future Roadmap

### AVX2 (x86-64)

- **Potential**: Could process **32 bytes** (32 slots) at once.
- **Requirement**: Would require changing the Swisstable `GROUP_SIZE` from 16 to 32.
- **Trade-off**: Larger groups might increase cache pressure for small maps. Current 16-byte group is a sweet spot for cache locality.

### AVX-512 (x86-64)

- **Potential**: Could process **64 bytes** (64 slots) at once.
- **Status**: Not currently implemented (diminishing returns for general-purpose maps).

## Verification

To verify SIMD usage in your build:

```c
#include "vex_macros.h"

#if VEX_SIMD_X86
  printf("Using SSE2/AVX\n");
#elif VEX_SIMD_NEON
  printf("Using NEON\n");
#else
  printf("Using Scalar Fallback\n");
#endif
```
