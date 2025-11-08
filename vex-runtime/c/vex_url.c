// vex_url.c - URL encoding/decoding with SIMD acceleration
#include "vex.h"
#include <stdint.h>
#include <stdbool.h>
#include <string.h>
#include <stdlib.h>

// vex.h already includes vex_macros.h with VEX_SIMD_X86 and VEX_SIMD_NEON

// ============================================================================
// URL ENCODING
// ============================================================================

static inline bool is_url_safe(unsigned char c)
{
  // Unreserved characters: A-Z a-z 0-9 - _ . ~
  return (c >= 'A' && c <= 'Z') ||
         (c >= 'a' && c <= 'z') ||
         (c >= '0' && c <= '9') ||
         c == '-' || c == '_' || c == '.' || c == '~';
}

static const char hex_chars[] = "0123456789ABCDEF";

// Scalar URL encode
static size_t url_encode_scalar(const char *src, size_t src_len, char *dst)
{
  size_t dst_pos = 0;

  for (size_t i = 0; i < src_len; i++)
  {
    unsigned char c = (unsigned char)src[i];

    if (is_url_safe(c))
    {
      if (dst)
        dst[dst_pos] = c;
      dst_pos++;
    }
    else if (c == ' ')
    {
      // Space can be encoded as '+'
      if (dst)
        dst[dst_pos] = '+';
      dst_pos++;
    }
    else
    {
      // Percent-encode: %XX
      if (dst)
      {
        dst[dst_pos] = '%';
        dst[dst_pos + 1] = hex_chars[c >> 4];
        dst[dst_pos + 2] = hex_chars[c & 0xF];
      }
      dst_pos += 3;
    }
  }

  return dst_pos;
}

#if VEX_SIMD_X86
// SIMD-accelerated URL encode (x86)
static size_t url_encode_simd_x86(const char *src, size_t src_len, char *dst)
{
  size_t i = 0;
  size_t dst_pos = 0;

  // Process 16 bytes at a time
  while (i + 16 <= src_len)
  {
    __m128i chunk = _mm_loadu_si128((const __m128i *)(src + i));

    // Check if all characters are safe (A-Z, a-z, 0-9, -, _, ., ~)
    __m128i is_upper = _mm_and_si128(
        _mm_cmpgt_epi8(chunk, _mm_set1_epi8('A' - 1)),
        _mm_cmpgt_epi8(_mm_set1_epi8('Z' + 1), chunk));
    __m128i is_lower = _mm_and_si128(
        _mm_cmpgt_epi8(chunk, _mm_set1_epi8('a' - 1)),
        _mm_cmpgt_epi8(_mm_set1_epi8('z' + 1), chunk));
    __m128i is_digit = _mm_and_si128(
        _mm_cmpgt_epi8(chunk, _mm_set1_epi8('0' - 1)),
        _mm_cmpgt_epi8(_mm_set1_epi8('9' + 1), chunk));

    __m128i is_safe = _mm_or_si128(_mm_or_si128(is_upper, is_lower), is_digit);

    // Check for special safe chars (-, _, ., ~)
    __m128i is_dash = _mm_cmpeq_epi8(chunk, _mm_set1_epi8('-'));
    __m128i is_under = _mm_cmpeq_epi8(chunk, _mm_set1_epi8('_'));
    __m128i is_dot = _mm_cmpeq_epi8(chunk, _mm_set1_epi8('.'));
    __m128i is_tilde = _mm_cmpeq_epi8(chunk, _mm_set1_epi8('~'));

    is_safe = _mm_or_si128(is_safe, _mm_or_si128(
                                        _mm_or_si128(is_dash, is_under),
                                        _mm_or_si128(is_dot, is_tilde)));

    int mask = _mm_movemask_epi8(is_safe);

    if (mask == 0xFFFF)
    {
      // All safe - copy directly
      if (dst)
      {
        _mm_storeu_si128((__m128i *)(dst + dst_pos), chunk);
      }
      dst_pos += 16;
      i += 16;
    }
    else
    {
      // Fallback to scalar for this chunk
      size_t encoded = url_encode_scalar(src + i, 16, dst ? dst + dst_pos : NULL);
      dst_pos += encoded;
      i += 16;
    }
  }

  // Handle remainder
  if (i < src_len)
  {
    size_t encoded = url_encode_scalar(src + i, src_len - i, dst ? dst + dst_pos : NULL);
    dst_pos += encoded;
  }

  return dst_pos;
}
#endif

char *vex_url_encode(const char *str)
{
  if (!str)
    return NULL;

  size_t src_len = vex_strlen(str);

  // First pass: calculate required size
  size_t dst_len;
#if VEX_SIMD_X86
  dst_len = url_encode_simd_x86(str, src_len, NULL);
#else
  dst_len = url_encode_scalar(str, src_len, NULL);
#endif

  // Allocate buffer
  char *result = (char *)vex_malloc(dst_len + 1);

  // Second pass: encode
#if VEX_SIMD_X86
  url_encode_simd_x86(str, src_len, result);
#else
  url_encode_scalar(str, src_len, result);
#endif

  result[dst_len] = '\0';
  return result;
}

// ============================================================================
// URL DECODING
// ============================================================================

static inline int hex_digit_to_int(char c)
{
  if (c >= '0' && c <= '9')
    return c - '0';
  if (c >= 'A' && c <= 'F')
    return c - 'A' + 10;
  if (c >= 'a' && c <= 'f')
    return c - 'a' + 10;
  return -1;
}

char *vex_url_decode(const char *str)
{
  if (!str)
    return NULL;

  size_t src_len = vex_strlen(str);
  char *result = (char *)vex_malloc(src_len + 1); // Decoded will be <= original
  size_t dst_pos = 0;

  for (size_t i = 0; i < src_len; i++)
  {
    char c = str[i];

    if (c == '%' && i + 2 < src_len)
    {
      // Percent-encoded character
      int high = hex_digit_to_int(str[i + 1]);
      int low = hex_digit_to_int(str[i + 2]);

      if (high >= 0 && low >= 0)
      {
        result[dst_pos++] = (char)((high << 4) | low);
        i += 2;
      }
      else
      {
        // Invalid encoding - keep as-is
        result[dst_pos++] = c;
      }
    }
    else if (c == '+')
    {
      // '+' decodes to space
      result[dst_pos++] = ' ';
    }
    else
    {
      result[dst_pos++] = c;
    }
  }

  result[dst_pos] = '\0';
  return result;
}

// ============================================================================
// QUERY STRING PARSING
// ============================================================================

typedef struct
{
  char *key;
  char *value;
} VexUrlParam;

VexMap *vex_url_parse_query(const char *query)
{
  if (!query)
    return NULL;

  VexMap *params = (VexMap *)vex_malloc(sizeof(VexMap));
  vex_map_new(params, 16);

  // Make a copy to modify
  char *query_copy = vex_strdup(query);
  char *saveptr = NULL;

  // Split by '&'
  char *pair = strtok_r(query_copy, "&", &saveptr);
  while (pair)
  {
    // Split by '='
    char *eq = strchr(pair, '=');
    if (eq)
    {
      *eq = '\0';
      char *key = vex_url_decode(pair);
      char *value = vex_url_decode(eq + 1);
      vex_map_insert(params, key, value);
    }
    else
    {
      // Key without value
      char *key = vex_url_decode(pair);
      vex_map_insert(params, key, vex_strdup(""));
    }

    pair = strtok_r(NULL, "&", &saveptr);
  }

  vex_free(query_copy);
  return params;
}

// ============================================================================
// URL PARSING
// ============================================================================

VexUrl *vex_url_parse(const char *url_str)
{
  if (!url_str)
    return NULL;

  VexUrl *url = (VexUrl *)vex_calloc(1, sizeof(VexUrl));
  const char *p = url_str;

  // Parse scheme (http://)
  const char *scheme_end = strstr(p, "://");
  if (scheme_end)
  {
    size_t scheme_len = scheme_end - p;
    url->scheme = (char *)vex_malloc(scheme_len + 1);
    vex_memcpy(url->scheme, p, scheme_len);
    url->scheme[scheme_len] = '\0';
    p = scheme_end + 3;
  }

  // Parse host and port
  const char *path_start = strchr(p, '/');
  const char *query_start = strchr(p, '?');
  const char *fragment_start = strchr(p, '#');

  const char *host_end = path_start ? path_start : (query_start ? query_start : (fragment_start ? fragment_start : p + vex_strlen(p)));

  // Check for port
  const char *colon = strchr(p, ':');
  if (colon && colon < host_end)
  {
    size_t host_len = colon - p;
    url->host = (char *)vex_malloc(host_len + 1);
    vex_memcpy(url->host, p, host_len);
    url->host[host_len] = '\0';

    url->port = atoi(colon + 1);
    p = host_end;
  }
  else
  {
    size_t host_len = host_end - p;
    url->host = (char *)vex_malloc(host_len + 1);
    vex_memcpy(url->host, p, host_len);
    url->host[host_len] = '\0';

    url->port = -1; // Default port
    p = host_end;
  }

  // Parse path
  if (*p == '/')
  {
    const char *path_end = query_start ? query_start : (fragment_start ? fragment_start : p + vex_strlen(p));
    size_t path_len = path_end - p;
    url->path = (char *)vex_malloc(path_len + 1);
    vex_memcpy(url->path, p, path_len);
    url->path[path_len] = '\0';
    p = path_end;
  }

  // Parse query
  if (*p == '?')
  {
    p++; // Skip '?'
    const char *query_end = fragment_start ? fragment_start : p + vex_strlen(p);
    size_t query_len = query_end - p;
    url->query = (char *)vex_malloc(query_len + 1);
    vex_memcpy(url->query, p, query_len);
    url->query[query_len] = '\0';
    p = query_end;
  }

  // Parse fragment
  if (*p == '#')
  {
    p++; // Skip '#'
    url->fragment = vex_strdup(p);
  }

  return url;
}

void vex_url_free(VexUrl *url)
{
  if (!url)
    return;

  if (url->scheme)
    vex_free(url->scheme);
  if (url->host)
    vex_free(url->host);
  if (url->path)
    vex_free(url->path);
  if (url->query)
    vex_free(url->query);
  if (url->fragment)
    vex_free(url->fragment);

  vex_free(url);
}
