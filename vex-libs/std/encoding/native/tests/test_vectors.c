#include "vex_fastenc.h"
#include <stdio.h>
#include <string.h>

static int test_hex(){
  const char* s="hello";
  char enc[64]; uint8_t dec[64];
  size_t n = vex_hex_encode((const uint8_t*)s, 5, enc, 0);
  if (strncmp(enc, "68656c6c6f", n)!=0){ printf("hex enc fail: %.*s\n",(int)n,enc); return -1; }
  ssize_t m = vex_hex_decode(enc,n,dec);
  if (m!=5 || memcmp(dec,s,5)!=0){ printf("hex dec fail\n"); return -1; }
  return 0;
}

static int test_b64(){
  const char* s="foobar";
  char enc[64]; vex_b64_cfg cfg={VEX_B64_STD,1,0};
  size_t n = vex_base64_encode((const uint8_t*)s, 6, enc, cfg); enc[n]=0;
  if (strcmp(enc,"Zm9vYmFy")==0){} else { printf("b64 enc fail: %s\n",enc); return -1; }
  uint8_t dec[64]; ssize_t m = vex_base64_decode(enc,n,dec,VEX_B64_STD);
  if (m!=6 || memcmp(dec,s,6)!=0){ printf("b64 dec fail\n"); return -1; }
  return 0;
}
static int test_b32(){
  const uint8_t in[]={0x66,0x6f,0x6f,0x62,0x61,0x72};
  char enc[64]; vex_b32_cfg cfg={VEX_B32_RFC,1};
  size_t n = vex_base32_encode(in, sizeof in, enc, cfg); enc[n]=0;
  if (strcmp(enc,"MZXW6YTBOI======")!=0){ printf("b32 enc fail: %s\n",enc); return -1; }
  uint8_t dec[64]; ssize_t m = vex_base32_decode(enc,n,dec,VEX_B32_RFC);
  if (m!=6 || memcmp(dec,in,6)!=0){ printf("b32 dec fail\n"); return -1; }
  return 0;
}
static int test_uuid(){
  vex_uuid u; char s[37];
  vex_uuid_v4(&u); vex_uuid_format(s,&u);
  printf("uuid4: %s\n", s);
  vex_uuid out; if (vex_uuid_parse(s,&out)!=0 || memcmp(&u,&out,sizeof u)!=0){ printf("uuid parse/format fail\n"); return -1; }
  return 0;
}

int main(void){
  if (test_hex()!=0) return 1;
  if (test_b64()!=0) return 1;
  if (test_b32()!=0) return 1;
  if (test_uuid()!=0) return 1;
  printf("OK\n");
  return 0;
}
