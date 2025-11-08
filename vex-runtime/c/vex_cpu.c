// vex_cpu.c - CPU feature detection and SIMD capability query
#include "vex.h"
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>

#if defined(__x86_64__) || defined(_M_X64) || defined(__i386__) || defined(_M_IX86)
#define VEX_ARCH_X86 1
#if defined(_MSC_VER)
#include <intrin.h>
#else
#include <cpuid.h>
#endif
#else
#define VEX_ARCH_X86 0
#endif

#if defined(__aarch64__) || defined(__ARM_NEON) || defined(__ARM_NEON__)
#define VEX_ARCH_ARM 1
#if defined(__linux__)
#include <sys/auxv.h>
#include <asm/hwcap.h>
#endif
#else
#define VEX_ARCH_ARM 0
#endif

// ============================================================================
// CPU FEATURES STRUCTURE (internal)
// ============================================================================

static struct
{
  // x86/x64 features
  bool sse;
  bool sse2;
  bool sse3;
  bool ssse3;
  bool sse4_1;
  bool sse4_2;
  bool avx;
  bool avx2;
  bool avx512f;
  bool avx512bw;
  bool avx512vl;
  bool fma;
  bool bmi1;
  bool bmi2;
  bool popcnt;
  bool aes;

  // ARM features
  bool neon;
  bool sve;
  bool sve2;

  // General info
  const char *vendor;
  const char *brand;
  int cores;
  int logical_processors;
} g_cpu_features = {0};
static bool g_cpu_features_initialized = false;

// ============================================================================
// X86/X64 CPUID
// ============================================================================

#if VEX_ARCH_X86

static void cpuid(uint32_t func, uint32_t subfunc, uint32_t *eax, uint32_t *ebx, uint32_t *ecx, uint32_t *edx)
{
#if defined(_MSC_VER)
  int regs[4];
  __cpuidex(regs, func, subfunc);
  *eax = regs[0];
  *ebx = regs[1];
  *ecx = regs[2];
  *edx = regs[3];
#else
  __cpuid_count(func, subfunc, *eax, *ebx, *ecx, *edx);
#endif
}

static void detect_x86_features()
{
  uint32_t eax, ebx, ecx, edx;

  // Get vendor
  cpuid(0, 0, &eax, &ebx, &ecx, &edx);
  static char vendor_str[13] = {0};
  *(uint32_t *)(vendor_str + 0) = ebx;
  *(uint32_t *)(vendor_str + 4) = edx;
  *(uint32_t *)(vendor_str + 8) = ecx;
  g_cpu_features.vendor = vendor_str;

  // Feature flags (function 1)
  cpuid(1, 0, &eax, &ebx, &ecx, &edx);

  // EDX flags
  g_cpu_features.sse = (edx >> 25) & 1;
  g_cpu_features.sse2 = (edx >> 26) & 1;

  // ECX flags
  g_cpu_features.sse3 = (ecx >> 0) & 1;
  g_cpu_features.ssse3 = (ecx >> 9) & 1;
  g_cpu_features.sse4_1 = (ecx >> 19) & 1;
  g_cpu_features.sse4_2 = (ecx >> 20) & 1;
  g_cpu_features.popcnt = (ecx >> 23) & 1;
  g_cpu_features.aes = (ecx >> 25) & 1;
  g_cpu_features.avx = (ecx >> 28) & 1;
  g_cpu_features.fma = (ecx >> 12) & 1;

  // Extended features (function 7)
  cpuid(7, 0, &eax, &ebx, &ecx, &edx);

  g_cpu_features.bmi1 = (ebx >> 3) & 1;
  g_cpu_features.avx2 = (ebx >> 5) & 1;
  g_cpu_features.bmi2 = (ebx >> 8) & 1;
  g_cpu_features.avx512f = (ebx >> 16) & 1;
  g_cpu_features.avx512bw = (ebx >> 30) & 1;
  g_cpu_features.avx512vl = (ebx >> 31) & 1;

  // Processor info
  cpuid(1, 0, &eax, &ebx, &ecx, &edx);
  g_cpu_features.logical_processors = (ebx >> 16) & 0xFF;
}

#endif

// ============================================================================
// ARM FEATURE DETECTION
// ============================================================================

#if VEX_ARCH_ARM

static void detect_arm_features()
{
  g_cpu_features.vendor = "ARM";

#if defined(__linux__)
  // Linux: Use getauxval to check hardware capabilities
  unsigned long hwcaps = getauxval(AT_HWCAP);

#if defined(__aarch64__)
  // ARM64
  g_cpu_features.neon = true; // NEON is mandatory on ARM64

#ifdef HWCAP_SVE
  g_cpu_features.sve = (hwcaps & HWCAP_SVE) != 0;
#endif

#ifdef HWCAP2_SVE2
  unsigned long hwcaps2 = getauxval(AT_HWCAP2);
  g_cpu_features.sve2 = (hwcaps2 & HWCAP2_SVE2) != 0;
#endif

#else
  // ARM32
#ifdef HWCAP_NEON
  g_cpu_features.neon = (hwcaps & HWCAP_NEON) != 0;
#endif
#endif

#elif defined(__APPLE__)
  // macOS/iOS: NEON is always available on Apple Silicon
  g_cpu_features.neon = true;
#else
  // Other platforms: assume NEON if compiled with it
#ifdef __ARM_NEON
  g_cpu_features.neon = true;
#endif
#endif
}

#endif

// ============================================================================
// PUBLIC API
// ============================================================================

const VexCpuFeatures *vex_cpu_detect()
{
  if (g_cpu_features_initialized)
  {
    return &g_cpu_features;
  }

#if VEX_ARCH_X86
  detect_x86_features();
#elif VEX_ARCH_ARM
  detect_arm_features();
#endif

  g_cpu_features_initialized = true;
  return &g_cpu_features;
}

bool vex_cpu_has_sse2()
{
  if (!g_cpu_features_initialized)
    vex_cpu_detect();
  return g_cpu_features.sse2;
}

bool vex_cpu_has_avx2()
{
  if (!g_cpu_features_initialized)
    vex_cpu_detect();
  return g_cpu_features.avx2;
}

bool vex_cpu_has_avx512()
{
  if (!g_cpu_features_initialized)
    vex_cpu_detect();
  return g_cpu_features.avx512f;
}

bool vex_cpu_has_neon()
{
  if (!g_cpu_features_initialized)
    vex_cpu_detect();
  return g_cpu_features.neon;
}

const char *vex_cpu_vendor()
{
  if (!g_cpu_features_initialized)
    vex_cpu_detect();
  return g_cpu_features.vendor ? g_cpu_features.vendor : "Unknown";
}

// ============================================================================
// BEST SIMD INSTRUCTION SET
// ============================================================================

VexSimdLevel vex_cpu_best_simd()
{
  if (!g_cpu_features_initialized)
    vex_cpu_detect();

#if VEX_ARCH_X86
  if (g_cpu_features.avx512f)
    return VEX_SIMD_LEVEL_AVX512;
  if (g_cpu_features.avx2)
    return VEX_SIMD_LEVEL_AVX2;
  if (g_cpu_features.avx)
    return VEX_SIMD_LEVEL_AVX;
  if (g_cpu_features.sse2)
    return VEX_SIMD_LEVEL_SSE2;
#elif VEX_ARCH_ARM
  if (g_cpu_features.sve)
    return VEX_SIMD_LEVEL_SVE;
  if (g_cpu_features.neon)
    return VEX_SIMD_LEVEL_NEON;
#endif

  return VEX_SIMD_LEVEL_NONE;
}

const char *vex_cpu_simd_name(VexSimdLevel level)
{
  switch (level)
  {
  case VEX_SIMD_LEVEL_NONE:
    return "None";
  case VEX_SIMD_LEVEL_SSE2:
    return "SSE2";
  case VEX_SIMD_LEVEL_AVX:
    return "AVX";
  case VEX_SIMD_LEVEL_AVX2:
    return "AVX2";
  case VEX_SIMD_LEVEL_AVX512:
    return "AVX-512";
  case VEX_SIMD_LEVEL_NEON:
    return "NEON";
  case VEX_SIMD_LEVEL_SVE:
    return "SVE";
  default:
    return "Unknown";
  }
}

// ============================================================================
// COMPILER INFO (what was used to build runtime)
// ============================================================================

const char *vex_runtime_compiler()
{
#if defined(__clang__)
  return "Clang " __clang_version__;
#elif defined(__GNUC__)
  return "GCC " __VERSION__;
#elif defined(_MSC_VER)
  return "MSVC";
#else
  return "Unknown";
#endif
}

const char *vex_runtime_arch()
{
#if defined(__x86_64__) || defined(_M_X64)
  return "x86_64";
#elif defined(__i386__) || defined(_M_IX86)
  return "i386";
#elif defined(__aarch64__)
  return "aarch64";
#elif defined(__arm__)
  return "arm";
#elif defined(__riscv)
  return "riscv";
#else
  return "unknown";
#endif
}

const char *vex_runtime_build_flags()
{
  static char flags[256] = {0};
  if (flags[0] != '\0')
    return flags;

  char *p = flags;

#ifdef __AVX512F__
  p += sprintf(p, "AVX512 ");
#endif
#ifdef __AVX2__
  p += sprintf(p, "AVX2 ");
#endif
#ifdef __AVX__
  p += sprintf(p, "AVX ");
#endif
#ifdef __SSE4_2__
  p += sprintf(p, "SSE4.2 ");
#endif
#ifdef __SSE4_1__
  p += sprintf(p, "SSE4.1 ");
#endif
#ifdef __SSSE3__
  p += sprintf(p, "SSSE3 ");
#endif
#ifdef __SSE3__
  p += sprintf(p, "SSE3 ");
#endif
#ifdef __SSE2__
  p += sprintf(p, "SSE2 ");
#endif
#ifdef __ARM_NEON
  p += sprintf(p, "NEON ");
#endif
#ifdef __ARM_FEATURE_SVE
  p += sprintf(p, "SVE ");
#endif

  if (p == flags)
  {
    sprintf(flags, "None");
  }
  else
  {
    // Remove trailing space
    *(p - 1) = '\0';
  }

  return flags;
}
