/*
 * SwissTable Optimizations - Phase 1 (Quick Wins)
 * Target: Beat Google Abseil Swiss Tables
 */

#ifndef VEX_SWISSTABLE_OPTIMIZED_H
#define VEX_SWISSTABLE_OPTIMIZED_H

#include <stdint.h>
#include <string.h>

// Force inline for critical hot paths
#define VEX_ALWAYS_INLINE __attribute__((always_inline)) inline
#define VEX_HOT __attribute__((hot))
#define VEX_FLATTEN __attribute__((flatten))

// ============================================================================
// OPTIMIZATION 1: Fast hash for small keys (<=16 bytes)
// ============================================================================

// Single-pass hash without strlen (for keys up to 16 bytes)
VEX_ALWAYS_INLINE uint64_t fast_hash_small_key(const char *s) {
    uint64_t h = 0xa0761d6478bd642full;
    const uint8_t *p = (const uint8_t *)s;
    
    // Unrolled loop for first 16 bytes
    // Most variable/function names are < 16 chars
    uint64_t chunk1 = 0, chunk2 = 0;
    
    // Load 8 bytes or until null
    for (int i = 0; i < 8; i++) {
        if (p[i] == 0) {
            return h ^ chunk1 ^ (uint64_t)i;
        }
        chunk1 |= ((uint64_t)p[i]) << (i * 8);
    }
    
    // Load next 8 bytes or until null
    for (int i = 0; i < 8; i++) {
        if (p[8 + i] == 0) {
            h ^= chunk1 * 0x2d358dccaa6c78a5ull;
            h ^= chunk2 * 0x8bb84b93962eacc9ull;
            return h ^ (uint64_t)(8 + i);
        }
        chunk2 |= ((uint64_t)p[8 + i]) << (i * 8);
    }
    
    // Long key: mix and continue with wyhash
    h ^= chunk1 * 0x2d358dccaa6c78a5ull;
    h ^= chunk2 * 0x8bb84b93962eacc9ull;
    return h;  // For keys > 16, caller should use full wyhash
}

// ============================================================================
// OPTIMIZATION 2: SIMD-optimized small key comparison
// ============================================================================

#if defined(__ARM_NEON) || defined(__ARM_NEON__)
#include <arm_neon.h>

// Compare two 16-byte keys using NEON (zero-copy)
VEX_ALWAYS_INLINE int fast_key_eq_16(const char *k1, const char *k2) {
    uint8x16_t v1 = vld1q_u8((const uint8_t *)k1);
    uint8x16_t v2 = vld1q_u8((const uint8_t *)k2);
    uint8x16_t cmp = vceqq_u8(v1, v2);
    
    // All bytes must match
    uint64x2_t cmp64 = vreinterpretq_u64_u8(cmp);
    return vgetq_lane_u64(cmp64, 0) == 0xFFFFFFFFFFFFFFFFull &&
           vgetq_lane_u64(cmp64, 1) == 0xFFFFFFFFFFFFFFFFull;
}

// Compare two 8-byte keys
VEX_ALWAYS_INLINE int fast_key_eq_8(const char *k1, const char *k2) {
    uint64_t v1, v2;
    memcpy(&v1, k1, 8);
    memcpy(&v2, k2, 8);
    return v1 == v2;
}

#elif defined(__AVX2__)
#include <immintrin.h>

VEX_ALWAYS_INLINE int fast_key_eq_16(const char *k1, const char *k2) {
    __m128i v1 = _mm_loadu_si128((const __m128i *)k1);
    __m128i v2 = _mm_loadu_si128((const __m128i *)k2);
    __m128i cmp = _mm_cmpeq_epi8(v1, v2);
    return _mm_movemask_epi8(cmp) == 0xFFFF;
}

VEX_ALWAYS_INLINE int fast_key_eq_8(const char *k1, const char *k2) {
    uint64_t v1, v2;
    memcpy(&v1, k1, 8);
    memcpy(&v2, k2, 8);
    return v1 == v2;
}

#else
// Scalar fallback
VEX_ALWAYS_INLINE int fast_key_eq_16(const char *k1, const char *k2) {
    uint64_t v1a, v1b, v2a, v2b;
    memcpy(&v1a, k1, 8);
    memcpy(&v1b, k1 + 8, 8);
    memcpy(&v2a, k2, 8);
    memcpy(&v2b, k2 + 8, 8);
    return (v1a == v2a) & (v1b == v2b);
}

VEX_ALWAYS_INLINE int fast_key_eq_8(const char *k1, const char *k2) {
    uint64_t v1, v2;
    memcpy(&v1, k1, 8);
    memcpy(&v2, k2, 8);
    return v1 == v2;
}
#endif

// ============================================================================
// OPTIMIZATION 3: Branchless bit operations
// ============================================================================

// Branchless select: returns a if mask==0, else b
VEX_ALWAYS_INLINE uint32_t branchless_select(uint32_t a, uint32_t b, int mask) {
    // mask is 0 or -1 (0xFFFFFFFF)
    return (a & ~mask) | (b & mask);
}

// Branchless min
VEX_ALWAYS_INLINE size_t branchless_min(size_t a, size_t b) {
    return b + ((a - b) & -(a < b));
}

// Count trailing zeros (branchless when possible)
VEX_ALWAYS_INLINE int fast_ctz(uint32_t x) {
#if defined(__GNUC__) || defined(__clang__)
    return __builtin_ctz(x | 1);  // |1 ensures x is never 0
#else
    // De Bruijn sequence fallback
    static const int table[32] = {
        0, 1, 28, 2, 29, 14, 24, 3, 30, 22, 20, 15, 25, 17, 4, 8,
        31, 27, 13, 23, 21, 19, 16, 7, 26, 12, 18, 6, 11, 5, 10, 9
    };
    return table[((uint32_t)((x & -x) * 0x077CB531U)) >> 27];
#endif
}

// ============================================================================
// OPTIMIZATION 4: Aggressive prefetching
// ============================================================================

VEX_ALWAYS_INLINE void prefetch_read_t0(const void *addr) {
    __builtin_prefetch(addr, 0, 3);  // Read, high temporal locality
}

VEX_ALWAYS_INLINE void prefetch_read_t1(const void *addr) {
    __builtin_prefetch(addr, 0, 2);  // Read, medium temporal locality
}

VEX_ALWAYS_INLINE void prefetch_read_nta(const void *addr) {
    __builtin_prefetch(addr, 0, 0);  // Read, non-temporal (evict soon)
}

VEX_ALWAYS_INLINE void prefetch_write(const void *addr) {
    __builtin_prefetch(addr, 1, 1);  // Write prefetch
}

// Prefetch multiple cache lines ahead
VEX_ALWAYS_INLINE void prefetch_stride(const void *base, size_t stride, int count) {
    const char *p = (const char *)base;
    for (int i = 0; i < count; i++) {
        prefetch_read_t1(p + i * stride);
    }
}

// ============================================================================
// OPTIMIZATION 5: Fast key length estimation
// ============================================================================

// Estimate key length category without computing exact length
VEX_ALWAYS_INLINE int estimate_key_size(const char *s) {
    // Quick check for small keys (most common)
    if (s[7] == 0) return 0;  // <= 7 bytes
    if (s[15] == 0) return 1; // 8-15 bytes
    return 2; // 16+ bytes
}

// ============================================================================
// OPTIMIZATION 6: Compile-time constants
// ============================================================================

// Ensure these are compile-time constants for optimization
#define GROUP_SIZE_LOG2 4
#define GROUP_SIZE (1 << GROUP_SIZE_LOG2)
#define GROUP_MASK (GROUP_SIZE - 1)

// Fast modulo for power-of-2 sizes
VEX_ALWAYS_INLINE size_t fast_mod_pow2(size_t x, size_t pow2_size) {
    return x & (pow2_size - 1);
}

// Fast division by GROUP_SIZE
VEX_ALWAYS_INLINE size_t fast_div_group_size(size_t x) {
    return x >> GROUP_SIZE_LOG2;
}

// Fast multiplication by GROUP_SIZE
VEX_ALWAYS_INLINE size_t fast_mul_group_size(size_t x) {
    return x << GROUP_SIZE_LOG2;
}

#endif /* VEX_SWISSTABLE_OPTIMIZED_H */

