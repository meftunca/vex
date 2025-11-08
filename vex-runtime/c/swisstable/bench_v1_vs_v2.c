/*
 * V1 vs V2 Comparison + Rust/C++ Comparison
 * Goal: CRUSH Rust HashMap and C++ Abseil!
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <time.h>
#include "vex.h"

// Forward declarations for V2
extern bool vex_map_new_v2(VexMap *map, size_t initial_capacity);
extern bool vex_map_insert_v2(VexMap *map, const char *key, void *value);
extern void *vex_map_get_v2(const VexMap *map, const char *key);
extern bool vex_map_remove_v2(VexMap *map, const char *key);
extern size_t vex_map_len_v2(const VexMap *map);
extern void vex_map_free_v2(VexMap *map);

static double now_sec(void) {
#if defined(_POSIX_C_SOURCE) && _POSIX_C_SOURCE >= 199309L
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (double)ts.tv_sec + (double)ts.tv_nsec * 1e-9;
#else
    return (double)clock() / (double)CLOCKS_PER_SEC;
#endif
}

static uint32_t xorshift32(uint32_t *state) {
    uint32_t x = *state;
    x ^= x << 13; x ^= x >> 17; x ^= x << 5;
    *state = x;
    return x;
}

static char **gen_keys(size_t N, uint32_t seed) {
    char **keys = malloc(N * sizeof(char *));
    uint32_t st = seed;
    for (size_t i = 0; i < N; ++i) {
        int len = 8 + (xorshift32(&st) % 9);  // 8-16 bytes (typical var names)
        keys[i] = malloc(len + 1);
        snprintf(keys[i], len + 1, "var_%zu_%x", i, xorshift32(&st));
    }
    return keys;
}

static void free_keys(char **keys, size_t N) {
    for (size_t i = 0; i < N; ++i) free(keys[i]);
    free(keys);
}

// ============================================================================
// Benchmark Insert
// ============================================================================
static void bench_insert(size_t N, const char *label, int version) {
    printf("\n[%s] Insert Benchmark (N=%zu)\n", label, N);
    
    VexMap m;
    if (version == 1) vex_map_new(&m, 32);
    else vex_map_new_v2(&m, 32);
    
    char **keys = gen_keys(N, 0xDEADBEEF);
    uint64_t *vals = malloc(N * sizeof(uint64_t));
    for (size_t i = 0; i < N; ++i) vals[i] = i;
    
    double t0 = now_sec();
    for (size_t i = 0; i < N; ++i) {
        if (version == 1) vex_map_insert(&m, keys[i], &vals[i]);
        else vex_map_insert_v2(&m, keys[i], &vals[i]);
    }
    double t1 = now_sec();
    
    double elapsed = t1 - t0;
    double ns_per_op = elapsed * 1e9 / N;
    double m_ops_s = N / elapsed / 1e6;
    
    printf("  ⏱️  Time: %.4f s\n", elapsed);
    printf("  📊 Throughput: %.2f M inserts/s\n", m_ops_s);
    printf("  📊 Latency: %.1f ns/insert\n", ns_per_op);
    
    free_keys(keys, N);
    free(vals);
    if (version == 1) vex_map_free(&m);
    else vex_map_free_v2(&m);
}

// ============================================================================
// Benchmark Lookup
// ============================================================================
static void bench_lookup(size_t N, const char *label, int version) {
    printf("\n[%s] Lookup Benchmark (N=%zu)\n", label, N);
    
    VexMap m;
    if (version == 1) vex_map_new(&m, 32);
    else vex_map_new_v2(&m, 32);
    
    char **keys = gen_keys(N, 0xCAFEBABE);
    uint64_t *vals = malloc(N * sizeof(uint64_t));
    
    // Insert all
    for (size_t i = 0; i < N; ++i) {
        vals[i] = i * 7;
        if (version == 1) vex_map_insert(&m, keys[i], &vals[i]);
        else vex_map_insert_v2(&m, keys[i], &vals[i]);
    }
    
    // Lookup all (repeat 2x for better measurement)
    size_t found = 0;
    double t0 = now_sec();
    for (int round = 0; round < 2; round++) {
        for (size_t i = 0; i < N; ++i) {
            uint64_t *p;
            if (version == 1) p = vex_map_get(&m, keys[i]);
            else p = vex_map_get_v2(&m, keys[i]);
            if (p && *p == vals[i]) found++;
        }
    }
    double t1 = now_sec();
    
    double elapsed = t1 - t0;
    double total_ops = N * 2;
    double ns_per_op = elapsed * 1e9 / total_ops;
    double m_ops_s = total_ops / elapsed / 1e6;
    
    printf("  ⏱️  Time: %.4f s\n", elapsed);
    printf("  📊 Throughput: %.2f M lookups/s\n", m_ops_s);
    printf("  📊 Latency: %.1f ns/lookup\n", ns_per_op);
    printf("  ✅ Hit rate: %.2f%%\n", 100.0 * found / total_ops);
    
    free_keys(keys, N);
    free(vals);
    if (version == 1) vex_map_free(&m);
    else vex_map_free_v2(&m);
}

// ============================================================================
// Summary comparison
// ============================================================================
static void print_comparison(void) {
    printf("\n");
    printf("═══════════════════════════════════════════════════════════\n");
    printf("  🏆 PERFORMANCE COMPARISON\n");
    printf("═══════════════════════════════════════════════════════════\n");
    printf("\n");
    
    printf("Reference Benchmarks (typical values, 100K items):\n\n");
    
    printf("1️⃣  C++ Abseil Swiss Tables (x86-64/AVX2):\n");
    printf("   Insert:  50-80 ns   (12-20M ops/s)\n");
    printf("   Lookup:  30-50 ns   (20-33M ops/s)\n");
    printf("   Delete:  40-70 ns   (14-25M ops/s)\n");
    printf("   Note: Industry gold standard\n\n");
    
    printf("2️⃣  Rust HashMap (std::collections):\n");
    printf("   Insert:  80-120 ns  (8-12M ops/s)\n");
    printf("   Lookup:  50-80 ns   (12-20M ops/s)\n");
    printf("   Delete:  60-100 ns  (10-16M ops/s)\n");
    printf("   Note: SipHash by default (slower but secure)\n\n");
    
    printf("3️⃣  Rust hashbrown (used in std via ahash):\n");
    printf("   Insert:  60-90 ns   (11-16M ops/s)\n");
    printf("   Lookup:  40-60 ns   (16-25M ops/s)\n");
    printf("   Delete:  50-80 ns   (12-20M ops/s)\n");
    printf("   Note: Swiss tables port with ahash\n\n");
    
    printf("4️⃣  Go map (runtime.hmap):\n");
    printf("   Insert:  100-150 ns (6-10M ops/s)\n");
    printf("   Lookup:  60-100 ns  (10-16M ops/s)\n");
    printf("   Delete:  80-120 ns  (8-12M ops/s)\n\n");
    
    printf("═══════════════════════════════════════════════════════════\n\n");
}

int main(void) {
    printf("═══════════════════════════════════════════════════════════\n");
    printf("  Vex SwissTable: V1 vs V2 ULTIMATE SHOWDOWN\n");
    printf("  Goal: CRUSH Rust AND C++!\n");
    printf("═══════════════════════════════════════════════════════════\n");
    
#if defined(__ARM_NEON) || defined(__aarch64__)
    printf("  Platform: ARM64 (NEON)\n");
#elif defined(__AVX2__)
    printf("  Platform: x86-64 (AVX2)\n");
#else
    printf("  Platform: Scalar\n");
#endif
    
    printf("═══════════════════════════════════════════════════════════\n");
    
    // Small workload (10K)
    printf("\n━━━━━━━━━━━━━━━━ 10K Items (Warm-up) ━━━━━━━━━━━━━━━━━\n");
    bench_insert(10000, "V1", 1);
    bench_insert(10000, "V2 OPTIMIZED", 2);
    bench_lookup(10000, "V1", 1);
    bench_lookup(10000, "V2 OPTIMIZED", 2);
    
    // Medium workload (100K) - Primary test
    printf("\n━━━━━━━━━━━━━━━━ 100K Items (PRIMARY TEST) ━━━━━━━━━━━━━━━━━\n");
    bench_insert(100000, "V1", 1);
    bench_insert(100000, "V2 OPTIMIZED", 2);
    bench_lookup(100000, "V1", 1);
    bench_lookup(100000, "V2 OPTIMIZED", 2);
    
    // Large workload (500K)
    printf("\n━━━━━━━━━━━━━━━━ 500K Items (Stress Test) ━━━━━━━━━━━━━━━━━\n");
    bench_insert(500000, "V1", 1);
    bench_insert(500000, "V2 OPTIMIZED", 2);
    bench_lookup(500000, "V1", 1);
    bench_lookup(500000, "V2 OPTIMIZED", 2);
    
    // Print comparison
    print_comparison();
    
    printf("🎯 Analysis:\n");
    printf("   If V2 > 15M inserts/s  → BEATING Rust hashbrown! ✅\n");
    printf("   If V2 > 20M lookups/s  → BEATING Rust hashbrown! ✅\n");
    printf("   If V2 > 18M ops/s      → COMPETING with Abseil! 🔥\n");
    printf("\n");
    printf("═══════════════════════════════════════════════════════════\n");
    printf("  🚀 Benchmark Complete!\n");
    printf("═══════════════════════════════════════════════════════════\n");
    
    return 0;
}

