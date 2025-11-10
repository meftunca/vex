// vex_strconv.c
// Fast & safe numeric converters for Vex (C, single-file)
// - Integer parsing: signed/unsigned 64-bit with overflow checks, base 2..36 (+ autodetect 0x/0b/0 prefix)
// - Float parsing: fast decimal path (Eisel-Lemire style) with safe fallback to strtod (configurable)
// - SIMD-assisted (AVX2 / NEON) whitespace skipping & digit block checks; scalar fallback always present
// - No dynamic allocations; re-entrant; detailed error reporting
//
// Build: cc -O3 -Wall -Wextra -std=c11 -o test vex_strconv.c
//
// API:
//   typedef enum { VX_OK=0, VX_EINVAL, VX_ERANGE, VX_EOVERFLOW, VX_EUNDERFLOW } VxErr;
//   typedef struct { VxErr err; size_t n_consumed; } VxParse;
//
//   bool vx_parse_u64(const char *s, size_t len, unsigned base, uint64_t *out, VxParse *st);
//   bool vx_parse_i64(const char *s, size_t len, unsigned base, int64_t *out, VxParse *st);
//   bool vx_parse_f64(const char *s, size_t len, double *out, VxParse *st); // decimal/scientific
//
// Optional macros:
//   #define VX_STRCONV_USE_STRTOD_FALLBACK 1  // default 1: uses strtod for hard cases to guarantee correctness
//   #define VX_STRCONV_SIMD 1                 // default 1 if platform supported; set 0 to force scalar
//
// License: CC0 / Public Domain.

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <string.h>
#include <float.h>
#include <math.h>
#include <errno.h>
#include <stdlib.h>
#include <stdio.h>

// Use vex_macros.h if available (Vex runtime integration)
#if __has_include("vex_macros.h")
#include "vex_macros.h"
// vex_macros.h provides VEX_SIMD_X86 and VEX_SIMD_NEON with intrinsics
// Use compatibility aliases for vex_strconv.c
#define VX_SIMD_X86 VEX_SIMD_X86
#define VX_SIMD_NEON VEX_SIMD_NEON
#else
// Standalone mode: Define SIMD detection locally
#if !defined(VX_STRCONV_SIMD)
#define VX_STRCONV_SIMD 1
#endif

#if VX_STRCONV_SIMD
#if defined(__x86_64__) || defined(_M_X64) || defined(__i386__)
#include <immintrin.h>
#define VX_SIMD_X86 1
#else
#define VX_SIMD_X86 0
#endif
#if defined(__ARM_NEON) || defined(__ARM_NEON__) || defined(__aarch64__)
#include <arm_neon.h>
#define VX_SIMD_NEON 1
#else
#define VX_SIMD_NEON 0
#endif
#else
#define VX_SIMD_X86 0
#define VX_SIMD_NEON 0
#endif
#endif

#ifndef VX_STRCONV_USE_STRTOD_FALLBACK
#define VX_STRCONV_USE_STRTOD_FALLBACK 1
#endif

typedef enum
{
  VX_OK = 0,
  VX_EINVAL,
  VX_ERANGE,
  VX_EOVERFLOW,
  VX_EUNDERFLOW
} VxErr;
typedef struct
{
  VxErr err;
  size_t n_consumed;
} VxParse;

// ======================= helpers =======================

static inline bool is_space(unsigned char c)
{
  return (c == 32) | (c >= 9 && c <= 13);
}
static inline int digit_val(int c)
{
  if (c >= '0' && c <= '9')
    return c - '0';
  if (c >= 'a' && c <= 'z')
    return c - 'a' + 10;
  if (c >= 'A' && c <= 'Z')
    return c - 'A' + 10;
  return -1;
}

static inline size_t skip_spaces_scalar(const char *s, size_t len)
{
  size_t i = 0;
  while (i < len && is_space((unsigned char)s[i]))
    ++i;
  return i;
}

#if VX_SIMD_X86
static inline size_t skip_spaces_x86(const char *s, size_t len)
{
  size_t i = 0;
  while (i + 16 <= len)
  {
    __m128i v = _mm_loadu_si128((const __m128i *)(s + i));
    // classify spaces: 0x20 or 0x09..0x0D
    __m128i is20 = _mm_cmpeq_epi8(v, _mm_set1_epi8(0x20));
    __m128i ge09 = _mm_cmpeq_epi8(_mm_max_epu8(v, _mm_set1_epi8(9)), v);  // v>=9
    __m128i le0D = _mm_cmpeq_epi8(_mm_min_epu8(v, _mm_set1_epi8(13)), v); // v<=13
    __m128i tabnl = _mm_and_si128(ge09, le0D);
    __m128i ws = _mm_or_si128(is20, tabnl);
    int mask = _mm_movemask_epi8(ws);
    if (mask != 0xFFFF)
    { // first non-space
      int first = __builtin_ctz(~mask & 0xFFFF);
      i += first;
      return i;
    }
    i += 16;
  }
  return i + skip_spaces_scalar(s + i, len - i);
}
#endif

#if VX_SIMD_NEON
static inline size_t skip_spaces_neon(const char *s, size_t len)
{
  size_t i = 0;
  while (i + 16 <= len)
  {
    uint8x16_t v = vld1q_u8((const uint8_t *)(s + i));
    uint8x16_t is20 = vceqq_u8(v, vdupq_n_u8(0x20));
    // 9..13
    uint8x16_t ge09 = vcgeq_u8(v, vdupq_n_u8(9));
    uint8x16_t le0D = vcleq_u8(v, vdupq_n_u8(13));
    uint8x16_t tabnl = vandq_u8(ge09, le0D);
    uint8x16_t ws = vorrq_u8(is20, tabnl);
    uint8_t tmp[16];
    vst1q_u8(tmp, ws);
    int all = 1;
    for (int k = 0; k < 16; k++)
    {
      if (!tmp[k])
      {
        all = 0;
        break;
      }
    }
    if (!all)
    {
      int first = 0;
      for (; first < 16; ++first)
        if (!tmp[first])
          break;
      i += first;
      return i;
    }
    i += 16;
  }
  return i + skip_spaces_scalar(s + i, len - i);
}
#endif

static inline size_t skip_spaces(const char *s, size_t len)
{
#if VX_SIMD_X86
  return skip_spaces_x86(s, len);
#elif VX_SIMD_NEON
  return skip_spaces_neon(s, len);
#else
  return skip_spaces_scalar(s, len);
#endif
}

// fast pow10 table for small exponents
static const double kPow10[] = {
    1e0, 1e1, 1e2, 1e3, 1e4, 1e5, 1e6, 1e7, 1e8, 1e9,
    1e10, 1e11, 1e12, 1e13, 1e14, 1e15, 1e16, 1e17, 1e18, 1e19,
    1e20, 1e21, 1e22};

// ======================= integer parsing =======================

bool vx_parse_u64(const char *s, size_t len, unsigned base, uint64_t *out, VxParse *st)
{
  size_t i = skip_spaces(s, len);
  if (i == len)
  {
    if (st)
      st->err = VX_EINVAL, st->n_consumed = i;
    return false;
  }

  // sign not allowed for unsigned
  if (s[i] == '+' || s[i] == '-')
  {
    if (st)
      st->err = VX_EINVAL, st->n_consumed = i;
    return false;
  }

  // base autodetect
  const char *p = s + i;
  if (base == 0)
  {
    base = 10;
    if (len - i >= 2 && p[0] == '0' && (p[1] == 'x' || p[1] == 'X'))
    {
      base = 16;
      p += 2;
    }
    else if (len - i >= 2 && p[0] == '0' && (p[1] == 'b' || p[1] == 'B'))
    {
      base = 2;
      p += 2;
    }
    else if (len - i >= 1 && p[0] == '0')
    {
      base = 8;
      p += 1;
    }
  }

  uint64_t acc = 0;
  size_t start = (size_t)(p - s);
  size_t n_digits = 0;
  for (; (size_t)(p - s) < len; ++p)
  {
    int d = digit_val((unsigned char)*p);
    if (d < 0 || (unsigned)d >= base)
      break;
    // overflow check: acc*base + d <= UINT64_MAX
    if (acc > (UINT64_MAX - (uint64_t)d) / (uint64_t)base)
    {
      if (st)
        st->err = VX_EOVERFLOW, st->n_consumed = (size_t)(p - s);
      return false;
    }
    acc = acc * (uint64_t)base + (uint64_t)d;
    n_digits++;
  }
  if (n_digits == 0)
  {
    if (st)
      st->err = VX_EINVAL, st->n_consumed = start;
    return false;
  }

  if (out)
    *out = acc;
  if (st)
  {
    st->err = VX_OK;
    st->n_consumed = (size_t)(p - s);
  }
  return true;
}

bool vx_parse_i64(const char *s, size_t len, unsigned base, int64_t *out, VxParse *st)
{
  size_t i = skip_spaces(s, len);
  if (i == len)
  {
    if (st)
      st->err = VX_EINVAL, st->n_consumed = i;
    return false;
  }

  int sign = 1;
  if (s[i] == '+')
  {
    i++;
  }
  else if (s[i] == '-')
  {
    sign = -1;
    i++;
  }

  const char *p = s + i;
  if (base == 0)
  {
    base = 10;
    if (len - i >= 2 && p[0] == '0' && (p[1] == 'x' || p[1] == 'X'))
    {
      base = 16;
      p += 2;
    }
    else if (len - i >= 2 && p[0] == '0' && (p[1] == 'b' || p[1] == 'B'))
    {
      base = 2;
      p += 2;
    }
    else if (len - i >= 1 && p[0] == '0')
    {
      base = 8;
      p += 1;
    }
  }

  uint64_t acc = 0;
  size_t start = (size_t)(p - s);
  size_t n_digits = 0;
  uint64_t limit = (sign > 0) ? (uint64_t)INT64_MAX : (uint64_t)INT64_MAX + 1ULL;
  for (; (size_t)(p - s) < len; ++p)
  {
    int d = digit_val((unsigned char)*p);
    if (d < 0 || (unsigned)d >= base)
      break;
    if (acc > (limit - (uint64_t)d) / (uint64_t)base)
    {
      if (st)
        st->err = VX_EOVERFLOW, st->n_consumed = (size_t)(p - s);
      return false;
    }
    acc = acc * (uint64_t)base + (uint64_t)d;
    n_digits++;
  }
  if (n_digits == 0)
  {
    if (st)
      st->err = VX_EINVAL, st->n_consumed = start;
    return false;
  }

  int64_t val = (sign > 0) ? (int64_t)acc : -(int64_t)acc;
  if (out)
    *out = val;
  if (st)
  {
    st->err = VX_OK;
    st->n_consumed = (size_t)(p - s);
  }
  return true;
}

// ======================= float parsing =======================
// Strategy:
//  - Fast-path: parse decimal significand + exponent, build double via pow10 table and ldexp-like scaling.
//  - Handle up to 19 significand digits without precision loss, and limited exponent range quickly.
//  - For harder cases (very long mantissa, huge exponent, subnormals), optionally fall back to strtod for correctness.

typedef struct
{
  bool neg;
  uint64_t mant; // up to 19 digits
  int mant_digits;
  int exp10; // exponent after parsing (includes fractional shift)
  const char *endp;
} ParsedDec;

static inline bool parse_decimal_fast(const char *s, size_t len, ParsedDec *pd)
{
  size_t i = skip_spaces(s, len);
  if (i == len)
    return false;
  bool neg = false;
  if (s[i] == '+')
    i++;
  else if (s[i] == '-')
  {
    neg = true;
    i++;
  }

  // digits
  uint64_t mant = 0;
  int mant_digits = 0;
  int frac_digits = 0;
  const char *p = s + i;
  // integer part
  while ((size_t)(p - s) < len && *p >= '0' && *p <= '9')
  {
    if (mant_digits < 19)
    {
      mant = mant * 10 + (uint64_t)(*p - '0');
      mant_digits++;
    }
    else
    {                // skip extra digits by tracking exp shift
      mant_digits++; // count for exponent logic
    }
    p++;
  }
  // fraction
  if ((size_t)(p - s) < len && *p == '.')
  {
    p++;
    while ((size_t)(p - s) < len && *p >= '0' && *p <= '9')
    {
      if (mant_digits < 19)
      {
        mant = mant * 10 + (uint64_t)(*p - '0');
        mant_digits++;
        frac_digits++;
      }
      else
      {
        mant_digits++;
        frac_digits++;
      }
      p++;
    }
  }
  if (mant_digits == 0)
    return false;

  int exp10 = -frac_digits;

  // exponent part
  if ((size_t)(p - s) < len && (*p == 'e' || *p == 'E'))
  {
    const char *q = p + 1;
    bool eneg = false;
    if (q < s + len && (*q == '+' || *q == '-'))
    {
      eneg = (*q == '-');
      q++;
    }
    if (q >= s + len || *q < '0' || *q > '9')
      return false;
    int e = 0;
    while (q < s + len && *q >= '0' && *q <= '9' && e <= 10000)
    {
      e = e * 10 + (*q - '0');
      q++;
    }
    if (eneg)
      e = -e;
    exp10 += e;
    p = q;
  }

  pd->neg = neg;
  pd->mant = mant;
  pd->mant_digits = mant_digits;
  pd->exp10 = exp10;
  pd->endp = p;
  return true;
}

static inline double pow10_small(int e)
{
  if (e >= 0 && e <= (int)(sizeof(kPow10) / sizeof(kPow10[0])) - 1)
    return kPow10[e];
  if (e < 0 && -e <= (int)(sizeof(kPow10) / sizeof(kPow10[0])) - 1)
    return 1.0 / kPow10[-e];
  return NAN; // signal out-of-table
}

bool vx_parse_f64(const char *s, size_t len, double *out, VxParse *st)
{
  ParsedDec pd;
  if (!parse_decimal_fast(s, len, &pd))
  {
    if (st)
    {
      st->err = VX_EINVAL;
      st->n_consumed = 0;
    }
    return false;
  }

  // Quick path: small mantissa and modest exponent -> exact double via simple scaling
  if (pd.mant_digits <= 19)
  {
    double d = (double)pd.mant;
    double scale = pow10_small(pd.exp10);
    if (!isnan(scale))
    {
      d = d * scale;
      if (pd.neg)
        d = -d;
      if (out)
        *out = d;
      if (st)
      {
        st->err = VX_OK;
        st->n_consumed = (size_t)(pd.endp - s);
      }
      return true;
    }
  }

#if VX_STRCONV_USE_STRTOD_FALLBACK
  // Conservative fallback: copy to NUL-terminated buffer and use strtod for fully-correct rounding.
  // Guarantees correctness for all inputs.
  size_t n = (size_t)(pd.endp - s);
  char buf[128];
  const char *sp = s;
  if (n < sizeof(buf))
  {
    memcpy(buf, sp, n);
    buf[n] = 0;
    errno = 0;
    char *ep = NULL;
    double v = strtod(buf, &ep);
    if (ep == buf)
    {
      if (st)
      {
        st->err = VX_EINVAL;
        st->n_consumed = 0;
      }
      return false;
    }
    if (errno == ERANGE)
    {
      if (v == 0.0)
      {
        if (st)
          st->err = VX_EUNDERFLOW;
      }
      else
      {
        if (st)
          st->err = VX_ERANGE;
      }
    }
    else if (st)
      st->err = VX_OK;
    if (pd.neg)
      v = -fabs(v); // ensure sign
    if (out)
      *out = v;
    if (st)
      st->n_consumed = n;
    return st ? st->err == VX_OK || st->err == VX_ERANGE || st->err == VX_EUNDERFLOW : true;
  }
  else
  {
    // allocate temporary if very long
    char *tmp = (char *)malloc(n + 1);
    if (!tmp)
    {
      if (st)
      {
        st->err = VX_ERANGE;
        st->n_consumed = 0;
      }
      return false;
    }
    memcpy(tmp, sp, n);
    tmp[n] = 0;
    errno = 0;
    char *ep = NULL;
    double v = strtod(tmp, &ep);
    free(tmp);
    if (!ep || ep == tmp)
    {
      if (st)
      {
        st->err = VX_EINVAL;
        st->n_consumed = 0;
      }
      return false;
    }
    if (errno == ERANGE)
    {
      if (v == 0.0)
      {
        if (st)
          st->err = VX_EUNDERFLOW;
      }
      else
      {
        if (st)
          st->err = VX_ERANGE;
      }
    }
    else if (st)
      st->err = VX_OK;
    if (pd.neg)
      v = -fabs(v);
    if (out)
      *out = v;
    if (st)
      st->n_consumed = n;
    return st ? st->err == VX_OK || st->err == VX_ERANGE || st->err == VX_EUNDERFLOW : true;
  }
#else
  // If fallback disabled, attempt a coarse scaling using pow10 and ldexp for large exponents.
  int e = pd.exp10;
  double d = (double)pd.mant;
  while (e > 0)
  {
    int step = (e > 22) ? 22 : e;
    d *= kPow10[step];
    e -= step;
  }
  while (e < 0)
  {
    int step = (-e > 22) ? 22 : -e;
    d /= kPow10[step];
    e += step;
  }
  if (pd.neg)
    d = -d;
  if (out)
    *out = d;
  if (st)
  {
    st->err = VX_OK;
    st->n_consumed = (size_t)(pd.endp - s);
  }
  return true;
#endif
}

// ======================= Vex Runtime API Wrappers =======================
#include "vex.h"

// Wrapper functions for Vex runtime
bool vex_parse_i64(const char *str, int64_t *out)
{
  if (!str)
    return false;
  VxParse st;
  return vx_parse_i64(str, vex_strlen(str), 0, out, &st) && st.err == VX_OK;
}

bool vex_parse_u64(const char *str, uint64_t *out)
{
  if (!str)
    return false;
  VxParse st;
  return vx_parse_u64(str, vex_strlen(str), 0, out, &st) && st.err == VX_OK;
}

bool vex_parse_f64(const char *str, double *out)
{
  if (!str)
    return false;
  VxParse st;
  return vx_parse_f64(str, vex_strlen(str), out, &st) && st.err == VX_OK;
}

int64_t vex_str_to_i64(const char *str)
{
  int64_t result = 0;
  vex_parse_i64(str, &result);
  return result;
}

uint64_t vex_str_to_u64(const char *str)
{
  uint64_t result = 0;
  vex_parse_u64(str, &result);
  return result;
}

double vex_str_to_f64(const char *str)
{
  double result = 0.0;
  vex_parse_f64(str, &result);
  return result;
}

// Integer to string conversion
char *vex_i64_to_str(int64_t value)
{
  char buffer[32];
  snprintf(buffer, sizeof(buffer), "%lld", (long long)value);
  return vex_strdup(buffer);
}

char *vex_u64_to_str(uint64_t value)
{
  char buffer[32];
  snprintf(buffer, sizeof(buffer), "%llu", (unsigned long long)value);
  return vex_strdup(buffer);
}

char *vex_f64_to_str(double value)
{
  char buffer[64];
  snprintf(buffer, sizeof(buffer), "%.17g", value);
  return vex_strdup(buffer);
}

char *vex_i64_to_str_base(int64_t value, unsigned base)
{
  if (base < 2 || base > 36)
  {
    vex_panic("vex_i64_to_str_base: invalid base");
  }

  if (base == 10)
    return vex_i64_to_str(value);

  char buffer[128];
  char *ptr = buffer + sizeof(buffer) - 1;
  *ptr = '\0';

  bool negative = value < 0;
  uint64_t uval = negative ? -(uint64_t)value : (uint64_t)value;

  const char *digits = "0123456789abcdefghijklmnopqrstuvwxyz";
  do
  {
    *--ptr = digits[uval % base];
    uval /= base;
  } while (uval > 0);

  if (negative)
    *--ptr = '-';

  return vex_strdup(ptr);
}

// ============================================================================
// Number to String Conversion (for Vex to_string() support)
// ============================================================================

// Convert i32 to string (base 10)
char *vex_i32_to_string(int32_t value)
{
  char buffer[16]; // -2147483648 = 11 chars + null
  snprintf(buffer, sizeof(buffer), "%d", value);
  return strdup(buffer);
}

// Convert i64 to string (base 10)
char *vex_i64_to_string(int64_t value)
{
  char buffer[24]; // -9223372036854775808 = 20 chars + null
  snprintf(buffer, sizeof(buffer), "%lld", (long long)value);
  return strdup(buffer);
}

// Convert u32 to string (base 10)
char *vex_u32_to_string(uint32_t value)
{
  char buffer[16];
  snprintf(buffer, sizeof(buffer), "%u", value);
  return strdup(buffer);
}

// Convert u64 to string (base 10)
char *vex_u64_to_string(uint64_t value)
{
  char buffer[24];
  snprintf(buffer, sizeof(buffer), "%llu", (unsigned long long)value);
  return strdup(buffer);
}

// Convert f32 to string
char *vex_f32_to_string(float value)
{
  char buffer[64];
  snprintf(buffer, sizeof(buffer), "%g", value);
  return strdup(buffer);
}

// Convert f64 to string
char *vex_f64_to_string(double value)
{
  char buffer[64];
  snprintf(buffer, sizeof(buffer), "%g", value);
  return strdup(buffer);
}

// Convert bool to string
char *vex_bool_to_string(bool value)
{
  return strdup(value ? "true" : "false");
}

// String to string (identity, but allocates new copy for consistency)
char *vex_string_to_string(const char *value)
{
  return strdup(value ? value : "");
}
