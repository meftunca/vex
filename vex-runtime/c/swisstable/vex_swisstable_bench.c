/*
 * Comprehensive SwissTable Performance Benchmark
 * Tests various scenarios critical for Vex runtime
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <time.h>
#include "vex.h"

#if defined(_POSIX_C_SOURCE) && _POSIX_C_SOURCE >= 199309L
#define HAVE_CLOCK_GETTIME 1
#else
#define HAVE_CLOCK_GETTIME 0
#endif

static double now_sec(void)
{
#if HAVE_CLOCK_GETTIME
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (double)ts.tv_sec + (double)ts.tv_nsec * 1e-9;
#else
    return (double)clock() / (double)CLOCKS_PER_SEC;
#endif
}

static uint32_t xorshift32(uint32_t *state)
{
    uint32_t x = *state;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    *state = x;
    return x;
}

static char **gen_keys(size_t N, uint32_t seed)
{
    char **keys = (char **)malloc(N * sizeof(char *));
    if (!keys) return NULL;

    uint32_t st = seed ? seed : 0x12345678u;
    for (size_t i = 0; i < N; ++i) {
        int len = 8 + (int)(xorshift32(&st) % 17);
        char *s = (char *)malloc((size_t)len + 32);
        if (!s) {
            for (size_t j = 0; j < i; ++j) free(keys[j]);
            free(keys);
            return NULL;
        }
        snprintf(s, (size_t)len + 32, "key_%zu_%08x", i, xorshift32(&st));
        keys[i] = s;
    }
    return keys;
}

static void free_keys(char **keys, size_t N)
{
    if (!keys) return;
    for (size_t i = 0; i < N; ++i) free(keys[i]);
    free(keys);
}

// ============================================================================
// BENCHMARK 1: Sequential Insert
// ============================================================================
static void bench_sequential_insert(size_t N, int initial_cap)
{
    printf("\n[BENCH 1] Sequential Insert (N=%zu, cap=%d)\n", N, initial_cap);
    
    VexMap m;
    if (!vex_map_new(&m, (size_t)initial_cap)) {
        fprintf(stderr, "  ❌ map_new failed\n");
        return;
    }

    char **keys = gen_keys(N, 0xDEADBEEF);
    uint64_t *vals = (uint64_t *)malloc(N * sizeof(uint64_t));
    
    for (size_t i = 0; i < N; ++i) vals[i] = i;

    double t0 = now_sec();
    for (size_t i = 0; i < N; ++i) {
        if (!vex_map_insert(&m, keys[i], strlen(keys[i]), &vals[i])) {
            fprintf(stderr, "  ❌ insert failed at %zu\n", i);
            break;
        }
    }
    double t1 = now_sec();
    
    double elapsed = t1 - t0;
    double ops_per_sec = (double)N / elapsed;
    double ns_per_op = elapsed * 1e9 / (double)N;
    
    printf("  ⏱️  Time: %.3f s\n", elapsed);
    printf("  📊 Throughput: %.1f M inserts/s\n", ops_per_sec / 1e6);
    printf("  📊 Latency: %.1f ns/insert\n", ns_per_op);
    printf("  📦 Final size: %zu\n", vex_map_len(&m));

    free_keys(keys, N);
    free(vals);
    vex_map_free(&m);
}

// ============================================================================
// BENCHMARK 2: Random Lookup
// ============================================================================
static void bench_random_lookup(size_t N)
{
    printf("\n[BENCH 2] Random Lookup (N=%zu)\n", N);
    
    VexMap m;
    if (!vex_map_new(&m, 32)) {
        fprintf(stderr, "  ❌ map_new failed\n");
        return;
    }

    char **keys = gen_keys(N, 0xCAFEBABE);
    uint64_t *vals = (uint64_t *)malloc(N * sizeof(uint64_t));
    
    // Insert all
    for (size_t i = 0; i < N; ++i) {
        vals[i] = i * 7 + 13;
        vex_map_insert(&m, keys[i], strlen(keys[i]), &vals[i]);
    }

    // Random lookup
    uint32_t rng = 0x87654321;
    size_t hits = 0, misses = 0;
    
    double t0 = now_sec();
    for (size_t i = 0; i < N * 2; ++i) {
        size_t idx = xorshift32(&rng) % N;
        uint64_t *p = (uint64_t *)vex_map_get(&m, keys[idx], strlen(keys[idx]));
        if (p && *p == vals[idx]) hits++;
        else misses++;
    }
    double t1 = now_sec();
    
    double elapsed = t1 - t0;
    double ops_per_sec = (double)(N * 2) / elapsed;
    double ns_per_op = elapsed * 1e9 / (double)(N * 2);
    
    printf("  ⏱️  Time: %.3f s\n", elapsed);
    printf("  📊 Throughput: %.1f M lookups/s\n", ops_per_sec / 1e6);
    printf("  📊 Latency: %.1f ns/lookup\n", ns_per_op);
    printf("  ✅ Hit rate: %.2f%% (%zu hits, %zu misses)\n", 
           100.0 * hits / (hits + misses), hits, misses);

    free_keys(keys, N);
    free(vals);
    vex_map_free(&m);
}

// ============================================================================
// BENCHMARK 3: Mixed Operations
// ============================================================================
static void bench_mixed_operations(size_t N)
{
    printf("\n[BENCH 3] Mixed Operations (N=%zu)\n", N);
    printf("  Pattern: 60%% lookup, 30%% insert, 10%% update\n");
    
    VexMap m;
    if (!vex_map_new(&m, 32)) {
        fprintf(stderr, "  ❌ map_new failed\n");
        return;
    }

    char **keys = gen_keys(N * 2, 0xBEEFCAFE);
    uint64_t *vals = (uint64_t *)malloc(N * 2 * sizeof(uint64_t));
    
    // Pre-populate 50%
    for (size_t i = 0; i < N; ++i) {
        vals[i] = i;
        vex_map_insert(&m, keys[i], strlen(keys[i]), &vals[i]);
    }

    uint32_t rng = 0x11223344;
    size_t lookups = 0, inserts = 0, updates = 0;
    
    double t0 = now_sec();
    for (size_t i = 0; i < N * 3; ++i) {
        uint32_t dice = xorshift32(&rng) % 100;
        
        if (dice < 60) {
            // 60% lookup
            size_t idx = xorshift32(&rng) % (N * 2);
            vex_map_get(&m, keys[idx], strlen(keys[idx]));
            lookups++;
        } else if (dice < 90) {
            // 30% insert new
            size_t idx = N + (xorshift32(&rng) % N);
            vals[idx] = idx * 17;
            vex_map_insert(&m, keys[idx], strlen(keys[idx]), &vals[idx]);
            inserts++;
        } else {
            // 10% update existing
            size_t idx = xorshift32(&rng) % N;
            vals[idx] = xorshift32(&rng);
            vex_map_insert(&m, keys[idx], strlen(keys[idx]), &vals[idx]);
            updates++;
        }
    }
    double t1 = now_sec();
    
    double elapsed = t1 - t0;
    double total_ops = lookups + inserts + updates;
    double ops_per_sec = total_ops / elapsed;
    double ns_per_op = elapsed * 1e9 / total_ops;
    
    printf("  ⏱️  Time: %.3f s\n", elapsed);
    printf("  📊 Throughput: %.1f M ops/s\n", ops_per_sec / 1e6);
    printf("  📊 Latency: %.1f ns/op\n", ns_per_op);
    printf("  📈 Operations: %zu lookups, %zu inserts, %zu updates\n", 
           lookups, inserts, updates);
    printf("  📦 Final size: %zu\n", vex_map_len(&m));

    free_keys(keys, N * 2);
    free(vals);
    vex_map_free(&m);
}

// ============================================================================
// BENCHMARK 4: Small Key Performance (typical variable names)
// ============================================================================
static void bench_small_keys(size_t N)
{
    printf("\n[BENCH 4] Small Keys - Variable Names (N=%zu)\n", N);
    
    VexMap m;
    if (!vex_map_new(&m, 32)) {
        fprintf(stderr, "  ❌ map_new failed\n");
        return;
    }

    // Typical variable/function name lengths (3-15 chars)
    const char *prefixes[] = {"var", "temp", "result", "data", "value", "item", "obj", "fn", "my", "get"};
    size_t num_prefixes = sizeof(prefixes) / sizeof(prefixes[0]);
    
    char **keys = (char **)malloc(N * sizeof(char *));
    uint64_t *vals = (uint64_t *)malloc(N * sizeof(uint64_t));
    
    uint32_t rng = 0xABCD1234;
    for (size_t i = 0; i < N; ++i) {
        keys[i] = (char *)malloc(32);
        const char *prefix = prefixes[xorshift32(&rng) % num_prefixes];
        snprintf(keys[i], 32, "%s_%u", prefix, (uint32_t)i);
        vals[i] = i;
    }

    double t0 = now_sec();
    for (size_t i = 0; i < N; ++i) {
        vex_map_insert(&m, keys[i], strlen(keys[i]), &vals[i]);
    }
    double t1 = now_sec();
    
    // Lookup all
    size_t found = 0;
    double t2 = now_sec();
    for (size_t i = 0; i < N; ++i) {
        // The original instruction had a typo `)d++` and an incorrect comparison.
        // Assuming the intent was to check if the value was found and correct,
        // similar to other benchmarks.
        uint64_t *p = (uint64_t *)vex_map_get(&m, keys[i], strlen(keys[i]));
        if (p && *p == vals[i]) found++;
    }
    double t3 = now_sec();
    
    printf("  ⏱️  Insert time: %.3f s (%.1f M inserts/s)\n", 
           t1 - t0, (double)N / (t1 - t0) / 1e6);
    printf("  ⏱️  Lookup time: %.3f s (%.1f M lookups/s)\n", 
           t3 - t2, (double)N / (t3 - t2) / 1e6);
    printf("  ✅ Found: %zu/%zu\n", found, N);

    free_keys(keys, N);
    free(vals);
    vex_map_free(&m);
}

// ============================================================================
// BENCHMARK 5: Collision Stress Test
// ============================================================================
static void bench_collision_stress(size_t N)
{
    printf("\n[BENCH 5] Collision Stress Test (N=%zu)\n", N);
    printf("  Using keys with same prefix to force collisions\n");
    
    VexMap m;
    if (!vex_map_new(&m, 32)) {
        fprintf(stderr, "  ❌ map_new failed\n");
        return;
    }

    char **keys = (char **)malloc(N * sizeof(char *));
    uint64_t *vals = (uint64_t *)malloc(N * sizeof(uint64_t));
    
    // All keys share prefix - forces H2 collisions
    for (size_t i = 0; i < N; ++i) {
        keys[i] = (char *)malloc(32);
        snprintf(keys[i], 32, "prefix_%08zu", i);
        vals[i] = i * 3;
    }

    double t0 = now_sec();
    for (size_t i = 0; i < N; ++i) {
        vex_map_insert(&m, keys[i], strlen(keys[i]), &vals[i]);
    }
    double t1 = now_sec();
    
    // Verify all
    size_t errors = 0;
    double t2 = now_sec();
    for (size_t i = 0; i < N; ++i) {
        uint64_t *p = (uint64_t *)vex_map_get(&m, keys[i], strlen(keys[i]));
        if (!p || *p != vals[i]) errors++;
    }
    double t3 = now_sec();
    
    printf("  ⏱️  Insert time: %.3f s (%.1f M inserts/s)\n", 
           t1 - t0, (double)N / (t1 - t0) / 1e6);
    printf("  ⏱️  Lookup time: %.3f s (%.1f M lookups/s)\n", 
           t3 - t2, (double)N / (t3 - t2) / 1e6);
    printf("  %s Errors: %zu\n", errors ? "❌" : "✅", errors);

    free_keys(keys, N);
    free(vals);
    vex_map_free(&m);
}

// ============================================================================
// MAIN
// ============================================================================
int main(void)
{
    printf("═══════════════════════════════════════════════════════════\n");
    printf("  Vex SwissTable Comprehensive Benchmark\n");
    printf("═══════════════════════════════════════════════════════════\n");

#if defined(__ARM_NEON) || defined(__ARM_NEON__) || defined(__aarch64__)
    printf("  Platform: ARM64/AArch64 (NEON)\n");
#elif defined(__AVX2__)
    printf("  Platform: x86-64 (AVX2)\n");
#elif defined(__SSE2__)
    printf("  Platform: x86-64 (SSE2)\n");
#else
    printf("  Platform: Scalar\n");
#endif
    printf("═══════════════════════════════════════════════════════════\n");

    // Small dataset
    bench_sequential_insert(10000, 8);
    bench_random_lookup(10000);
    
    // Medium dataset
    bench_sequential_insert(100000, 32);
    bench_random_lookup(100000);
    
    // Large dataset
    bench_sequential_insert(500000, 64);
    bench_random_lookup(500000);
    
    // Realistic workloads
    bench_mixed_operations(100000);
    bench_small_keys(50000);
    bench_collision_stress(50000);

    printf("\n═══════════════════════════════════════════════════════════\n");
    printf("  🎉 Benchmark Complete!\n");
    printf("═══════════════════════════════════════════════════════════\n");

    return 0;
}

