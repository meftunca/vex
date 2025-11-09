#include "simd_detect.h"
#include <string.h>

#if defined(__x86_64__) || defined(_M_X64) || defined(__i386__) || defined(_M_IX86)
#define VEX_X86 1
#include <cpuid.h>
#elif defined(__aarch64__) || defined(_M_ARM64) || defined(__arm__) || defined(_M_ARM)
#define VEX_ARM 1
#if defined(__APPLE__)
#include <sys/sysctl.h>
#elif defined(__linux__)
#include <sys/auxv.h>
#include <asm/hwcap.h>
#endif
#endif

static SIMDFeatures g_detected_features = 0;
static int g_features_initialized = 0;

#ifdef VEX_X86
static void cpuid(uint32_t leaf, uint32_t subleaf, uint32_t* eax, uint32_t* ebx, uint32_t* ecx, uint32_t* edx) {
    __cpuid_count(leaf, subleaf, *eax, *ebx, *ecx, *edx);
}

static SIMDFeatures detect_x86_features(void) {
    SIMDFeatures features = SIMD_NONE;
    uint32_t eax, ebx, ecx, edx;
    
    /* Check CPUID support */
    cpuid(0, 0, &eax, &ebx, &ecx, &edx);
    if (eax < 1) return features;
    
    /* Check SSE2 (CPUID.01H:EDX[26]) */
    cpuid(1, 0, &eax, &ebx, &ecx, &edx);
    if (edx & (1 << 26)) {
        features |= SIMD_SSE2;
    }
    
    /* Check AVX2 (CPUID.07H:EBX[5]) */
    cpuid(7, 0, &eax, &ebx, &ecx, &edx);
    if (ebx & (1 << 5)) {
        features |= SIMD_AVX2;
    }
    
    /* Check AVX-512F (CPUID.07H:EBX[16]) */
    if (ebx & (1 << 16)) {
        features |= SIMD_AVX512;
    }
    
    return features;
}
#endif

#ifdef VEX_ARM
static SIMDFeatures detect_arm_features(void) {
    SIMDFeatures features = SIMD_NONE;
    
#if defined(__APPLE__)
    /* macOS: Use sysctl */
    int has_neon = 0;
    size_t len = sizeof(has_neon);
    if (sysctlbyname("hw.optional.neon", &has_neon, &len, NULL, 0) == 0 && has_neon) {
        features |= SIMD_NEON;
    }
#elif defined(__linux__)
    /* Linux: Use getauxval */
    unsigned long hwcaps = getauxval(AT_HWCAP);
    #ifdef HWCAP_NEON
    if (hwcaps & HWCAP_NEON) {
        features |= SIMD_NEON;
    }
    #endif
#else
    /* Assume NEON on ARMv8+ */
    #if defined(__aarch64__) || defined(_M_ARM64)
    features |= SIMD_NEON;
    #endif
#endif
    
    return features;
}
#endif

SIMDFeatures simd_detect_features(void) {
    if (g_features_initialized) {
        return g_detected_features;
    }
    
#ifdef VEX_X86
    g_detected_features = detect_x86_features();
#elif defined(VEX_ARM)
    g_detected_features = detect_arm_features();
#else
    g_detected_features = SIMD_NONE;
#endif
    
    g_features_initialized = 1;
    return g_detected_features;
}

const char* simd_feature_name(SIMDFeatures f) {
    if (f & SIMD_AVX512) return "AVX-512";
    if (f & SIMD_AVX2) return "AVX2";
    if (f & SIMD_NEON) return "NEON";
    if (f & SIMD_SSE2) return "SSE2";
    return "Scalar";
}

