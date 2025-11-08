#include "vex_fastenc.h"
#if defined(__aarch64__) || defined(__ARM_NEON) || defined(__ARM_NEON__)
#include <arm_neon.h>
#include <string.h>

static inline void hex16_neon(const uint8_t* src, char* dst, int uppercase){
  uint8x16_t v = vld1q_u8(src);
  uint8x16_t hi = vshrq_n_u8(v,4);
  uint8x16_t lo = vandq_u8(v, vdupq_n_u8(0x0F));
  uint8x16_t base = vdupq_n_u8('0');
  uint8x16_t adj  = vdupq_n_u8(uppercase?7:39);

  uint8x16_t hi_cmp = vcgtq_u8(hi, vdupq_n_u8(9));
  uint8x16_t lo_cmp = vcgtq_u8(lo, vdupq_n_u8(9));

  uint8x16_t ahi = vaddq_u8(hi, base);
  uint8x16_t alo = vaddq_u8(lo, base);
  ahi = vaddq_u8(ahi, vandq_u8(hi_cmp, adj));
  alo = vaddq_u8(alo, vandq_u8(lo_cmp, adj));

  /* interleave */
  uint8x16x2_t zip0 = vzipq_u8(ahi, alo); /* [h0,l0,h1,l1,...] in two registers: lo and hi halves */
  vst1q_u8((uint8_t*)dst,       zip0.val[0]);
  vst1q_u8((uint8_t*)(dst+16),  zip0.val[1]);
}

size_t vex_hex_encode(const uint8_t* src, size_t n, char* dst, int uppercase){
  size_t i=0, o=0;
  if (vex_cpu_has_neon()){
    for (; i+16<=n; i+=16){
      hex16_neon(src+i, dst+o, uppercase);
      o += 32;
    }
  }
  for (; i<n; i++){
    uint8_t b=src[i];
    uint8_t hi=(b>>4)&0xF, lo=b&0xF;
    dst[o++] = (char)(hi<10? '0'+hi : (uppercase? 'A'+(hi-10) : 'a'+(hi-10)));
    dst[o++] = (char)(lo<10? '0'+lo : (uppercase? 'A'+(lo-10) : 'a'+(lo-10)));
  }
  return o;
}
#endif
