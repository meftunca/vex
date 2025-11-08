#include "vex_fastenc.h"
#include <string.h>

static const char B64_STD[] =
  "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
static const char B64_URL[] =
  "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

size_t vex_base64_max_decoded_len(size_t n){ return (n/4)*3 + 3; }

size_t vex_base64_encoded_len(size_t n, vex_b64_cfg cfg){
  size_t full = (n/3)*4;
  size_t rem  = n%3;
  size_t out  = full + (rem? (cfg.pad?4: (rem==1?2:3)) : 0);
  if (cfg.wrap>0){
    size_t lines = out / cfg.wrap;
    size_t extra = lines; /* '\n' per line */
    if (out % cfg.wrap == 0 && lines>0) extra -= 1; /* no trailing newline */
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
    else { /* unpadded */ }
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

/* tolerant decoder: ignores whitespace; stops at first invalid */
ssize_t vex_base64_decode(const char* src, size_t n, uint8_t* dst, vex_b64_alphabet alpha){
  size_t o=0; uint32_t buf=0; int k=0;
  for (size_t i=0;i<n;i++){
    unsigned char c = (unsigned char)src[i];
    if (c=='\r'||c=='\n'||c=='\t'||c==' ') continue;
    if (c=='='){ /* flush */ 
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
