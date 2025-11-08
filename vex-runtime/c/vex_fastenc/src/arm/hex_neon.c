/**
 * HEX (Base16) NEON Implementation for ARM64
 * Process 16 bytes at a time using ARM NEON SIMD
 */

#include "vex_fastenc.h"

#if defined(__aarch64__) || defined(_M_ARM64)
#if defined(__ARM_NEON) || defined(__ARM_NEON__)

#include <arm_neon.h>
#include <string.h>

/* ============================================================================
   HEX ENCODE (16 bytes → 32 chars)
   ============================================================================ */

static inline void hex16_neon_encode(const uint8_t* src, char* dst, int uppercase){
  uint8x16_t v = vld1q_u8(src);
  uint8x16_t hi = vshrq_n_u8(v, 4);
  uint8x16_t lo = vandq_u8(v, vdupq_n_u8(0x0F));
  
  uint8x16_t base = vdupq_n_u8('0');
  uint8x16_t adj = vdupq_n_u8(uppercase ? 7 : 39);  /* 'A'-'0'-10 or 'a'-'0'-10 */
  
  uint8x16_t hi_cmp = vcgtq_u8(hi, vdupq_n_u8(9));
  uint8x16_t lo_cmp = vcgtq_u8(lo, vdupq_n_u8(9));
  
  uint8x16_t ahi = vaddq_u8(hi, base);
  uint8x16_t alo = vaddq_u8(lo, base);
  ahi = vaddq_u8(ahi, vandq_u8(hi_cmp, adj));
  alo = vaddq_u8(alo, vandq_u8(lo_cmp, adj));
  
  /* Interleave hi and lo nibbles */
  uint8x16x2_t zip = vzipq_u8(ahi, alo);
  vst1q_u8((uint8_t*)dst, zip.val[0]);
  vst1q_u8((uint8_t*)(dst + 16), zip.val[1]);
}

size_t vex_hex_encode_neon(const uint8_t* src, size_t n, char* dst, int uppercase){
  size_t i = 0;
  size_t o = 0;
  
  /* Process 16 bytes at a time */
  for (; i + 16 <= n; i += 16){
    hex16_neon_encode(src + i, dst + o, uppercase);
    o += 32;
  }
  
  /* Scalar fallback for remainder */
  for (; i < n; i++){
    uint8_t b = src[i];
    uint8_t hi = (b >> 4) & 0xF;
    uint8_t lo = b & 0xF;
    dst[o++] = (char)(hi < 10 ? '0' + hi : (uppercase ? 'A' + (hi - 10) : 'a' + (hi - 10)));
    dst[o++] = (char)(lo < 10 ? '0' + lo : (uppercase ? 'A' + (lo - 10) : 'a' + (lo - 10)));
  }
  
  return o;
}

/* ============================================================================
   HEX DECODE (32 chars → 16 bytes)
   ============================================================================ */

static inline int hex16_neon_decode(const char* src, uint8_t* dst){
  uint8x16_t hi_chars = vld1q_u8((const uint8_t*)src);
  uint8x16_t lo_chars = vld1q_u8((const uint8_t*)(src + 16));
  
  /* Deinterleave (every other character) */
  uint8x16x2_t deinterleaved;
  deinterleaved.val[0] = hi_chars;
  deinterleaved.val[1] = lo_chars;
  uint8x16x2_t unzipped = vuzpq_u8(deinterleaved.val[0], deinterleaved.val[1]);
  uint8x16_t hi = unzipped.val[0];
  uint8x16_t lo = unzipped.val[1];
  
  /* Decode nibbles: '0'-'9' → 0-9, 'A'-'F' → 10-15, 'a'-'f' → 10-15 */
  
  /* Digit check: '0' <= c <= '9' */
  uint8x16_t hi_is_digit = vandq_u8(
    vcgeq_u8(hi, vdupq_n_u8('0')),
    vcleq_u8(hi, vdupq_n_u8('9'))
  );
  uint8x16_t lo_is_digit = vandq_u8(
    vcgeq_u8(lo, vdupq_n_u8('0')),
    vcleq_u8(lo, vdupq_n_u8('9'))
  );
  
  /* Upper case: 'A' <= c <= 'F' */
  uint8x16_t hi_is_upper = vandq_u8(
    vcgeq_u8(hi, vdupq_n_u8('A')),
    vcleq_u8(hi, vdupq_n_u8('F'))
  );
  uint8x16_t lo_is_upper = vandq_u8(
    vcgeq_u8(lo, vdupq_n_u8('A')),
    vcleq_u8(lo, vdupq_n_u8('F'))
  );
  
  /* Lower case: 'a' <= c <= 'f' */
  uint8x16_t hi_is_lower = vandq_u8(
    vcgeq_u8(hi, vdupq_n_u8('a')),
    vcleq_u8(hi, vdupq_n_u8('f'))
  );
  uint8x16_t lo_is_lower = vandq_u8(
    vcgeq_u8(lo, vdupq_n_u8('a')),
    vcleq_u8(lo, vdupq_n_u8('f'))
  );
  
  /* Decode */
  uint8x16_t hi_val = vbslq_u8(hi_is_digit, vsubq_u8(hi, vdupq_n_u8('0')), vdupq_n_u8(0));
  hi_val = vbslq_u8(hi_is_upper, vsubq_u8(hi, vdupq_n_u8('A' - 10)), hi_val);
  hi_val = vbslq_u8(hi_is_lower, vsubq_u8(hi, vdupq_n_u8('a' - 10)), hi_val);
  
  uint8x16_t lo_val = vbslq_u8(lo_is_digit, vsubq_u8(lo, vdupq_n_u8('0')), vdupq_n_u8(0));
  lo_val = vbslq_u8(lo_is_upper, vsubq_u8(lo, vdupq_n_u8('A' - 10)), lo_val);
  lo_val = vbslq_u8(lo_is_lower, vsubq_u8(lo, vdupq_n_u8('a' - 10)), lo_val);
  
  /* Combine nibbles */
  uint8x16_t result = vorrq_u8(vshlq_n_u8(hi_val, 4), lo_val);
  vst1q_u8(dst, result);
  
  return 0;  /* Success */
}

ssize_t vex_hex_decode_neon(const char* src, size_t n, uint8_t* dst){
  if (n % 2) return -1;
  
  size_t i = 0;
  size_t o = 0;
  
  /* Process 32 chars (16 bytes output) at a time */
  for (; i + 32 <= n; i += 32){
    if (hex16_neon_decode(src + i, dst + o) != 0) {
      return -1;  /* Invalid hex */
    }
    o += 16;
  }
  
  /* Scalar fallback for remainder */
  for (; i < n; i += 2){
    unsigned char hi_char = (unsigned char)src[i];
    unsigned char lo_char = (unsigned char)src[i+1];
    
    int hi = -1, lo = -1;
    
    if (hi_char >= '0' && hi_char <= '9') hi = hi_char - '0';
    else if (hi_char >= 'a' && hi_char <= 'f') hi = hi_char - 'a' + 10;
    else if (hi_char >= 'A' && hi_char <= 'F') hi = hi_char - 'A' + 10;
    
    if (lo_char >= '0' && lo_char <= '9') lo = lo_char - '0';
    else if (lo_char >= 'a' && lo_char <= 'f') lo = lo_char - 'a' + 10;
    else if (lo_char >= 'A' && lo_char <= 'F') lo = lo_char - 'A' + 10;
    
    if (hi < 0 || lo < 0) return -1;
    
    dst[o++] = (uint8_t)((hi << 4) | lo);
  }
  
  return (ssize_t)o;
}

#endif /* __ARM_NEON */
#endif /* __aarch64__ */
