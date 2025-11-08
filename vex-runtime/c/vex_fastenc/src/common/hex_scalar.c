#include "vex_fastenc.h"
#include <string.h>
#include <ctype.h>

size_t vex_hex_encoded_len(size_t nbytes){ return nbytes*2; }
size_t vex_hex_decoded_len(size_t nchars){ return nchars/2; }

static inline char hex_digit_low(int v){ return (v<10)?(char)('0'+v):(char)('a'+(v-10)); }
static inline char hex_digit_up (int v){ return (v<10)?(char)('0'+v):(char)('A'+(v-10)); }

size_t vex_hex_encode(const uint8_t* src, size_t n, char* dst, int uppercase){
  const char* hexd = uppercase? "0123456789ABCDEF" : "0123456789abcdef";
  for (size_t i=0;i<n;i++){
    uint8_t b=src[i];
    dst[2*i+0] = hexd[(b>>4)&0xF];
    dst[2*i+1] = hexd[(b    )&0xF];
  }
  return n*2;
}

static inline int hex_val(int c){
  if (c>='0'&&c<='9') return c-'0';
  if (c>='a'&&c<='f') return c-'a'+10;
  if (c>='A'&&c<='F') return c-'A'+10;
  return -1;
}

ssize_t vex_hex_decode(const char* src, size_t n, uint8_t* dst){
  if (n%2) return -1;
  for (size_t i=0;i<n; i+=2){
    int hi = hex_val((unsigned char)src[i]);
    int lo = hex_val((unsigned char)src[i+1]);
    if (hi<0 || lo<0) return -1;
    dst[i/2] = (uint8_t)((hi<<4)|lo);
  }
  return (ssize_t)(n/2);
}
