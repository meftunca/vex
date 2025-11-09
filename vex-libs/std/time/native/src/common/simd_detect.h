#ifndef VEX_TIME_SIMD_DETECT_H
#define VEX_TIME_SIMD_DETECT_H

#include <stdint.h>

/* CPU Feature Detection */
typedef enum {
    SIMD_NONE   = 0,
    SIMD_SSE2   = 1 << 0,
    SIMD_AVX2   = 1 << 1,
    SIMD_AVX512 = 1 << 2,
    SIMD_NEON   = 1 << 3
} SIMDFeatures;

/* Detect CPU features at runtime */
SIMDFeatures simd_detect_features(void);

/* Get human-readable name */
const char* simd_feature_name(SIMDFeatures f);

#endif /* VEX_TIME_SIMD_DETECT_H */

