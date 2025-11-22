/**
 * Vex String Operations
 * SIMD-optimized, zero-overhead implementations
 */

#include "vex.h"
#include <stdio.h>  // For snprintf
#include <string.h> // For strstr, memcpy

// vex.h already includes vex_macros.h which provides:
// - VEX_SIMD_X86, VEX_SIMD_NEON (with proper intrinsics)
// - VEX_LIKELY, VEX_UNLIKELY
// - VEX_RESTRICT, VEX_INLINE, etc.

// ============================================================================
// OPTIMIZED STRING OPERATIONS
// ============================================================================

/**
 * strlen - SIMD optimized
 * Strategy: Process 16/32 bytes at a time, find null byte with vector instructions
 */
size_t vex_strlen(const char *s)
{
    const char *start = s;

#if VEX_SIMD_X86
    // x86 SSE2 version (16 bytes at a time)
    while (((uintptr_t)s & 15) != 0) // Align to 16-byte boundary
    {
        if (*s == '\0')
            return s - start;
        s++;
    }

    __m128i zero = _mm_setzero_si128();

    while (1)
    {
        __m128i chunk = _mm_load_si128((const __m128i *)s);
        __m128i cmp = _mm_cmpeq_epi8(chunk, zero);
        int mask = _mm_movemask_epi8(cmp);

        if (mask != 0)
        {
            // Found null byte
            return s - start + __builtin_ctz(mask);
        }

        s += 16;
    }
#elif VEX_SIMD_NEON
    // ARM NEON version (16 bytes at a time)
    while (((uintptr_t)s & 15) != 0) // Align to 16-byte boundary
    {
        if (*s == '\0')
            return s - start;
        s++;
    }

    uint8x16_t zero = vdupq_n_u8(0);

    while (1)
    {
        uint8x16_t chunk = vld1q_u8((const uint8_t *)s);
        uint8x16_t cmp = vceqq_u8(chunk, zero);

        // Check if any byte matched
        uint8x8_t narrow = vorr_u8(vget_low_u8(cmp), vget_high_u8(cmp));
        if (vget_lane_u64((uint64x1_t)narrow, 0) != 0)
        {
            // Found null byte in this chunk - scan byte by byte
            for (int i = 0; i < 16; i++)
            {
                if (s[i] == '\0')
                    return s - start + i;
            }
        }

        s += 16;
    }
#else
    // Scalar fallback - but optimized (unrolled)
    while (1)
    {
        if (s[0] == '\0')
            return s - start;
        if (s[1] == '\0')
            return s - start + 1;
        if (s[2] == '\0')
            return s - start + 2;
        if (s[3] == '\0')
            return s - start + 3;
        s += 4;
    }
#endif
}

/**
 * strcmp - SIMD optimized
 * Strategy: Compare 16 bytes at a time
 */
int vex_strcmp(const char *s1, const char *s2)
{
#if VEX_SIMD_X86
    // Fast path: compare 16 bytes at a time
    while (((uintptr_t)s1 & 15) != 0 || ((uintptr_t)s2 & 15) != 0)
    {
        if (*s1 != *s2 || *s1 == '\0')
            return *(unsigned char *)s1 - *(unsigned char *)s2;
        s1++;
        s2++;
    }

    __m128i zero = _mm_setzero_si128();

    while (1)
    {
        __m128i v1 = _mm_load_si128((const __m128i *)s1);
        __m128i v2 = _mm_load_si128((const __m128i *)s2);

        // Check for null bytes
        __m128i null_check = _mm_cmpeq_epi8(v1, zero);
        int null_mask = _mm_movemask_epi8(null_check);

        // Compare bytes
        __m128i cmp = _mm_cmpeq_epi8(v1, v2);
        int cmp_mask = _mm_movemask_epi8(cmp);

        if (null_mask != 0 || cmp_mask != 0xFFFF)
        {
            // Found difference or null - scan byte by byte
            for (int i = 0; i < 16; i++)
            {
                if (s1[i] != s2[i] || s1[i] == '\0')
                    return (unsigned char)s1[i] - (unsigned char)s2[i];
            }
        }

        s1 += 16;
        s2 += 16;
    }
#elif VEX_SIMD_NEON
    // ARM NEON version
    while (((uintptr_t)s1 & 15) != 0 || ((uintptr_t)s2 & 15) != 0)
    {
        if (*s1 != *s2 || *s1 == '\0')
            return *(unsigned char *)s1 - *(unsigned char *)s2;
        s1++;
        s2++;
    }

    uint8x16_t zero = vdupq_n_u8(0);

    while (1)
    {
        uint8x16_t v1 = vld1q_u8((const uint8_t *)s1);
        uint8x16_t v2 = vld1q_u8((const uint8_t *)s2);

        // Check for null or difference
        uint8x16_t null_check = vceqq_u8(v1, zero);
        uint8x16_t cmp = vceqq_u8(v1, v2);

        uint8x16_t any_diff = vbicq_u8(vdupq_n_u8(0xFF), cmp);
        uint8x16_t combined = vorrq_u8(null_check, any_diff);

        uint8x8_t narrow = vorr_u8(vget_low_u8(combined), vget_high_u8(combined));
        if (vget_lane_u64((uint64x1_t)narrow, 0) != 0)
        {
            // Found difference or null - scan byte by byte
            for (int i = 0; i < 16; i++)
            {
                if (s1[i] != s2[i] || s1[i] == '\0')
                    return (unsigned char)s1[i] - (unsigned char)s2[i];
            }
        }

        s1 += 16;
        s2 += 16;
    }
#else
    // Optimized scalar with 4-byte unrolling
    while (1)
    {
        if (*s1 != *s2 || *s1 == '\0')
            return *(unsigned char *)s1 - *(unsigned char *)s2;
        s1++;
        s2++;
        if (*s1 != *s2 || *s1 == '\0')
            return *(unsigned char *)s1 - *(unsigned char *)s2;
        s1++;
        s2++;
        if (*s1 != *s2 || *s1 == '\0')
            return *(unsigned char *)s1 - *(unsigned char *)s2;
        s1++;
        s2++;
        if (*s1 != *s2 || *s1 == '\0')
            return *(unsigned char *)s1 - *(unsigned char *)s2;
        s1++;
        s2++;
    }
#endif
}

/**
 * strncmp - Optimized n-byte comparison
 */
int vex_strncmp(const char *s1, const char *s2, size_t n)
{
    if (n == 0)
        return 0;

#if VEX_SIMD_X86
    // Process 16 bytes at a time if n is large enough
    while (n >= 16)
    {
        __m128i v1 = _mm_loadu_si128((const __m128i *)s1);
        __m128i v2 = _mm_loadu_si128((const __m128i *)s2);
        __m128i cmp = _mm_cmpeq_epi8(v1, v2);
        int mask = _mm_movemask_epi8(cmp);

        if (mask != 0xFFFF)
        {
            // Found difference - locate it
            for (size_t i = 0; i < 16 && i < n; i++)
            {
                if (s1[i] != s2[i] || s1[i] == '\0')
                    return (unsigned char)s1[i] - (unsigned char)s2[i];
            }
        }

        // Check for null terminator
        __m128i zero = _mm_setzero_si128();
        __m128i null_check = _mm_cmpeq_epi8(v1, zero);
        if (_mm_movemask_epi8(null_check) != 0)
            return 0; // Strings are equal up to null

        s1 += 16;
        s2 += 16;
        n -= 16;
    }
#endif

    // Scalar for remaining bytes
    for (size_t i = 0; i < n; i++)
    {
        if (s1[i] != s2[i])
            return (unsigned char)s1[i] - (unsigned char)s2[i];
        if (s1[i] == '\0')
            return 0;
    }
    return 0;
}

char *vex_strcpy(char *dest, const char *src)
{
    char *d = dest;
    while ((*d++ = *src++))
        ;
    return dest;
}

char *vex_strcat(char *dest, const char *src)
{
    char *d = dest;
    // Find end of dest
    while (*d)
    {
        d++;
    }
    // Copy src to end of dest
    while ((*d++ = *src++))
        ;
    return dest;
}

// ⭐ NEW: String concatenation (allocates new string)
char *vex_strcat_new(const char *s1, const char *s2)
{
    size_t len1 = vex_strlen(s1);
    size_t len2 = vex_strlen(s2);
    size_t total_len = len1 + len2 + 1; // +1 for null terminator

    char *result = (char *)vex_malloc(total_len);
    if (!result)
    {
        return NULL; // Allocation failed
    }

    // Copy s1
    vex_memcpy(result, s1, len1);
    // Copy s2
    vex_memcpy(result + len1, s2, len2 + 1); // +1 to include null terminator

    return result;
}

// vex_strdup removed - duplicate definition in vex_alloc.c

// ============================================================================
// UTF-8/UTF-16/UTF-32 OPERATIONS (SIMD-accelerated)
// ============================================================================

// ----------------------------------------------------------------------------
// UTF-16/UTF-32 Validation and Conversion
// ----------------------------------------------------------------------------

/**
 * UTF-16 validation
 */
static inline bool utf16_validate_scalar(const uint16_t *s, size_t len)
{
    size_t i = 0;
    while (i < len)
    {
        uint16_t w1 = s[i++];
        if (w1 >= 0xD800 && w1 <= 0xDBFF)
        { // high surrogate
            if (i >= len)
                return false;
            uint16_t w2 = s[i++];
            if (!(w2 >= 0xDC00 && w2 <= 0xDFFF))
                return false;
        }
        else if (w1 >= 0xDC00 && w1 <= 0xDFFF)
        {
            return false; // lone low surrogate
        }
    }
    return true;
}

bool vex_utf16_validate(const uint16_t *s, size_t len)
{
    return utf16_validate_scalar(s, len);
}

/**
 * UTF-32 validation
 */
static inline bool utf32_validate_scalar(const uint32_t *s, size_t len)
{
    for (size_t i = 0; i < len; ++i)
    {
        uint32_t cp = s[i];
        if (cp > 0x10FFFF)
            return false;
        if (cp >= 0xD800 && cp <= 0xDFFF)
            return false;
    }
    return true;
}

bool vex_utf32_validate(const uint32_t *s, size_t len)
{
    return utf32_validate_scalar(s, len);
}

/**
 * UTF-8 -> UTF-16 conversion
 * Returns number of UTF-16 units written, or (size_t)-1 on error
 */
size_t vex_utf8_to_utf16(const uint8_t *src, size_t len, uint16_t *dst)
{
    size_t i = 0, j = 0;
    while (i < len)
    {
        uint8_t c = src[i];
        if (c < 0x80)
        {
            dst[j++] = c;
            i++;
        }
        else if ((c >> 5) == 0x6)
        { // 2-byte
            if (i + 1 >= len)
                return (size_t)-1;
            uint8_t c2 = src[i + 1];
            if ((c2 & 0xC0) != 0x80 || c < 0xC2)
                return (size_t)-1;
            dst[j++] = ((c & 0x1F) << 6) | (c2 & 0x3F);
            i += 2;
        }
        else if ((c >> 4) == 0xE)
        { // 3-byte
            if (i + 2 >= len)
                return (size_t)-1;
            uint8_t c2 = src[i + 1], c3 = src[i + 2];
            if ((c2 & 0xC0) != 0x80 || (c3 & 0xC0) != 0x80)
                return (size_t)-1;
            uint32_t cp = ((c & 0x0F) << 12) | ((c2 & 0x3F) << 6) | (c3 & 0x3F);
            if (cp >= 0xD800 && cp <= 0xDFFF)
                return (size_t)-1;
            if (c == 0xE0 && c2 < 0xA0)
                return (size_t)-1;
            if (c == 0xED && c2 > 0x9F)
                return (size_t)-1;
            dst[j++] = (uint16_t)cp;
            i += 3;
        }
        else if ((c >> 3) == 0x1E)
        { // 4-byte
            if (i + 3 >= len)
                return (size_t)-1;
            uint8_t c2 = src[i + 1], c3 = src[i + 2], c4 = src[i + 3];
            if ((c2 & 0xC0) != 0x80 || (c3 & 0xC0) != 0x80 || (c4 & 0xC0) != 0x80)
                return (size_t)-1;
            uint32_t cp = ((c & 0x07) << 18) | ((c2 & 0x3F) << 12) | ((c3 & 0x3F) << 6) | (c4 & 0x3F);
            if (cp > 0x10FFFF)
                return (size_t)-1;
            if (c == 0xF0 && c2 < 0x90)
                return (size_t)-1;
            if (c == 0xF4 && c2 > 0x8F)
                return (size_t)-1;
            cp -= 0x10000;
            dst[j++] = (uint16_t)(0xD800 | (cp >> 10));
            dst[j++] = (uint16_t)(0xDC00 | (cp & 0x3FF));
            i += 4;
        }
        else
        {
            return (size_t)-1;
        }
    }
    return j;
}

/**
 * UTF-8 -> UTF-32 conversion
 * Returns number of UTF-32 units written, or (size_t)-1 on error
 */
size_t vex_utf8_to_utf32(const uint8_t *src, size_t len, uint32_t *dst)
{
    size_t i = 0, j = 0;
    while (i < len)
    {
        uint8_t c = src[i];
        if (c < 0x80)
        {
            dst[j++] = c;
            i++;
        }
        else if ((c >> 5) == 0x6)
        { // 2-byte
            if (i + 1 >= len)
                return (size_t)-1;
            uint8_t c2 = src[i + 1];
            if ((c2 & 0xC0) != 0x80 || c < 0xC2)
                return (size_t)-1;
            dst[j++] = ((c & 0x1F) << 6) | (c2 & 0x3F);
            i += 2;
        }
        else if ((c >> 4) == 0xE)
        { // 3-byte
            if (i + 2 >= len)
                return (size_t)-1;
            uint8_t c2 = src[i + 1], c3 = src[i + 2];
            if ((c2 & 0xC0) != 0x80 || (c3 & 0xC0) != 0x80)
                return (size_t)-1;
            uint32_t cp = ((c & 0x0F) << 12) | ((c2 & 0x3F) << 6) | (c3 & 0x3F);
            if (cp >= 0xD800 && cp <= 0xDFFF)
                return (size_t)-1;
            if (c == 0xE0 && c2 < 0xA0)
                return (size_t)-1;
            if (c == 0xED && c2 > 0x9F)
                return (size_t)-1;
            dst[j++] = cp;
            i += 3;
        }
        else if ((c >> 3) == 0x1E)
        { // 4-byte
            if (i + 3 >= len)
                return (size_t)-1;
            uint8_t c2 = src[i + 1], c3 = src[i + 2], c4 = src[i + 3];
            if ((c2 & 0xC0) != 0x80 || (c3 & 0xC0) != 0x80 || (c4 & 0xC0) != 0x80)
                return (size_t)-1;
            uint32_t cp = ((c & 0x07) << 18) | ((c2 & 0x3F) << 12) | ((c3 & 0x3F) << 6) | (c4 & 0x3F);
            if (cp > 0x10FFFF)
                return (size_t)-1;
            if (c == 0xF0 && c2 < 0x90)
                return (size_t)-1;
            if (c == 0xF4 && c2 > 0x8F)
                return (size_t)-1;
            dst[j++] = cp;
            i += 4;
        }
        else
        {
            return (size_t)-1;
        }
    }
    return j;
}

// ----------------------------------------------------------------------------
// UTF-8 Operations
// ----------------------------------------------------------------------------

/**
 * Check if byte is a UTF-8 continuation byte (10xxxxxx)
 */
static inline bool vex_utf8_is_continuation(unsigned char byte)
{
    return (byte & 0xC0) == 0x80;
}

/**
 * Get the length of a UTF-8 character from its first byte
 * Returns 0 for invalid UTF-8
 */
static inline size_t vex_utf8_char_len(unsigned char first_byte)
{
    if ((first_byte & 0x80) == 0x00)
    {
        // 0xxxxxxx - 1 byte (ASCII)
        return 1;
    }
    else if ((first_byte & 0xE0) == 0xC0)
    {
        // 110xxxxx - 2 bytes
        return 2;
    }
    else if ((first_byte & 0xF0) == 0xE0)
    {
        // 1110xxxx - 3 bytes
        return 3;
    }
    else if ((first_byte & 0xF8) == 0xF0)
    {
        // 11110xxx - 4 bytes
        return 4;
    }
    // Invalid UTF-8
    return 0;
}

/**
 * Validate UTF-8 string (SIMD-accelerated scalar fallback)
 * Uses algorithm from vex_simd_utf.c
 * Returns true if valid UTF-8, false otherwise
 */
static inline bool utf8_validate_scalar(const uint8_t *s, size_t len)
{
    size_t i = 0;
    while (i < len)
    {
        uint8_t c = s[i];
        if (c < 0x80)
        {
            i++;
            continue;
        }
        else if ((c >> 5) == 0x6)
        { // 110xxxxx - 2 bytes
            if (i + 1 >= len)
                return false;
            uint8_t c2 = s[i + 1];
            if ((c2 & 0xC0) != 0x80)
                return false;
            if (c < 0xC2)
                return false; // overlong
            i += 2;
        }
        else if ((c >> 4) == 0xE)
        { // 1110xxxx - 3 bytes
            if (i + 2 >= len)
                return false;
            uint8_t c2 = s[i + 1], c3 = s[i + 2];
            if ((c2 & 0xC0) != 0x80 || (c3 & 0xC0) != 0x80)
                return false;
            uint32_t cp = ((c & 0x0F) << 12) | ((c2 & 0x3F) << 6) | (c3 & 0x3F);
            if (cp >= 0xD800 && cp <= 0xDFFF)
                return false; // surrogates not allowed
            if (c == 0xE0 && c2 < 0xA0)
                return false; // overlong 3-byte
            if (c == 0xED && c2 > 0x9F)
                return false; // surrogate region
            i += 3;
        }
        else if ((c >> 3) == 0x1E)
        { // 11110xxx - 4 bytes
            if (i + 3 >= len)
                return false;
            uint8_t c2 = s[i + 1], c3 = s[i + 2], c4 = s[i + 3];
            if ((c2 & 0xC0) != 0x80 || (c3 & 0xC0) != 0x80 || (c4 & 0xC0) != 0x80)
                return false;
            uint32_t cp = ((c & 0x07) << 18) | ((c2 & 0x3F) << 12) | ((c3 & 0x3F) << 6) | (c4 & 0x3F);
            if (cp > 0x10FFFF)
                return false;
            if (c == 0xF0 && c2 < 0x90)
                return false; // overlong 4-byte
            if (c == 0xF4 && c2 > 0x8F)
                return false; // > U+10FFFF
            i += 4;
        }
        else
        {
            return false;
        }
    }
    return true;
}

/**
 * Validate UTF-8 string with SIMD acceleration
 * Fast path: scan 16/32-byte chunks for all-ASCII
 * Slow path: fall back to rigorous scalar validation
 */
bool vex_utf8_valid(const char *s, size_t byte_len)
{
    if (!s)
        return false;

    const uint8_t *str = (const uint8_t *)s;
    size_t i = 0;

#if VEX_SIMD_X86
#if defined(__AVX2__)
    // AVX2: 32 bytes at a time
    while (i + 32 <= byte_len)
    {
        __m256i v = _mm256_loadu_si256((const __m256i *)(str + i));
        // Check if all bytes < 0x80 (ASCII fast path)
        if (_mm256_movemask_epi8(v) != 0)
        {
            // Non-ASCII detected: validate this chunk rigorously
            size_t chunk_end = i + 32;
            if (!utf8_validate_scalar(str + i, chunk_end - i))
                return false;
            i = chunk_end;
            continue;
        }
        i += 32;
    }
#endif

    // SSE2: 16 bytes at a time
    while (i + 16 <= byte_len)
    {
        __m128i v = _mm_loadu_si128((const __m128i *)(str + i));
        if (_mm_movemask_epi8(v) != 0)
        {
            // Non-ASCII: validate rigorously
            size_t chunk_end = i + 16;
            if (!utf8_validate_scalar(str + i, chunk_end - i))
                return false;
            i = chunk_end;
            continue;
        }
        i += 16;
    }
#elif VEX_SIMD_NEON
    // ARM NEON: 16 bytes at a time
    while (i + 16 <= byte_len)
    {
        uint8x16_t v = vld1q_u8(str + i);
        // Check for non-ASCII (high bit set)
        uint8x16_t msb = vandq_u8(v, vdupq_n_u8(0x80));
        uint8x8_t or1 = vorr_u8(vget_low_u8(msb), vget_high_u8(msb));
        uint8x8_t or2 = vpmax_u8(or1, or1);
        uint8x8_t or3 = vpmax_u8(or2, or2);
        uint8x8_t or4 = vpmax_u8(or3, or3);
        if (vget_lane_u8(or4, 0))
        {
            // Non-ASCII: validate rigorously
            size_t chunk_end = i + 16;
            if (!utf8_validate_scalar(str + i, chunk_end - i))
                return false;
            i = chunk_end;
            continue;
        }
        i += 16;
    }
#endif

    // Tail: validate remaining bytes
    if (i < byte_len)
    {
        if (!utf8_validate_scalar(str + i, byte_len - i))
            return false;
    }

    return true;
}

/**
 * Count UTF-8 characters (not bytes) in a string
 * Returns character count, or 0 if invalid UTF-8
 */
size_t vex_utf8_char_count(const char *s)
{
    if (!s)
        return 0;

    const unsigned char *str = (const unsigned char *)s;
    size_t char_count = 0;
    size_t i = 0;

    while (str[i] != '\0')
    {
        size_t char_len = vex_utf8_char_len(str[i]);

        if (char_len == 0)
        {
            // Invalid UTF-8
            vex_panic("utf8_char_count: invalid UTF-8 sequence");
        }

        // Validate continuation bytes
        for (size_t j = 1; j < char_len; j++)
        {
            if (!vex_utf8_is_continuation(str[i + j]))
            {
                vex_panic("utf8_char_count: invalid UTF-8 continuation byte");
            }
        }

        char_count++;
        i += char_len;
    }

    return char_count;
}

/**
 * Get pointer to the Nth UTF-8 character (0-indexed)
 * Returns NULL if index out of bounds or invalid UTF-8
 */
const char *vex_utf8_char_at(const char *s, size_t char_index)
{
    if (!s)
    {
        vex_panic("utf8_char_at: NULL string pointer");
    }

    const unsigned char *str = (const unsigned char *)s;
    size_t current_char = 0;
    size_t i = 0;

    while (str[i] != '\0')
    {
        if (current_char == char_index)
        {
            return (const char *)&str[i];
        }

        size_t char_len = vex_utf8_char_len(str[i]);

        if (char_len == 0)
        {
            vex_panic("utf8_char_at: invalid UTF-8 sequence");
        }

        // Validate continuation bytes
        for (size_t j = 1; j < char_len; j++)
        {
            if (!vex_utf8_is_continuation(str[i + j]))
            {
                vex_panic("utf8_char_at: invalid UTF-8 continuation byte");
            }
        }

        current_char++;
        i += char_len;
    }

    // Index out of bounds
    char msg[128];
    vex_sprintf(msg, "utf8_char_at: index out of bounds (index: %zu, length: %zu)",
                char_index, current_char);
    vex_panic(msg);
    return NULL; // Never reached
}

/**
 * Convert UTF-8 character index to byte index
 * Returns byte index, or panics if invalid
 */
size_t vex_utf8_char_to_byte_index(const char *s, size_t char_index)
{
    if (!s)
    {
        vex_panic("utf8_char_to_byte_index: NULL string pointer");
    }

    const unsigned char *str = (const unsigned char *)s;
    size_t current_char = 0;
    size_t byte_index = 0;

    while (str[byte_index] != '\0')
    {
        if (current_char == char_index)
        {
            return byte_index;
        }

        size_t char_len = vex_utf8_char_len(str[byte_index]);

        if (char_len == 0)
        {
            vex_panic("utf8_char_to_byte_index: invalid UTF-8 sequence");
        }

        current_char++;
        byte_index += char_len;
    }

    // Index out of bounds
    char msg[128];
    vex_sprintf(msg, "utf8_char_to_byte_index: index out of bounds (index: %zu, length: %zu)",
                char_index, current_char);
    vex_panic(msg);
    return 0; // Never reached
}

/**
 * Extract a single UTF-8 character at index and return as new string
 * Allocates memory (must be freed)
 */
char *vex_utf8_char_extract(const char *s, size_t char_index)
{
    const char *char_ptr = vex_utf8_char_at(s, char_index);
    size_t char_len = vex_utf8_char_len(*((unsigned char *)char_ptr));

    char *result = (char *)vex_malloc(char_len + 1);
    if (!result)
    {
        vex_panic("utf8_char_extract: out of memory");
    }

    vex_memcpy(result, char_ptr, char_len);
    result[char_len] = '\0';

    return result;
}

/**
 * Decode UTF-8 character to Unicode code point
 * Returns code point (0-0x10FFFF) or 0xFFFFFFFF on error
 */
uint32_t vex_utf8_decode(const char *s)
{
    if (!s)
        return 0xFFFFFFFF;

    const unsigned char *str = (const unsigned char *)s;
    size_t char_len = vex_utf8_char_len(str[0]);

    if (char_len == 0)
        return 0xFFFFFFFF;

    uint32_t code_point;

    if (char_len == 1)
    {
        code_point = str[0];
    }
    else if (char_len == 2)
    {
        code_point = ((str[0] & 0x1F) << 6) | (str[1] & 0x3F);
    }
    else if (char_len == 3)
    {
        code_point = ((str[0] & 0x0F) << 12) |
                     ((str[1] & 0x3F) << 6) |
                     (str[2] & 0x3F);
    }
    else
    { // char_len == 4
        code_point = ((str[0] & 0x07) << 18) |
                     ((str[1] & 0x3F) << 12) |
                     ((str[2] & 0x3F) << 6) |
                     (str[3] & 0x3F);
    }

    return code_point;
}

/**
 * Encode Unicode code point to UTF-8
 * Writes to buf (must have at least 5 bytes)
 * Returns number of bytes written, or 0 on error
 */
size_t vex_utf8_encode(uint32_t code_point, char *buf)
{
    if (!buf)
        return 0;

    if (code_point <= 0x7F)
    {
        // 1 byte: 0xxxxxxx
        buf[0] = (char)code_point;
        buf[1] = '\0';
        return 1;
    }
    else if (code_point <= 0x7FF)
    {
        // 2 bytes: 110xxxxx 10xxxxxx
        buf[0] = (char)(0xC0 | (code_point >> 6));
        buf[1] = (char)(0x80 | (code_point & 0x3F));
        buf[2] = '\0';
        return 2;
    }
    else if (code_point <= 0xFFFF)
    {
        // 3 bytes: 1110xxxx 10xxxxxx 10xxxxxx
        // Check for surrogates
        if (code_point >= 0xD800 && code_point <= 0xDFFF)
        {
            return 0; // Invalid (surrogate range)
        }
        buf[0] = (char)(0xE0 | (code_point >> 12));
        buf[1] = (char)(0x80 | ((code_point >> 6) & 0x3F));
        buf[2] = (char)(0x80 | (code_point & 0x3F));
        buf[3] = '\0';
        return 3;
    }
    else if (code_point <= 0x10FFFF)
    {
        // 4 bytes: 11110xxx 10xxxxxx 10xxxxxx 10xxxxxx
        buf[0] = (char)(0xF0 | (code_point >> 18));
        buf[1] = (char)(0x80 | ((code_point >> 12) & 0x3F));
        buf[2] = (char)(0x80 | ((code_point >> 6) & 0x3F));
        buf[3] = (char)(0x80 | (code_point & 0x3F));
        buf[4] = '\0';
        return 4;
    }

    return 0; // Invalid code point
}

// ============================================================================
// NUMERIC TO STRING CONVERSIONS
// ============================================================================

/**
 * Convert i32 to string
 * Returns heap-allocated string (caller must free)
 */
/*
char *vex_i32_to_string(int32_t value)
{
    // Max length: "-2147483648" = 11 chars + null terminator
    char *buf = (char *)vex_malloc(12);
    if (!buf)
        return NULL;

    snprintf(buf, 12, "%d", value);
    return buf;
}

/**
 * Convert i64 to string
 * Returns heap-allocated string (caller must free)
 */
/*
char *vex_i64_to_string(int64_t value)
{
    // Max length: "-9223372036854775808" = 20 chars + null terminator
    char *buf = (char *)vex_malloc(21);
    if (!buf)
        return NULL;

    snprintf(buf, 21, "%lld", (long long)value);
    return buf;
}

char *vex_u32_to_string(uint32_t value)
{
    // Max length: "4294967295" = 10 chars + null terminator
    char *buf = (char *)vex_malloc(11);
    if (!buf)
        return NULL;

    snprintf(buf, 11, "%u", value);
    return buf;
}

char *vex_u64_to_string(uint64_t value)
{
    // Max length: "18446744073709551615" = 20 chars + null terminator
    char *buf = (char *)vex_malloc(21);
    if (!buf)
        return NULL;

    snprintf(buf, 21, "%llu", (unsigned long long)value);
    return buf;
}

char *vex_f32_to_string(float value)
{
    // Max length: "-3.402823e+38" + some buffer = 32 chars
    char *buf = (char *)vex_malloc(32);
    if (!buf)
        return NULL;

    snprintf(buf, 32, "%g", value); // %g uses shortest representation
    return buf;
}
*/

// ============================================================================
// GO-LIKE STRING OPERATIONS
// ============================================================================

/**
 * Contains - Check if string contains substring
 * Returns true if substr is found in s
 */
bool vex_str_contains(const char *s, const char *substr)
{
    if (!s || !substr)
        return false;
    if (*substr == '\0')
        return true; // Empty string is contained in any string

    return strstr(s, substr) != NULL;
}

/**
 * HasPrefix - Check if string starts with prefix
 */
bool vex_str_has_prefix(const char *s, const char *prefix)
{
    if (!s || !prefix)
        return false;

    size_t prefix_len = vex_strlen(prefix);
    return vex_strncmp(s, prefix, prefix_len) == 0;
}

/**
 * HasSuffix - Check if string ends with suffix
 */
bool vex_str_has_suffix(const char *s, const char *suffix)
{
    if (!s || !suffix)
        return false;

    size_t s_len = vex_strlen(s);
    size_t suffix_len = vex_strlen(suffix);

    if (suffix_len > s_len)
        return false;

    return vex_strcmp(s + (s_len - suffix_len), suffix) == 0;
}

/**
 * ToUpper - Convert string to uppercase (ASCII only)
 * Returns heap-allocated string (caller must free)
 */
char *vex_str_to_upper(const char *s)
{
    if (!s)
        return NULL;

    size_t len = vex_strlen(s);
    char *result = (char *)vex_malloc(len + 1);
    if (!result)
        return NULL;

    for (size_t i = 0; i <= len; i++)
    {
        char c = s[i];
        result[i] = (c >= 'a' && c <= 'z') ? (c - 32) : c;
    }

    return result;
}

/**
 * ToLower - Convert string to lowercase (ASCII only)
 * Returns heap-allocated string (caller must free)
 */
char *vex_str_to_lower(const char *s)
{
    if (!s)
        return NULL;

    size_t len = vex_strlen(s);
    char *result = (char *)vex_malloc(len + 1);
    if (!result)
        return NULL;

    for (size_t i = 0; i <= len; i++)
    {
        char c = s[i];
        result[i] = (c >= 'A' && c <= 'Z') ? (c + 32) : c;
    }

    return result;
}

/**
 * Trim - Remove leading and trailing whitespace
 * Returns heap-allocated string (caller must free)
 */
char *vex_str_trim(const char *s)
{
    if (!s)
        return NULL;

    // Skip leading whitespace
    while (*s && (*s == ' ' || *s == '\t' || *s == '\n' || *s == '\r'))
        s++;

    if (*s == '\0')
    {
        // All whitespace - return empty string
        char *result = (char *)vex_malloc(1);
        if (result)
            result[0] = '\0';
        return result;
    }

    // Find end (last non-whitespace)
    const char *end = s + vex_strlen(s) - 1;
    while (end > s && (*end == ' ' || *end == '\t' || *end == '\n' || *end == '\r'))
        end--;

    size_t len = end - s + 1;
    char *result = (char *)vex_malloc(len + 1);
    if (!result)
        return NULL;

    memcpy(result, s, len);
    result[len] = '\0';

    return result;
}

/**
 * Replace - Replace all occurrences of old with new
 * Returns heap-allocated string (caller must free)
 */
char *vex_str_replace(const char *s, const char *old_str, const char *new_str)
{
    if (!s || !old_str || !new_str)
        return NULL;

    size_t old_len = vex_strlen(old_str);
    size_t new_len = vex_strlen(new_str);

    if (old_len == 0)
    {
        // Can't replace empty string - return copy
        return vex_strdup(s);
    }

    // Count occurrences
    size_t count = 0;
    const char *tmp = s;
    while ((tmp = strstr(tmp, old_str)) != NULL)
    {
        count++;
        tmp += old_len;
    }

    if (count == 0)
    {
        // No replacements - return copy
        return vex_strdup(s);
    }

    // Allocate result
    size_t s_len = vex_strlen(s);
    size_t result_len = s_len + count * (new_len - old_len);
    char *result = (char *)vex_malloc(result_len + 1);
    if (!result)
        return NULL;

    // Perform replacements
    char *dst = result;
    const char *src = s;

    while (*src)
    {
        const char *match = strstr(src, old_str);

        if (match == NULL)
        {
            // Copy rest of string
            vex_strcpy(dst, src);
            break;
        }

        // Copy up to match
        size_t prefix_len = match - src;
        memcpy(dst, src, prefix_len);
        dst += prefix_len;

        // Copy replacement
        memcpy(dst, new_str, new_len);
        dst += new_len;

        src = match + old_len;
    }

    return result;
}

/**
 * Split - Split string by delimiter
 * Returns array of strings (NULL-terminated)
 * Caller must free each string and the array itself
 */
char **vex_str_split(const char *s, const char *delim, size_t *out_count)
{
    if (!s || !delim || !out_count)
        return NULL;

    size_t delim_len = vex_strlen(delim);
    if (delim_len == 0)
    {
        // Empty delimiter - return single element (copy of s)
        char **result = (char **)vex_malloc(2 * sizeof(char *));
        if (!result)
            return NULL;
        result[0] = vex_strdup(s);
        result[1] = NULL;
        *out_count = 1;
        return result;
    }

    // Count occurrences
    size_t count = 1; // At least one part
    const char *tmp = s;
    while ((tmp = strstr(tmp, delim)) != NULL)
    {
        count++;
        tmp += delim_len;
    }

    // Allocate array (count + 1 for NULL terminator)
    char **result = (char **)vex_malloc((count + 1) * sizeof(char *));
    if (!result)
        return NULL;

    // Split
    size_t idx = 0;
    const char *start = s;
    const char *end;

    while ((end = strstr(start, delim)) != NULL)
    {
        size_t part_len = end - start;
        result[idx] = (char *)vex_malloc(part_len + 1);
        if (!result[idx])
        {
            // Cleanup on failure
            for (size_t i = 0; i < idx; i++)
                vex_free(result[i]);
            vex_free(result);
            return NULL;
        }
        memcpy(result[idx], start, part_len);
        result[idx][part_len] = '\0';
        idx++;
        start = end + delim_len;
    }

    // Last part
    result[idx] = vex_strdup(start);
    if (!result[idx])
    {
        for (size_t i = 0; i < idx; i++)
            vex_free(result[i]);
        vex_free(result);
        return NULL;
    }
    idx++;

    result[idx] = NULL; // NULL terminator
    *out_count = count;
    return result;
}

/**
 * Convert f64 to string
 * Returns heap-allocated string (caller must free)
 */
/*
char *vex_f64_to_string(double value)
{
    // Max length: "-1.797693e+308" + some buffer = 32 chars
    char *buf = (char *)vex_malloc(32);
    if (!buf)
        return NULL;

    snprintf(buf, 32, "%g", value); // %g uses shortest representation
    return buf;
}
*/

/**
 * Convert bool to string
 * Returns heap-allocated string (caller must free)
 */
/*
char *vex_bool_to_string(bool value)
{
    if (value)
        return vex_strdup("true");
    else
        return vex_strdup("false");
}
*/

/**
 * String to string (identity function for consistency)
 * Returns heap-allocated copy (caller must free)
 */
/*
char *vex_string_to_string(const char *value)
{
    return vex_strdup(value ? value : "");
}
*/

// ============================================================================
// STRING INDEXING AND SLICING (v0.1.2)
// ============================================================================

#include <stdlib.h> // For abort()

/**
 * Get byte at index (not UTF-8 character!)
 * Used for: text[3]
 * Returns: u8 byte value at index
 * Panics: If index >= length
 */
uint8_t vex_string_index(const char *str, size_t index)
{
    size_t len = vex_strlen(str);

    if (index >= len)
    {
        fprintf(stderr, "String index out of bounds: %zu >= %zu\n", index, len);
        abort(); // Runtime panic
    }

    return (uint8_t)str[index];
}

/**
 * Check if index is on UTF-8 character boundary
 * Returns: 1 if valid boundary, 0 if in middle of multi-byte char
 */
static int vex_is_utf8_boundary(const char *str, size_t index)
{
    // UTF-8 continuation bytes start with 0b10xxxxxx (0x80-0xBF)
    // Valid start positions: 0b0xxxxxxx or 0b11xxxxxx
    uint8_t byte = (uint8_t)str[index];
    return (byte & 0xC0) != 0x80; // Not a continuation byte
}

/**
 * String slicing - creates new substring
 * Used for: text[start..end], text[..end], text[start..], text[..]
 *
 * Args:
 *   str: Source string (null-terminated)
 *   start: Start index (inclusive), use 0 for text[..end]
 *   end: End index (exclusive), use -1 for text[start..]
 *
 * Returns: Heap-allocated substring (caller must free)
 * Panics: If indices out of bounds or split UTF-8 character
 */
char *vex_string_substr(const char *str, int64_t start, int64_t end)
{
    size_t len = vex_strlen(str);

    // Handle negative indices (not yet supported, but for future)
    if (start < 0)
        start = 0;
    if (end < 0)
        end = (int64_t)len; // -1 means "to end"

    // Bounds check
    if (start > (int64_t)len || end > (int64_t)len || start > end)
    {
        fprintf(stderr, "String slice out of bounds: [%lld..%lld] (len=%zu)\n",
                (long long)start, (long long)end, len);
        abort();
    }

    // UTF-8 boundary check
    if (!vex_is_utf8_boundary(str, start))
    {
        fprintf(stderr, "String slice splits UTF-8 character at start=%lld\n",
                (long long)start);
        abort();
    }

    if (end < (int64_t)len && !vex_is_utf8_boundary(str, end))
    {
        fprintf(stderr, "String slice splits UTF-8 character at end=%lld\n",
                (long long)end);
        abort();
    }

    // Allocate new string
    size_t slice_len = (size_t)(end - start);
    char *result = (char *)vex_malloc(slice_len + 1);
    if (!result)
    {
        fprintf(stderr, "String slice allocation failed\n");
        abort();
    }

    // Copy substring
    memcpy(result, str + start, slice_len);
    result[slice_len] = '\0';

    return result;
}

/**
 * Helper: Get string length (exposed for Vex code)
 * Note: Different from vex_string_len which takes vex_string_t*
 */
size_t vex_string_length(const char *str)
{
    return vex_strlen(str);
}