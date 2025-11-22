/*
    single_header.c - High-performance, single-file HTTP/1.x streaming parser
                      + IPv4/IPv6 UDP parser with multi-ISA SIMD acceleration
                      + minimal HTTP/2 preface/frame-header skeleton.

    ÖZET:
      - HTTP/1.x request parser:
          * Incremental / streaming API
          * Tek seferde (one-shot) parse convenience API
          * Content-Length gövde
          * Transfer-Encoding: chunked (gövdeyi decode eder, tek parça haline getirir)
      - HTTP connection wrapper:
          * Keep-alive / pipelining: aynı buffer içinde birden fazla istek
      - UDP:
          * IPv4 + UDP
          * IPv6 + UDP (extension header yok varsayımı)
      - SIMD:
          * AVX-512, AVX2, SSE2, ARM NEON, SCALAR fallback
      - HTTP/2:
          * Client preface string match
          * Frame header (9 byte) parse iskeleti
          * Tam HTTP/2 implementasyonu DEĞİL.
      - HTTP/3:
          * Bu dosyada yok (QUIC gerektiriyor).

    Derleme (Godbolt için tipik):
      gcc -std=c11 -O2 -Wall -Wextra -pedantic

    Streaming notu:
      - Streaming API, buffer pointer'ının çağrılar arasında sabit kaldığını varsayar.
        Yani hep aynı başlangıç adresine yazıp "len"i büyütüyormuşsun gibi düşün.
*/

#include <stdint.h>
#include <stddef.h>
#include <string.h>
#include <stdio.h>
#include <time.h>

/* ====================== SIMD feature detection ====================== */

#if defined(__x86_64__) || defined(_M_X64) || defined(__i386) || defined(_M_IX86)
#   define FASTNET_ARCH_X86 1
#else
#   define FASTNET_ARCH_X86 0
#endif

#if defined(__aarch64__) || defined(__arm__)
#   define FASTNET_ARCH_ARM 1
#else
#   define FASTNET_ARCH_ARM 0
#endif

/* x86 SIMD */
#if FASTNET_ARCH_X86
#   include <immintrin.h>
#   if defined(__AVX512F__)
#       define FASTNET_HAVE_AVX512 1
#   else
#       define FASTNET_HAVE_AVX512 0
#   endif
#   if defined(__AVX2__)
#       define FASTNET_HAVE_AVX2 1
#   else
#       define FASTNET_HAVE_AVX2 0
#   endif
#   if defined(__SSE2__) || (defined(_M_IX86_FP) && _M_IX86_FP >= 2)
#       define FASTNET_HAVE_SSE2 1
#   else
#       define FASTNET_HAVE_SSE2 0
#   endif
#else
#   define FASTNET_HAVE_AVX512 0
#   define FASTNET_HAVE_AVX2   0
#   define FASTNET_HAVE_SSE2   0
#endif

/* ARM NEON */
#if FASTNET_ARCH_ARM
#   include <arm_neon.h>
#   if defined(__ARM_NEON) || defined(__ARM_NEON__)
#       define FASTNET_HAVE_NEON 1
#   else
#       define FASTNET_HAVE_NEON 0
#   endif
#else
#   define FASTNET_HAVE_NEON 0
#endif

#ifndef FASTNET_MAX_HEADERS
#   define FASTNET_MAX_HEADERS 32
#endif

typedef enum {
    FASTNET_OK               = 0,
    FASTNET_ERR_TRUNCATED    = -1, /* streaming için: daha fazla veri lazım */
    FASTNET_ERR_BAD_REQUEST  = -2,
    FASTNET_ERR_TOO_MANY_HDR = -3,
    FASTNET_ERR_NOT_UDP      = -4,
    FASTNET_ERR_BAD_IP       = -5
} fastnet_status_t;

/* ================== Small helpers =================== */

static inline size_t fastnet_min_size_t(size_t a, size_t b) {
    return a < b ? a : b;
}

static const char *fastnet_simd_name(void) {
#if FASTNET_HAVE_AVX512
    return "AVX-512";
#elif FASTNET_HAVE_AVX2
    return "AVX2";
#elif FASTNET_HAVE_SSE2
    return "SSE2";
#elif FASTNET_HAVE_NEON
    return "ARM NEON";
#else
    return "SCALAR";
#endif
}

/* SIMD hızlandırmalı karakter arama (fallback ile birlikte) */
static size_t fastnet_find_char(const char *buf, size_t len, char c) {
#if FASTNET_HAVE_AVX512
    if (len >= 64) {
        __m512i target = _mm512_set1_epi8((char)c);
        size_t i = 0;
        size_t n = len & ~(size_t)63;
        for (; i < n; i += 64) {
            __m512i data   = _mm512_loadu_si512((const void *)(buf + i));
            __mmask64 mask = _mm512_cmpeq_epi8_mask(data, target);
            if (mask != 0) {
                for (int j = 0; j < 64; ++j) {
                    if (mask & ((uint64_t)1 << j)) {
                        size_t idx = i + (size_t)j;
                        if (idx < len) return idx;
                        return len;
                    }
                }
            }
        }
        for (; i < len; ++i) {
            if (buf[i] == c) return i;
        }
        return len;
    }
#elif FASTNET_HAVE_AVX2
    if (len >= 32) {
        __m256i target = _mm256_set1_epi8((char)c);
        size_t i = 0;
        size_t n = len & ~(size_t)31;
        for (; i < n; i += 32) {
            __m256i data = _mm256_loadu_si256((const __m256i *)(buf + i));
            __m256i eq   = _mm256_cmpeq_epi8(data, target);
            int mask     = _mm256_movemask_epi8(eq);
            if (mask != 0) {
                for (int j = 0; j < 32; ++j) {
                    if (mask & (1 << j)) {
                        size_t idx = i + (size_t)j;
                        if (idx < len) return idx;
                        return len;
                    }
                }
            }
        }
        for (; i < len; ++i) {
            if (buf[i] == c) return i;
        }
        return len;
    }
#elif FASTNET_HAVE_SSE2
    if (len >= 16) {
        __m128i target = _mm_set1_epi8((char)c);
        size_t i = 0;
        size_t n = len & ~(size_t)15;
        for (; i < n; i += 16) {
            __m128i data = _mm_loadu_si128((const __m128i *)(buf + i));
            __m128i eq   = _mm_cmpeq_epi8(data, target);
            int mask     = _mm_movemask_epi8(eq);
            if (mask != 0) {
                for (int j = 0; j < 16; ++j) {
                    if (mask & (1 << j)) {
                        size_t idx = i + (size_t)j;
                        if (idx < len) return idx;
                        return len;
                    }
                }
            }
        }
        for (; i < len; ++i) {
            if (buf[i] == c) return i;
        }
        return len;
    }
#elif FASTNET_HAVE_NEON
    if (len >= 16) {
        uint8x16_t target = vdupq_n_u8((uint8_t)c);
        size_t i = 0;
        size_t n = len & ~(size_t)15;
        for (; i < n; i += 16) {
            uint8x16_t data = vld1q_u8((const uint8_t *)(buf + i));
            uint8x16_t cmp  = vceqq_u8(data, target);
            uint8_t tmp[16];
            vst1q_u8(tmp, cmp);
            for (int j = 0; j < 16; ++j) {
                if (tmp[j]) {
                    size_t idx = i + (size_t)j;
                    if (idx < len) return idx;
                    return len;
                }
            }
        }
        for (; i < len; ++i) {
            if (buf[i] == c) return i;
        }
        return len;
    }
#endif
    for (size_t i = 0; i < len; ++i) {
        if (buf[i] == c) return i;
    }
    return len;
}

/* ================== HTTP structs =================== */

typedef struct {
    const char *name;
    uint16_t    name_len;
    const char *value;
    uint16_t    value_len;
} fastnet_http_header_t;

typedef struct {
    const char *method;
    size_t      method_len;
    const char *uri;
    size_t      uri_len;
    int         http_major;
    int         http_minor;
} fastnet_http_request_line_t;

typedef struct {
    fastnet_http_request_line_t request_line;
    fastnet_http_header_t       headers[FASTNET_MAX_HEADERS];
    size_t                      header_count;
    const char                 *body;
    size_t                      body_len;
} fastnet_http_request_t;

/* Streaming parser state */
typedef enum {
    FASTNET_HTTP_PARSER_REQ_LINE = 0,
    FASTNET_HTTP_PARSER_HEADERS,
    FASTNET_HTTP_PARSER_BODY_CL,
    FASTNET_HTTP_PARSER_BODY_CHUNK_SIZE,
    FASTNET_HTTP_PARSER_BODY_CHUNK_DATA,
    FASTNET_HTTP_PARSER_BODY_CHUNK_TRAILERS,
    FASTNET_HTTP_PARSER_DONE,
    FASTNET_HTTP_PARSER_ERROR
} fastnet_http_stream_state_t;

typedef struct {
    fastnet_http_request_t     *req;
    fastnet_http_stream_state_t state;
    size_t                      pos;
    size_t                      content_length;
    int                         has_content_length;
    int                         is_chunked;
    size_t                      body_start;
    size_t                      body_written;
    size_t                      chunk_bytes_remaining;
} fastnet_http_stream_t;

/* ================== HTTP helper fonksiyonları =================== */

static int fastnet_is_token_char(unsigned char c) {
    if (c <= 32 || c >= 127) return 0;
    switch (c) {
        case '(': case ')': case '<': case '>': case '@':
        case ',': case ';': case ':': case '\\': case '"':
        case '/': case '[': case ']': case '?': case '=':
        case '{': case '}':
            return 0;
        default:
            return 1;
    }
}

/* Baş / son boşluk (SP, HTAB) ve sondaki CR'ı kırp */
static void fastnet_trim(const char **start, const char **end) {
    const char *s = *start;
    const char *e = *end;
    while (s < e && (*s == ' ' || *s == '\t')) s++;
    while (e > s && (e[-1] == ' ' || e[-1] == '\t' || e[-1] == '\r')) e--;
    *start = s;
    *end   = e;
}

/* "HTTP/1.x" parse */
static int fastnet_parse_http_version(const char *p, size_t len, int *maj, int *min) {
    if (len < 8) return FASTNET_ERR_BAD_REQUEST;
    if (memcmp(p, "HTTP/", 5) != 0) return FASTNET_ERR_BAD_REQUEST;
    if (p[6] != '.') return FASTNET_ERR_BAD_REQUEST;
    if (p[5] < '0' || p[5] > '9' || p[7] < '0' || p[7] > '9') {
        return FASTNET_ERR_BAD_REQUEST;
    }
    *maj = p[5] - '0';
    *min = p[7] - '0';
    return FASTNET_OK;
}

/* ASCII case-insensitive karşılaştırma: a (len) vs sabit b (lowercase literal) */
static int fastnet_ieq_lit(const char *a, size_t alen, const char *b_lit) {
    size_t blen = strlen(b_lit);
    if (alen != blen) return 0;
    for (size_t i = 0; i < alen; ++i) {
        char ca = a[i];
        if (ca >= 'A' && ca <= 'Z') ca = (char)(ca - 'A' + 'a');
        if (ca != b_lit[i]) return 0;
    }
    return 1;
}

/* Case-insensitive substring search */
static int fastnet_contains_ci(const char *s, size_t len, const char *needle) {
    size_t nlen = strlen(needle);
    if (nlen == 0 || len < nlen) return 0;
    for (size_t i = 0; i + nlen <= len; ++i) {
        size_t j = 0;
        for (; j < nlen; ++j) {
            char cs = s[i + j];
            char cn = needle[j];
            if (cs >= 'A' && cs <= 'Z') cs = (char)(cs - 'A' + 'a');
            if (cn >= 'A' && cn <= 'Z') cn = (char)(cn - 'A' + 'a');
            if (cs != cn) break;
        }
        if (j == nlen) return 1;
    }
    return 0;
}

/* Decimal size parse */
static int fastnet_parse_size_dec(const char *s, size_t len, size_t *out) {
    size_t v = 0;
    if (len == 0) return -1;
    for (size_t i = 0; i < len; ++i) {
        unsigned char c = (unsigned char)s[i];
        if (c < '0' || c > '9') return -1;
        size_t digit = (size_t)(c - '0');
        size_t nv = v * 10 + digit;
        if (nv < v) {
            v = (size_t)-1;
            break;
        }
        v = nv;
    }
    *out = v;
    return 0;
}

/* Hex chunk-size parse */
static int fastnet_parse_size_hex(const char *s, size_t len, size_t *out) {
    size_t v = 0;
    int have_digit = 0;
    size_t i = 0;
    for (; i < len; ++i) {
        unsigned char c = (unsigned char)s[i];
        if (c == ';' || c == ' ' || c == '\t') break;
        int d;
        if (c >= '0' && c <= '9') d = c - '0';
        else if (c >= 'a' && c <= 'f') d = 10 + (c - 'a');
        else if (c >= 'A' && c <= 'F') d = 10 + (c - 'A');
        else return -1;
        have_digit = 1;
        size_t nv = (v << 4) | (size_t)d;
        if (nv < v) {
            v = (size_t)-1;
            break;
        }
        v = nv;
    }
    if (!have_digit) return -1;
    *out = v;
    return 0;
}

/* ---------- Request-line'i parça parça parse ---------- */
static fastnet_status_t fastnet_parse_request_line_partial(
    const char *buf, size_t len, size_t *pos, fastnet_http_request_t *out)
{
    size_t p   = *pos;
    size_t rem = (p < len) ? (len - p) : 0;

    if (rem < 14) {
        return FASTNET_ERR_TRUNCATED;
    }

    size_t sp1 = fastnet_find_char(buf + p, rem, ' ');
    if (sp1 == rem || sp1 == 0) {
        if (sp1 == rem)
            return FASTNET_ERR_TRUNCATED;
        return FASTNET_ERR_BAD_REQUEST;
    }

    out->request_line.method     = buf + p;
    out->request_line.method_len = sp1;

    for (size_t i = 0; i < sp1; ++i) {
        if (!fastnet_is_token_char((unsigned char)buf[p + i])) {
            return FASTNET_ERR_BAD_REQUEST;
        }
    }

    p += sp1;

    while (p < len && buf[p] == ' ') p++;
    if (p >= len) return FASTNET_ERR_TRUNCATED;

    rem = len - p;
    size_t sp2 = fastnet_find_char(buf + p, rem, ' ');
    if (sp2 == rem || sp2 == 0) {
        if (sp2 == rem)
            return FASTNET_ERR_TRUNCATED;
        return FASTNET_ERR_BAD_REQUEST;
    }

    out->request_line.uri     = buf + p;
    out->request_line.uri_len = sp2;

    p += sp2;

    while (p < len && buf[p] == ' ') p++;
    if (p + 10 > len) return FASTNET_ERR_TRUNCATED; /* en az "HTTP/1.0\r\n" */

    const char *version_start = buf + p;
    rem = len - p;
    size_t cr = fastnet_find_char(version_start, rem, '\r');
    if (cr == rem) {
        return FASTNET_ERR_TRUNCATED;
    }
    if (p + cr + 1 >= len) return FASTNET_ERR_TRUNCATED;
    if (buf[p + cr + 1] != '\n') {
        return FASTNET_ERR_BAD_REQUEST;
    }

    int maj, min;
    fastnet_status_t st = fastnet_parse_http_version(version_start, cr, &maj, &min);
    if (st != FASTNET_OK) return st;
    out->request_line.http_major = maj;
    out->request_line.http_minor = min;

    p += cr + 2;
    *pos = p;
    return FASTNET_OK;
}

/* ---------- Header'ları parça parça parse ---------- */
static fastnet_status_t fastnet_parse_headers_partial(
    const char *buf,
    size_t      len,
    size_t     *pos,
    fastnet_http_request_t *out,
    size_t     *content_length,
    int        *has_content_length,
    int        *is_chunked)
{
    size_t p = *pos;

    while (1) {
        if (p + 2 <= len && buf[p] == '\r' && buf[p + 1] == '\n') {
            p += 2;
            *pos = p;
            return FASTNET_OK;
        }
        if (p >= len) return FASTNET_ERR_TRUNCATED;

        if (out->header_count >= FASTNET_MAX_HEADERS) {
            return FASTNET_ERR_TOO_MANY_HDR;
        }

        const char *line     = buf + p;
        size_t      line_rem = len - p;
        size_t      crlf     = fastnet_find_char(line, line_rem, '\r');
        if (crlf == line_rem) {
            return FASTNET_ERR_TRUNCATED;
        }
        if (p + crlf + 1 >= len) return FASTNET_ERR_TRUNCATED;
        if (buf[p + crlf + 1] != '\n') {
            return FASTNET_ERR_BAD_REQUEST;
        }

        size_t colon = fastnet_find_char(line, crlf, ':');
        if (colon == crlf || colon == 0) {
            return FASTNET_ERR_BAD_REQUEST;
        }

        const char *name_start = line;
        const char *name_end   = line + colon;
        fastnet_trim(&name_start, &name_end);
        if (name_start >= name_end) return FASTNET_ERR_BAD_REQUEST;

        const char *val_start = line + colon + 1;
        const char *val_end   = line + crlf;
        fastnet_trim(&val_start, &val_end);

        size_t name_len  = (size_t)(name_end - name_start);
        size_t value_len = (size_t)(val_end - val_start);
        if (name_len > 0xFFFFu || value_len > 0xFFFFu) {
            return FASTNET_ERR_BAD_REQUEST;
        }

        fastnet_http_header_t *h = &out->headers[out->header_count++];
        h->name      = name_start;
        h->name_len  = (uint16_t)name_len;
        h->value     = val_start;
        h->value_len = (uint16_t)value_len;

        if (fastnet_ieq_lit(h->name, h->name_len, "content-length")) {
            size_t cl = 0;
            if (fastnet_parse_size_dec(h->value, h->value_len, &cl) != 0) {
                return FASTNET_ERR_BAD_REQUEST;
            }
            *content_length     = cl;
            *has_content_length = 1;
        } else if (fastnet_ieq_lit(h->name, h->name_len, "transfer-encoding")) {
            if (fastnet_contains_ci(h->value, h->value_len, "chunked")) {
                *is_chunked = 1;
            }
        }

        p += crlf + 2;
    }
}

/* ================== Streaming HTTP API =================== */

static void fastnet_http_stream_init(fastnet_http_stream_t *st, fastnet_http_request_t *req) {
    memset(req, 0, sizeof(*req));
    st->req                = req;
    st->state              = FASTNET_HTTP_PARSER_REQ_LINE;
    st->pos                = 0;
    st->content_length     = 0;
    st->has_content_length = 0;
    st->is_chunked         = 0;
    st->body_start         = 0;
    st->body_written       = 0;
    st->chunk_bytes_remaining = 0;
}

/* Streaming parse (HTTP/1.x, Content-Length + chunked) */
static fastnet_status_t fastnet_http_stream_execute(
    fastnet_http_stream_t *st,
    const char            *buf,
    size_t                 len,
    size_t                *bytes_parsed)
{
    size_t start_pos = st->pos;
    fastnet_status_t status = FASTNET_OK;

    for (;;) {
        switch (st->state) {

        case FASTNET_HTTP_PARSER_REQ_LINE: {
            status = fastnet_parse_request_line_partial(buf, len, &st->pos, st->req);
            if (status == FASTNET_OK) {
                st->state = FASTNET_HTTP_PARSER_HEADERS;
            } else {
                goto done;
            }
        } break;

        case FASTNET_HTTP_PARSER_HEADERS: {
            status = fastnet_parse_headers_partial(
                buf, len, &st->pos, st->req,
                &st->content_length, &st->has_content_length, &st->is_chunked);
            if (status == FASTNET_OK) {
                if (st->is_chunked) {
                    /* Chunked gövde: decode edilmiş body'yi body_start'tan itibaren memmove ile yazarız */
                    st->body_start   = st->pos;
                    st->body_written = 0;
                    st->chunk_bytes_remaining = 0;
                    st->req->body     = buf + st->body_start;
                    st->req->body_len = 0;
                    st->state         = FASTNET_HTTP_PARSER_BODY_CHUNK_SIZE;
                } else if (st->has_content_length) {
                    st->req->body = buf + st->pos;
                    size_t available = (st->pos <= len) ? (len - st->pos) : 0;
                    if (available >= st->content_length) {
                        st->req->body_len = st->content_length;
                        st->pos          += st->content_length;
                        st->state         = FASTNET_HTTP_PARSER_DONE;
                        status            = FASTNET_OK;
                        goto done;
                    } else {
                        st->req->body_len = available;
                        st->state         = FASTNET_HTTP_PARSER_BODY_CL;
                        status            = FASTNET_ERR_TRUNCATED;
                        goto done;
                    }
                } else {
                    /* Gövde yok */
                    st->req->body     = buf + st->pos;
                    st->req->body_len = 0;
                    st->state         = FASTNET_HTTP_PARSER_DONE;
                    status            = FASTNET_OK;
                    goto done;
                }
            } else {
                goto done;
            }
        } break;

        case FASTNET_HTTP_PARSER_BODY_CL: {
            if (!st->has_content_length) {
                st->state = FASTNET_HTTP_PARSER_ERROR;
                status    = FASTNET_ERR_BAD_REQUEST;
                goto done;
            }
            size_t have_total   = st->req->body_len;
            size_t needed_total = st->content_length;

            if (have_total >= needed_total) {
                st->state = FASTNET_HTTP_PARSER_DONE;
                status    = FASTNET_OK;
                goto done;
            }

            size_t available = (len > st->pos) ? (len - st->pos) : 0;
            size_t missing   = needed_total - have_total;

            if (available >= missing) {
                st->req->body_len = needed_total;
                st->pos          += missing;
                st->state         = FASTNET_HTTP_PARSER_DONE;
                status            = FASTNET_OK;
                goto done;
            } else {
                st->req->body_len += available;
                st->pos            = len;
                status             = FASTNET_ERR_TRUNCATED;
                goto done;
            }
        } break;

        case FASTNET_HTTP_PARSER_BODY_CHUNK_SIZE: {
            size_t p   = st->pos;
            if (p >= len) {
                status = FASTNET_ERR_TRUNCATED;
                goto done;
            }
            size_t rem = len - p;
            size_t cr  = fastnet_find_char(buf + p, rem, '\r');
            if (cr == rem) {
                status = FASTNET_ERR_TRUNCATED;
                goto done;
            }
            if (p + cr + 1 >= len) {
                status = FASTNET_ERR_TRUNCATED;
                goto done;
            }
            if (buf[p + cr + 1] != '\n') {
                status = FASTNET_ERR_BAD_REQUEST;
                goto done;
            }

            size_t chunk_size = 0;
            fastnet_status_t sthex = fastnet_parse_size_hex(buf + p, cr, &chunk_size);
            if (sthex != 0) {
                status = FASTNET_ERR_BAD_REQUEST;
                goto done;
            }

            st->pos = p + cr + 2;
            if (chunk_size == 0) {
                st->chunk_bytes_remaining = 0;
                st->state = FASTNET_HTTP_PARSER_BODY_CHUNK_TRAILERS;
            } else {
                st->chunk_bytes_remaining = chunk_size;
                st->state = FASTNET_HTTP_PARSER_BODY_CHUNK_DATA;
            }
        } break;

        case FASTNET_HTTP_PARSER_BODY_CHUNK_DATA: {
            if (st->chunk_bytes_remaining == 0) {
                st->state = FASTNET_HTTP_PARSER_BODY_CHUNK_SIZE;
                continue;
            }
            if (st->pos >= len) {
                status = FASTNET_ERR_TRUNCATED;
                goto done;
            }
            size_t available = len - st->pos;
            size_t to_copy   = fastnet_min_size_t(available, st->chunk_bytes_remaining);

            char *dest = (char *)buf + st->body_start + st->body_written;
            const char *src = buf + st->pos;
            memmove(dest, src, to_copy);

            st->body_written         += to_copy;
            st->req->body_len         = st->body_written;
            st->chunk_bytes_remaining -= to_copy;
            st->pos                  += to_copy;

            if (st->chunk_bytes_remaining > 0) {
                status = FASTNET_ERR_TRUNCATED;
                goto done;
            } else {
                /* Chunk data bitti; sıradaki CRLF'i tüket */
                if (st->pos + 2 > len) {
                    status = FASTNET_ERR_TRUNCATED;
                    goto done;
                }
                if (buf[st->pos] != '\r' || buf[st->pos + 1] != '\n') {
                    status = FASTNET_ERR_BAD_REQUEST;
                    goto done;
                }
                st->pos += 2;
                st->state = FASTNET_HTTP_PARSER_BODY_CHUNK_SIZE;
            }
        } break;

        case FASTNET_HTTP_PARSER_BODY_CHUNK_TRAILERS: {
            /* Trailer header'ları parse et, ama saklama. Boş satır (CRLF) ile biter. */
            while (1) {
                if (st->pos + 2 <= len &&
                    buf[st->pos] == '\r' &&
                    buf[st->pos + 1] == '\n') {
                    st->pos += 2;
                    st->state = FASTNET_HTTP_PARSER_DONE;
                    status    = FASTNET_OK;
                    goto done;
                }
                if (st->pos >= len) {
                    status = FASTNET_ERR_TRUNCATED;
                    goto done;
                }
                size_t rem = len - st->pos;
                size_t cr  = fastnet_find_char(buf + st->pos, rem, '\r');
                if (cr == rem) {
                    status = FASTNET_ERR_TRUNCATED;
                    goto done;
                }
                if (st->pos + cr + 1 >= len) {
                    status = FASTNET_ERR_TRUNCATED;
                    goto done;
                }
                if (buf[st->pos + cr + 1] != '\n') {
                    status = FASTNET_ERR_BAD_REQUEST;
                    goto done;
                }
                /* Bu trailer satırını ignore edip devam */
                st->pos += cr + 2;
            }
        } break;

        case FASTNET_HTTP_PARSER_DONE:
            status = FASTNET_OK;
            goto done;

        case FASTNET_HTTP_PARSER_ERROR:
        default:
            status = FASTNET_ERR_BAD_REQUEST;
            goto done;
        }
    }

done:
    if (bytes_parsed) {
        *bytes_parsed = (st->pos >= start_pos) ? (st->pos - start_pos) : 0;
    }
    return status;
}

static int fastnet_http_stream_is_done(const fastnet_http_stream_t *st) {
    return st->state == FASTNET_HTTP_PARSER_DONE;
}

/* ================== One-shot HTTP API =================== */

static fastnet_status_t fastnet_http_parse(const char *buf, size_t len, fastnet_http_request_t *out) {
    fastnet_http_stream_t st;
    fastnet_http_stream_init(&st, out);

    size_t consumed = 0;
    fastnet_status_t r = fastnet_http_stream_execute(&st, buf, len, &consumed);
    if (r == FASTNET_OK && fastnet_http_stream_is_done(&st)) {
        return FASTNET_OK;
    }
    if (r == FASTNET_ERR_TRUNCATED) {
        return FASTNET_ERR_TRUNCATED;
    }
    return r;
}

/* ================== HTTP Connection wrapper (pipelining) =================== */

typedef struct {
    fastnet_http_stream_t parser;
} fastnet_http_connection_t;

static void fastnet_http_connection_init(fastnet_http_connection_t *conn,
                                         fastnet_http_request_t *first_req)
{
    fastnet_http_stream_init(&conn->parser, first_req);
}

static void fastnet_http_connection_next_request(fastnet_http_connection_t *conn,
                                                 fastnet_http_request_t *next_req)
{
    /* Aynı buffer içinde kaldığımız yerden devam edeceğiz; sadece parser state'i resetle */
    memset(next_req, 0, sizeof(*next_req));
    conn->parser.req                = next_req;
    conn->parser.state              = FASTNET_HTTP_PARSER_REQ_LINE;
    conn->parser.content_length     = 0;
    conn->parser.has_content_length = 0;
    conn->parser.is_chunked         = 0;
    conn->parser.body_start         = 0;
    conn->parser.body_written       = 0;
    conn->parser.chunk_bytes_remaining = 0;
}

/* ================== UDP (IPv4 / IPv6) parser =================== */

typedef struct {
    uint8_t       version;
    uint8_t       ihl_bytes;
    uint8_t       protocol;
    uint32_t      src_ip;       /* network byte order */
    uint32_t      dst_ip;       /* network byte order */
    uint16_t      src_port;     /* host byte order */
    uint16_t      dst_port;     /* host byte order */
    uint16_t      length;       /* UDP length, host byte order */
    const uint8_t *payload;
    uint16_t      payload_len;
} fastnet_udp4_packet_t;

typedef struct {
    uint8_t       version;      /* 6 */
    uint8_t       protocol;     /* 17 (UDP) */
    uint8_t       src_ip[16];
    uint8_t       dst_ip[16];
    uint16_t      src_port;
    uint16_t      dst_port;
    uint16_t      length;       /* UDP length, host byte order */
    const uint8_t *payload;
    uint16_t      payload_len;
} fastnet_udp6_packet_t;

static inline uint16_t fastnet_be16(const uint8_t *p) {
    return (uint16_t)((uint16_t)p[0] << 8 | (uint16_t)p[1]);
}

static inline uint32_t fastnet_be32(const uint8_t *p) {
    return ((uint32_t)p[0] << 24) |
           ((uint32_t)p[1] << 16) |
           ((uint32_t)p[2] << 8)  |
           ((uint32_t)p[3]);
}

/* IPv4 + UDP parse */
static fastnet_status_t fastnet_udp_parse_ipv4(const uint8_t *buf, size_t len, fastnet_udp4_packet_t *out) {
    if (len < 20 + 8) return FASTNET_ERR_TRUNCATED;

    uint8_t vihl    = buf[0];
    uint8_t version = (uint8_t)(vihl >> 4);
    uint8_t ihl     = (uint8_t)(vihl & 0x0F);

    if (version != 4) return FASTNET_ERR_BAD_IP;
    if (ihl < 5)      return FASTNET_ERR_BAD_IP;

    uint8_t ip_header_len = (uint8_t)(ihl * 4);
    if (len < (size_t)ip_header_len + 8u) return FASTNET_ERR_TRUNCATED;

    uint16_t total_length = fastnet_be16(buf + 2);
    if (total_length < (uint16_t)(ip_header_len + 8)) return FASTNET_ERR_BAD_IP;
    if (len < (size_t)total_length) return FASTNET_ERR_TRUNCATED;

    uint8_t protocol = buf[9];
    if (protocol != 17) {
        return FASTNET_ERR_NOT_UDP;
    }

    out->version   = version;
    out->ihl_bytes = ip_header_len;
    out->protocol  = protocol;
    out->src_ip    = fastnet_be32(buf + 12);
    out->dst_ip    = fastnet_be32(buf + 16);

    const uint8_t *udp    = buf + ip_header_len;
    uint16_t       sport  = fastnet_be16(udp + 0);
    uint16_t       dport  = fastnet_be16(udp + 2);
    uint16_t       udplen = fastnet_be16(udp + 4);

    if (udplen < 8) return FASTNET_ERR_BAD_IP;
    if ((size_t)ip_header_len + udplen > len) return FASTNET_ERR_TRUNCATED;

    out->src_port = sport;
    out->dst_port = dport;
    out->length   = udplen;
    out->payload  = udp + 8;
    out->payload_len = (uint16_t)(udplen - 8);

    return FASTNET_OK;
}

/* IPv6 + UDP parse (extension header yok varsayımı) */
static fastnet_status_t fastnet_udp_parse_ipv6(const uint8_t *buf, size_t len, fastnet_udp6_packet_t *out) {
    if (len < 40 + 8) return FASTNET_ERR_TRUNCATED;

    uint8_t version = (uint8_t)(buf[0] >> 4);
    if (version != 6) return FASTNET_ERR_BAD_IP;

    uint16_t payload_len = fastnet_be16(buf + 4);
    uint8_t  next_header = buf[6];

    if (next_header != 17) {
        return FASTNET_ERR_NOT_UDP;
    }

    size_t total_len = 40u + (size_t)payload_len;
    if (len < total_len) {
        return FASTNET_ERR_TRUNCATED;
    }

    memcpy(out->src_ip, buf + 8,  16);
    memcpy(out->dst_ip, buf + 24, 16);
    out->version  = 6;
    out->protocol = next_header;

    const uint8_t *udp = buf + 40;
    uint16_t sport  = fastnet_be16(udp + 0);
    uint16_t dport  = fastnet_be16(udp + 2);
    uint16_t udplen = fastnet_be16(udp + 4);

    if (udplen < 8) return FASTNET_ERR_BAD_IP;
    if ((size_t)40 + udplen > len) return FASTNET_ERR_TRUNCATED;

    out->src_port = sport;
    out->dst_port = dport;
    out->length   = udplen;
    out->payload  = udp + 8;
    out->payload_len = (uint16_t)(udplen - 8);

    return FASTNET_OK;
}

/* IPv4 -> "x.x.x.x" */
static void fastnet_ipv4_to_str(uint32_t ip_net_order, char *buf16) {
    unsigned char b0 = (unsigned char)((ip_net_order >> 24) & 0xFF);
    unsigned char b1 = (unsigned char)((ip_net_order >> 16) & 0xFF);
    unsigned char b2 = (unsigned char)((ip_net_order >> 8)  & 0xFF);
    unsigned char b3 = (unsigned char)(ip_net_order & 0xFF);
    sprintf(buf16, "%u.%u.%u.%u",
            (unsigned)b0, (unsigned)b1, (unsigned)b2, (unsigned)b3);
}

/* IPv6 -> basit "xxxx:xxxx:...:xxxx" */
static void fastnet_ipv6_to_str(const uint8_t ip[16], char *buf64) {
    sprintf(buf64,
            "%02x%02x:%02x%02x:%02x%02x:%02x%02x:"
            "%02x%02x:%02x%02x:%02x%02x:%02x%02x",
            ip[0], ip[1], ip[2], ip[3],
            ip[4], ip[5], ip[6], ip[7],
            ip[8], ip[9], ip[10], ip[11],
            ip[12], ip[13], ip[14], ip[15]);
}

/* ================== HTTP/2 preface + frame header skeleton =================== */

#define FASTNET_HTTP2_PREFACE_LEN 24

static const char fastnet_http2_client_preface[FASTNET_HTTP2_PREFACE_LEN] =
    "PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

typedef struct {
    uint32_t length;
    uint8_t  type;
    uint8_t  flags;
    uint32_t stream_id;
} fastnet_http2_frame_header_t;

static int fastnet_http2_is_preface(const uint8_t *buf, size_t len) {
    if (len < FASTNET_HTTP2_PREFACE_LEN) return 0;
    return memcmp(buf, fastnet_http2_client_preface, FASTNET_HTTP2_PREFACE_LEN) == 0;
}

static fastnet_status_t fastnet_http2_parse_frame_header(const uint8_t *buf, size_t len,
                                                         fastnet_http2_frame_header_t *out)
{
    if (len < 9) return FASTNET_ERR_TRUNCATED;
    uint32_t length = ((uint32_t)buf[0] << 16) | ((uint32_t)buf[1] << 8) | (uint32_t)buf[2];
    uint8_t type    = buf[3];
    uint8_t flags   = buf[4];
    uint32_t sid    = ((uint32_t)buf[5] << 24) |
                      ((uint32_t)buf[6] << 16) |
                      ((uint32_t)buf[7] << 8)  |
                      ((uint32_t)buf[8]);
    sid &= 0x7FFFFFFFu;

    out->length    = length;
    out->type      = type;
    out->flags     = flags;
    out->stream_id = sid;
    return FASTNET_OK;
}

/* ================== HTTP benchmark =================== */

static void fastnet_http_benchmark(void) {
    const char *sample =
        "POST /bench/test HTTP/1.1\r\n"
        "Host: bench.example\r\n"
        "User-Agent: fastnet-bench/1.0\r\n"
        "Content-Length: 16\r\n"
        "\r\n"
        "0123456789ABCDEF";

    size_t len   = strlen(sample);
    const size_t iters = 100000;

    clock_t start = clock();
    for (size_t i = 0; i < iters; ++i) {
        fastnet_http_request_t req;
        fastnet_status_t st = fastnet_http_parse(sample, len, &req);
        if (st != FASTNET_OK) {
            printf("Benchmark parse error at iter %zu: %d\n", i, st);
            return;
        }
    }
    clock_t end   = clock();
    double secs   = (double)(end - start) / (double)CLOCKS_PER_SEC;
    if (secs <= 0.0) secs = 1e-9;

    double msec   = secs * 1000.0;
    double rps    = (double)iters / secs;

    printf("HTTP benchmark (one-shot):\n");
    printf("  iters      : %zu\n", iters);
    printf("  time       : %.3f ms\n", msec);
    printf("  req/sec    : %.0f\n\n", rps);
}

/* ================== Demo main =================== */

int main(void) {
    printf("FASTNET SIMD backend : %s\n\n", fastnet_simd_name());

    /* -------- One-shot HTTP demo (Content-Length) -------- */
    const char *http_sample =
        "POST /hello/world?x=1 HTTP/1.1\r\n"
        "Host: example.com\r\n"
        "User-Agent: fastnet-demo/1.0\r\n"
        "Content-Length: 17\r\n"
        "\r\n"
        "Body payload here";

    fastnet_http_request_t req1;
    fastnet_status_t hs1 = fastnet_http_parse(http_sample, strlen(http_sample), &req1);
    printf("One-shot HTTP parse status = %d\n", hs1);
    if (hs1 == FASTNET_OK) {
        printf("  Method : %.*s\n",
               (int)req1.request_line.method_len, req1.request_line.method);
        printf("  URI    : %.*s\n",
               (int)req1.request_line.uri_len, req1.request_line.uri);
        printf("  HTTP   : %d.%d\n",
               req1.request_line.http_major, req1.request_line.http_minor);

        printf("  Headers (%zu):\n", req1.header_count);
        for (size_t i = 0; i < req1.header_count; ++i) {
            fastnet_http_header_t *h = &req1.headers[i];
            printf("    %.*s: %.*s\n",
                   (int)h->name_len,  h->name,
                   (int)h->value_len, h->value);
        }

        printf("  Body (%zu bytes): \"%.*s\"\n\n",
               req1.body_len, (int)req1.body_len, req1.body);
    }
    printf("-----------------------------------------------------\n\n");

    /* -------- Streaming + chunked demo -------- */
    const char *http_chunked =
        "POST /chunked HTTP/1.1\r\n"
        "Host: chunk.example\r\n"
        "Transfer-Encoding: chunked\r\n"
        "\r\n"
        "4\r\n"
        "Wiki\r\n"
        "5\r\n"
        "pedia\r\n"
        "0\r\n"
        "X-Trailer: value\r\n"
        "\r\n";

    size_t total_len = strlen(http_chunked);
    char buf[512];
    memcpy(buf, http_chunked, total_len);

    fastnet_http_request_t req2;
    fastnet_http_stream_t  stream;
    fastnet_http_stream_init(&stream, &req2);

    size_t part1_len = total_len / 2;
    size_t consumed = 0;
    fastnet_status_t hs2 =
        fastnet_http_stream_execute(&stream, buf, part1_len, &consumed);

    printf("Chunked streaming call #1: status=%d, consumed=%zu, done=%d\n",
           hs2, consumed, fastnet_http_stream_is_done(&stream));

    size_t consumed2 = 0;
    fastnet_status_t hs3 =
        fastnet_http_stream_execute(&stream, buf, total_len, &consumed2);

    printf("Chunked streaming call #2: status=%d, consumed=%zu, done=%d\n",
           hs3, consumed2, fastnet_http_stream_is_done(&stream));

    if (hs3 == FASTNET_OK && fastnet_http_stream_is_done(&stream)) {
        printf("\nChunked HTTP parsed request:\n");
        printf("  Method : %.*s\n",
               (int)req2.request_line.method_len, req2.request_line.method);
        printf("  URI    : %.*s\n",
               (int)req2.request_line.uri_len, req2.request_line.uri);
        printf("  HTTP   : %d.%d\n",
               req2.request_line.http_major, req2.request_line.http_minor);

        printf("  Headers (%zu):\n", req2.header_count);
        for (size_t i = 0; i < req2.header_count; ++i) {
            fastnet_http_header_t *h = &req2.headers[i];
            printf("    %.*s: %.*s\n",
                   (int)h->name_len,  h->name,
                   (int)h->value_len, h->value);
        }

        printf("  Decoded body (%zu bytes): \"%.*s\"\n\n",
               req2.body_len, (int)req2.body_len, req2.body);
    }

    printf("=====================================================\n\n");

    /* -------- Connection wrapper / pipelining demo -------- */
    const char *http_pipelined =
        "GET /a HTTP/1.1\r\n"
        "Host: example\r\n"
        "\r\n"
        "GET /b HTTP/1.1\r\n"
        "Host: example\r\n"
        "\r\n";

    size_t pipe_len = strlen(http_pipelined);
    char pipe_buf[256];
    memcpy(pipe_buf, http_pipelined, pipe_len);

    fastnet_http_connection_t conn;
    fastnet_http_request_t    preq1, preq2;

    fastnet_http_connection_init(&conn, &preq1);

    size_t pcons = 0;
    fastnet_status_t ps1 =
        fastnet_http_stream_execute(&conn.parser, pipe_buf, pipe_len, &pcons);

    printf("Pipelined req1: status=%d, consumed=%zu, done=%d\n",
           ps1, pcons, fastnet_http_stream_is_done(&conn.parser));

    if (ps1 == FASTNET_OK && fastnet_http_stream_is_done(&conn.parser)) {
        printf("  Req1 URI: %.*s\n",
               (int)preq1.request_line.uri_len, preq1.request_line.uri);
    }

    fastnet_http_connection_next_request(&conn, &preq2);
    size_t pcons2 = 0;
    fastnet_status_t ps2 =
        fastnet_http_stream_execute(&conn.parser, pipe_buf, pipe_len, &pcons2);

    printf("Pipelined req2: status=%d, consumed=%zu, done=%d\n",
           ps2, pcons2, fastnet_http_stream_is_done(&conn.parser));

    if (ps2 == FASTNET_OK && fastnet_http_stream_is_done(&conn.parser)) {
        printf("  Req2 URI: %.*s\n",
               (int)preq2.request_line.uri_len, preq2.request_line.uri);
    }

    printf("=====================================================\n\n");

    /* -------- HTTP benchmark -------- */
    fastnet_http_benchmark();

    printf("=====================================================\n\n");

    /* -------- UDP IPv4 demo -------- */
    uint8_t packet4[128];
    memset(packet4, 0, sizeof(packet4));

    size_t ip_header_len = 20;
    packet4[0] = 0x45;
    packet4[1] = 0x00;

    const char *udp_payload4 = "hello over udp4";
    size_t payload4_len      = strlen(udp_payload4);
    uint16_t total4 = (uint16_t)(ip_header_len + 8 + payload4_len);

    packet4[2] = (uint8_t)(total4 >> 8);
    packet4[3] = (uint8_t)(total4 & 0xFF);
    packet4[4] = 0x00; packet4[5] = 0x01;
    packet4[6] = 0x40; packet4[7] = 0x00;
    packet4[8] = 64;
    packet4[9] = 17;

    packet4[12] = 192; packet4[13] = 168; packet4[14] = 1; packet4[15] = 10;
    packet4[16] = 192; packet4[17] = 168; packet4[18] = 1; packet4[19] = 20;

    size_t udp4_off  = ip_header_len;
    uint16_t sport4  = 50000;
    uint16_t dport4  = 60000;
    uint16_t udplen4 = (uint16_t)(8 + payload4_len);

    packet4[udp4_off + 0] = (uint8_t)(sport4 >> 8);
    packet4[udp4_off + 1] = (uint8_t)(sport4 & 0xFF);
    packet4[udp4_off + 2] = (uint8_t)(dport4 >> 8);
    packet4[udp4_off + 3] = (uint8_t)(dport4 & 0xFF);
    packet4[udp4_off + 4] = (uint8_t)(udplen4 >> 8);
    packet4[udp4_off + 5] = (uint8_t)(udplen4 & 0xFF);
    packet4[udp4_off + 6] = 0;
    packet4[udp4_off + 7] = 0;

    memcpy(packet4 + udp4_off + 8, udp_payload4, payload4_len);

    fastnet_udp4_packet_t up4;
    fastnet_status_t us4 = fastnet_udp_parse_ipv4(packet4, total4, &up4);
    printf("UDP IPv4 parse status = %d\n", us4);
    if (us4 == FASTNET_OK) {
        char src_ip_str[16];
        char dst_ip_str[16];
        fastnet_ipv4_to_str(up4.src_ip, src_ip_str);
        fastnet_ipv4_to_str(up4.dst_ip, dst_ip_str);

        printf("  src: %s:%u\n", src_ip_str, (unsigned)up4.src_port);
        printf("  dst: %s:%u\n", dst_ip_str, (unsigned)up4.dst_port);
        printf("  udp length   : %u\n", (unsigned)up4.length);
        printf("  payload_len  : %u\n", (unsigned)up4.payload_len);
        printf("  payload      : \"%.*s\"\n\n",
               (int)up4.payload_len, (const char *)up4.payload);
    }

    /* -------- UDP IPv6 demo -------- */
    uint8_t packet6[128];
    memset(packet6, 0, sizeof(packet6));

    packet6[0] = 0x60;
    packet6[1] = 0x00;
    packet6[2] = 0x00;
    packet6[3] = 0x00;

    const char *udp_payload6 = "hello over udp6";
    size_t payload6_len      = strlen(udp_payload6);
    uint16_t udplen6         = (uint16_t)(8 + payload6_len);

    packet6[4] = (uint8_t)(udplen6 >> 8);
    packet6[5] = (uint8_t)(udplen6 & 0xFF);
    packet6[6] = 17;
    packet6[7] = 64;

    uint8_t src6[16] = {
        0x20,0x01,0x0d,0xb8, 0,0,0,1,
        0,0,0,0, 0,0,0,1
    };
    uint8_t dst6[16] = {
        0x20,0x01,0x0d,0xb8, 0,0,0,2,
        0,0,0,0, 0,0,0,2
    };
    memcpy(packet6 + 8,  src6, 16);
    memcpy(packet6 + 24, dst6, 16);

    size_t udp6_off = 40;
    uint16_t sport6 = 40000;
    uint16_t dport6 = 40001;

    packet6[udp6_off + 0] = (uint8_t)(sport6 >> 8);
    packet6[udp6_off + 1] = (uint8_t)(sport6 & 0xFF);
    packet6[udp6_off + 2] = (uint8_t)(dport6 >> 8);
    packet6[udp6_off + 3] = (uint8_t)(dport6 & 0xFF);
    packet6[udp6_off + 4] = (uint8_t)(udplen6 >> 8);
    packet6[udp6_off + 5] = (uint8_t)(udplen6 & 0xFF);
    packet6[udp6_off + 6] = 0;
    packet6[udp6_off + 7] = 0;

    memcpy(packet6 + udp6_off + 8, udp_payload6, payload6_len);

    size_t total6 = 40 + (size_t)udplen6;
    fastnet_udp6_packet_t up6;
    fastnet_status_t us6 = fastnet_udp_parse_ipv6(packet6, total6, &up6);
    printf("UDP IPv6 parse status = %d\n", us6);
    if (us6 == FASTNET_OK) {
        char src6_str[64];
        char dst6_str[64];
        fastnet_ipv6_to_str(up6.src_ip, src6_str);
        fastnet_ipv6_to_str(up6.dst_ip, dst6_str);

        printf("  src: [%s]:%u\n", src6_str, (unsigned)up6.src_port);
        printf("  dst: [%s]:%u\n", dst6_str, (unsigned)up6.dst_port);
        printf("  udp length   : %u\n", (unsigned)up6.length);
        printf("  payload_len  : %u\n", (unsigned)up6.payload_len);
        printf("  payload      : \"%.*s\"\n\n",
               (int)up6.payload_len, (const char *)up6.payload);
    }

    /* -------- HTTP/2 preface + frame header demo -------- */
    uint8_t h2buf[64];
    memcpy(h2buf, fastnet_http2_client_preface, FASTNET_HTTP2_PREFACE_LEN);
    /* Ardına basit bir SETTINGS frame header koy: length=4, type=0x4, flags=0x04, stream_id=1 */
    h2buf[24] = 0x00;
    h2buf[25] = 0x00;
    h2buf[26] = 0x04;
    h2buf[27] = 0x01;
    h2buf[28] = 0x04;
    h2buf[29] = 0x00;
    h2buf[30] = 0x00;
    h2buf[31] = 0x00;
    h2buf[32] = 0x01;

    printf("HTTP/2 preface match: %d\n",
           fastnet_http2_is_preface(h2buf, FASTNET_HTTP2_PREFACE_LEN));

    fastnet_http2_frame_header_t fh;
    fastnet_status_t h2st = fastnet_http2_parse_frame_header(h2buf + FASTNET_HTTP2_PREFACE_LEN,
                                                             9, &fh);
    printf("HTTP/2 frame header parse status = %d\n", h2st);
    if (h2st == FASTNET_OK) {
        printf("  len=%u type=%u flags=0x%02x stream_id=%u\n",
               fh.length, fh.type, fh.flags, fh.stream_id);
    }

    return 0;
}
