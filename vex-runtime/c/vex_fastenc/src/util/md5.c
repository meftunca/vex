/* public-domain style small MD5 (for UUID v3) */
#include "vex_fastenc.h"
#include <string.h>
static uint32_t rol(uint32_t x, int s){ return (x<<s)|(x>>(32-s)); }
static void md5_core(const uint8_t* data, size_t n, uint32_t h[4]){
  static const uint32_t K[64]={
    0xd76aa478,0xe8c7b756,0x242070db,0xc1bdceee,0xf57c0faf,0x4787c62a,0xa8304613,0xfd469501,
    0x698098d8,0x8b44f7af,0xffff5bb1,0x895cd7be,0x6b901122,0xfd987193,0xa679438e,0x49b40821,
    0xf61e2562,0xc040b340,0x265e5a51,0xe9b6c7aa,0xd62f105d,0x02441453,0xd8a1e681,0xe7d3fbc8,
    0x21e1cde6,0xc33707d6,0xf4d50d87,0x455a14ed,0xa9e3e905,0xfcefa3f8,0x676f02d9,0x8d2a4c8a,
    0xfffa3942,0x8771f681,0x6d9d6122,0xfde5380c,0xa4beea44,0x4bdecfa9,0xf6bb4b60,0xbebfbc70,
    0x289b7ec6,0xeaa127fa,0xd4ef3085,0x04881d05,0xd9d4d039,0xe6db99e5,0x1fa27cf8,0xc4ac5665,
    0xf4292244,0x432aff97,0xab9423a7,0xfc93a039,0x655b59c3,0x8f0ccc92,0xffeff47d,0x85845dd1,
    0x6fa87e4f,0xfe2ce6e0,0xa3014314,0x4e0811a1,0xf7537e82,0xbd3af235,0x2ad7d2bb,0xeb86d391};
  static const int S[64]={7,12,17,22, 5,9,14,20, 4,11,16,23, 6,10,15,21};
  for (size_t i=0;i<n;i+=64){
    uint32_t a=h[0],b=h[1],c=h[2],d=h[3];
    uint32_t X[16]; for (int j=0;j<16;j++){
      X[j] = (uint32_t)data[i+4*j] | ((uint32_t)data[i+4*j+1]<<8) | ((uint32_t)data[i+4*j+2]<<16) | ((uint32_t)data[i+4*j+3]<<24);
    }
    for (int t=0;t<64;t++){
      uint32_t F,g;
      if (t<16){ F=(b&c)|((~b)&d); g=t; }
      else if (t<32){ F=(d&b)|((~d)&c); g=(5*t+1)&15; }
      else if (t<48){ F=b^c^d; g=(3*t+5)&15; }
      else { F=c^(b|(~d)); g=(7*t)&15; }
      uint32_t tmp = d;
      d=c; c=b;
      uint32_t add = a + F + K[t] + X[g];
      b = b + rol(add, S[(t/16)*4 + (t%4)]);
      a=tmp;
    }
    h[0]+=a; h[1]+=b; h[2]+=c; h[3]+=d;
  }
}
void vex_md5(const void* data, size_t len, uint8_t out16[16]){
  uint64_t bitlen = (uint64_t)len * 8;
  size_t nfull = len & ~((size_t)63);
  uint32_t h[4]={0x67452301,0xefcdab89,0x98badcfe,0x10325476};
  md5_core((const uint8_t*)data, nfull, h);
  uint8_t tail[128]; size_t rem=len-nfull;
  memcpy(tail, (const uint8_t*)data + nfull, rem);
  tail[rem++] = 0x80;
  size_t pad = (rem%64<=56) ? (56 - rem%64) : (56 + 64 - rem%64);
  memset(tail+rem, 0, pad); rem += pad;
  for (int i=0;i<8;i++) tail[rem++] = (uint8_t)((bitlen>>(8*i))&0xFF);
  md5_core(tail, rem, h);
  for (int i=0;i<4;i++){
    out16[4*i+0] = (uint8_t)(h[i] & 0xFF);
    out16[4*i+1] = (uint8_t)((h[i]>>8) & 0xFF);
    out16[4*i+2] = (uint8_t)((h[i]>>16)& 0xFF);
    out16[4*i+3] = (uint8_t)((h[i]>>24)& 0xFF);
  }
}
