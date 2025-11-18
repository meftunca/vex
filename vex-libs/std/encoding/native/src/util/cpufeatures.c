#include "vex_fastenc.h"

#if defined(__x86_64__) || defined(_M_X64) || defined(__i386) || defined(_M_IX86)
#if defined(_MSC_VER)
#include <intrin.h>
static void cpuidex(int regs[4], int leaf, int subleaf){ __cpuidex(regs, leaf, subleaf); }
#else
#include <cpuid.h>
static void cpuidex(int regs[4], int leaf, int subleaf){
  unsigned int a,b,c,d;
  __cpuid_count(leaf, subleaf, a,b,c,d);
  regs[0]=(int)a; regs[1]=(int)b; regs[2]=(int)c; regs[3]=(int)d;
}
#endif

static int os_avx_support(){
#if defined(_MSC_VER)
  return ( ( __cpuidex, 1 ), ( (unsigned long long)_xgetbv(0) & 0x6 ) == 0x6 );
#elif defined(__GNUC__) || defined(__clang__)
  unsigned int a,b,c,d; (void)a;(void)b;(void)c;(void)d;
  int regs[4]; cpuidex(regs,1,0);
  unsigned int osxsave = (regs[2] >> 27) & 1;
  if (!osxsave) return 0;
  unsigned int xcr0_lo=0, xcr0_hi=0;
  __asm__ volatile ("xgetbv" : "=a"(xcr0_lo), "=d"(xcr0_hi) : "c"(0) );
  unsigned long long xcr0 = ((unsigned long long)xcr0_hi<<32) | xcr0_lo;
  return ( (xcr0 & 0x6) == 0x6 );
#else
  return 0;
#endif
}

int vex_cpu_has_avx2(void){
  int r[4]; cpuidex(r,7,0);
  int avx2 = (r[1] >> 5) & 1; /* EBX bit 5 */
  return avx2 && os_avx_support();
}
int vex_cpu_has_avx512bw(void){
  int r[4]; cpuidex(r,7,0);
  int avx512f  = (r[1] >> 16) & 1; /* EBX bit 16 */
  int avx512bw = (r[1] >> 30) & 1; /* EBX bit 30 */
  /* require OS support for ZMM state */
#if defined(__GNUC__) || defined(__clang__)
  unsigned int xcr0_lo=0, xcr0_hi=0;
  __asm__ volatile ("xgetbv" : "=a"(xcr0_lo), "=d"(xcr0_hi) : "c"(0) );
  unsigned long long xcr0 = ((unsigned long long)xcr0_hi<<32) | xcr0_lo;
  int zmm = ((xcr0 & 0xE0) == 0xE0); /* Opmask|ZMM_Hi256|Hi16_ZMM */
  return avx512f && avx512bw && zmm;
#else
  return avx512f && avx512bw;
#endif
}

#elif defined(__aarch64__) || defined(__arm__)
#if defined(__APPLE__)
#include <TargetConditionals.h>
#endif
#include <stdlib.h>
#if defined(__linux__)
#include <sys/auxv.h>
#include <asm/hwcap.h>
#endif
int vex_cpu_has_avx2(void){ return 0; }
int vex_cpu_has_avx512bw(void){ return 0; }
int vex_cpu_has_neon(void){
#if defined(__aarch64__)
  return 1; /* NEON mandatory */
#elif defined(__ARM_NEON) || defined(__ARM_NEON__)
  return 1;
#elif defined(__linux__)
  unsigned long hw = getauxval(AT_HWCAP);
  return (hw & HWCAP_NEON) != 0;
#else
  return 0;
#endif
}
#else
int vex_cpu_has_avx2(void){ return 0; }
int vex_cpu_has_avx512bw(void){ return 0; }
int vex_cpu_has_neon(void){ return 0; }
#endif
