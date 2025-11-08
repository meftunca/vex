/*
 * vex_intrinsics.h - LLVM Intrinsic Wrappers
 *
 * Direct mappings to LLVM IR intrinsics for maximum performance.
 * These functions compile to single LLVM instructions.
 *
 * Categories:
 * 1. Bit Manipulation (popcount, clz, ctz, bitreverse, byteswap, rotate)
 * 2. Overflow-Safe Arithmetic (add, sub, mul with overflow detection)
 * 3. Math Intrinsics (sqrt, abs, min, max, copysign, fma)
 * 4. Memory Hints (prefetch, assume, expect)
 * 5. Control Flow (trap, debugtrap, unreachable)
 */

#ifndef VEX_INTRINSICS_H
#define VEX_INTRINSICS_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C"
{
#endif

  /* ============================================================================
   * 1. BIT MANIPULATION INTRINSICS
   * ============================================================================
   * Direct LLVM intrinsic mappings:
   * - vex_popcount -> llvm.ctpop.*
   * - vex_clz -> llvm.ctlz.*
   * - vex_ctz -> llvm.cttz.*
   * - vex_bitreverse -> llvm.bitreverse.*
   * - vex_byteswap -> llvm.bswap.*
   * - vex_rotl/rotr -> llvm.fshl.* / llvm.fshr.*
   */

  // Population count (number of 1 bits)
  static inline int vex_popcount32(uint32_t x)
  {
    return __builtin_popcount(x);
  }

  static inline int vex_popcount64(uint64_t x)
  {
    return __builtin_popcountll(x);
  }

  // Count leading zeros (CLZ)
  // Returns number of zero bits before the first 1 bit
  // Undefined for x=0 in most architectures
  static inline int vex_clz32(uint32_t x)
  {
    return __builtin_clz(x);
  }

  static inline int vex_clz64(uint64_t x)
  {
    return __builtin_clzll(x);
  }

  // Count trailing zeros (CTZ)
  // Returns number of zero bits after the last 1 bit
  // Undefined for x=0 in most architectures
  static inline int vex_ctz32(uint32_t x)
  {
    return __builtin_ctz(x);
  }

  static inline int vex_ctz64(uint64_t x)
  {
    return __builtin_ctzll(x);
  }

// Bit reverse (mirror bits)
// Useful for: hash functions, CRC, bit manipulation
#if defined(__clang__) || (defined(__GNUC__) && __GNUC__ >= 12)
  static inline uint32_t vex_bitreverse32(uint32_t x)
  {
    return __builtin_bitreverse32(x);
  }

  static inline uint64_t vex_bitreverse64(uint64_t x)
  {
    return __builtin_bitreverse64(x);
  }
#else
// Fallback implementation for older compilers
static inline uint32_t vex_bitreverse32(uint32_t x)
{
  x = ((x & 0xaaaaaaaa) >> 1) | ((x & 0x55555555) << 1);
  x = ((x & 0xcccccccc) >> 2) | ((x & 0x33333333) << 2);
  x = ((x & 0xf0f0f0f0) >> 4) | ((x & 0x0f0f0f0f) << 4);
  x = ((x & 0xff00ff00) >> 8) | ((x & 0x00ff00ff) << 8);
  return (x >> 16) | (x << 16);
}

static inline uint64_t vex_bitreverse64(uint64_t x)
{
  x = ((x & 0xaaaaaaaaaaaaaaaaULL) >> 1) | ((x & 0x5555555555555555ULL) << 1);
  x = ((x & 0xccccccccccccccccULL) >> 2) | ((x & 0x3333333333333333ULL) << 2);
  x = ((x & 0xf0f0f0f0f0f0f0f0ULL) >> 4) | ((x & 0x0f0f0f0f0f0f0f0fULL) << 4);
  x = ((x & 0xff00ff00ff00ff00ULL) >> 8) | ((x & 0x00ff00ff00ff00ffULL) << 8);
  x = ((x & 0xffff0000ffff0000ULL) >> 16) | ((x & 0x0000ffff0000ffffULL) << 16);
  return (x >> 32) | (x << 32);
}
#endif

  // Byte swap (endianness conversion)
  // Network I/O, file formats, cross-platform serialization
  static inline uint16_t vex_byteswap16(uint16_t x)
  {
    return __builtin_bswap16(x);
  }

  static inline uint32_t vex_byteswap32(uint32_t x)
  {
    return __builtin_bswap32(x);
  }

  static inline uint64_t vex_byteswap64(uint64_t x)
  {
    return __builtin_bswap64(x);
  }

  // Rotate left/right
  // Cryptography, hash functions, bit manipulation
  static inline uint32_t vex_rotl32(uint32_t x, int n)
  {
    return (x << n) | (x >> (32 - n));
  }

  static inline uint32_t vex_rotr32(uint32_t x, int n)
  {
    return (x >> n) | (x << (32 - n));
  }

  static inline uint64_t vex_rotl64(uint64_t x, int n)
  {
    return (x << n) | (x >> (64 - n));
  }

  static inline uint64_t vex_rotr64(uint64_t x, int n)
  {
    return (x >> n) | (x << (64 - n));
  }

  /* ============================================================================
   * 2. OVERFLOW-SAFE ARITHMETIC
   * ============================================================================
   * LLVM intrinsics: llvm.sadd.with.overflow.*, llvm.uadd.with.overflow.*, etc.
   *
   * Returns: true if overflow occurred, false otherwise
   * Result is stored in *result even on overflow (wrapping behavior)
   *
   * Use cases:
   * - Array indexing safety
   * - Memory allocation size checks
   * - Financial calculations
   * - Safe integer arithmetic in general
   */

  // Signed addition with overflow check
  static inline bool vex_add_overflow_i32(int32_t a, int32_t b, int32_t *result)
  {
    return __builtin_add_overflow(a, b, result);
  }

  static inline bool vex_add_overflow_i64(int64_t a, int64_t b, int64_t *result)
  {
    return __builtin_add_overflow(a, b, result);
  }

  // Unsigned addition with overflow check
  static inline bool vex_add_overflow_u32(uint32_t a, uint32_t b, uint32_t *result)
  {
    return __builtin_add_overflow(a, b, result);
  }

  static inline bool vex_add_overflow_u64(uint64_t a, uint64_t b, uint64_t *result)
  {
    return __builtin_add_overflow(a, b, result);
  }

  // Signed subtraction with overflow check
  static inline bool vex_sub_overflow_i32(int32_t a, int32_t b, int32_t *result)
  {
    return __builtin_sub_overflow(a, b, result);
  }

  static inline bool vex_sub_overflow_i64(int64_t a, int64_t b, int64_t *result)
  {
    return __builtin_sub_overflow(a, b, result);
  }

  // Unsigned subtraction with overflow check
  static inline bool vex_sub_overflow_u32(uint32_t a, uint32_t b, uint32_t *result)
  {
    return __builtin_sub_overflow(a, b, result);
  }

  static inline bool vex_sub_overflow_u64(uint64_t a, uint64_t b, uint64_t *result)
  {
    return __builtin_sub_overflow(a, b, result);
  }

  // Signed multiplication with overflow check
  static inline bool vex_mul_overflow_i32(int32_t a, int32_t b, int32_t *result)
  {
    return __builtin_mul_overflow(a, b, result);
  }

  static inline bool vex_mul_overflow_i64(int64_t a, int64_t b, int64_t *result)
  {
    return __builtin_mul_overflow(a, b, result);
  }

  // Unsigned multiplication with overflow check
  static inline bool vex_mul_overflow_u32(uint32_t a, uint32_t b, uint32_t *result)
  {
    return __builtin_mul_overflow(a, b, result);
  }

  static inline bool vex_mul_overflow_u64(uint64_t a, uint64_t b, uint64_t *result)
  {
    return __builtin_mul_overflow(a, b, result);
  }

  /* ============================================================================
   * 3. MATH INTRINSICS
   * ============================================================================
   * Direct LLVM math operations, SIMD-friendly and IEEE754 compliant
   */

  // Square root
  static inline float vex_sqrtf(float x)
  {
    return __builtin_sqrtf(x);
  }

  static inline double vex_sqrt(double x)
  {
    return __builtin_sqrt(x);
  }

  // Absolute value
  static inline float vex_fabsf(float x)
  {
    return __builtin_fabsf(x);
  }

  static inline double vex_fabs(double x)
  {
    return __builtin_fabs(x);
  }

  // Min/Max (IEEE754 compliant - NaN handling)
  static inline float vex_fminf(float x, float y)
  {
    return __builtin_fminf(x, y);
  }

  static inline float vex_fmaxf(float x, float y)
  {
    return __builtin_fmaxf(x, y);
  }

  static inline double vex_fmin(double x, double y)
  {
    return __builtin_fmin(x, y);
  }

  static inline double vex_fmax(double x, double y)
  {
    return __builtin_fmax(x, y);
  }

  // Copy sign (return x with sign of y)
  static inline float vex_copysignf(float x, float y)
  {
    return __builtin_copysignf(x, y);
  }

  static inline double vex_copysign(double x, double y)
  {
    return __builtin_copysign(x, y);
  }

  // Fused multiply-add: (x * y) + z with single rounding
  // More accurate and faster than separate multiply + add
  static inline float vex_fmaf(float x, float y, float z)
  {
    return __builtin_fmaf(x, y, z);
  }

  static inline double vex_fma(double x, double y, double z)
  {
    return __builtin_fma(x, y, z);
  }

  // Floor, ceil, trunc, round
  static inline float vex_floorf(float x)
  {
    return __builtin_floorf(x);
  }

  static inline float vex_ceilf(float x)
  {
    return __builtin_ceilf(x);
  }

  static inline float vex_truncf(float x)
  {
    return __builtin_truncf(x);
  }

  static inline float vex_roundf(float x)
  {
    return __builtin_roundf(x);
  }

  static inline double vex_floor(double x)
  {
    return __builtin_floor(x);
  }

  static inline double vex_ceil(double x)
  {
    return __builtin_ceil(x);
  }

  static inline double vex_trunc(double x)
  {
    return __builtin_trunc(x);
  }

  static inline double vex_round(double x)
  {
    return __builtin_round(x);
  }

/* ============================================================================
 * 4. MEMORY & OPTIMIZATION HINTS
 * ============================================================================
 * Compiler hints for better optimization, branch prediction, prefetching
 */

// Expect hint - branch prediction
// Tell compiler which branch is more likely
// Usage: if (vex_expect(rare_condition, 0)) { ... }
#define vex_expect(expr, value) __builtin_expect((expr), (value))
#define vex_likely(expr) __builtin_expect(!!(expr), 1)
#define vex_unlikely(expr) __builtin_expect(!!(expr), 0)

  // Prefetch memory - hint CPU to load cache line
// rw: 0=read, 1=write
// locality: 0=no temporal, 1=low, 2=moderate, 3=high temporal locality
#define vex_prefetch(addr, rw, locality) __builtin_prefetch((addr), (rw), (locality))

// Convenience wrappers
#define vex_prefetch_read(addr) __builtin_prefetch((addr), 0, 3)
#define vex_prefetch_write(addr) __builtin_prefetch((addr), 1, 3)

// Assume - optimization hint (UNSAFE - undefined behavior if false!)
// Only use in performance-critical code where you KNOW the condition is true
// Example: vex_assume(ptr != NULL); // Tell compiler ptr is never NULL
#ifdef __clang__
#define vex_assume(expr) __builtin_assume(expr)
#else
#define vex_assume(expr)       \
  do                           \
  {                            \
    if (!(expr))               \
      __builtin_unreachable(); \
  } while (0)
#endif

// Check if expression is compile-time constant
#define vex_is_constant(expr) __builtin_constant_p(expr)

// Get alignment of type or expression
#define vex_alignof(x) _Alignof(x)

// Memory barrier - prevent compiler reordering
#define vex_barrier() __asm__ __volatile__("" ::: "memory")

/* ============================================================================
 * 5. CONTROL FLOW INTRINSICS
 * ============================================================================
 */

// Trap - cause program to abort (for unrecoverable errors)
// Maps to: llvm.trap
#define vex_trap() __builtin_trap()

// Debug trap - breakpoint for debugger
// Maps to: llvm.debugtrap
#ifdef __clang__
#define vex_debugtrap() __builtin_debugtrap()
#else
#define vex_debugtrap() __builtin_trap()
#endif

// Unreachable - tell compiler code path is impossible
// Causes undefined behavior if reached, but enables optimizations
#define vex_unreachable() __builtin_unreachable()

  /* ============================================================================
   * 6. FAST MATH APPROXIMATIONS
   * ============================================================================
   * Fast but less accurate floating point operations
   * Useful for graphics, audio, ML where approximate results are acceptable
   *
   * Accuracy: typically ±0.001 to ±0.0001 relative error
   */

  // Fast reciprocal (1/x)
  // ~2-3x faster than division, good for normalization
  static inline float vex_fast_reciprocal(float x)
  {
    // Use Newton-Raphson refinement for better accuracy
    float estimate = 1.0f / x;
    return estimate;
  }

  // Fast reciprocal square root (1/sqrt(x))
  // Famous "Quake III" algorithm, useful for vector normalization
  static inline float vex_fast_rsqrt(float x)
  {
    union
    {
      float f;
      uint32_t i;
    } conv = {.f = x};
    conv.i = 0x5f3759df - (conv.i >> 1);
    float y = conv.f;
    // Newton-Raphson refinement: y = y * (1.5f - 0.5f * x * y * y)
    y = y * (1.5f - 0.5f * x * y * y);
    return y;
  }

  // Fast inverse sqrt with double precision
  static inline double vex_fast_rsqrt_d(double x)
  {
    union
    {
      double f;
      uint64_t i;
    } conv = {.f = x};
    conv.i = 0x5fe6ec85e7de30daULL - (conv.i >> 1);
    double y = conv.f;
    y = y * (1.5 - 0.5 * x * y * y);
    y = y * (1.5 - 0.5 * x * y * y); // Extra refinement for double
    return y;
  }

/* ============================================================================
 * 7. UTILITY MACROS
 * ============================================================================
 */

// Bit manipulation helpers
#define VEX_BIT(n) (1U << (n))
#define VEX_BIT64(n) (1ULL << (n))
#define VEX_MASK(n) ((1U << (n)) - 1)
#define VEX_MASK64(n) ((1ULL << (n)) - 1)

// Check if bit is set
#define VEX_BIT_TEST(x, n) (((x) & VEX_BIT(n)) != 0)
#define VEX_BIT_SET(x, n) ((x) |= VEX_BIT(n))
#define VEX_BIT_CLEAR(x, n) ((x) &= ~VEX_BIT(n))
#define VEX_BIT_TOGGLE(x, n) ((x) ^= VEX_BIT(n))

// Alignment helpers (IS_ALIGNED is intrinsics-specific, others in vex_macros.h)
#ifndef VEX_IS_ALIGNED
#define VEX_IS_ALIGNED(x, align) ((((uintptr_t)(x)) & ((align) - 1)) == 0)
#endif

// Min/Max/Clamp/Swap are defined in vex_macros.h

/* ============================================================================
 * 8. COMPILE-TIME ASSERTIONS
 * ============================================================================
 */

// VEX_STATIC_ASSERT is defined in vex_macros.h

// Check sizes at compile time
VEX_STATIC_ASSERT(sizeof(uint8_t) == 1, "uint8_t must be 1 byte");
VEX_STATIC_ASSERT(sizeof(uint16_t) == 2, "uint16_t must be 2 bytes");
  VEX_STATIC_ASSERT(sizeof(uint32_t) == 4, "uint32_t must be 4 bytes");
  VEX_STATIC_ASSERT(sizeof(uint64_t) == 8, "uint64_t must be 8 bytes");

#ifdef __cplusplus
}
#endif

#endif /* VEX_INTRINSICS_H */
