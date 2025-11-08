/**
 * VEX Runtime Macros - Central Macro Management
 * 
 * This header provides a single source of truth for all common macros
 * used across the Vex runtime. Include this file instead of redefining
 * macros in individual source files.
 * 
 * Categories:
 *   - Platform Detection
 *   - SIMD Detection
 *   - Compiler Hints & Attributes
 *   - Utility Macros
 */

#ifndef VEX_MACROS_H
#define VEX_MACROS_H

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

// ============================================================================
// PLATFORM DETECTION
// ============================================================================

/**
 * Operating System Detection
 */
#if defined(__linux__)
  #define VEX_OS_LINUX 1
  #define VEX_OS_MACOS 0
  #define VEX_OS_WINDOWS 0
  #define VEX_OS_BSD 0
#elif defined(__APPLE__) && defined(__MACH__)
  #define VEX_OS_LINUX 0
  #define VEX_OS_MACOS 1
  #define VEX_OS_WINDOWS 0
  #define VEX_OS_BSD 0
#elif defined(_WIN32) || defined(_WIN64)
  #define VEX_OS_LINUX 0
  #define VEX_OS_MACOS 0
  #define VEX_OS_WINDOWS 1
  #define VEX_OS_BSD 0
#elif defined(__FreeBSD__) || defined(__OpenBSD__) || defined(__NetBSD__)
  #define VEX_OS_LINUX 0
  #define VEX_OS_MACOS 0
  #define VEX_OS_WINDOWS 0
  #define VEX_OS_BSD 1
#else
  #define VEX_OS_LINUX 0
  #define VEX_OS_MACOS 0
  #define VEX_OS_WINDOWS 0
  #define VEX_OS_BSD 0
#endif

/**
 * POSIX-like systems (Unix, Linux, macOS, BSD)
 */
#define VEX_OS_POSIX (VEX_OS_LINUX || VEX_OS_MACOS || VEX_OS_BSD)

/**
 * Architecture Detection
 */
#if defined(__x86_64__) || defined(_M_X64)
  #define VEX_ARCH_X86_64 1
  #define VEX_ARCH_X86_32 0
  #define VEX_ARCH_ARM64 0
  #define VEX_ARCH_ARM32 0
#elif defined(__i386__) || defined(_M_IX86)
  #define VEX_ARCH_X86_64 0
  #define VEX_ARCH_X86_32 1
  #define VEX_ARCH_ARM64 0
  #define VEX_ARCH_ARM32 0
#elif defined(__aarch64__) || defined(_M_ARM64)
  #define VEX_ARCH_X86_64 0
  #define VEX_ARCH_X86_32 0
  #define VEX_ARCH_ARM64 1
  #define VEX_ARCH_ARM32 0
#elif defined(__arm__) || defined(_M_ARM)
  #define VEX_ARCH_X86_64 0
  #define VEX_ARCH_X86_32 0
  #define VEX_ARCH_ARM64 0
  #define VEX_ARCH_ARM32 1
#else
  #define VEX_ARCH_X86_64 0
  #define VEX_ARCH_X86_32 0
  #define VEX_ARCH_ARM64 0
  #define VEX_ARCH_ARM32 0
#endif

// Combined arch groups
#define VEX_ARCH_X86 (VEX_ARCH_X86_64 || VEX_ARCH_X86_32)
#define VEX_ARCH_ARM (VEX_ARCH_ARM64 || VEX_ARCH_ARM32)

/**
 * Compiler Detection
 */
#if defined(__clang__)
  #define VEX_COMPILER_CLANG 1
  #define VEX_COMPILER_GCC 0
  #define VEX_COMPILER_MSVC 0
#elif defined(__GNUC__) && !defined(__clang__)
  #define VEX_COMPILER_CLANG 0
  #define VEX_COMPILER_GCC 1
  #define VEX_COMPILER_MSVC 0
#elif defined(_MSC_VER)
  #define VEX_COMPILER_CLANG 0
  #define VEX_COMPILER_GCC 0
  #define VEX_COMPILER_MSVC 1
#else
  #define VEX_COMPILER_CLANG 0
  #define VEX_COMPILER_GCC 0
  #define VEX_COMPILER_MSVC 0
#endif

// ============================================================================
// SIMD DETECTION & INTRINSICS
// ============================================================================

/**
 * x86/x86_64 SIMD (SSE, AVX, AVX2, AVX-512)
 */
#if VEX_ARCH_X86
  #if defined(__has_include)
    #if __has_include(<x86intrin.h>)
      #include <x86intrin.h>
      #define VEX_SIMD_X86 1
    #elif __has_include(<immintrin.h>)
      #include <immintrin.h>
      #define VEX_SIMD_X86 1
    #else
      #define VEX_SIMD_X86 0
    #endif
  #elif defined(_MSC_VER)
    #include <intrin.h>
    #define VEX_SIMD_X86 1
  #else
    #define VEX_SIMD_X86 0
  #endif
#else
  #define VEX_SIMD_X86 0
#endif

/**
 * ARM NEON SIMD
 */
#if VEX_ARCH_ARM
  #if defined(__ARM_NEON) || defined(__ARM_NEON__)
    #include <arm_neon.h>
    #define VEX_SIMD_NEON 1
  #elif VEX_ARCH_ARM64
    // ARM64 always has NEON
    #include <arm_neon.h>
    #define VEX_SIMD_NEON 1
  #else
    #define VEX_SIMD_NEON 0
  #endif
#else
  #define VEX_SIMD_NEON 0
#endif

/**
 * Check if ANY SIMD is available
 */
#define VEX_SIMD_AVAILABLE (VEX_SIMD_X86 || VEX_SIMD_NEON)

// ============================================================================
// COMPILER HINTS & ATTRIBUTES
// ============================================================================

/**
 * Branch prediction hints
 */
#if defined(__GNUC__) || defined(__clang__)
  #define VEX_LIKELY(x)   __builtin_expect(!!(x), 1)
  #define VEX_UNLIKELY(x) __builtin_expect(!!(x), 0)
#else
  #define VEX_LIKELY(x)   (x)
  #define VEX_UNLIKELY(x) (x)
#endif

/**
 * Restrict keyword for pointer aliasing
 */
#if defined(__GNUC__) || defined(__clang__)
  #define VEX_RESTRICT __restrict__
#elif defined(_MSC_VER)
  #define VEX_RESTRICT __restrict
#else
  #define VEX_RESTRICT
#endif

/**
 * Function inlining hints
 */
#if defined(__GNUC__) || defined(__clang__)
  #define VEX_INLINE static inline __attribute__((always_inline))
  #define VEX_FORCE_INLINE static inline __attribute__((always_inline))
  #define VEX_NO_INLINE __attribute__((noinline))
#elif defined(_MSC_VER)
  #define VEX_INLINE static inline __forceinline
  #define VEX_FORCE_INLINE static inline __forceinline
  #define VEX_NO_INLINE __declspec(noinline)
#else
  #define VEX_INLINE static inline
  #define VEX_FORCE_INLINE static inline
  #define VEX_NO_INLINE
#endif

/**
 * Hot/cold path hints for code layout optimization
 */
#if defined(__GNUC__) || defined(__clang__)
  #define VEX_HOT __attribute__((hot))
  #define VEX_COLD __attribute__((cold))
#else
  #define VEX_HOT
  #define VEX_COLD
#endif

/**
 * Function attribute: flatten (inline all callees)
 */
#if defined(__GNUC__) || defined(__clang__)
  #define VEX_FLATTEN __attribute__((flatten))
#else
  #define VEX_FLATTEN
#endif

/**
 * Function attribute: pure (no side effects, only depends on args)
 */
#if defined(__GNUC__) || defined(__clang__)
  #define VEX_PURE __attribute__((pure))
  #define VEX_CONST __attribute__((const))
#else
  #define VEX_PURE
  #define VEX_CONST
#endif

/**
 * Memory prefetch hint
 */
#if defined(__GNUC__) || defined(__clang__)
  #define VEX_PREFETCH(ptr, rw, locality) __builtin_prefetch((ptr), (rw), (locality))
#else
  #define VEX_PREFETCH(ptr, rw, locality) ((void)0)
#endif

/**
 * Compiler memory barrier (prevent reordering)
 */
#if defined(_MSC_VER)
  #define VEX_BARRIER() _ReadWriteBarrier()
#elif defined(__GNUC__) || defined(__clang__)
  #define VEX_BARRIER() __asm__ __volatile__("" ::: "memory")
#else
  #define VEX_BARRIER() ((void)0)
#endif

/**
 * Unreachable code hint (for optimization)
 */
#if defined(__GNUC__) || defined(__clang__)
  #define VEX_UNREACHABLE() __builtin_unreachable()
#elif defined(_MSC_VER)
  #define VEX_UNREACHABLE() __assume(0)
#else
  #define VEX_UNREACHABLE() ((void)0)
#endif

/**
 * Mark variable as potentially unused (suppress warnings)
 */
#if defined(__GNUC__) || defined(__clang__)
  #define VEX_UNUSED __attribute__((unused))
#else
  #define VEX_UNUSED
#endif

/**
 * Alignment hints
 */
#if defined(__GNUC__) || defined(__clang__)
  #define VEX_ALIGN(n) __attribute__((aligned(n)))
#elif defined(_MSC_VER)
  #define VEX_ALIGN(n) __declspec(align(n))
#else
  #define VEX_ALIGN(n)
#endif

/**
 * Packed struct (no padding)
 */
#if defined(__GNUC__) || defined(__clang__)
  #define VEX_PACKED __attribute__((packed))
#elif defined(_MSC_VER)
  #define VEX_PACKED
  #pragma pack(push, 1)
#else
  #define VEX_PACKED
#endif

// ============================================================================
// UTILITY MACROS
// ============================================================================

/**
 * Array length (compile-time constant)
 */
#define VEX_ARRAY_LEN(arr) (sizeof(arr) / sizeof((arr)[0]))

/**
 * Min/Max macros (with type safety)
 */
#define VEX_MIN(a, b) ((a) < (b) ? (a) : (b))
#define VEX_MAX(a, b) ((a) > (b) ? (a) : (b))

/**
 * Clamp value between min and max
 */
#define VEX_CLAMP(x, min, max) (VEX_MIN(VEX_MAX((x), (min)), (max)))

/**
 * Align value up/down to multiple of alignment
 */
#define VEX_ALIGN_UP(x, align) (((x) + (align) - 1) & ~((align) - 1))
#define VEX_ALIGN_DOWN(x, align) ((x) & ~((align) - 1))

/**
 * Check if value is power of 2
 */
#define VEX_IS_POWER_OF_2(x) (((x) > 0) && (((x) & ((x) - 1)) == 0))

/**
 * String concatenation in macros
 */
#define VEX_CAT_(a, b) a##b
#define VEX_CAT(a, b) VEX_CAT_(a, b)

/**
 * Stringification
 */
#define VEX_STRINGIFY_(x) #x
#define VEX_STRINGIFY(x) VEX_STRINGIFY_(x)

/**
 * Swap two values (requires typeof or C23 auto)
 */
#if defined(__GNUC__) || defined(__clang__)
  #define VEX_SWAP(a, b) do { \
    __typeof__(a) _tmp = (a); \
    (a) = (b); \
    (b) = _tmp; \
  } while (0)
#else
  #define VEX_SWAP(a, b) do { \
    auto _tmp = (a); \
    (a) = (b); \
    (b) = _tmp; \
  } while (0)
#endif

// ============================================================================
// CACHE LINE SIZE
// ============================================================================

/**
 * Common cache line size (64 bytes on most modern CPUs)
 */
#ifndef VEX_CACHE_LINE_SIZE
  #define VEX_CACHE_LINE_SIZE 64
#endif

// ============================================================================
// DEBUGGING & DIAGNOSTICS
// ============================================================================

/**
 * Static assert (C11+ or compiler builtin)
 */
#if __STDC_VERSION__ >= 201112L
  #define VEX_STATIC_ASSERT(cond, msg) _Static_assert(cond, msg)
#elif defined(__GNUC__) || defined(__clang__)
  #define VEX_STATIC_ASSERT(cond, msg) _Static_assert(cond, msg)
#else
  #define VEX_STATIC_ASSERT(cond, msg) typedef char VEX_CAT(static_assert_, __LINE__)[(cond) ? 1 : -1]
#endif

/**
 * Debug-only code (removed in release builds)
 */
#ifdef NDEBUG
  #define VEX_DEBUG_ONLY(code) ((void)0)
#else
  #define VEX_DEBUG_ONLY(code) do { code } while (0)
#endif

// ============================================================================
// VERSION INFO
// ============================================================================

#define VEX_MACROS_VERSION_MAJOR 1
#define VEX_MACROS_VERSION_MINOR 0
#define VEX_MACROS_VERSION_PATCH 0

#endif // VEX_MACROS_H

