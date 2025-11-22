/*
 * http_parser.c - HTTP/1.x streaming parser
 * Supports: Content-Length, Transfer-Encoding: chunked, pipelining
 */

#include "protocols/http_parser.h"
#include "protocols/simd_utils.h"
#include <string.h>

/* ========== Helper Functions ========== */

static inline size_t min_size(size_t a, size_t b) {
    return a < b ? a : b;
}

static int is_token_char(unsigned char c) {
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

static void trim_whitespace(const char **start, const char **end) {
    const char *s = *start;
    const char *e = *end;
    while (s < e && (*s == ' ' || *s == '\t')) s++;
    while (e > s && (e[-1] == ' ' || e[-1] == '\t' || e[-1] == '\r')) e--;
    *start = s;
    *end   = e;
}

static int parse_http_version(const char *p, size_t len, int *maj, int *min) {
    if (len < 8) return VEX_HTTP_ERR_BAD_REQUEST;
    if (memcmp(p, "HTTP/", 5) != 0) return VEX_HTTP_ERR_BAD_REQUEST;
    if (p[6] != '.') return VEX_HTTP_ERR_BAD_REQUEST;
    if (p[5] < '0' || p[5] > '9' || p[7] < '0' || p[7] > '9') {
        return VEX_HTTP_ERR_BAD_REQUEST;
    }
    *maj = p[5] - '0';
    *min = p[7] - '0';
    return VEX_HTTP_OK;
}

static int str_equals_ci(const char *a, size_t alen, const char *b_lit) {
    size_t blen = strlen(b_lit);
    if (alen != blen) return 0;
    for (size_t i = 0; i < alen; ++i) {
        char ca = a[i];
        if (ca >= 'A' && ca <= 'Z') ca = (char)(ca - 'A' + 'a');
        if (ca != b_lit[i]) return 0;
    }
    return 1;
}

static int contains_ci(const char *s, size_t len, const char *needle) {
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

static int parse_decimal(const char *s, size_t len, size_t *out) {
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

static int parse_hex(const char *s, size_t len, size_t *out) {
    size_t v = 0;
    int have_digit = 0;
    for (size_t i = 0; i < len; ++i) {
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

/* ========== Request Line Parser ========== */

static int parse_request_line_partial(
    const char *buf, size_t len, size_t *pos, vex_http_request_t *out)
{
    size_t p   = *pos;
    size_t rem = (p < len) ? (len - p) : 0;

    if (rem < 14) {
        return VEX_HTTP_ERR_TRUNCATED;
    }

    size_t sp1 = vex_simd_find_char(buf + p, rem, ' ');
    if (sp1 == rem || sp1 == 0) {
        if (sp1 == rem) return VEX_HTTP_ERR_TRUNCATED;
        return VEX_HTTP_ERR_BAD_REQUEST;
    }

    out->request_line.method     = buf + p;
    out->request_line.method_len = sp1;

    for (size_t i = 0; i < sp1; ++i) {
        if (!is_token_char((unsigned char)buf[p + i])) {
            return VEX_HTTP_ERR_BAD_REQUEST;
        }
    }

    p += sp1;
    while (p < len && buf[p] == ' ') p++;
    if (p >= len) return VEX_HTTP_ERR_TRUNCATED;

    rem = len - p;
    size_t sp2 = vex_simd_find_char(buf + p, rem, ' ');
    if (sp2 == rem || sp2 == 0) {
        if (sp2 == rem) return VEX_HTTP_ERR_TRUNCATED;
        return VEX_HTTP_ERR_BAD_REQUEST;
    }

    out->request_line.uri     = buf + p;
    out->request_line.uri_len = sp2;

    p += sp2;
    while (p < len && buf[p] == ' ') p++;
    if (p + 10 > len) return VEX_HTTP_ERR_TRUNCATED;

    const char *version_start = buf + p;
    rem = len - p;
    size_t cr = vex_simd_find_char(version_start, rem, '\r');
    if (cr == rem) {
        return VEX_HTTP_ERR_TRUNCATED;
    }
    if (p + cr + 1 >= len) return VEX_HTTP_ERR_TRUNCATED;
    if (buf[p + cr + 1] != '\n') {
        return VEX_HTTP_ERR_BAD_REQUEST;
    }

    int maj, min;
    int st = parse_http_version(version_start, cr, &maj, &min);
    if (st != VEX_HTTP_OK) return st;
    out->request_line.http_major = maj;
    out->request_line.http_minor = min;

    p += cr + 2;
    *pos = p;
    return VEX_HTTP_OK;
}

/* ========== Header Parser ========== */

static int parse_headers_partial(
    const char *buf, size_t len, size_t *pos, vex_http_request_t *out,
    size_t *content_length, int *has_content_length, int *is_chunked)
{
    size_t p = *pos;

    while (1) {
        if (p + 2 <= len && buf[p] == '\r' && buf[p + 1] == '\n') {
            p += 2;
            *pos = p;
            return VEX_HTTP_OK;
        }
        if (p >= len) return VEX_HTTP_ERR_TRUNCATED;

        if (out->header_count >= VEX_HTTP_MAX_HEADERS) {
            return VEX_HTTP_ERR_TOO_MANY_HDR;
        }

        const char *line     = buf + p;
        size_t      line_rem = len - p;
        size_t      crlf     = vex_simd_find_char(line, line_rem, '\r');
        if (crlf == line_rem) {
            return VEX_HTTP_ERR_TRUNCATED;
        }
        if (p + crlf + 1 >= len) return VEX_HTTP_ERR_TRUNCATED;
        if (buf[p + crlf + 1] != '\n') {
            return VEX_HTTP_ERR_BAD_REQUEST;
        }

        size_t colon = vex_simd_find_char(line, crlf, ':');
        if (colon == crlf || colon == 0) {
            return VEX_HTTP_ERR_BAD_REQUEST;
        }

        const char *name_start = line;
        const char *name_end   = line + colon;
        trim_whitespace(&name_start, &name_end);
        if (name_start >= name_end) return VEX_HTTP_ERR_BAD_REQUEST;

        const char *val_start = line + colon + 1;
        const char *val_end   = line + crlf;
        trim_whitespace(&val_start, &val_end);

        size_t name_len  = (size_t)(name_end - name_start);
        size_t value_len = (size_t)(val_end - val_start);
        if (name_len > 0xFFFFu || value_len > 0xFFFFu) {
            return VEX_HTTP_ERR_BAD_REQUEST;
        }

        vex_http_header_t *h = &out->headers[out->header_count++];
        h->name      = name_start;
        h->name_len  = (uint16_t)name_len;
        h->value     = val_start;
        h->value_len = (uint16_t)value_len;

        if (str_equals_ci(h->name, h->name_len, "content-length")) {
            size_t cl = 0;
            if (parse_decimal(h->value, h->value_len, &cl) != 0) {
                return VEX_HTTP_ERR_BAD_REQUEST;
            }
            *content_length     = cl;
            *has_content_length = 1;
        } else if (str_equals_ci(h->name, h->name_len, "transfer-encoding")) {
            if (contains_ci(h->value, h->value_len, "chunked")) {
                *is_chunked = 1;
            }
        }

        p += crlf + 2;
    }
}

/* ========== Streaming Parser ========== */

void vex_http_stream_init(vex_http_stream_t *st, vex_http_request_t *req) {
    memset(req, 0, sizeof(*req));
    st->req                = req;
    st->state              = VEX_HTTP_PARSER_REQ_LINE;
    st->pos                = 0;
    st->content_length     = 0;
    st->has_content_length = 0;
    st->is_chunked         = 0;
    st->body_start         = 0;
    st->body_written       = 0;
    st->chunk_bytes_remaining = 0;
}

int vex_http_stream_execute(vex_http_stream_t *st, const char *buf, size_t len,
                            size_t *bytes_parsed)
{
    size_t start_pos = st->pos;
    int status = VEX_HTTP_OK;

    for (;;) {
        switch (st->state) {

        case VEX_HTTP_PARSER_REQ_LINE: {
            status = parse_request_line_partial(buf, len, &st->pos, st->req);
            if (status == VEX_HTTP_OK) {
                st->state = VEX_HTTP_PARSER_HEADERS;
            } else {
                goto done;
            }
        } break;

        case VEX_HTTP_PARSER_HEADERS: {
            status = parse_headers_partial(
                buf, len, &st->pos, st->req,
                &st->content_length, &st->has_content_length, &st->is_chunked);
            if (status == VEX_HTTP_OK) {
                if (st->is_chunked) {
                    st->body_start   = st->pos;
                    st->body_written = 0;
                    st->chunk_bytes_remaining = 0;
                    st->req->body     = buf + st->body_start;
                    st->req->body_len = 0;
                    st->state         = VEX_HTTP_PARSER_BODY_CHUNK_SIZE;
                } else if (st->has_content_length) {
                    st->req->body = buf + st->pos;
                    size_t available = (st->pos <= len) ? (len - st->pos) : 0;
                    if (available >= st->content_length) {
                        st->req->body_len = st->content_length;
                        st->pos          += st->content_length;
                        st->state         = VEX_HTTP_PARSER_DONE;
                        status            = VEX_HTTP_OK;
                        goto done;
                    } else {
                        st->req->body_len = available;
                        st->state         = VEX_HTTP_PARSER_BODY_CL;
                        status            = VEX_HTTP_ERR_TRUNCATED;
                        goto done;
                    }
                } else {
                    st->req->body     = buf + st->pos;
                    st->req->body_len = 0;
                    st->state         = VEX_HTTP_PARSER_DONE;
                    status            = VEX_HTTP_OK;
                    goto done;
                }
            } else {
                goto done;
            }
        } break;

        case VEX_HTTP_PARSER_BODY_CL: {
            if (!st->has_content_length) {
                st->state = VEX_HTTP_PARSER_ERROR;
                status    = VEX_HTTP_ERR_BAD_REQUEST;
                goto done;
            }
            size_t have_total   = st->req->body_len;
            size_t needed_total = st->content_length;

            if (have_total >= needed_total) {
                st->state = VEX_HTTP_PARSER_DONE;
                status    = VEX_HTTP_OK;
                goto done;
            }

            size_t available = (len > st->pos) ? (len - st->pos) : 0;
            size_t missing   = needed_total - have_total;

            if (available >= missing) {
                st->req->body_len = needed_total;
                st->pos          += missing;
                st->state         = VEX_HTTP_PARSER_DONE;
                status            = VEX_HTTP_OK;
                goto done;
            } else {
                st->req->body_len += available;
                st->pos            = len;
                status             = VEX_HTTP_ERR_TRUNCATED;
                goto done;
            }
        } break;

        case VEX_HTTP_PARSER_BODY_CHUNK_SIZE: {
            size_t p   = st->pos;
            if (p >= len) {
                status = VEX_HTTP_ERR_TRUNCATED;
                goto done;
            }
            size_t rem = len - p;
            size_t cr  = vex_simd_find_char(buf + p, rem, '\r');
            if (cr == rem) {
                status = VEX_HTTP_ERR_TRUNCATED;
                goto done;
            }
            if (p + cr + 1 >= len) {
                status = VEX_HTTP_ERR_TRUNCATED;
                goto done;
            }
            if (buf[p + cr + 1] != '\n') {
                status = VEX_HTTP_ERR_BAD_REQUEST;
                goto done;
            }

            size_t chunk_size = 0;
            int sthex = parse_hex(buf + p, cr, &chunk_size);
            if (sthex != 0) {
                status = VEX_HTTP_ERR_BAD_REQUEST;
                goto done;
            }

            st->pos = p + cr + 2;
            if (chunk_size == 0) {
                st->chunk_bytes_remaining = 0;
                st->state = VEX_HTTP_PARSER_BODY_CHUNK_TRAILERS;
            } else {
                st->chunk_bytes_remaining = chunk_size;
                st->state = VEX_HTTP_PARSER_BODY_CHUNK_DATA;
            }
        } break;

        case VEX_HTTP_PARSER_BODY_CHUNK_DATA: {
            if (st->chunk_bytes_remaining == 0) {
                st->state = VEX_HTTP_PARSER_BODY_CHUNK_SIZE;
                continue;
            }
            if (st->pos >= len) {
                status = VEX_HTTP_ERR_TRUNCATED;
                goto done;
            }
            size_t available = len - st->pos;
            size_t to_copy   = min_size(available, st->chunk_bytes_remaining);

            char *dest = (char *)buf + st->body_start + st->body_written;
            const char *src = buf + st->pos;
            memmove(dest, src, to_copy);

            st->body_written         += to_copy;
            st->req->body_len         = st->body_written;
            st->chunk_bytes_remaining -= to_copy;
            st->pos                  += to_copy;

            if (st->chunk_bytes_remaining > 0) {
                status = VEX_HTTP_ERR_TRUNCATED;
                goto done;
            } else {
                if (st->pos + 2 > len) {
                    status = VEX_HTTP_ERR_TRUNCATED;
                    goto done;
                }
                if (buf[st->pos] != '\r' || buf[st->pos + 1] != '\n') {
                    status = VEX_HTTP_ERR_BAD_REQUEST;
                    goto done;
                }
                st->pos += 2;
                st->state = VEX_HTTP_PARSER_BODY_CHUNK_SIZE;
            }
        } break;

        case VEX_HTTP_PARSER_BODY_CHUNK_TRAILERS: {
            while (1) {
                if (st->pos + 2 <= len &&
                    buf[st->pos] == '\r' &&
                    buf[st->pos + 1] == '\n') {
                    st->pos += 2;
                    st->state = VEX_HTTP_PARSER_DONE;
                    status    = VEX_HTTP_OK;
                    goto done;
                }
                if (st->pos >= len) {
                    status = VEX_HTTP_ERR_TRUNCATED;
                    goto done;
                }
                size_t rem = len - st->pos;
                size_t cr  = vex_simd_find_char(buf + st->pos, rem, '\r');
                if (cr == rem) {
                    status = VEX_HTTP_ERR_TRUNCATED;
                    goto done;
                }
                if (st->pos + cr + 1 >= len) {
                    status = VEX_HTTP_ERR_TRUNCATED;
                    goto done;
                }
                if (buf[st->pos + cr + 1] != '\n') {
                    status = VEX_HTTP_ERR_BAD_REQUEST;
                    goto done;
                }
                st->pos += cr + 2;
            }
        } break;

        case VEX_HTTP_PARSER_DONE:
            status = VEX_HTTP_OK;
            goto done;

        case VEX_HTTP_PARSER_ERROR:
        default:
            status = VEX_HTTP_ERR_BAD_REQUEST;
            goto done;
        }
    }

done:
    if (bytes_parsed) {
        *bytes_parsed = (st->pos >= start_pos) ? (st->pos - start_pos) : 0;
    }
    return status;
}

int vex_http_stream_is_done(const vex_http_stream_t *st) {
    return st->state == VEX_HTTP_PARSER_DONE;
}

/* ========== One-Shot Parser ========== */

int vex_http_parse(const char *buf, size_t len, vex_http_request_t *out) {
    vex_http_stream_t st;
    vex_http_stream_init(&st, out);

    size_t consumed = 0;
    int r = vex_http_stream_execute(&st, buf, len, &consumed);
    if (r == VEX_HTTP_OK && vex_http_stream_is_done(&st)) {
        return VEX_HTTP_OK;
    }
    if (r == VEX_HTTP_ERR_TRUNCATED) {
        return VEX_HTTP_ERR_TRUNCATED;
    }
    return r;
}

/* ========== Connection Wrapper ========== */

void vex_http_connection_init(vex_http_connection_t *conn,
                              vex_http_request_t *first_req)
{
    vex_http_stream_init(&conn->parser, first_req);
}

void vex_http_connection_next_request(vex_http_connection_t *conn,
                                      vex_http_request_t *next_req)
{
    memset(next_req, 0, sizeof(*next_req));
    conn->parser.req                = next_req;
    conn->parser.state              = VEX_HTTP_PARSER_REQ_LINE;
    conn->parser.content_length     = 0;
    conn->parser.has_content_length = 0;
    conn->parser.is_chunked         = 0;
    conn->parser.body_start         = 0;
    conn->parser.body_written       = 0;
    conn->parser.chunk_bytes_remaining = 0;
}
