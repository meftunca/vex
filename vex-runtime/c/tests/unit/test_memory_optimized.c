/**
 * Benchmark: vex_memory.c optimized functions
 * Compare SIMD-optimized vs naive implementations
 */

#include "vex.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <assert.h>

#define ITERATIONS 1000000
#define SMALL_SIZE 32
#define MEDIUM_SIZE 1024
#define LARGE_SIZE 65536

// Timing helper
static inline double get_time_ns()
{
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return ts.tv_sec * 1e9 + ts.tv_nsec;
}

// ============================================================================
// CORRECTNESS TESTS
// ============================================================================

void test_memcpy()
{
    printf("[TEST] vex_memcpy correctness...\n");

    char src[256];
    char dst[256];

    for (int i = 0; i < 256; i++)
    {
        src[i] = (char)i;
    }

    // Test various sizes
    for (size_t size = 0; size <= 256; size++)
    {
        memset(dst, 0, 256);
        vex_memcpy(dst, src, size);
        assert(memcmp(dst, src, size) == 0);
    }

    printf("  ✓ PASS\n");
}

void test_memmove()
{
    printf("[TEST] vex_memmove correctness...\n");

    char buf[256];

    // Test overlapping forward
    for (int i = 0; i < 256; i++)
    {
        buf[i] = (char)i;
    }
    vex_memmove(buf + 10, buf, 100);
    for (int i = 0; i < 100; i++)
    {
        assert(buf[i + 10] == (char)i);
    }

    // Test overlapping backward
    for (int i = 0; i < 256; i++)
    {
        buf[i] = (char)i;
    }
    vex_memmove(buf, buf + 10, 100);
    for (int i = 0; i < 100; i++)
    {
        assert(buf[i] == (char)(i + 10));
    }

    printf("  ✓ PASS\n");
}

void test_memset()
{
    printf("[TEST] vex_memset correctness...\n");

    char buf[256];

    for (int value = 0; value < 256; value++)
    {
        memset(buf, 0, 256);
        vex_memset(buf, value, 256);
        for (int i = 0; i < 256; i++)
        {
            assert(buf[i] == (char)value);
        }
    }

    printf("  ✓ PASS\n");
}

void test_memcmp()
{
    printf("[TEST] vex_memcmp correctness...\n");

    char buf1[256];
    char buf2[256];

    for (int i = 0; i < 256; i++)
    {
        buf1[i] = (char)i;
        buf2[i] = (char)i;
    }

    // Test equal
    assert(vex_memcmp(buf1, buf2, 256) == 0);

    // Test different
    buf2[128] = 99;
    assert(vex_memcmp(buf1, buf2, 256) != 0);
    assert(vex_memcmp(buf1, buf2, 128) == 0);

    printf("  ✓ PASS\n");
}

// ============================================================================
// PERFORMANCE BENCHMARKS
// ============================================================================

void bench_memcpy(size_t size, const char *label)
{
    void *src = vex_malloc(size);
    void *dst = vex_malloc(size);

    memset(src, 0xAA, size);

    double start = get_time_ns();
    for (int i = 0; i < ITERATIONS; i++)
    {
        vex_memcpy(dst, src, size);
    }
    double end = get_time_ns();

    double time_per_op = (end - start) / ITERATIONS;
    double throughput_gbps = (size / time_per_op) * 1e9 / (1024.0 * 1024.0 * 1024.0);

    printf("  %s: %.2f ns/op (%.2f GB/s)\n", label, time_per_op, throughput_gbps);

    vex_free(src);
    vex_free(dst);
}

void bench_memset(size_t size, const char *label)
{
    void *buf = vex_malloc(size);

    double start = get_time_ns();
    for (int i = 0; i < ITERATIONS; i++)
    {
        vex_memset(buf, 0xAA, size);
    }
    double end = get_time_ns();

    double time_per_op = (end - start) / ITERATIONS;
    double throughput_gbps = (size / time_per_op) * 1e9 / (1024.0 * 1024.0 * 1024.0);

    printf("  %s: %.2f ns/op (%.2f GB/s)\n", label, time_per_op, throughput_gbps);

    vex_free(buf);
}

void bench_memcmp(size_t size, const char *label)
{
    void *buf1 = vex_malloc(size);
    void *buf2 = vex_malloc(size);

    memset(buf1, 0xAA, size);
    memset(buf2, 0xAA, size);

    double start = get_time_ns();
    for (int i = 0; i < ITERATIONS; i++)
    {
        vex_memcmp(buf1, buf2, size);
    }
    double end = get_time_ns();

    double time_per_op = (end - start) / ITERATIONS;
    double throughput_gbps = (size / time_per_op) * 1e9 / (1024.0 * 1024.0 * 1024.0);

    printf("  %s: %.2f ns/op (%.2f GB/s)\n", label, time_per_op, throughput_gbps);

    vex_free(buf1);
    vex_free(buf2);
}

// ============================================================================
// MAIN
// ============================================================================

int main()
{
    printf("═══════════════════════════════════════════════════════════\n");
    printf("  VEX MEMORY OPERATIONS - OPTIMIZED BENCHMARK\n");
    printf("═══════════════════════════════════════════════════════════\n\n");

// Platform detection
#if VEX_SIMD_X86
#ifdef __AVX__
    printf("🔧 Platform: x86-64 with AVX\n");
#else
    printf("🔧 Platform: x86-64 with SSE2\n");
#endif
#elif VEX_SIMD_NEON
    printf("🔧 Platform: ARM64 with NEON\n");
#else
    printf("🔧 Platform: Scalar (no SIMD)\n");
#endif

    printf("\n═══════════════════════════════════════════════════════════\n");
    printf("  CORRECTNESS TESTS\n");
    printf("═══════════════════════════════════════════════════════════\n\n");

    test_memcpy();
    test_memmove();
    test_memset();
    test_memcmp();

    printf("\n═══════════════════════════════════════════════════════════\n");
    printf("  PERFORMANCE BENCHMARKS (%d iterations)\n", ITERATIONS);
    printf("═══════════════════════════════════════════════════════════\n\n");

    printf("[BENCH] memcpy\n");
    bench_memcpy(SMALL_SIZE, "Small (32 bytes)   ");
    bench_memcpy(MEDIUM_SIZE, "Medium (1 KB)      ");
    bench_memcpy(LARGE_SIZE, "Large (64 KB)      ");

    printf("\n[BENCH] memset\n");
    bench_memset(SMALL_SIZE, "Small (32 bytes)   ");
    bench_memset(MEDIUM_SIZE, "Medium (1 KB)      ");
    bench_memset(LARGE_SIZE, "Large (64 KB)      ");

    printf("\n[BENCH] memcmp\n");
    bench_memcmp(SMALL_SIZE, "Small (32 bytes)   ");
    bench_memcmp(MEDIUM_SIZE, "Medium (1 KB)      ");
    bench_memcmp(LARGE_SIZE, "Large (64 KB)      ");

    printf("\n═══════════════════════════════════════════════════════════\n");
    printf("  ✅ ALL TESTS PASSED!\n");
    printf("═══════════════════════════════════════════════════════════\n\n");

    printf("🚀 Optimized memory operations:\n");
    printf("   • SIMD-accelerated (16-32 bytes at a time)\n");
    printf("   • Branch prediction hints (VEX_LIKELY/UNLIKELY)\n");
    printf("   • Pointer aliasing hints (VEX_RESTRICT)\n");
    printf("   • Efficient scalar fallback (8-byte chunks)\n\n");

    return 0;
}
