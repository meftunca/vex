/*
 * simd_utils.c - SIMD-accelerated utility functions
 * Supports: AVX-512, AVX2, SSE2, ARM NEON with scalar fallback
 */

#include "protocols/simd_utils.h"

/* ====================== Platform Detection ====================== */

#if defined(__x86_64__) || defined(_M_X64) || defined(__i386) || defined(_M_IX86)
#   define VEX_SIMD_ARCH_X86 1
#else
#   define VEX_SIMD_ARCH_X86 0
#endif

#if defined(__aarch64__) || defined(__arm__)
#   define VEX_SIMD_ARCH_ARM 1
#else
#   define VEX_SIMD_ARCH_ARM 0
#endif

/* x86 SIMD feature detection */
#if VEX_SIMD_ARCH_X86
#   include <immintrin.h>
#   if defined(__AVX512F__)
#       define VEX_SIMD_HAVE_AVX512 1
#   else
#       define VEX_SIMD_HAVE_AVX512 0
#   endif
#   if defined(__AVX2__)
#       define VEX_SIMD_HAVE_AVX2 1
#   else
#       define VEX_SIMD_HAVE_AVX2 0
#   endif
#   if defined(__SSE2__) || (defined(_M_IX86_FP) && _M_IX86_FP >= 2)
#       define VEX_SIMD_HAVE_SSE2 1
#   else
#       define VEX_SIMD_HAVE_SSE2 0
#   endif
#else
#   define VEX_SIMD_HAVE_AVX512 0
#   define VEX_SIMD_HAVE_AVX2   0
#   define VEX_SIMD_HAVE_SSE2   0
#endif

/* ARM NEON feature detection */
#if VEX_SIMD_ARCH_ARM
#   include <arm_neon.h>
#   if defined(__ARM_NEON) || defined(__ARM_NEON__)
#       define VEX_SIMD_HAVE_NEON 1
#   else
#       define VEX_SIMD_HAVE_NEON 0
#   endif
#else
#   define VEX_SIMD_HAVE_NEON 0
#endif

/* ====================== Public API ====================== */

const char* vex_simd_backend(void) {
#if VEX_SIMD_HAVE_AVX512
    return "AVX-512";
#elif VEX_SIMD_HAVE_AVX2
    return "AVX2";
#elif VEX_SIMD_HAVE_SSE2
    return "SSE2";
#elif VEX_SIMD_HAVE_NEON
    return "ARM NEON";
#else
    return "SCALAR";
#endif
}

size_t vex_simd_find_char(const char *buf, size_t len, char c) {
#if VEX_SIMD_HAVE_AVX512
    if (len >= 64) {
        __m512i target = _mm512_set1_epi8((char)c);
        size_t i = 0;
        size_t n = len & ~(size_t)63;
        for (; i < n; i += 64) {
            __m512i data   = _mm512_loadu_si512((const void *)(buf + i));
            __mmask64 mask = _mm512_cmpeq_epi8_mask(data, target);
            if (mask != 0) {
                return i + (size_t)__builtin_ctzll(mask);
            }
        }
        /* Tail */
        for (; i < len; ++i) {
            if (buf[i] == c) return i;
        }
        return len;
    }
#elif VEX_SIMD_HAVE_AVX2
    if (len >= 32) {
        __m256i target = _mm256_set1_epi8((char)c);
        size_t i = 0;
        size_t n = len & ~(size_t)31;
        for (; i < n; i += 32) {
            __m256i data = _mm256_loadu_si256((const __m256i *)(buf + i));
            __m256i eq   = _mm256_cmpeq_epi8(data, target);
            int mask     = _mm256_movemask_epi8(eq);
            if (mask != 0) {
                return i + (size_t)__builtin_ctz(mask);
            }
        }
        /* Tail */
        for (; i < len; ++i) {
            if (buf[i] == c) return i;
        }
        return len;
    }
#elif VEX_SIMD_HAVE_SSE2
    if (len >= 16) {
        __m128i target = _mm_set1_epi8((char)c);
        size_t i = 0;
        size_t n = len & ~(size_t)15;
        for (; i < n; i += 16) {
            __m128i data = _mm_loadu_si128((const __m128i *)(buf + i));
            __m128i eq   = _mm_cmpeq_epi8(data, target);
            int mask     = _mm_movemask_epi8(eq);
            if (mask != 0) {
                return i + (size_t)__builtin_ctz(mask);
            }
        }
        /* Tail */
        for (; i < len; ++i) {
            if (buf[i] == c) return i;
        }
        return len;
    }
#elif VEX_SIMD_HAVE_NEON
    if (len >= 16) {
        uint8x16_t target = vdupq_n_u8((uint8_t)c);
        size_t i = 0;
        size_t n = len & ~(size_t)15;
        for (; i < n; i += 16) {
            uint8x16_t data = vld1q_u8((const uint8_t *)(buf + i));
            uint8x16_t cmp  = vceqq_u8(data, target);
            /* Check if any byte matched */
            uint64x2_t pair = vpaddlq_u32(vpaddlq_u16(vpaddlq_u8(cmp)));
            if (vgetq_lane_u64(pair, 0) | vgetq_lane_u64(pair, 1)) {
                /* Found match, find index */
                uint8_t tmp[16];
                vst1q_u8(tmp, cmp);
                for (int j = 0; j < 16; ++j) {
                    if (tmp[j]) return i + j;
                }
            }
        }
        /* Tail */
        for (; i < len; ++i) {
            if (buf[i] == c) return i;
        }
        return len;
    }
#endif

    /* Scalar fallback */
    for (size_t i = 0; i < len; ++i) {
        if (buf[i] == c) return i;
    }
    return len;
}

size_t vex_simd_find_set2(const char *buf, size_t len, char c1, char c2) {
#if VEX_SIMD_HAVE_AVX512
    if (len >= 64) {
        __m512i t1 = _mm512_set1_epi8(c1);
        __m512i t2 = _mm512_set1_epi8(c2);
        size_t i = 0;
        size_t n = len & ~(size_t)63;
        for (; i < n; i += 64) {
            __m512i data = _mm512_loadu_si512((const void *)(buf + i));
            __mmask64 m1 = _mm512_cmpeq_epi8_mask(data, t1);
            __mmask64 m2 = _mm512_cmpeq_epi8_mask(data, t2);
            __mmask64 mask = m1 | m2;
            if (mask != 0) {
                return i + (size_t)__builtin_ctzll(mask);
            }
        }
        for (; i < len; ++i) if (buf[i] == c1 || buf[i] == c2) return i;
        return len;
    }
#elif VEX_SIMD_HAVE_AVX2
    if (len >= 32) {
        __m256i t1 = _mm256_set1_epi8(c1);
        __m256i t2 = _mm256_set1_epi8(c2);
        size_t i = 0;
        size_t n = len & ~(size_t)31;
        for (; i < n; i += 32) {
            __m256i data = _mm256_loadu_si256((const __m256i *)(buf + i));
            __m256i eq1  = _mm256_cmpeq_epi8(data, t1);
            __m256i eq2  = _mm256_cmpeq_epi8(data, t2);
            __m256i eq   = _mm256_or_si256(eq1, eq2);
            int mask     = _mm256_movemask_epi8(eq);
            if (mask != 0) {
                return i + (size_t)__builtin_ctz(mask);
            }
        }
        for (; i < len; ++i) if (buf[i] == c1 || buf[i] == c2) return i;
        return len;
    }
#elif VEX_SIMD_HAVE_SSE2
    if (len >= 16) {
        __m128i t1 = _mm_set1_epi8(c1);
        __m128i t2 = _mm_set1_epi8(c2);
        size_t i = 0;
        size_t n = len & ~(size_t)15;
        for (; i < n; i += 16) {
            __m128i data = _mm_loadu_si128((const __m128i *)(buf + i));
            __m128i eq1  = _mm_cmpeq_epi8(data, t1);
            __m128i eq2  = _mm_cmpeq_epi8(data, t2);
            __m128i eq   = _mm_or_si128(eq1, eq2);
            int mask     = _mm_movemask_epi8(eq);
            if (mask != 0) {
                return i + (size_t)__builtin_ctz(mask);
            }
        }
        for (; i < len; ++i) if (buf[i] == c1 || buf[i] == c2) return i;
        return len;
    }
#elif VEX_SIMD_HAVE_NEON
    if (len >= 16) {
        uint8x16_t t1 = vdupq_n_u8((uint8_t)c1);
        uint8x16_t t2 = vdupq_n_u8((uint8_t)c2);
        size_t i = 0;
        size_t n = len & ~(size_t)15;
        for (; i < n; i += 16) {
            uint8x16_t data = vld1q_u8((const uint8_t *)(buf + i));
            uint8x16_t eq1  = vceqq_u8(data, t1);
            uint8x16_t eq2  = vceqq_u8(data, t2);
            uint8x16_t eq   = vorrq_u8(eq1, eq2);
            
            uint64x2_t pair = vpaddlq_u32(vpaddlq_u16(vpaddlq_u8(eq)));
            if (vgetq_lane_u64(pair, 0) | vgetq_lane_u64(pair, 1)) {
                uint8_t tmp[16];
                vst1q_u8(tmp, eq);
                for (int j = 0; j < 16; ++j) if (tmp[j]) return i + j;
            }
        }
        for (; i < len; ++i) if (buf[i] == c1 || buf[i] == c2) return i;
        return len;
    }
#endif
    for (size_t i = 0; i < len; ++i) if (buf[i] == c1 || buf[i] == c2) return i;
    return len;
}

size_t vex_simd_find_set4(const char *buf, size_t len, char c1, char c2, char c3, char c4) {
#if VEX_SIMD_HAVE_AVX512
    if (len >= 64) {
        __m512i t1 = _mm512_set1_epi8(c1);
        __m512i t2 = _mm512_set1_epi8(c2);
        __m512i t3 = _mm512_set1_epi8(c3);
        __m512i t4 = _mm512_set1_epi8(c4);
        size_t i = 0;
        size_t n = len & ~(size_t)63;
        for (; i < n; i += 64) {
            __m512i data = _mm512_loadu_si512((const void *)(buf + i));
            __mmask64 m1 = _mm512_cmpeq_epi8_mask(data, t1);
            __mmask64 m2 = _mm512_cmpeq_epi8_mask(data, t2);
            __mmask64 m3 = _mm512_cmpeq_epi8_mask(data, t3);
            __mmask64 m4 = _mm512_cmpeq_epi8_mask(data, t4);
            __mmask64 mask = m1 | m2 | m3 | m4;
            if (mask != 0) return i + (size_t)__builtin_ctzll(mask);
        }
        for (; i < len; ++i) if (buf[i] == c1 || buf[i] == c2 || buf[i] == c3 || buf[i] == c4) return i;
        return len;
    }
#elif VEX_SIMD_HAVE_AVX2
    if (len >= 32) {
        __m256i t1 = _mm256_set1_epi8(c1);
        __m256i t2 = _mm256_set1_epi8(c2);
        __m256i t3 = _mm256_set1_epi8(c3);
        __m256i t4 = _mm256_set1_epi8(c4);
        size_t i = 0;
        size_t n = len & ~(size_t)31;
        for (; i < n; i += 32) {
            __m256i data = _mm256_loadu_si256((const __m256i *)(buf + i));
            __m256i eq1  = _mm256_cmpeq_epi8(data, t1);
            __m256i eq2  = _mm256_cmpeq_epi8(data, t2);
            __m256i eq3  = _mm256_cmpeq_epi8(data, t3);
            __m256i eq4  = _mm256_cmpeq_epi8(data, t4);
            __m256i eq   = _mm256_or_si256(_mm256_or_si256(eq1, eq2), _mm256_or_si256(eq3, eq4));
            int mask     = _mm256_movemask_epi8(eq);
            if (mask != 0) return i + (size_t)__builtin_ctz(mask);
        }
        for (; i < len; ++i) if (buf[i] == c1 || buf[i] == c2 || buf[i] == c3 || buf[i] == c4) return i;
        return len;
    }
#elif VEX_SIMD_HAVE_SSE2
    if (len >= 16) {
        __m128i t1 = _mm_set1_epi8(c1);
        __m128i t2 = _mm_set1_epi8(c2);
        __m128i t3 = _mm_set1_epi8(c3);
        __m128i t4 = _mm_set1_epi8(c4);
        size_t i = 0;
        size_t n = len & ~(size_t)15;
        for (; i < n; i += 16) {
            __m128i data = _mm_loadu_si128((const __m128i *)(buf + i));
            __m128i eq1  = _mm_cmpeq_epi8(data, t1);
            __m128i eq2  = _mm_cmpeq_epi8(data, t2);
            __m128i eq3  = _mm_cmpeq_epi8(data, t3);
            __m128i eq4  = _mm_cmpeq_epi8(data, t4);
            __m128i eq   = _mm_or_si128(_mm_or_si128(eq1, eq2), _mm_or_si128(eq3, eq4));
            int mask     = _mm_movemask_epi8(eq);
            if (mask != 0) return i + (size_t)__builtin_ctz(mask);
        }
        for (; i < len; ++i) if (buf[i] == c1 || buf[i] == c2 || buf[i] == c3 || buf[i] == c4) return i;
        return len;
    }
#elif VEX_SIMD_HAVE_NEON
    if (len >= 16) {
        uint8x16_t t1 = vdupq_n_u8((uint8_t)c1);
        uint8x16_t t2 = vdupq_n_u8((uint8_t)c2);
        uint8x16_t t3 = vdupq_n_u8((uint8_t)c3);
        uint8x16_t t4 = vdupq_n_u8((uint8_t)c4);
        size_t i = 0;
        size_t n = len & ~(size_t)15;
        for (; i < n; i += 16) {
            uint8x16_t data = vld1q_u8((const uint8_t *)(buf + i));
            uint8x16_t eq1  = vceqq_u8(data, t1);
            uint8x16_t eq2  = vceqq_u8(data, t2);
            uint8x16_t eq3  = vceqq_u8(data, t3);
            uint8x16_t eq4  = vceqq_u8(data, t4);
            uint8x16_t eq   = vorrq_u8(vorrq_u8(eq1, eq2), vorrq_u8(eq3, eq4));
            
            uint64x2_t pair = vpaddlq_u32(vpaddlq_u16(vpaddlq_u8(eq)));
            if (vgetq_lane_u64(pair, 0) | vgetq_lane_u64(pair, 1)) {
                uint8_t tmp[16];
                vst1q_u8(tmp, eq);
                for (int j = 0; j < 16; ++j) if (tmp[j]) return i + j;
            }
        }
        for (; i < len; ++i) if (buf[i] == c1 || buf[i] == c2 || buf[i] == c3 || buf[i] == c4) return i;
        return len;
    }
#endif
    for (size_t i = 0; i < len; ++i) if (buf[i] == c1 || buf[i] == c2 || buf[i] == c3 || buf[i] == c4) return i;
    return len;
}

void vex_simd_xor_stream(uint8_t *buf, size_t len, const uint8_t key[4]) {
#if VEX_SIMD_HAVE_AVX512
    if (len >= 64) {
        /* Broadcast 4-byte key to 64 bytes */
        uint32_t k32 = *(const uint32_t*)key;
        __m512i k = _mm512_set1_epi32(k32);
        size_t i = 0;
        size_t n = len & ~(size_t)63;
        for (; i < n; i += 64) {
            __m512i data = _mm512_loadu_si512((const void *)(buf + i));
            _mm512_storeu_si512((void *)(buf + i), _mm512_xor_si512(data, k));
        }
        /* Tail */
        for (; i < len; ++i) buf[i] ^= key[i % 4];
        return;
    }
#elif VEX_SIMD_HAVE_AVX2
    if (len >= 32) {
        uint32_t k32 = *(const uint32_t*)key;
        __m256i k = _mm256_set1_epi32(k32);
        size_t i = 0;
        size_t n = len & ~(size_t)31;
        for (; i < n; i += 32) {
            __m256i data = _mm256_loadu_si256((const __m256i *)(buf + i));
            _mm256_storeu_si256((__m256i *)(buf + i), _mm256_xor_si256(data, k));
        }
        for (; i < len; ++i) buf[i] ^= key[i % 4];
        return;
    }
#elif VEX_SIMD_HAVE_SSE2
    if (len >= 16) {
        uint32_t k32 = *(const uint32_t*)key;
        __m128i k = _mm_set1_epi32(k32);
        size_t i = 0;
        size_t n = len & ~(size_t)15;
        for (; i < n; i += 16) {
            __m128i data = _mm_loadu_si128((const __m128i *)(buf + i));
            _mm_storeu_si128((__m128i *)(buf + i), _mm_xor_si128(data, k));
        }
        for (; i < len; ++i) buf[i] ^= key[i % 4];
        return;
    }
#elif VEX_SIMD_HAVE_NEON
    if (len >= 16) {
        uint32_t k32 = *(const uint32_t*)key;
        uint32x4_t k = vdupq_n_u32(k32);
        size_t i = 0;
        size_t n = len & ~(size_t)15;
        for (; i < n; i += 16) {
            uint32x4_t data = vld1q_u32((const uint32_t *)(buf + i));
            vst1q_u32((uint32_t *)(buf + i), veorq_u32(data, k));
        }
        for (; i < len; ++i) buf[i] ^= key[i % 4];
        return;
    }
#endif
    for (size_t i = 0; i < len; ++i) buf[i] ^= key[i % 4];
}
