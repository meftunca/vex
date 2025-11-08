/**
 * HEX (Base16) Encoding/Decoding with SIMD Acceleration
 * 
 * Supports:
 * - x86-64: AVX2, AVX-512BW
 * - ARM64: NEON
 * - Fallback: Optimized scalar
 */

#include "vex_fastenc.h"
#include <string.h>
#include <ctype.h>

/* ============================================================================
   SIMD FUNCTION DECLARATIONS (implemented in arch-specific files)
   ============================================================================ */

/* x86-64 SIMD */
#if defined(__x86_64__) || defined(_M_X64)
  extern size_t vex_hex_encode_avx2(const uint8_t* src, size_t n, char* dst, int uppercase);
  extern ssize_t vex_hex_decode_avx2(const char* src, size_t n, uint8_t* dst);
  
  #if defined(__AVX512BW__) && defined(__AVX512F__)
    extern size_t vex_hex_encode_avx512(const uint8_t* src, size_t n, char* dst, int uppercase);
    extern ssize_t vex_hex_decode_avx512(const char* src, size_t n, uint8_t* dst);
  #endif
#endif

/* ARM64 SIMD */
#if defined(__aarch64__) || defined(_M_ARM64)
  #if defined(__ARM_NEON) || defined(__ARM_NEON__)
    extern size_t vex_hex_encode_neon(const uint8_t* src, size_t n, char* dst, int uppercase);
    extern ssize_t vex_hex_decode_neon(const char* src, size_t n, uint8_t* dst);
  #endif
#endif

/* ============================================================================
   OPTIMIZED SCALAR FALLBACK
   ============================================================================ */

size_t vex_hex_encoded_len(size_t nbytes){ return nbytes*2; }
size_t vex_hex_decoded_len(size_t nchars){ return nchars/2; }

static inline char hex_digit_low(int v){ return (v<10)?(char)('0'+v):(char)('a'+(v-10)); }
static inline char hex_digit_up (int v){ return (v<10)?(char)('0'+v):(char)('A'+(v-10)); }

/* Optimized scalar encode (unrolled) */
static size_t hex_encode_scalar(const uint8_t* src, size_t n, char* dst, int uppercase){
  const char* hexd = uppercase? "0123456789ABCDEF" : "0123456789abcdef";
  
  size_t i = 0;
  
  /* Process 4 bytes at a time (unrolled) */
  for (; i + 4 <= n; i += 4) {
    uint8_t b0 = src[i];
    uint8_t b1 = src[i+1];
    uint8_t b2 = src[i+2];
    uint8_t b3 = src[i+3];
    
    dst[2*i+0] = hexd[(b0>>4)&0xF];
    dst[2*i+1] = hexd[(b0   )&0xF];
    dst[2*i+2] = hexd[(b1>>4)&0xF];
    dst[2*i+3] = hexd[(b1   )&0xF];
    dst[2*i+4] = hexd[(b2>>4)&0xF];
    dst[2*i+5] = hexd[(b2   )&0xF];
    dst[2*i+6] = hexd[(b3>>4)&0xF];
    dst[2*i+7] = hexd[(b3   )&0xF];
  }
  
  /* Handle remainder */
  for (; i < n; i++){
    uint8_t b = src[i];
    dst[2*i+0] = hexd[(b>>4)&0xF];
    dst[2*i+1] = hexd[(b   )&0xF];
  }
  
  return n*2;
}

static inline int hex_val(int c){
  if (c>='0'&&c<='9') return c-'0';
  if (c>='a'&&c<='f') return c-'a'+10;
  if (c>='A'&&c<='F') return c-'A'+10;
  return -1;
}

/* Optimized scalar decode (unrolled) */
static ssize_t hex_decode_scalar(const char* src, size_t n, uint8_t* dst){
  if (n%2) return -1;
  
  size_t i = 0;
  
  /* Process 8 chars (4 bytes) at a time (unrolled) */
  for (; i + 8 <= n; i += 8) {
    int hi0 = hex_val((unsigned char)src[i]);
    int lo0 = hex_val((unsigned char)src[i+1]);
    int hi1 = hex_val((unsigned char)src[i+2]);
    int lo1 = hex_val((unsigned char)src[i+3]);
    int hi2 = hex_val((unsigned char)src[i+4]);
    int lo2 = hex_val((unsigned char)src[i+5]);
    int hi3 = hex_val((unsigned char)src[i+6]);
    int lo3 = hex_val((unsigned char)src[i+7]);
    
    if ((hi0|lo0|hi1|lo1|hi2|lo2|hi3|lo3) < 0) return -1;
    
    dst[i/2+0] = (uint8_t)((hi0<<4)|lo0);
    dst[i/2+1] = (uint8_t)((hi1<<4)|lo1);
    dst[i/2+2] = (uint8_t)((hi2<<4)|lo2);
    dst[i/2+3] = (uint8_t)((hi3<<4)|lo3);
  }
  
  /* Handle remainder */
  for (; i < n; i += 2){
    int hi = hex_val((unsigned char)src[i]);
    int lo = hex_val((unsigned char)src[i+1]);
    if (hi<0 || lo<0) return -1;
    dst[i/2] = (uint8_t)((hi<<4)|lo);
  }
  
  return (ssize_t)(n/2);
}

/* ============================================================================
   SIMD DISPATCHERS (choose best implementation at runtime)
   ============================================================================ */

size_t vex_hex_encode(const uint8_t* src, size_t n, char* dst, int uppercase){
  /* Use SIMD if available and data is large enough */
  
#if defined(__AVX512BW__) && defined(__AVX512F__)
  if (n >= 64) {
    return vex_hex_encode_avx512(src, n, dst, uppercase);
  }
#endif

#if defined(__AVX2__)
  if (n >= 32) {
    return vex_hex_encode_avx2(src, n, dst, uppercase);
  }
#endif

#if defined(__aarch64__) && (defined(__ARM_NEON) || defined(__ARM_NEON__))
  if (n >= 16) {
    return vex_hex_encode_neon(src, n, dst, uppercase);
  }
#endif

  /* Fallback to optimized scalar */
  return hex_encode_scalar(src, n, dst, uppercase);
}

ssize_t vex_hex_decode(const char* src, size_t n, uint8_t* dst){
  /* Use SIMD if available and data is large enough */
  
#if defined(__AVX512BW__) && defined(__AVX512F__)
  if (n >= 128) {
    return vex_hex_decode_avx512(src, n, dst);
  }
#endif

#if defined(__AVX2__)
  if (n >= 64) {
    return vex_hex_decode_avx2(src, n, dst);
  }
#endif

#if defined(__aarch64__) && (defined(__ARM_NEON) || defined(__ARM_NEON__))
  if (n >= 32) {
    return vex_hex_decode_neon(src, n, dst);
  }
#endif

  /* Fallback to optimized scalar */
  return hex_decode_scalar(src, n, dst);
}
