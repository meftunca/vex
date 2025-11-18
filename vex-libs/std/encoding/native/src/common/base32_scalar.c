/**
 * Base32 Encoding/Decoding with SIMD Optimization
 * 
 * Supports:
 * - RFC 4648 Base32 (A-Z, 2-7)
 * - Base32hex (0-9, A-V)
 * - Crockford (0-9, A-Z excluding I, L, O, U)
 * 
 * SIMD: NEON (ARM64) and AVX2 (x86-64) for decode
 */

#include "vex_fastenc.h"
#include <string.h>
#include <ctype.h>

/* ============================================================================
   SIMD Platform Detection
   ============================================================================ */

#if defined(__aarch64__) || defined(_M_ARM64)
  #if defined(__ARM_NEON) || defined(__ARM_NEON__)
    #define VEX_BASE32_NEON 1
    #include <arm_neon.h>
  #endif
#elif defined(__x86_64__) || defined(_M_X64)
  #if defined(__AVX2__)
    #define VEX_BASE32_AVX2 1
    #include <immintrin.h>
  #endif
#endif

/* ============================================================================
   LOOKUP TABLES
   ============================================================================ */

static const char B32_RFC[] = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567"; /* RFC 4648 */
static const char B32_HEX[] = "0123456789ABCDEFGHIJKLMNOPQRSTUV"; /* Base32hex */
static const char B32_CRK[] = "0123456789ABCDEFGHJKMNPQRSTVWXYZ"; /* Crockford */

size_t vex_base32_max_decoded_len(size_t n){ return (n*5)/8 + 8; }

size_t vex_base32_encoded_len(size_t n, vex_b32_cfg cfg){
  size_t full = (n/5)*8;
  size_t rem  = n%5;
  size_t out  = full;
  if (rem){
    if (cfg.pad){
      out += 8;
    } else {
      static const int O[5]={0,2,4,5,7};
      out += O[rem];
    }
  }
  return out;
}

static inline const char* alpha_str(vex_b32_alphabet a){
  switch(a){
    case VEX_B32_HEX: return B32_HEX;
    case VEX_B32_CROCKFORD: return B32_CRK;
    default: return B32_RFC;
  }
}

/* ============================================================================
   OPTIMIZED SCALAR ENCODE (with unrolling)
   ============================================================================ */

size_t vex_base32_encode(const uint8_t* src, size_t n, char* dst, vex_b32_cfg cfg){
  const char* ABC = alpha_str(cfg.alpha);
  size_t i=0, o=0;
  
  /* Process 5 bytes at a time (unrolled 2x for 10 bytes) */
  while (i + 10 <= n){
    /* First 5 bytes → 8 chars */
    uint64_t v1 = ((uint64_t)src[i]<<32) | ((uint64_t)src[i+1]<<24) | 
                  ((uint64_t)src[i+2]<<16) | ((uint64_t)src[i+3]<<8) | (uint64_t)src[i+4];
    dst[o++] = ABC[(v1>>35)&31]; dst[o++] = ABC[(v1>>30)&31];
    dst[o++] = ABC[(v1>>25)&31]; dst[o++] = ABC[(v1>>20)&31];
    dst[o++] = ABC[(v1>>15)&31]; dst[o++] = ABC[(v1>>10)&31];
    dst[o++] = ABC[(v1>>5 )&31]; dst[o++] = ABC[(v1    )&31];
    
    /* Second 5 bytes → 8 chars */
    uint64_t v2 = ((uint64_t)src[i+5]<<32) | ((uint64_t)src[i+6]<<24) | 
                  ((uint64_t)src[i+7]<<16) | ((uint64_t)src[i+8]<<8) | (uint64_t)src[i+9];
    dst[o++] = ABC[(v2>>35)&31]; dst[o++] = ABC[(v2>>30)&31];
    dst[o++] = ABC[(v2>>25)&31]; dst[o++] = ABC[(v2>>20)&31];
    dst[o++] = ABC[(v2>>15)&31]; dst[o++] = ABC[(v2>>10)&31];
    dst[o++] = ABC[(v2>>5 )&31]; dst[o++] = ABC[(v2    )&31];
    
    i += 10;
  }
  
  /* Handle remainder 5 bytes at a time */
  while (i+5<=n){
    uint64_t v = ((uint64_t)src[i]<<32) | ((uint64_t)src[i+1]<<24) | 
                 ((uint64_t)src[i+2]<<16) | ((uint64_t)src[i+3]<<8) | (uint64_t)src[i+4];
    dst[o++] = ABC[(v>>35)&31]; dst[o++] = ABC[(v>>30)&31];
    dst[o++] = ABC[(v>>25)&31]; dst[o++] = ABC[(v>>20)&31];
    dst[o++] = ABC[(v>>15)&31]; dst[o++] = ABC[(v>>10)&31];
    dst[o++] = ABC[(v>>5 )&31]; dst[o++] = ABC[(v    )&31];
    i+=5;
  }
  
  /* Handle final partial block */
  size_t rem = n-i;
  if (rem){
    uint64_t v=0; 
    for (size_t k=0;k<rem;k++) v |= (uint64_t)src[i+k] << (8*(4-k));
    int out_chars = (int)((rem*8 + 4)/5);
    for (int j=0;j<out_chars;j++){
      int sh = 35 - 5*j;
      dst[o++] = ABC[(v>>sh)&31];
    }
    if (cfg.pad){
      while (o%8) dst[o++]='=';
    }
  }
  return o;
}

/* ============================================================================
   DECODE LOOKUP TABLES (for SIMD)
   ============================================================================ */

static inline uint8_t de_rfc(int c){
  if (c>='A'&&c<='Z') return (uint8_t)(c-'A');
  if (c>='2'&&c<='7') return (uint8_t)(c-'2'+26);
  return 0xFF;
}
static inline uint8_t de_hex(int c){
  if (c>='0'&&c<='9') return (uint8_t)(c-'0');
  if (c>='A'&&c<='V') return (uint8_t)(c-'A'+10);
  return 0xFF;
}
static inline uint8_t de_crock(int c){
  if (c>='0'&&c<='9') return (uint8_t)(c-'0');
  c = toupper(c);
  if (c=='O') c='0'; if (c=='I'||c=='L') c='1';
  const char* p = strchr(B32_CRK, c);
  if (!p) return 0xFF;
  return (uint8_t)(p - B32_CRK);
}

/* ============================================================================
   SIMD BASE32 DECODE (RFC only for simplicity)
   ============================================================================ */

#ifdef VEX_BASE32_NEON

/* NEON: Decode 16 chars → 10 bytes */
static inline size_t base32_decode_neon_rfc(const char* src, size_t n, uint8_t* dst){
  size_t i = 0;
  size_t o = 0;
  
  /* Process 16 input chars → 10 output bytes */
  while (i + 16 <= n){
    uint8x16_t in = vld1q_u8((const uint8_t*)&src[i]);
    
    /* Check for special chars (padding, whitespace) */
    uint8x16_t eq_pad = vceqq_u8(in, vdupq_n_u8('='));
    uint8x16_t eq_space = vceqq_u8(in, vdupq_n_u8(' '));
    uint8x16_t eq_nl = vceqq_u8(in, vdupq_n_u8('\n'));
    uint8x16_t skip = vorrq_u8(vorrq_u8(eq_pad, eq_space), eq_nl);
    
    if (vmaxvq_u8(skip) != 0) {
      break;  /* Fall back to scalar */
    }
    
    /* Decode: A-Z → 0-25, 2-7 → 26-31 */
    uint8x16_t ge_A = vcgeq_u8(in, vdupq_n_u8('A'));
    uint8x16_t le_Z = vcleq_u8(in, vdupq_n_u8('Z'));
    uint8x16_t is_upper = vandq_u8(ge_A, le_Z);
    uint8x16_t upper_dec = vsubq_u8(in, vdupq_n_u8('A'));
    
    uint8x16_t ge_2 = vcgeq_u8(in, vdupq_n_u8('2'));
    uint8x16_t le_7 = vcleq_u8(in, vdupq_n_u8('7'));
    uint8x16_t is_digit = vandq_u8(ge_2, le_7);
    uint8x16_t digit_dec = vaddq_u8(vsubq_u8(in, vdupq_n_u8('2')), vdupq_n_u8(26));
    
    /* Combine */
    uint8x16_t decoded = vbslq_u8(is_upper, upper_dec, vdupq_n_u8(0));
    decoded = vbslq_u8(is_digit, digit_dec, decoded);
    
    /* Pack 16×5-bit → 10×8-bit (scalar for now) */
    uint8_t temp[16];
    vst1q_u8(temp, decoded);
    
    /* Pack: 8 chars (40 bits) → 5 bytes */
    if (o + 10 <= n * 5 / 8) {
      uint64_t buf = 0;
      for (int j = 0; j < 8; j++) {
        buf = (buf << 5) | temp[j];
      }
      /* Extract 5 bytes */
      dst[o++] = (buf >> 32) & 0xFF;
      dst[o++] = (buf >> 24) & 0xFF;
      dst[o++] = (buf >> 16) & 0xFF;
      dst[o++] = (buf >> 8) & 0xFF;
      dst[o++] = buf & 0xFF;
      
      /* Second 8 chars → 5 bytes */
      buf = 0;
      for (int j = 8; j < 16; j++) {
        buf = (buf << 5) | temp[j];
      }
      dst[o++] = (buf >> 32) & 0xFF;
      dst[o++] = (buf >> 24) & 0xFF;
      dst[o++] = (buf >> 16) & 0xFF;
      dst[o++] = (buf >> 8) & 0xFF;
      dst[o++] = buf & 0xFF;
    }
    
    i += 16;
  }
  
  return i;
}

#endif /* VEX_BASE32_NEON */

#ifdef VEX_BASE32_AVX2

/* AVX2: Decode 32 chars → 20 bytes */
static inline size_t base32_decode_avx2_rfc(const char* src, size_t n, uint8_t* dst){
  size_t i = 0;
  size_t o = 0;
  
  /* Process 32 input chars → 20 output bytes */
  while (i + 32 <= n){
    __m256i in = _mm256_loadu_si256((const __m256i*)&src[i]);
    
    /* Check for special chars */
    __m256i eq_pad = _mm256_cmpeq_epi8(in, _mm256_set1_epi8('='));
    __m256i eq_space = _mm256_cmpeq_epi8(in, _mm256_set1_epi8(' '));
    __m256i skip = _mm256_or_si256(eq_pad, eq_space);
    
    if (_mm256_movemask_epi8(skip) != 0) {
      break;
    }
    
    /* Decode */
    __m256i ge_A = _mm256_cmpgt_epi8(in, _mm256_set1_epi8('A'-1));
    __m256i le_Z = _mm256_cmpgt_epi8(_mm256_set1_epi8('Z'+1), in);
    __m256i is_upper = _mm256_and_si256(ge_A, le_Z);
    __m256i upper_dec = _mm256_sub_epi8(in, _mm256_set1_epi8('A'));
    
    __m256i ge_2 = _mm256_cmpgt_epi8(in, _mm256_set1_epi8('2'-1));
    __m256i le_7 = _mm256_cmpgt_epi8(_mm256_set1_epi8('7'+1), in);
    __m256i is_digit = _mm256_and_si256(ge_2, le_7);
    __m256i digit_dec = _mm256_add_epi8(_mm256_sub_epi8(in, _mm256_set1_epi8('2')), _mm256_set1_epi8(26));
    
    __m256i decoded = _mm256_blendv_epi8(_mm256_setzero_si256(), upper_dec, is_upper);
    decoded = _mm256_blendv_epi8(decoded, digit_dec, is_digit);
    
    /* Pack (scalar) */
    uint8_t temp[32];
    _mm256_storeu_si256((__m256i*)temp, decoded);
    
    /* Pack 32 chars → 20 bytes (4 groups of 8→5) */
    for (int g = 0; g < 4 && o + 5 <= n * 5 / 8; g++) {
      uint64_t buf = 0;
      for (int j = 0; j < 8; j++) {
        buf = (buf << 5) | temp[g*8 + j];
      }
      dst[o++] = (buf >> 32) & 0xFF;
      dst[o++] = (buf >> 24) & 0xFF;
      dst[o++] = (buf >> 16) & 0xFF;
      dst[o++] = (buf >> 8) & 0xFF;
      dst[o++] = buf & 0xFF;
    }
    
    i += 32;
  }
  
  return i;
}

#endif /* VEX_BASE32_AVX2 */

/* ============================================================================
   SCALAR DECODE (optimized with unrolling)
   ============================================================================ */

static ssize_t base32_decode_scalar(const char* src, size_t n, uint8_t* dst, vex_b32_alphabet alpha, size_t start_i, size_t start_o){
  size_t o = start_o;
  int k = 0;
  uint64_t buf = 0;
  
  for (size_t i = start_i; i < n; i++){
    unsigned char c = (unsigned char)src[i];
    if (c=='\r'||c=='\n'||c=='\t'||c==' ') continue;
    if (c=='=') break;
    
    uint8_t v;
    switch(alpha){
      case VEX_B32_HEX: v=de_hex(c); break;
      case VEX_B32_CROCKFORD: v=de_crock(c); break;
      default: v=de_rfc(c); break;
    }
    
    if (v==0xFF) return -1;
    
    buf = (buf<<5) | v;
    k += 5;
    
    /* Unrolled: extract multiple bytes at once */
    while (k >= 16) {
      k -= 8; dst[o++] = (uint8_t)((buf>>k)&0xFF);
      k -= 8; dst[o++] = (uint8_t)((buf>>k)&0xFF);
    }
    while (k >= 8) {
      k -= 8;
      dst[o++] = (uint8_t)((buf>>k)&0xFF);
    }
  }
  
  return (ssize_t)o;
}

/* ============================================================================
   MAIN DISPATCHER
   ============================================================================ */

ssize_t vex_base32_decode(const char* src, size_t n, uint8_t* dst, vex_b32_alphabet alpha){
  size_t i = 0;
  size_t o = 0;
  
  /* Use SIMD only for RFC alphabet (simplest) */
  if (alpha == VEX_B32_RFC) {
#ifdef VEX_BASE32_AVX2
    if (n >= 32) {
      i = base32_decode_avx2_rfc(src, n, dst);
      o = (i / 8) * 5;
    }
#elif defined(VEX_BASE32_NEON)
    if (n >= 16) {
      i = base32_decode_neon_rfc(src, n, dst);
      o = (i / 8) * 5;
    }
#endif
  }
  
  /* Scalar fallback */
  return base32_decode_scalar(src, n, dst, alpha, i, o);
}
