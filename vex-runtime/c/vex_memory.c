/**
 * Vex Memory Operations
 * SIMD-optimized, high-performance implementations
 */

#include "vex.h"
#include <string.h>

// vex.h includes vex_macros.h with VEX_SIMD_X86, VEX_SIMD_NEON

// ============================================================================
// OPTIMIZED MEMORY OPERATIONS
// ============================================================================

/**
 * memcpy - Optimized with unrolled loops and SIMD
 * Strategy: 
 *   1. Copy small amounts byte-by-byte
 *   2. Align destination to word boundary
 *   3. Copy in large chunks (64 bytes at a time)
 *   4. Handle tail bytes
 */
void* vex_memcpy(void* VEX_RESTRICT dest, const void* VEX_RESTRICT src, size_t n) {
    char* VEX_RESTRICT d = (char*)dest;
    const char* VEX_RESTRICT s = (const char*)src;
    
    // Fast path for small copies
    if (VEX_UNLIKELY(n < 16)) {
        for (size_t i = 0; i < n; i++) {
            d[i] = s[i];
        }
        return dest;
    }

#if VEX_SIMD_X86
    // x86 SIMD: Use SSE2 (16 bytes) or AVX (32 bytes)
    #ifdef __AVX__
    // AVX: 32-byte chunks
    while (n >= 32) {
        __m256i chunk = _mm256_loadu_si256((const __m256i*)s);
        _mm256_storeu_si256((__m256i*)d, chunk);
        s += 32;
        d += 32;
        n -= 32;
    }
    #endif
    
    // SSE2: 16-byte chunks (fallback or tail handling)
    while (n >= 16) {
        __m128i chunk = _mm_loadu_si128((const __m128i*)s);
        _mm_storeu_si128((__m128i*)d, chunk);
        s += 16;
        d += 16;
        n -= 16;
    }
#elif VEX_SIMD_NEON
    // ARM NEON: 16-byte chunks
    while (n >= 16) {
        uint8x16_t chunk = vld1q_u8((const uint8_t*)s);
        vst1q_u8((uint8_t*)d, chunk);
        s += 16;
        d += 16;
        n -= 16;
    }
#else
    // Scalar: 8-byte (uint64_t) chunks
    while (n >= 8) {
        *(uint64_t*)d = *(const uint64_t*)s;
        s += 8;
        d += 8;
        n -= 8;
    }
#endif

    // Handle tail bytes (0-15 bytes remaining)
    while (n > 0) {
        *d++ = *s++;
        n--;
    }
    
    return dest;
}

/**
 * memmove - Handles overlapping regions correctly
 * Strategy: Detect overlap direction and copy accordingly
 */
void* vex_memmove(void* dest, const void* src, size_t n) {
    char* d = (char*)dest;
    const char* s = (const char*)src;
    
    // No overlap or same address - just copy
    if (VEX_UNLIKELY(d == s || n == 0)) {
        return dest;
    }
    
    // Overlap detection
    if (d < s) {
        // Forward copy (dest is before src)
        return vex_memcpy(dest, src, n);
    } else {
        // Backward copy (dest is after src, overlapping)
        // Copy from end to beginning
        d += n;
        s += n;
        
        // Handle tail bytes first
        size_t tail = n & 7;
        while (tail--) {
            *--d = *--s;
        }
        n -= (n & 7);
        
        // 8-byte chunks backward
        while (n >= 8) {
            d -= 8;
            s -= 8;
            *(uint64_t*)d = *(const uint64_t*)s;
            n -= 8;
        }
    }
    
    return dest;
}

/**
 * memset - Optimized with unrolled loops and SIMD
 * Strategy: Broadcast value to vector registers and write in large chunks
 */
void* vex_memset(void* s, int c, size_t n) {
    unsigned char* p = (unsigned char*)s;
    unsigned char value = (unsigned char)c;
    
    // Fast path for small sets
    if (VEX_UNLIKELY(n < 16)) {
        for (size_t i = 0; i < n; i++) {
            p[i] = value;
        }
        return s;
    }
    
#if VEX_SIMD_X86
    // Broadcast byte to 16 bytes
    __m128i vec = _mm_set1_epi8((char)value);
    
    #ifdef __AVX__
    // AVX: Broadcast to 32 bytes
    __m256i vec32 = _mm256_set1_epi8((char)value);
    while (n >= 32) {
        _mm256_storeu_si256((__m256i*)p, vec32);
        p += 32;
        n -= 32;
    }
    #endif
    
    // SSE2: 16-byte chunks
    while (n >= 16) {
        _mm_storeu_si128((__m128i*)p, vec);
        p += 16;
        n -= 16;
    }
#elif VEX_SIMD_NEON
    // ARM NEON: Broadcast to 16 bytes
    uint8x16_t vec = vdupq_n_u8(value);
    while (n >= 16) {
        vst1q_u8(p, vec);
        p += 16;
        n -= 16;
    }
#else
    // Scalar: Broadcast to 8 bytes (uint64_t)
    uint64_t value64 = (uint64_t)value * 0x0101010101010101ULL;
    while (n >= 8) {
        *(uint64_t*)p = value64;
        p += 8;
        n -= 8;
    }
#endif

    // Handle tail bytes
    while (n > 0) {
        *p++ = value;
        n--;
    }
    
    return s;
}

/**
 * memcmp - Optimized with unrolled loops and SIMD
 * Strategy: Compare in large chunks, return early on mismatch
 */
int vex_memcmp(const void* s1, const void* s2, size_t n) {
    const unsigned char* p1 = (const unsigned char*)s1;
    const unsigned char* p2 = (const unsigned char*)s2;
    
    // Fast path for small compares
    if (VEX_UNLIKELY(n < 8)) {
        for (size_t i = 0; i < n; i++) {
            if (p1[i] != p2[i]) {
                return p1[i] - p2[i];
            }
        }
        return 0;
    }

#if VEX_SIMD_X86
    // SSE2: Compare 16 bytes at a time
    while (n >= 16) {
        __m128i v1 = _mm_loadu_si128((const __m128i*)p1);
        __m128i v2 = _mm_loadu_si128((const __m128i*)p2);
        __m128i cmp = _mm_cmpeq_epi8(v1, v2);
        int mask = _mm_movemask_epi8(cmp);
        
        if (VEX_UNLIKELY(mask != 0xFFFF)) {
            // Found mismatch - find first differing byte
            for (size_t i = 0; i < 16; i++) {
                if (p1[i] != p2[i]) {
                    return p1[i] - p2[i];
                }
            }
        }
        
        p1 += 16;
        p2 += 16;
        n -= 16;
    }
#elif VEX_SIMD_NEON
    // ARM NEON: Compare 16 bytes at a time
    while (n >= 16) {
        uint8x16_t v1 = vld1q_u8(p1);
        uint8x16_t v2 = vld1q_u8(p2);
        uint8x16_t cmp = vceqq_u8(v1, v2);
        
        // Check if all bytes are equal
        uint8x8_t cmp_low = vget_low_u8(cmp);
        uint8x8_t cmp_high = vget_high_u8(cmp);
        uint8x8_t cmp_and = vand_u8(cmp_low, cmp_high);
        uint8x8_t cmp_min = vpmin_u8(cmp_and, cmp_and);
        cmp_min = vpmin_u8(cmp_min, cmp_min);
        cmp_min = vpmin_u8(cmp_min, cmp_min);
        
        if (VEX_UNLIKELY(vget_lane_u8(cmp_min, 0) != 0xFF)) {
            // Found mismatch
            for (size_t i = 0; i < 16; i++) {
                if (p1[i] != p2[i]) {
                    return p1[i] - p2[i];
                }
            }
        }
        
        p1 += 16;
        p2 += 16;
        n -= 16;
    }
#else
    // Scalar: 8-byte chunks
    while (n >= 8) {
        uint64_t v1 = *(const uint64_t*)p1;
        uint64_t v2 = *(const uint64_t*)p2;
        
        if (VEX_UNLIKELY(v1 != v2)) {
            // Found mismatch - check byte by byte
            for (size_t i = 0; i < 8; i++) {
                if (p1[i] != p2[i]) {
                    return p1[i] - p2[i];
                }
            }
        }
        
        p1 += 8;
        p2 += 8;
        n -= 8;
    }
#endif

    // Handle tail bytes
    while (n > 0) {
        if (*p1 != *p2) {
            return *p1 - *p2;
        }
        p1++;
        p2++;
        n--;
    }
    
    return 0;
}
