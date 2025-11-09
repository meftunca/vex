#include "simd_rfc3339.h"
#include "simd_detect.h"
#include <string.h>
#include <stdlib.h>
#include <stdio.h>
#include <time.h>

/* Platform compatibility wrappers */
#if defined(_WIN32)
static inline time_t timegm_compat(struct tm* t) { return _mkgmtime(t); }
static inline void gmtime_compat(const time_t* tt, struct tm* out) { gmtime_s(out, tt); }
#else
static inline time_t timegm_compat(struct tm* t) { return timegm(t); }
static inline void gmtime_compat(const time_t* tt, struct tm* out) { gmtime_r(tt, out); }
#endif

/* Function pointers for runtime dispatch */
static int (*parse_func)(const char*, VexInstant*) = NULL;
static int (*format_func)(VexInstant, char*, size_t) = NULL;

/* ============================================================================
 * Scalar (fallback) implementation
 * ============================================================================ */

static inline int parse_digit(char c) {
    return (c >= '0' && c <= '9') ? (c - '0') : -1;
}

static inline int parse_2digits(const char* s) {
    int d1 = parse_digit(s[0]);
    int d2 = parse_digit(s[1]);
    return (d1 >= 0 && d2 >= 0) ? (d1 * 10 + d2) : -1;
}

static inline int parse_4digits(const char* s) {
    int d1 = parse_digit(s[0]);
    int d2 = parse_digit(s[1]);
    int d3 = parse_digit(s[2]);
    int d4 = parse_digit(s[3]);
    return (d1 >= 0 && d2 >= 0 && d3 >= 0 && d4 >= 0) 
           ? (d1 * 1000 + d2 * 100 + d3 * 10 + d4) : -1;
}

static int parse_rfc3339_scalar(const char* s, VexInstant* out) {
    /* Fast path: minimum length check */
    size_t len = strlen(s);
    if (len < 20) return -1;
    
    /* Parse date: 2024-11-07 */
    int year = parse_4digits(s);
    if (year < 0 || s[4] != '-') return -1;
    
    int month = parse_2digits(s + 5);
    if (month < 1 || month > 12 || s[7] != '-') return -1;
    
    int day = parse_2digits(s + 8);
    if (day < 1 || day > 31 || s[10] != 'T') return -1;
    
    /* Parse time: 12:34:56 */
    int hour = parse_2digits(s + 11);
    if (hour < 0 || hour > 23 || s[13] != ':') return -1;
    
    int minute = parse_2digits(s + 14);
    if (minute < 0 || minute > 59 || s[16] != ':') return -1;
    
    int second = parse_2digits(s + 17);
    if (second < 0 || second > 60) return -1; /* 60 for leap second */
    
    /* Parse fractional seconds (optional) */
    int nsec = 0;
    const char* p = s + 19;
    if (*p == '.') {
        p++;
        int digits = 0;
        while (*p >= '0' && *p <= '9' && digits < 9) {
            nsec = nsec * 10 + (*p - '0');
            digits++;
            p++;
        }
        /* Pad to nanoseconds */
        while (digits < 9) {
            nsec *= 10;
            digits++;
        }
        /* Skip remaining fractional digits */
        while (*p >= '0' && *p <= '9') p++;
    }
    
    /* Parse timezone */
    int tz_offset = 0;
    if (*p == 'Z') {
        /* UTC */
    } else if (*p == '+' || *p == '-') {
        int sign = (*p == '-') ? -1 : 1;
        p++;
        int tz_hour = parse_2digits(p);
        if (tz_hour < 0) return -1;
        p += 2;
        
        int tz_min = 0;
        if (*p == ':') {
            p++;
            tz_min = parse_2digits(p);
            if (tz_min < 0) return -1;
        }
        
        tz_offset = sign * (tz_hour * 3600 + tz_min * 60);
    } else {
        return -1;
    }
    
    /* Convert to Unix timestamp */
    /* Simplified: use standard library for date arithmetic */
    struct tm tm;
    memset(&tm, 0, sizeof(tm));
    tm.tm_year = year - 1900;
    tm.tm_mon = month - 1;
    tm.tm_mday = day;
    tm.tm_hour = hour;
    tm.tm_min = minute;
    tm.tm_sec = second;
    
    time_t unix_time = timegm_compat(&tm);
    
    /* Apply timezone offset */
    unix_time -= tz_offset;
    
    out->unix_sec = (int64_t)unix_time;
    out->nsec = (int32_t)nsec;
    out->_pad = 0;
    
    return 0;
}

/* ============================================================================
 * SSE2/AVX2 implementation (x86/x64)
 * ============================================================================ */

#if defined(__x86_64__) || defined(_M_X64) || defined(__i386__) || defined(_M_IX86)
#include <emmintrin.h>  /* SSE2 */
#ifdef __AVX2__
#include <immintrin.h>  /* AVX2 */
#endif

static int parse_rfc3339_sse2(const char* s, VexInstant* out) {
    /* Fast length check */
    if (strlen(s) < 20) return -1;
    
    /* Load 16 bytes: "2024-11-07T12:34" */
    __m128i chunk = _mm_loadu_si128((const __m128i*)s);
    __m128i ascii_zero = _mm_set1_epi8('0');
    
    /* Convert ASCII digits to integers: digit - '0' */
    __m128i digits = _mm_sub_epi8(chunk, ascii_zero);
    
    /* Store to array for extraction */
    uint8_t d[16];
    _mm_storeu_si128((__m128i*)d, digits);
    
    /* Quick validation: check separators */
    if (s[4] != '-' || s[7] != '-' || s[10] != 'T' || s[13] != ':' || s[16] != ':') {
        return -1;
    }
    
    /* Parse with SIMD-extracted digits */
    int year = d[0] * 1000 + d[1] * 100 + d[2] * 10 + d[3];
    int month = d[5] * 10 + d[6];
    int day = d[8] * 10 + d[9];
    int hour = d[11] * 10 + d[12];
    int minute = d[14] * 10 + d[15];
    
    /* Parse seconds */
    int second = (s[17] - '0') * 10 + (s[18] - '0');
    
    /* Basic validation */
    if (month < 1 || month > 12 || day < 1 || day > 31 || 
        hour > 23 || minute > 59 || second > 60) {
        return -1;
    }
    
    /* Parse fractional seconds */
    int nsec = 0;
    const char* p = s + 19;
    if (*p == '.') {
        p++;
        int digits_count = 0;
        while (*p >= '0' && *p <= '9' && digits_count < 9) {
            nsec = nsec * 10 + (*p - '0');
            digits_count++;
            p++;
        }
        while (digits_count < 9) { nsec *= 10; digits_count++; }
        while (*p >= '0' && *p <= '9') p++;
    }
    
    /* Parse timezone */
    int tz_offset = 0;
    if (*p == 'Z') {
        /* UTC */
    } else if (*p == '+' || *p == '-') {
        int sign = (*p == '-') ? -1 : 1;
        p++;
        int tz_hour = (p[0] - '0') * 10 + (p[1] - '0');
        p += 2;
        int tz_min = 0;
        if (*p == ':') {
            p++;
            tz_min = (p[0] - '0') * 10 + (p[1] - '0');
        }
        tz_offset = sign * (tz_hour * 3600 + tz_min * 60);
    } else {
        return -1;
    }
    
    /* Convert to Unix timestamp */
    struct tm tm;
    memset(&tm, 0, sizeof(tm));
    tm.tm_year = year - 1900;
    tm.tm_mon = month - 1;
    tm.tm_mday = day;
    tm.tm_hour = hour;
    tm.tm_min = minute;
    tm.tm_sec = second;
    
    time_t unix_time = timegm_compat(&tm);
    unix_time -= tz_offset;
    
    out->unix_sec = (int64_t)unix_time;
    out->nsec = (int32_t)nsec;
    out->_pad = 0;
    
    return 0;
}

#ifdef __AVX2__
static int parse_rfc3339_avx2(const char* s, VexInstant* out) {
    /* AVX2 can process more in parallel, but RFC3339 is only 20-32 chars */
    /* SSE2 is sufficient for this use case */
    /* Use SSE2 implementation for now (already optimal for this size) */
    return parse_rfc3339_sse2(s, out);
}
#endif

#endif /* x86/x64 */

/* ============================================================================
 * NEON implementation (ARM)
 * ============================================================================ */

#if defined(__aarch64__) || defined(_M_ARM64) || defined(__ARM_NEON)
#include <arm_neon.h>

static int parse_rfc3339_neon(const char* s, VexInstant* out) {
    /* Fast length check */
    if (strlen(s) < 20) return -1;
    
    /* Load date part: "2024-11-07T12:34" (16 bytes) */
    uint8x16_t chunk1 = vld1q_u8((const uint8_t*)s);
    uint8x16_t ascii_zero = vdupq_n_u8('0');
    
    /* Convert ASCII digits to integers in parallel: digit - '0' */
    uint8x16_t digits1 = vsubq_u8(chunk1, ascii_zero);
    
    /* Extract year: digits at position 0,1,2,3 */
    uint8_t d[16];
    vst1q_u8(d, digits1);
    
    /* Quick validation: check separators */
    if (s[4] != '-' || s[7] != '-' || s[10] != 'T' || s[13] != ':' || s[16] != ':') {
        return -1;
    }
    
    /* Parse with SIMD-extracted digits */
    int year = d[0] * 1000 + d[1] * 100 + d[2] * 10 + d[3];
    int month = d[5] * 10 + d[6];
    int day = d[8] * 10 + d[9];
    int hour = d[11] * 10 + d[12];
    int minute = d[14] * 10 + d[15];
    
    /* Load next chunk for seconds: ":56.123456789Z" */
    int second = (s[17] - '0') * 10 + (s[18] - '0');
    
    /* Basic validation */
    if (month < 1 || month > 12 || day < 1 || day > 31 || 
        hour > 23 || minute > 59 || second > 60) {
        return -1;
    }
    
    /* Parse fractional seconds (if present) */
    int nsec = 0;
    const char* p = s + 19;
    if (*p == '.') {
        p++;
        int digits = 0;
        while (*p >= '0' && *p <= '9' && digits < 9) {
            nsec = nsec * 10 + (*p - '0');
            digits++;
            p++;
        }
        while (digits < 9) { nsec *= 10; digits++; }
        while (*p >= '0' && *p <= '9') p++;
    }
    
    /* Parse timezone */
    int tz_offset = 0;
    if (*p == 'Z') {
        /* UTC */
    } else if (*p == '+' || *p == '-') {
        int sign = (*p == '-') ? -1 : 1;
        p++;
        int tz_hour = (p[0] - '0') * 10 + (p[1] - '0');
        p += 2;
        int tz_min = 0;
        if (*p == ':') {
            p++;
            tz_min = (p[0] - '0') * 10 + (p[1] - '0');
        }
        tz_offset = sign * (tz_hour * 3600 + tz_min * 60);
    } else {
        return -1;
    }
    
    /* Convert to Unix timestamp */
    struct tm tm;
    memset(&tm, 0, sizeof(tm));
    tm.tm_year = year - 1900;
    tm.tm_mon = month - 1;
    tm.tm_mday = day;
    tm.tm_hour = hour;
    tm.tm_min = minute;
    tm.tm_sec = second;
    
    time_t unix_time = timegm_compat(&tm);
    unix_time -= tz_offset;
    
    out->unix_sec = (int64_t)unix_time;
    out->nsec = (int32_t)nsec;
    out->_pad = 0;
    
    return 0;
}

#endif /* ARM */

/* ============================================================================
 * Format (scalar for now, SIMD helps less here)
 * ============================================================================ */

static int format_rfc3339_scalar(VexInstant t, char* buf, size_t buflen) {
    if (!buf || buflen < 25) return -1;
    
    time_t sec = (time_t)t.unix_sec;
    struct tm tmv;
    gmtime_compat(&sec, &tmv);
    
    if (t.nsec) {
        snprintf(buf, buflen, "%04d-%02d-%02dT%02d:%02d:%02d.%09dZ",
                tmv.tm_year + 1900, tmv.tm_mon + 1, tmv.tm_mday,
                tmv.tm_hour, tmv.tm_min, tmv.tm_sec, t.nsec);
    } else {
        snprintf(buf, buflen, "%04d-%02d-%02dT%02d:%02d:%02dZ",
                tmv.tm_year + 1900, tmv.tm_mon + 1, tmv.tm_mday,
                tmv.tm_hour, tmv.tm_min, tmv.tm_sec);
    }
    
    return 0;
}

/* ============================================================================
 * Initialization and dispatch
 * ============================================================================ */

void vt_simd_init(void) {
    if (parse_func) return; /* Already initialized */
    
    SIMDFeatures features = simd_detect_features();
    
    /* Select best implementation */
#if defined(__AVX2__)
    if (features & SIMD_AVX2) {
        parse_func = parse_rfc3339_avx2;
        format_func = format_rfc3339_scalar;
        return;
    }
#endif

#if defined(__x86_64__) || defined(_M_X64) || defined(__i386__) || defined(_M_IX86)
    if (features & SIMD_SSE2) {
        parse_func = parse_rfc3339_sse2;
        format_func = format_rfc3339_scalar;
        return;
    }
#endif

#if defined(__aarch64__) || defined(_M_ARM64) || defined(__ARM_NEON)
    if (features & SIMD_NEON) {
        parse_func = parse_rfc3339_neon;
        format_func = format_rfc3339_scalar;
        return;
    }
#endif

    /* Fallback to scalar */
    parse_func = parse_rfc3339_scalar;
    format_func = format_rfc3339_scalar;
}

int vt_parse_rfc3339_simd(const char* s, VexInstant* out) {
    if (!parse_func) vt_simd_init();
    return parse_func(s, out);
}

int vt_format_rfc3339_utc_simd(VexInstant t, char* buf, size_t buflen) {
    if (!format_func) vt_simd_init();
    return format_func(t, buf, buflen);
}

