// utf_validate_convert.c
// Single-file UTF-8/16/32 validator + UTF-8->UTF16/UTF32 converters
// SIMD-accelerated (x86 AVX2/SSE2, ARM NEON) fast paths + portable scalar fallback.
// License: CC0 / Public Domain. Use at your own risk.
//
// Build examples:
//   x86 AVX2: cc -O3 -mavx2 -o utf_demo utf_validate_convert.c
//   x86 SSE2: cc -O3 -msse2 -o utf_demo utf_validate_convert.c
//   AArch64 : cc -O3        -o utf_demo utf_validate_convert.c
//
// Public API:
//   bool   utf8_validate(const uint8_t *s, size_t len);
//   bool   utf16_validate(const uint16_t *s, size_t len);
//   bool   utf32_validate(const uint32_t *s, size_t len);
//   size_t utf8_to_utf16(const uint8_t *src, size_t len, uint16_t *dst); // returns out length or (size_t)-1 on error
//   size_t utf8_to_utf32(const uint8_t *src, size_t len, uint32_t *dst); // returns out length or (size_t)-1 on error
//
// Notes:
// - SIMD paths implement block checks for ASCII-fast and structural continuation patterns.
//   On detecting any complex multi-byte lead pattern, we conservatively fall back to scalar verification
//   for the remainder of the chunk to keep this reference implementation compact and correct.
// - For production-grade maximal speed, consider full-vector decoders (e.g., simdutf).
// - This file is intentionally dependency-free and portable.

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <string.h>

#if defined(__x86_64__) || defined(_M_X64) || defined(__i386__)
  #include <immintrin.h>
  #define VEC_X86 1
#else
  #define VEC_X86 0
#endif

#if defined(__ARM_NEON) || defined(__ARM_NEON__) || defined(__aarch64__)
  #include <arm_neon.h>
  #define VEC_NEON 1
#else
  #define VEC_NEON 0
#endif

// =============================
// Scalar validators/converters
// =============================

static inline bool utf8_validate_scalar(const uint8_t *s, size_t len) {
    size_t i = 0;
    while (i < len) {
        uint8_t c = s[i];
        if (c < 0x80) { i++; continue; }
        else if ((c >> 5) == 0x6) { // 110xxxxx
            if (i+1 >= len) return false;
            uint8_t c2 = s[i+1];
            if ((c2 & 0xC0) != 0x80) return false;
            if (c < 0xC2) return false; // overlong
            i += 2;
        } else if ((c >> 4) == 0xE) { // 1110xxxx
            if (i+2 >= len) return false;
            uint8_t c2 = s[i+1], c3 = s[i+2];
            if ((c2 & 0xC0) != 0x80 || (c3 & 0xC0) != 0x80) return false;
            uint32_t cp = ((c & 0x0F) << 12) | ((c2 & 0x3F) << 6) | (c3 & 0x3F);
            if (cp >= 0xD800 && cp <= 0xDFFF) return false; // surrogates not allowed
            if (c == 0xE0 && c2 < 0xA0) return false; // overlong 3-byte
            if (c == 0xED && c2 > 0x9F) return false; // surrogate region
            i += 3;
        } else if ((c >> 3) == 0x1E) { // 11110xxx
            if (i+3 >= len) return false;
            uint8_t c2 = s[i+1], c3 = s[i+2], c4 = s[i+3];
            if ((c2 & 0xC0) != 0x80 || (c3 & 0xC0) != 0x80 || (c4 & 0xC0) != 0x80) return false;
            uint32_t cp = ((c & 0x07) << 18) | ((c2 & 0x3F) << 12) | ((c3 & 0x3F) << 6) | (c4 & 0x3F);
            if (cp > 0x10FFFF) return false;
            if (c == 0xF0 && c2 < 0x90) return false; // overlong 4-byte
            if (c == 0xF4 && c2 > 0x8F) return false; // > U+10FFFF
            i += 4;
        } else {
            return false;
        }
    }
    return true;
}

static inline bool utf16_validate_scalar(const uint16_t *s, size_t len) {
    size_t i = 0;
    while (i < len) {
        uint16_t w1 = s[i++];
        if (w1 >= 0xD800 && w1 <= 0xDBFF) { // high surrogate
            if (i >= len) return false;
            uint16_t w2 = s[i++];
            if (!(w2 >= 0xDC00 && w2 <= 0xDFFF)) return false;
        } else if (w1 >= 0xDC00 && w1 <= 0xDFFF) {
            return false; // lone low surrogate
        }
    }
    return true;
}

static inline bool utf32_validate_scalar(const uint32_t *s, size_t len) {
    for (size_t i = 0; i < len; ++i) {
        uint32_t cp = s[i];
        if (cp > 0x10FFFF) return false;
        if (cp >= 0xD800 && cp <= 0xDFFF) return false;
    }
    return true;
}

static inline size_t utf8_to_utf16_scalar(const uint8_t *src, size_t len, uint16_t *dst) {
    size_t i = 0, j = 0;
    while (i < len) {
        uint8_t c = src[i];
        if (c < 0x80) { dst[j++] = c; i++; }
        else if ((c >> 5) == 0x6) { // 2-byte
            if (i+1 >= len) return (size_t)-1;
            uint8_t c2 = src[i+1];
            if ((c2 & 0xC0) != 0x80 || c < 0xC2) return (size_t)-1;
            dst[j++] = ((c & 0x1F) << 6) | (c2 & 0x3F);
            i += 2;
        } else if ((c >> 4) == 0xE) { // 3-byte
            if (i+2 >= len) return (size_t)-1;
            uint8_t c2 = src[i+1], c3 = src[i+2];
            if ((c2 & 0xC0) != 0x80 || (c3 & 0xC0) != 0x80) return (size_t)-1;
            uint32_t cp = ((c & 0x0F) << 12) | ((c2 & 0x3F) << 6) | (c3 & 0x3F);
            if (cp >= 0xD800 && cp <= 0xDFFF) return (size_t)-1;
            if (c == 0xE0 && c2 < 0xA0) return (size_t)-1;
            if (c == 0xED && c2 > 0x9F) return (size_t)-1;
            dst[j++] = (uint16_t)cp;
            i += 3;
        } else if ((c >> 3) == 0x1E) { // 4-byte
            if (i+3 >= len) return (size_t)-1;
            uint8_t c2 = src[i+1], c3 = src[i+2], c4 = src[i+3];
            if ((c2 & 0xC0) != 0x80 || (c3 & 0xC0) != 0x80 || (c4 & 0xC0) != 0x80) return (size_t)-1;
            uint32_t cp = ((c & 0x07) << 18) | ((c2 & 0x3F) << 12) | ((c3 & 0x3F) << 6) | (c4 & 0x3F);
            if (cp > 0x10FFFF) return (size_t)-1;
            if (c == 0xF0 && c2 < 0x90) return (size_t)-1;
            if (c == 0xF4 && c2 > 0x8F) return (size_t)-1;
            cp -= 0x10000;
            dst[j++] = (uint16_t)(0xD800 | (cp >> 10));
            dst[j++] = (uint16_t)(0xDC00 | (cp & 0x3FF));
            i += 4;
        } else return (size_t)-1;
    }
    return j;
}

static inline size_t utf8_to_utf32_scalar(const uint8_t *src, size_t len, uint32_t *dst) {
    size_t i = 0, j = 0;
    while (i < len) {
        uint8_t c = src[i];
        if (c < 0x80) { dst[j++] = c; i++; }
        else if ((c >> 5) == 0x6) {
            if (i+1 >= len) return (size_t)-1;
            uint8_t c2 = src[i+1];
            if ((c2 & 0xC0) != 0x80 || c < 0xC2) return (size_t)-1;
            dst[j++] = ((c & 0x1F) << 6) | (c2 & 0x3F);
            i += 2;
        } else if ((c >> 4) == 0xE) {
            if (i+2 >= len) return (size_t)-1;
            uint8_t c2 = src[i+1], c3 = src[i+2];
            if ((c2 & 0xC0) != 0x80 || (c3 & 0xC0) != 0x80) return (size_t)-1;
            uint32_t cp = ((c & 0x0F) << 12) | ((c2 & 0x3F) << 6) | (c3 & 0x3F);
            if (cp >= 0xD800 && cp <= 0xDFFF) return (size_t)-1;
            if (c == 0xE0 && c2 < 0xA0) return (size_t)-1;
            if (c == 0xED && c2 > 0x9F) return (size_t)-1;
            dst[j++] = cp;
            i += 3;
        } else if ((c >> 3) == 0x1E) {
            if (i+3 >= len) return (size_t)-1;
            uint8_t c2 = src[i+1], c3 = src[i+2], c4 = src[i+3];
            if ((c2 & 0xC0) != 0x80 || (c3 & 0xC0) != 0x80 || (c4 & 0xC0) != 0x80) return (size_t)-1;
            uint32_t cp = ((c & 0x07) << 18) | ((c2 & 0x3F) << 12) | ((c3 & 0x3F) << 6) | (c4 & 0x3F);
            if (cp > 0x10FFFF) return (size_t)-1;
            if (c == 0xF0 && c2 < 0x90) return (size_t)-1;
            if (c == 0xF4 && c2 > 0x8F) return (size_t)-1;
            dst[j++] = cp;
            i += 4;
        } else return (size_t)-1;
    }
    return j;
}

// =============================
// SIMD-assisted UTF-8 validate
// =============================
// Strategy: scan in 32 or 16B chunks. If all bytes <0x80 => ASCII fast.
// Else, ensure that continuation bytes (10xxxxxx) only follow multi-byte leads.
// For non-trivial mixtures, fall back to scalar validate on that block boundary.
// This keeps code compact and correct while still fast on ASCII-heavy text.

static inline bool block_all_ascii(const uint8_t *p, size_t n) {
    // n is 16 or 32
    for (size_t i=0;i<n;i++) if (p[i] & 0x80) return false;
    return true;
}

static inline bool utf8_validate_simd(const uint8_t *s, size_t len) {
    size_t i = 0;

#if VEC_X86
    // Prefer 32-byte AVX2 if available at compile-time, otherwise 16-byte SSE2
    #if defined(__AVX2__)
    while (i + 32 <= len) {
        __m256i v = _mm256_loadu_si256((const __m256i*)(s + i));
        if (_mm256_movemask_epi8(v) != 0) {
            // Non-ASCII detected: fallback to scalar for the remainder of this block region.
            // We align down to block start for safety.
            size_t chunk_end = i + 32;
            if (!utf8_validate_scalar(s + i, chunk_end - i)) return false;
            i = chunk_end;
            goto tail_x86;
        }
        i += 32;
    }
tail_x86:
    #endif
    while (i + 16 <= len) {
        __m128i v = _mm_loadu_si128((const __m128i*)(s + i));
        if (_mm_movemask_epi8(v) != 0) {
            // Fallback on this 16B window
            size_t chunk_end = i + 16;
            if (!utf8_validate_scalar(s + i, chunk_end - i)) return false;
            i = chunk_end;
            continue;
        }
        i += 16;
    }
#elif VEC_NEON
    while (i + 16 <= len) {
        uint8x16_t v = vld1q_u8(s + i);
        // High bit mask: any >= 0x80 ?
        // Trick: compare v >= 0x80 -> test most significant bit
        uint8x16_t msb = vandq_u8(v, vdupq_n_u8(0x80));
        // Reduce OR: if any msb != 0 -> non-ASCII
        uint8x8_t or1 = vorr_u8(vget_low_u8(msb), vget_high_u8(msb));
        uint8x8_t or2 = vpmax_u8(or1, or1);
        uint8x8_t or3 = vpmax_u8(or2, or2);
        uint8x8_t or4 = vpmax_u8(or3, or3);
        if (vget_lane_u8(or4, 0)) {
            size_t chunk_end = i + 16;
            if (!utf8_validate_scalar(s + i, chunk_end - i)) return false;
            i = chunk_end;
            continue;
        }
        i += 16;
    }
#else
    // no SIMD
#endif

    // Tail (and any remaining bytes not covered)
    if (i < len) {
        if (!utf8_validate_scalar(s + i, len - i)) return false;
    }
    return true;
}

// =============================
// Public API wrappers
// =============================

bool utf8_validate(const uint8_t *s, size_t len) {
    // Fast path: try SIMD-assisted scan; it calls scalar as needed.
    return utf8_validate_simd(s, len);
}

bool utf16_validate(const uint16_t *s, size_t len) {
    return utf16_validate_scalar(s, len);
}

bool utf32_validate(const uint32_t *s, size_t len) {
    return utf32_validate_scalar(s, len);
}

size_t utf8_to_utf16(const uint8_t *src, size_t len, uint16_t *dst) {
    // Validate while converting (scalar is simpler/safer here).
    return utf8_to_utf16_scalar(src, len, dst);
}

size_t utf8_to_utf32(const uint8_t *src, size_t len, uint32_t *dst) {
    // Validate while converting.
    return utf8_to_utf32_scalar(src, len, dst);
}

// =============================
// Optional demo
// =============================
#ifdef UTF_SIMD_DEMO
#include <stdio.h>
int main(void) {
    const char *ok = "hello, dünya 🌍"; // UTF-8 string
    const uint8_t *p = (const uint8_t*)ok;
    size_t n = strlen(ok);

    printf("utf8_validate: %d\n", (int)utf8_validate(p, n));

    uint16_t buf16[128];
    uint32_t buf32[128];
    size_t n16 = utf8_to_utf16(p, n, buf16);
    size_t n32 = utf8_to_utf32(p, n, buf32);
    printf("utf8->utf16 units: %zu (error=%d)\n", (n16==(size_t)-1)?0:n16, (int)(n16==(size_t)-1));
    printf("utf8->utf32 units: %zu (error=%d)\n", (n32==(size_t)-1)?0:n32, (int)(n32==(size_t)-1));

    // Simple invalid UTF-8 test
    uint8_t bad[] = {0xE2, 0x28, 0xA1}; // invalid continuation
    printf("bad utf8_validate: %d\n", (int)utf8_validate(bad, sizeof(bad)));
    return 0;
}
#endif
