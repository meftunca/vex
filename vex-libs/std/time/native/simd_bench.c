/*
 * SIMD Benchmark Tool
 * 
 * Compares scalar vs SIMD performance for vex_time operations
 */

#include "include/vex_time.h"
#include "src/common/simd_detect.h"
#include "src/common/simd_rfc3339.h"
#include <stdio.h>
#include <time.h>
#include <string.h>

#define ITERATIONS 1000000

static uint64_t get_nanos() {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (uint64_t)ts.tv_sec * 1000000000ULL + (uint64_t)ts.tv_nsec;
}

static void bench_rfc3339_parse(void) {
    const char* test_input = "2024-11-07T12:34:56.123456789Z";
    VexInstant out;
    
    printf("\n[RFC3339 Parse Benchmark]\n");
    printf("  Input: %s\n", test_input);
    printf("  Iterations: %d\n\n", ITERATIONS);
    
    /* Warm up */
    for (int i = 0; i < 1000; i++) {
        vt_parse_rfc3339(test_input, &out);
    }
    
    /* Scalar baseline (old implementation) */
    uint64_t start = get_nanos();
    for (int i = 0; i < ITERATIONS; i++) {
        vt_parse_rfc3339(test_input, &out);
    }
    uint64_t scalar_time = get_nanos() - start;
    double scalar_ns_per_op = (double)scalar_time / ITERATIONS;
    
    /* SIMD version */
    start = get_nanos();
    for (int i = 0; i < ITERATIONS; i++) {
        vt_parse_rfc3339_simd(test_input, &out);
    }
    uint64_t simd_time = get_nanos() - start;
    double simd_ns_per_op = (double)simd_time / ITERATIONS;
    
    /* Results */
    SIMDFeatures features = simd_detect_features();
    double speedup = scalar_ns_per_op / simd_ns_per_op;
    
    printf("  Scalar: %.1f ns/op (%.1fM ops/s)\n",
           scalar_ns_per_op, 1000.0 / scalar_ns_per_op);
    printf("  SIMD (%s): %.1f ns/op (%.1fM ops/s)\n",
           simd_feature_name(features), simd_ns_per_op, 1000.0 / simd_ns_per_op);
    printf("  Speedup: %.2fx %s\n", speedup, speedup > 1.0 ? "🚀" : "");
}

static void bench_rfc3339_format(void) {
    VexInstant inst = vt_instant_from_unix(1699360496, 123456789);
    char buf[64];
    
    printf("\n[RFC3339 Format Benchmark]\n");
    printf("  Iterations: %d\n\n", ITERATIONS);
    
    /* Warm up */
    for (int i = 0; i < 1000; i++) {
        vt_format_rfc3339_utc(inst, buf, sizeof(buf));
    }
    
    /* Scalar */
    uint64_t start = get_nanos();
    for (int i = 0; i < ITERATIONS; i++) {
        vt_format_rfc3339_utc(inst, buf, sizeof(buf));
    }
    uint64_t scalar_time = get_nanos() - start;
    double scalar_ns_per_op = (double)scalar_time / ITERATIONS;
    
    /* SIMD */
    start = get_nanos();
    for (int i = 0; i < ITERATIONS; i++) {
        vt_format_rfc3339_utc_simd(inst, buf, sizeof(buf));
    }
    uint64_t simd_time = get_nanos() - start;
    double simd_ns_per_op = (double)simd_time / ITERATIONS;
    
    /* Results */
    SIMDFeatures features = simd_detect_features();
    double speedup = scalar_ns_per_op / simd_ns_per_op;
    
    printf("  Scalar: %.1f ns/op (%.1fM ops/s)\n",
           scalar_ns_per_op, 1000.0 / scalar_ns_per_op);
    printf("  SIMD (%s): %.1f ns/op (%.1fM ops/s)\n",
           simd_feature_name(features), simd_ns_per_op, 1000.0 / simd_ns_per_op);
    printf("  Speedup: %.2fx %s\n", speedup, speedup > 1.0 ? "🚀" : "");
}

int main(void) {
    printf("═══════════════════════════════════════════════════════════\n");
    printf("  vex_time SIMD Benchmark\n");
    printf("═══════════════════════════════════════════════════════════\n");
    
    /* Detect CPU features */
    SIMDFeatures features = simd_detect_features();
    printf("\nDetected CPU Features:\n");
    printf("  SIMD Support: %s\n", simd_feature_name(features));
    
    if (features & SIMD_SSE2) printf("  ✓ SSE2\n");
    if (features & SIMD_AVX2) printf("  ✓ AVX2\n");
    if (features & SIMD_AVX512) printf("  ✓ AVX-512\n");
    if (features & SIMD_NEON) printf("  ✓ NEON\n");
    if (features == SIMD_NONE) printf("  ✓ Scalar only (no SIMD)\n");
    
    /* Initialize SIMD */
    vt_simd_init();
    
    /* Run benchmarks */
    bench_rfc3339_parse();
    bench_rfc3339_format();
    
    printf("\n═══════════════════════════════════════════════════════════\n");
    printf("  Benchmark Complete\n");
    printf("═══════════════════════════════════════════════════════════\n\n");
    
    printf("💡 Tips:\n");
    printf("  - Compile with -march=native for best performance\n");
    printf("  - Use -mavx2 or -march=haswell for AVX2\n");
    printf("  - Use -mavx512f for AVX-512\n");
    printf("  - ARM: automatic NEON detection\n\n");
    
    return 0;
}

