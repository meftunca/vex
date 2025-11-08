#include "vex_fastenc.h"
#include <string.h>
#include <ctype.h>

static const char B32_RFC[] = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567"; /* RFC 4648 */
static const char B32_HEX[] = "0123456789ABCDEFGHIJKLMNOPQRSTUV"; /* Base32hex */
static const char B32_CRK[] = "0123456789ABCDEFGHJKMNPQRSTVWXYZ"; /* Crockford (no I L O U) */

size_t vex_base32_max_decoded_len(size_t n){ return (n*5)/8 + 8; }

size_t vex_base32_encoded_len(size_t n, vex_b32_cfg cfg){
  size_t full = (n/5)*8;
  size_t rem  = n%5;
  size_t out  = full;
  if (rem){
    if (cfg.pad){
      static const int P[5]={0,6,4,3,1}; /* how many '=' to add for rem=1..4 */
      out += 8;
    } else {
      static const int O[5]={0,2,4,5,7}; /* how many chars for rem bytes */
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

size_t vex_base32_encode(const uint8_t* src, size_t n, char* dst, vex_b32_cfg cfg){
  const char* ABC = alpha_str(cfg.alpha);
  size_t i=0, o=0;
  while (i+5<=n){
    uint64_t v = ((uint64_t)src[i]<<32) | ((uint64_t)src[i+1]<<24) | ((uint64_t)src[i+2]<<16) |
                 ((uint64_t)src[i+3]<<8) | (uint64_t)src[i+4];
    dst[o++] = ABC[(v>>35)&31];
    dst[o++] = ABC[(v>>30)&31];
    dst[o++] = ABC[(v>>25)&31];
    dst[o++] = ABC[(v>>20)&31];
    dst[o++] = ABC[(v>>15)&31];
    dst[o++] = ABC[(v>>10)&31];
    dst[o++] = ABC[(v>>5 )&31];
    dst[o++] = ABC[(v    )&31];
    i+=5;
  }
  size_t rem = n-i;
  if (rem){
    uint64_t v=0; for (size_t k=0;k<rem;k++) v |= (uint64_t)src[i+k] << (8*(4-k));
    int out_chars = (int)((rem*8 + 4)/5); /* ceil(rem*8/5) */
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

ssize_t vex_base32_decode(const char* src, size_t n, uint8_t* dst, vex_b32_alphabet alpha){
  size_t o=0; int k=0; uint64_t buf=0;
  for (size_t i=0;i<n;i++){
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
    buf = (buf<<5) | v; k += 5;
    while (k>=8){
      k-=8; dst[o++] = (uint8_t)((buf>>k)&0xFF);
    }
  }
  return (ssize_t)o;
}
