/* compact SHA-1 (for UUID v5) */
#include "vex_fastenc.h"
#include <string.h>
static uint32_t rol(uint32_t x, int s){ return (x<<s)|(x>>(32-s)); }
static void sha1_core(const uint8_t* d, size_t n, uint32_t h[5]){
  for (size_t i=0;i<n;i+=64){
    uint32_t w[80];
    for (int t=0;t<16;t++){
      w[t] = ((uint32_t)d[i+4*t]<<24)|((uint32_t)d[i+4*t+1]<<16)|((uint32_t)d[i+4*t+2]<<8)|(uint32_t)d[i+4*t+3];
    }
    for (int t=16;t<80;t++) w[t] = rol(w[t-3]^w[t-8]^w[t-14]^w[t-16],1);
    uint32_t a=h[0],b=h[1],c=h[2],d2=h[3],e=h[4];
    for (int t=0;t<80;t++){
      uint32_t f,k;
      if (t<20){ f=(b & c) | ((~b) & d2); k=0x5A827999; }
      else if (t<40){ f=b^c^d2; k=0x6ED9EBA1; }
      else if (t<60){ f=(b & c) | (b & d2) | (c & d2); k=0x8F1BBCDC; }
      else { f=b^c^d2; k=0xCA62C1D6; }
      uint32_t temp = rol(a,5) + f + e + k + w[t];
      e=d2; d2=c; c=rol(b,30); b=a; a=temp;
    }
    h[0]+=a; h[1]+=b; h[2]+=c; h[3]+=d2; h[4]+=e;
  }
}
void vex_sha1(const void* data, size_t len, uint8_t out20[20]){
  uint64_t bitlen = (uint64_t)len * 8;
  size_t nfull = len & ~((size_t)63);
  uint32_t h[5]={0x67452301,0xEFCDAB89,0x98BADCFE,0x10325476,0xC3D2E1F0};
  sha1_core((const uint8_t*)data, nfull, h);
  uint8_t tail[128]; size_t rem=len-nfull;
  memcpy(tail, (const uint8_t*)data + nfull, rem);
  tail[rem++] = 0x80;
  size_t pad = (rem%64<=56) ? (56 - rem%64) : (56 + 64 - rem%64);
  memset(tail+rem, 0, pad); rem += pad;
  for (int i=0;i<8;i++) tail[rem++] = (uint8_t)((bitlen>>(56-8*i))&0xFF);
  sha1_core(tail, rem, h);
  for (int i=0;i<5;i++){
    out20[4*i+0] = (uint8_t)((h[i]>>24)&0xFF);
    out20[4*i+1] = (uint8_t)((h[i]>>16)&0xFF);
    out20[4*i+2] = (uint8_t)((h[i]>>8 )&0xFF);
    out20[4*i+3] = (uint8_t)((h[i]    )&0xFF);
  }
}
