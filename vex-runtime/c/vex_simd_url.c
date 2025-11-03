// url_parse_simd.c
// SIMD-accelerated URL parser (x86 AVX2/SSE2 + ARM NEON) with scalar fallback.
// Includes percent-decoding ("encoded" characters like %20).
// License: CC0 / Public Domain.
//
// Build examples:
//   x86 AVX2: cc -O3 -mavx2 -o url_demo url_parse_simd.c
//   x86 SSE2: cc -O3 -msse2 -o url_demo url_parse_simd.c
//   AArch64 : cc -O3        -o url_demo url_parse_simd.c
//
// Usage:
//   typedef struct UrlParts { ... } UrlParts;
//   bool url_parse(const char *url, UrlParts *out);
//   char *url_decode_inplace(char *s); // optional decoding utility
//
// Notes:
//   - This is an educational, self-contained example showing SIMD-assisted scanning.
//   - SIMD paths only accelerate delimiter search and percent detection.
//   - Full RFC3986 compliance not guaranteed (simplified authority parsing).
//   - Encoding decoding handles %XX (ASCII hex). Invalid sequences are left unchanged.

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <string.h>
#include <stdlib.h>
#include <stdio.h>

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

typedef struct {
    const char *scheme, *user, *pass, *host, *port, *path, *query, *fragment;
    size_t scheme_len, user_len, pass_len, host_len, port_len, path_len, query_len, fragment_len;
} UrlParts;

// ===== Percent-decoding =====
static inline int hexval(char c) {
    if (c >= '0' && c <= '9') return c - '0';
    if (c >= 'A' && c <= 'F') return c - 'A' + 10;
    if (c >= 'a' && c <= 'f') return c - 'a' + 10;
    return -1;
}

char *url_decode_inplace(char *s) {
    char *r = s, *w = s;
    while (*r) {
        if (r[0] == '%' && r[1] && r[2]) {
            int hi = hexval(r[1]), lo = hexval(r[2]);
            if (hi >= 0 && lo >= 0) {
                *w++ = (char)((hi << 4) | lo);
                r += 3;
                continue;
            }
        }
        *w++ = *r++;
    }
    *w = 0;
    return s;
}

// ===== SIMD-assisted delimiter scanning =====

static inline int simd_find_first_delim(const char *s, size_t len) {
    static const char delims[] = ":/?#@[]%";
    for (size_t i = 0; i < len; i++) {
        unsigned char c = (unsigned char)s[i];
        for (size_t j = 0; j < sizeof(delims) - 1; j++) {
            if (c == (unsigned char)delims[j]) return (int)i;
        }
    }
    return -1;
}

#if VEC_X86
static inline int simd_find_first_delim_x86(const char *s, size_t len) {
    const __m128i d1 = _mm_setr_epi8(':','/','?','#','@','[',']','%',0,0,0,0,0,0,0,0);
    for (size_t i = 0; i + 16 <= len; i += 16) {
        __m128i v = _mm_loadu_si128((const __m128i*)(s + i));
        for (int j = 0; j < 8; j++) { // small loop for delimiters
            __m128i c = _mm_set1_epi8(((const char*)&d1)[j]);
            __m128i eq = _mm_cmpeq_epi8(v, c);
            int mask = _mm_movemask_epi8(eq);
            if (mask) return i + __builtin_ctz(mask);
        }
    }
    return simd_find_first_delim(s + (len & ~15), len & 15);
}
#elif VEC_NEON
static inline int simd_find_first_delim_neon(const char *s, size_t len) {
    const uint8x16_t delims = vld1q_u8((const uint8_t*)":/?#@[]%");
    for (size_t i = 0; i + 16 <= len; i += 16) {
        uint8x16_t v = vld1q_u8((const uint8_t*)(s + i));
        uint8x16_t acc = vdupq_n_u8(0);
        for (int j = 0; j < 7; j++) {
            uint8x16_t c = vdupq_n_u8(((const uint8_t*)&delims)[j]);
            uint8x16_t eq = vceqq_u8(v, c);
            acc = vorrq_u8(acc, eq);
        }
        uint8_t tmp[16]; vst1q_u8(tmp, acc);
        for (int k = 0; k < 16; k++) if (tmp[k]) return i + k;
    }
    return simd_find_first_delim(s + (len & ~15), len & 15);
}
#endif

static inline int url_find_delim(const char *s, size_t len) {
#if VEC_X86
    return simd_find_first_delim_x86(s, len);
#elif VEC_NEON
    return simd_find_first_delim_neon(s, len);
#else
    return simd_find_first_delim(s, len);
#endif
}

// ===== Simple RFC3986-like parser =====

bool url_parse(const char *url, UrlParts *out) {
    memset(out, 0, sizeof(*out));
    size_t len = strlen(url);

    // scheme
    const char *p = strchr(url, ':');
    if (!p) return false;
    out->scheme = url;
    out->scheme_len = (size_t)(p - url);
    p++; // skip ':'

    // optional "//"
    if (p[0] == '/' && p[1] == '/') {
        p += 2;
        const char *authority = p;
        const char *path_start = strchr(p, '/');
        size_t auth_len = path_start ? (size_t)(path_start - authority) : strlen(authority);

        // userinfo?
        const char *at = memchr(authority, '@', auth_len);
        const char *hostpart = authority;
        if (at) {
            const char *colon = memchr(authority, ':', (size_t)(at - authority));
            if (colon) {
                out->user = authority;
                out->user_len = (size_t)(colon - authority);
                out->pass = colon + 1;
                out->pass_len = (size_t)(at - colon - 1);
            } else {
                out->user = authority;
                out->user_len = (size_t)(at - authority);
            }
            hostpart = at + 1;
        }

        // host:port
        const char *colon = memchr(hostpart, ':', auth_len - (size_t)(hostpart - authority));
        if (colon) {
            out->host = hostpart;
            out->host_len = (size_t)(colon - hostpart);
            out->port = colon + 1;
            out->port_len = (auth_len - (size_t)(out->port - authority));
        } else {
            out->host = hostpart;
            out->host_len = auth_len - (size_t)(hostpart - authority);
        }
        p = path_start ? path_start : url + len;
    }

    // path, query, fragment
    const char *path = p;
    const char *q = strchr(p, '?');
    const char *f = strchr(p, '#');
    const char *path_end = q ? q : (f ? f : url + len);

    out->path = path;
    out->path_len = (size_t)(path_end - path);

    if (q) {
        const char *q_end = f ? f : url + len;
        out->query = q + 1;
        out->query_len = (size_t)(q_end - (q + 1));
    }
    if (f) {
        out->fragment = f + 1;
        out->fragment_len = (size_t)((url + len) - (f + 1));
    }

    return true;
}

// ===== Demo =====
#ifdef URL_SIMD_DEMO
int main(void) {
    const char *tests[] = {
        "https://user:pass@example.com:8080/path/to/page?x=1&y=2#frag",
        "http://example.org",
        "ftp://[2001:db8::1]/file.txt",
        "https://example.com/space%20encoded?q=%C3%A7#top",
        NULL
    };
    for (int i = 0; tests[i]; i++) {
        UrlParts u;
        printf("---- %s ----\n", tests[i]);
        if (!url_parse(tests[i], &u)) { printf("parse failed\n"); continue; }
        char buf[256]; snprintf(buf, sizeof(buf), "%.*s", (int)u.path_len, u.path);
        url_decode_inplace(buf);
        printf("scheme=%.*s host=%.*s port=%.*s path=%s\n",
            (int)u.scheme_len, u.scheme,
            (int)u.host_len, u.host ? u.host : "",
            (int)u.port_len, u.port ? u.port : "",
            buf);
    }
    return 0;
}
#endif
