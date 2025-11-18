#include "vex_fastenc.h"
#if (defined(__x86_64__) || defined(_M_X64)) && defined(__AVX512BW__)
#include <immintrin.h>

static inline void hex64_avx512(const uint8_t* src, char* dst, int uppercase){
  __m512i v = _mm512_loadu_si512((const void*)src);
  __m512i hi = _mm512_and_si512(_mm512_srli_epi16(v,4), _mm512_set1_epi8(0x0F));
  __m512i lo = _mm512_and_si512(v, _mm512_set1_epi8(0x0F));

  __m512i bias = _mm512_set1_epi8('0');
  __m512i t    = _mm512_set1_epi8(uppercase?7:39);

  __mmask64 mhi = _mm512_cmpgt_epi8_mask(hi, _mm512_set1_epi8(9));
  __mmask64 mlo = _mm512_cmpgt_epi8_mask(lo, _mm512_set1_epi8(9));

  __m512i ahi = _mm512_add_epi8(hi, bias);
  __m512i alo = _mm512_add_epi8(lo, bias);
  ahi = _mm512_mask_add_epi8(ahi, mhi, ahi, t);
  alo = _mm512_mask_add_epi8(alo, mlo, alo, t);

  __m512i p0 = _mm512_unpacklo_epi8(ahi, alo);
  __m512i p1 = _mm512_unpackhi_epi8(ahi, alo);
  _mm512_storeu_si512((void*)(dst), p0);
  _mm512_storeu_si512((void*)(dst+64), p1);
}

size_t vex_hex_encode(const uint8_t* src, size_t n, char* dst, int uppercase){
  size_t i=0, o=0;
  if (vex_cpu_has_avx512bw()){
    for (; i+64<=n; i+=64){
      hex64_avx512(src+i, dst+o, uppercase);
      o += 128;
    }
  }
  /* tail handled by AVX2 file or scalar; ensure we don't duplicate logic here */
  for (; i<n; i++){
    uint8_t b=src[i];
    uint8_t hi=(b>>4)&0xF, lo=b&0xF;
    dst[o++] = (char)(hi<10? '0'+hi : (uppercase? 'A'+(hi-10) : 'a'+(hi-10)));
    dst[o++] = (char)(lo<10? '0'+lo : (uppercase? 'A'+(lo-10) : 'a'+(lo-10)));
  }
  return o;
}
#endif
