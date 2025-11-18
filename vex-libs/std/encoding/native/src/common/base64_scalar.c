/**
 * Base64 Encoding/Decoding with Multi-Platform SIMD
 * 
 * Supports:
 * - x86-64: AVX2, SSSE3
 * - ARM64: NEON
 * - Fallback: Optimized scalar
 * 
 * Algorithm: Process chunks of input with SIMD classification and packing
 */

#include "vex_fastenc.h"
#include <string.h>

/* ============================================================================
   SIMD Platform Detection
   ============================================================================ */

#if defined(__aarch64__) || defined(_M_ARM64)
  #if defined(__ARM_NEON) || defined(__ARM_NEON__)
    #define VEX_BASE64_NEON 1
    #include <arm_neon.h>
  #endif
#elif defined(__x86_64__) || defined(_M_X64)
  #if defined(__AVX2__)
    #define VEX_BASE64_AVX2 1
    #include <immintrin.h>
  #elif defined(__SSSE3__)
    #define VEX_BASE64_SSSE3 1
    #include <tmmintrin.h>
  #endif
#endif

/* ============================================================================
   LOOKUP TABLES
   ============================================================================ */

static const char B64_STD[] =
  "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
static const char B64_URL[] =
  "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/* ============================================================================
   SIMD BASE64 DECODE
   ============================================================================ */

#ifdef VEX_BASE64_AVX2

/* AVX2 Base64 Decode (process 32 bytes → 24 bytes) */
static inline size_t base64_decode_avx2(const char* src, size_t n, uint8_t* dst) {
    size_t i = 0;
    size_t o = 0;
    
    /* Process 32 input chars → 24 output bytes */
    while (i + 32 <= n) {
        __m256i in = _mm256_loadu_si256((const __m256i*)&src[i]);
        
        /* Check for special chars (padding, whitespace) */
        __m256i eq_pad = _mm256_cmpeq_epi8(in, _mm256_set1_epi8('='));
        __m256i eq_space = _mm256_cmpeq_epi8(in, _mm256_set1_epi8(' '));
        __m256i eq_nl = _mm256_cmpeq_epi8(in, _mm256_set1_epi8('\n'));
        __m256i skip = _mm256_or_si256(_mm256_or_si256(eq_pad, eq_space), eq_nl);
        
        if (_mm256_movemask_epi8(skip) != 0) {
            break;  /* Fall back to scalar */
        }
        
        /* Decode characters using range checks */
        /* A-Z: 65-90 → 0-25 */
        __m256i ge_A = _mm256_cmpgt_epi8(in, _mm256_set1_epi8('A'-1));
        __m256i le_Z = _mm256_cmpgt_epi8(_mm256_set1_epi8('Z'+1), in);
        __m256i is_upper = _mm256_and_si256(ge_A, le_Z);
        __m256i upper_dec = _mm256_sub_epi8(in, _mm256_set1_epi8('A'));
        
        /* a-z: 97-122 → 26-51 */
        __m256i ge_a = _mm256_cmpgt_epi8(in, _mm256_set1_epi8('a'-1));
        __m256i le_z = _mm256_cmpgt_epi8(_mm256_set1_epi8('z'+1), in);
        __m256i is_lower = _mm256_and_si256(ge_a, le_z);
        __m256i lower_dec = _mm256_add_epi8(_mm256_sub_epi8(in, _mm256_set1_epi8('a')), _mm256_set1_epi8(26));
        
        /* 0-9: 48-57 → 52-61 */
        __m256i ge_0 = _mm256_cmpgt_epi8(in, _mm256_set1_epi8('0'-1));
        __m256i le_9 = _mm256_cmpgt_epi8(_mm256_set1_epi8('9'+1), in);
        __m256i is_digit = _mm256_and_si256(ge_0, le_9);
        __m256i digit_dec = _mm256_add_epi8(_mm256_sub_epi8(in, _mm256_set1_epi8('0')), _mm256_set1_epi8(52));
        
        /* +: 43 → 62 */
        __m256i is_plus = _mm256_cmpeq_epi8(in, _mm256_set1_epi8('+'));
        __m256i plus_dec = _mm256_set1_epi8(62);
        
        /* /: 47 → 63 */
        __m256i is_slash = _mm256_cmpeq_epi8(in, _mm256_set1_epi8('/'));
        __m256i slash_dec = _mm256_set1_epi8(63);
        
        /* Combine all ranges */
        __m256i decoded = _mm256_blendv_epi8(_mm256_setzero_si256(), upper_dec, is_upper);
        decoded = _mm256_blendv_epi8(decoded, lower_dec, is_lower);
        decoded = _mm256_blendv_epi8(decoded, digit_dec, is_digit);
        decoded = _mm256_blendv_epi8(decoded, plus_dec, is_plus);
        decoded = _mm256_blendv_epi8(decoded, slash_dec, is_slash);
        
        /* Pack 32×6-bit → 24×8-bit (scalar for now) */
        uint8_t temp[32];
        _mm256_storeu_si256((__m256i*)temp, decoded);
        
        /* Pack every 4 6-bit values into 3 bytes */
        for (int j = 0; j + 4 <= 32 && o + 3 <= n * 3 / 4; j += 4) {
            uint32_t v = ((uint32_t)temp[j] << 18) | ((uint32_t)temp[j+1] << 12) |
                         ((uint32_t)temp[j+2] << 6) | ((uint32_t)temp[j+3]);
            dst[o++] = (v >> 16) & 0xFF;
            dst[o++] = (v >> 8) & 0xFF;
            dst[o++] = v & 0xFF;
        }
        
        i += 32;
    }
    
    return i;
}

#endif /* VEX_BASE64_AVX2 */

#ifdef VEX_BASE64_NEON

/* NEON Base64 Decode (process 16 bytes) */
static inline size_t base64_decode_neon(const char* src, size_t n, uint8_t* dst) {
    size_t i = 0;
    size_t o = 0;
    
    while (i + 16 <= n) {
        uint8x16_t in = vld1q_u8((const uint8_t*)&src[i]);
        
        /* Check for special chars */
        uint8x16_t eq_pad = vceqq_u8(in, vdupq_n_u8('='));
        uint8x16_t eq_space = vceqq_u8(in, vdupq_n_u8(' '));
        uint8x16_t eq_nl = vceqq_u8(in, vdupq_n_u8('\n'));
        uint8x16_t skip = vorrq_u8(vorrq_u8(eq_pad, eq_space), eq_nl);
        
        if (vmaxvq_u8(skip) != 0) {
            break;
        }
        
        /* Decode using range checks */
        uint8x16_t ge_A = vcgeq_u8(in, vdupq_n_u8('A'));
        uint8x16_t le_Z = vcleq_u8(in, vdupq_n_u8('Z'));
        uint8x16_t is_upper = vandq_u8(ge_A, le_Z);
        uint8x16_t upper_dec = vsubq_u8(in, vdupq_n_u8('A'));
        
        uint8x16_t ge_a = vcgeq_u8(in, vdupq_n_u8('a'));
        uint8x16_t le_z = vcleq_u8(in, vdupq_n_u8('z'));
        uint8x16_t is_lower = vandq_u8(ge_a, le_z);
        uint8x16_t lower_dec = vaddq_u8(vsubq_u8(in, vdupq_n_u8('a')), vdupq_n_u8(26));
        
        uint8x16_t ge_0 = vcgeq_u8(in, vdupq_n_u8('0'));
        uint8x16_t le_9 = vcleq_u8(in, vdupq_n_u8('9'));
        uint8x16_t is_digit = vandq_u8(ge_0, le_9);
        uint8x16_t digit_dec = vaddq_u8(vsubq_u8(in, vdupq_n_u8('0')), vdupq_n_u8(52));
        
        uint8x16_t is_plus = vceqq_u8(in, vdupq_n_u8('+'));
        uint8x16_t plus_dec = vdupq_n_u8(62);
        
        uint8x16_t is_slash = vceqq_u8(in, vdupq_n_u8('/'));
        uint8x16_t slash_dec = vdupq_n_u8(63);
        
        /* Combine */
        uint8x16_t decoded = vbslq_u8(is_upper, upper_dec, vdupq_n_u8(0));
        decoded = vbslq_u8(is_lower, lower_dec, decoded);
        decoded = vbslq_u8(is_digit, digit_dec, decoded);
        decoded = vbslq_u8(is_plus, plus_dec, decoded);
        decoded = vbslq_u8(is_slash, slash_dec, decoded);
        
        /* Pack (scalar) */
        uint8_t temp[16];
        vst1q_u8(temp, decoded);
        
        for (int j = 0; j + 4 <= 16 && o + 3 <= n * 3 / 4; j += 4) {
            uint32_t v = ((uint32_t)temp[j] << 18) | ((uint32_t)temp[j+1] << 12) |
                         ((uint32_t)temp[j+2] << 6) | ((uint32_t)temp[j+3]);
            dst[o++] = (v >> 16) & 0xFF;
            dst[o++] = (v >> 8) & 0xFF;
            dst[o++] = v & 0xFF;
        }
        
        i += 16;
    }
    
    return i;
}

#endif /* VEX_BASE64_NEON */

/* ============================================================================
   SCALAR IMPLEMENTATION
   ============================================================================ */

size_t vex_base64_max_decoded_len(size_t n){ return (n/4)*3 + 3; }

size_t vex_base64_encoded_len(size_t n, vex_b64_cfg cfg){
  size_t full = (n/3)*4;
  size_t rem  = n%3;
  size_t out  = full + (rem? (cfg.pad?4: (rem==1?2:3)) : 0);
  if (cfg.wrap>0){
    size_t lines = out / cfg.wrap;
    size_t extra = lines;
    if (out % cfg.wrap == 0 && lines>0) extra -= 1;
    out += extra;
  }
  return out;
}

size_t vex_base64_encode(const uint8_t* src, size_t n, char* dst, vex_b64_cfg cfg){
  const char* ABC = (cfg.alpha==VEX_B64_URLSAFE)? B64_URL : B64_STD;
  size_t i=0, o=0, col=0, wrap = (cfg.wrap>0)? (size_t)cfg.wrap : (size_t)-1;
  while (i+3<=n){
    uint32_t v = (uint32_t)src[i]<<16 | (uint32_t)src[i+1]<<8 | (uint32_t)src[i+2];
    dst[o++] = ABC[(v>>18)&0x3F];
    dst[o++] = ABC[(v>>12)&0x3F];
    dst[o++] = ABC[(v>>6 )&0x3F];
    dst[o++] = ABC[(v    )&0x3F];
    i+=3; col+=4;
    if (col>=wrap){ dst[o++]='\n'; col=0; }
  }
  size_t rem = n - i;
  if (rem==1){
    uint32_t v = (uint32_t)src[i]<<16;
    dst[o++] = ABC[(v>>18)&0x3F];
    dst[o++] = ABC[(v>>12)&0x3F];
    if (cfg.pad){ dst[o++]='='; dst[o++]='='; }
  } else if (rem==2){
    uint32_t v = (uint32_t)src[i]<<16 | (uint32_t)src[i+1]<<8;
    dst[o++] = ABC[(v>>18)&0x3F];
    dst[o++] = ABC[(v>>12)&0x3F];
    dst[o++] = ABC[(v>>6 )&0x3F];
    if (cfg.pad){ dst[o++]='='; }
  }
  return o;
}

static inline uint8_t dtab_std(int c){
  if (c>='A'&&c<='Z') return (uint8_t)(c-'A');
  if (c>='a'&&c<='z') return (uint8_t)(c-'a'+26);
  if (c>='0'&&c<='9') return (uint8_t)(c-'0'+52);
  if (c=='+') return 62; if (c=='/') return 63;
  return 0xFF;
}
static inline uint8_t dtab_url(int c){
  if (c>='A'&&c<='Z') return (uint8_t)(c-'A');
  if (c>='a'&&c<='z') return (uint8_t)(c-'a'+26);
  if (c>='0'&&c<='9') return (uint8_t)(c-'0'+52);
  if (c=='-') return 62; if (c=='_') return 63;
  return 0xFF;
}

/* Scalar decode */
static ssize_t base64_decode_scalar(const char* src, size_t n, uint8_t* dst, vex_b64_alphabet alpha, size_t start_i, size_t start_o) {
  size_t o = start_o;
  uint32_t buf=0; int k=0;
  for (size_t i=start_i; i<n; i++){
    unsigned char c = (unsigned char)src[i];
    if (c=='\r'||c=='\n'||c=='\t'||c==' ') continue;
    if (c=='='){ 
      if (k==2){ dst[o++] = (uint8_t)((buf>>4)&0xFF); }
      else if (k==3){ dst[o++] = (uint8_t)((buf>>10)&0xFF); dst[o++] = (uint8_t)((buf>>2)&0xFF); }
      break;
    }
    uint8_t v = (alpha==VEX_B64_URLSAFE)? dtab_url(c) : dtab_std(c);
    if (v==0xFF) return -1;
    buf = (buf<<6) | v; k++;
    if (k==4){
      dst[o++] = (uint8_t)((buf>>16)&0xFF);
      dst[o++] = (uint8_t)((buf>>8 )&0xFF);
      dst[o++] = (uint8_t)((buf    )&0xFF);
      buf=0; k=0;
    }
  }
  return (ssize_t)o;
}

/* Main dispatcher */
ssize_t vex_base64_decode(const char* src, size_t n, uint8_t* dst, vex_b64_alphabet alpha){
  size_t i = 0;
  size_t o = 0;
  
  /* Use SIMD for standard alphabet only (URL-safe needs different lookup) */
  if (alpha == VEX_B64_STD) {
#ifdef VEX_BASE64_AVX2
    if (n >= 32) {
      i = base64_decode_avx2(src, n, dst);
      o = (i / 4) * 3;
    }
#elif defined(VEX_BASE64_NEON)
    if (n >= 16) {
      i = base64_decode_neon(src, n, dst);
      o = (i / 4) * 3;
    }
#endif
  }
  
  /* Scalar fallback */
  return base64_decode_scalar(src, n, dst, alpha, i, o);
}
