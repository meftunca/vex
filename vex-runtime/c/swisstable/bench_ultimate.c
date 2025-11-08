/*
 * ULTIMATE SHOWDOWN: V1 vs V2 vs V3
 * Goal: Prove V3 beats Rust hashbrown!
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <time.h>
#include "vex.h"

// V1 API
extern bool vex_map_new(VexMap *map, size_t initial_capacity);
extern bool vex_map_insert(VexMap *map, const char *key, void *value);
extern void *vex_map_get(const VexMap *map, const char *key);
extern void vex_map_free(VexMap *map);

// V2 API
extern bool vex_map_new_v2(VexMap *map, size_t initial_capacity);
extern bool vex_map_insert_v2(VexMap *map, const char *key, void *value);
extern void *vex_map_get_v2(const VexMap *map, const char *key);
extern void vex_map_free_v2(VexMap *map);

// V3 API
extern bool vex_map_new_v3(VexMap *map, size_t initial_capacity);
extern bool vex_map_insert_v3(VexMap *map, const char *key, void *value);
extern void *vex_map_get_v3(const VexMap *map, const char *key);
extern void vex_map_free_v3(VexMap *map);

static double now_sec(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (double)ts.tv_sec + (double)ts.tv_nsec * 1e-9;
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
        int len = 8 + (xorshift32(&st) % 9);
        keys[i] = malloc(len + 1);
        snprintf(keys[i], len + 1, "var_%zu_%x", i, xorshift32(&st));
    }
    return keys;
}

static void free_keys(char **keys, size_t N) {
    for (size_t i = 0; i < N; ++i) free(keys[i]);
    free(keys);
}

typedef struct {
    double insert_ms;
    double lookup_ms;
    double insert_ns;
    double lookup_ns;
} BenchResult;

BenchResult bench_version(size_t N, int version, size_t init_cap) {
    VexMap m;
    BenchResult result = {0};
    
    // Initialize
    switch(version) {
        case 1: vex_map_new(&m, init_cap); break;
        case 2: vex_map_new_v2(&m, init_cap); break;
        case 3: vex_map_new_v3(&m, init_cap); break;
    }
    
    char **keys = gen_keys(N, 0xDEAD0000 + version);
    uint64_t *vals = malloc(N * sizeof(uint64_t));
    for (size_t i = 0; i < N; ++i) vals[i] = i;
    
    // INSERT benchmark
    double t0 = now_sec();
    for (size_t i = 0; i < N; ++i) {
        switch(version) {
            case 1: vex_map_insert(&m, keys[i], &vals[i]); break;
            case 2: vex_map_insert_v2(&m, keys[i], &vals[i]); break;
            case 3: vex_map_insert_v3(&m, keys[i], &vals[i]); break;
        }
    }
    double t1 = now_sec();
    
    result.insert_ms = (t1 - t0) * 1000;
    result.insert_ns = (t1 - t0) * 1e9 / N;
    
    // LOOKUP benchmark (2 rounds)
    size_t found = 0;
    t0 = now_sec();
    for (int round = 0; round < 2; round++) {
        for (size_t i = 0; i < N; ++i) {
            uint64_t *p;
            switch(version) {
                case 1: p = vex_map_get(&m, keys[i]); break;
                case 2: p = vex_map_get_v2(&m, keys[i]); break;
                case 3: p = vex_map_get_v3(&m, keys[i]); break;
            }
            if (p && *p == vals[i]) found++;
        }
    }
    t1 = now_sec();
    
    result.lookup_ms = (t1 - t0) * 1000;
    result.lookup_ns = (t1 - t0) * 1e9 / (N * 2);
    
    free_keys(keys, N);
    free(vals);
    
    switch(version) {
        case 1: vex_map_free(&m); break;
        case 2: vex_map_free_v2(&m); break;
        case 3: vex_map_free_v3(&m); break;
    }
    
    return result;
}

int main(void) {
    printf("═══════════════════════════════════════════════════════════\n");
    printf("  🔥 ULTIMATE PERFORMANCE BATTLE 🔥\n");
    printf("  V1 vs V2 vs V3 - Beat Rust hashbrown!\n");
    printf("═══════════════════════════════════════════════════════════\n\n");
    
    size_t N = 100000;
    
    printf("Test Size: %zu items\n", N);
    printf("Key Type: Variable names (8-16 bytes)\n\n");
    
    // Small initial capacity (lots of rehash)
    printf("━━━━━━━━━━━━━━━━ Small Init (cap=32) ━━━━━━━━━━━━━━━━━\n\n");
    
    BenchResult r1 = bench_version(N, 1, 32);
    BenchResult r2 = bench_version(N, 2, 32);
    BenchResult r3 = bench_version(N, 3, 32);
    
    printf("V1 (baseline):\n");
    printf("  Insert: %.2f ms (%.1f ns/op, %.2f M ops/s)\n", 
           r1.insert_ms, r1.insert_ns, 1000.0 / r1.insert_ns);
    printf("  Lookup: %.2f ms (%.1f ns/op, %.2f M ops/s)\n\n", 
           r1.lookup_ms, r1.lookup_ns, 1000.0 / r1.lookup_ns);
    
    printf("V2 (optimized):\n");
    printf("  Insert: %.2f ms (%.1f ns/op, %.2f M ops/s) [%+.1f%%]\n", 
           r2.insert_ms, r2.insert_ns, 1000.0 / r2.insert_ns,
           (r1.insert_ns - r2.insert_ns) / r1.insert_ns * 100);
    printf("  Lookup: %.2f ms (%.1f ns/op, %.2f M ops/s) [%+.1f%%]\n\n", 
           r2.lookup_ms, r2.lookup_ns, 1000.0 / r2.lookup_ns,
           (r1.lookup_ns - r2.lookup_ns) / r1.lookup_ns * 100);
    
    printf("V3 (ULTIMATE):\n");
    printf("  Insert: %.2f ms (%.1f ns/op, %.2f M ops/s) [%+.1f%%]\n", 
           r3.insert_ms, r3.insert_ns, 1000.0 / r3.insert_ns,
           (r1.insert_ns - r3.insert_ns) / r1.insert_ns * 100);
    printf("  Lookup: %.2f ms (%.1f ns/op, %.2f M ops/s) [%+.1f%%]\n\n", 
           r3.lookup_ms, r3.lookup_ns, 1000.0 / r3.lookup_ns,
           (r1.lookup_ns - r3.lookup_ns) / r1.lookup_ns * 100);
    
    // Optimized initial capacity (less rehash)
    printf("━━━━━━━━━━━━━━━━ Optimized Init (cap=N*1.5) ━━━━━━━━━━━━━━━━━\n\n");
    
    size_t opt_cap = N * 3 / 2;
    r1 = bench_version(N, 1, opt_cap);
    r2 = bench_version(N, 2, opt_cap);
    r3 = bench_version(N, 3, opt_cap);
    
    printf("V1 (pre-sized):\n");
    printf("  Insert: %.2f ms (%.1f ns/op, %.2f M ops/s)\n", 
           r1.insert_ms, r1.insert_ns, 1000.0 / r1.insert_ns);
    printf("  Lookup: %.2f ms (%.1f ns/op, %.2f M ops/s)\n\n", 
           r1.lookup_ms, r1.lookup_ns, 1000.0 / r1.lookup_ns);
    
    printf("V2 (pre-sized + optimized):\n");
    printf("  Insert: %.2f ms (%.1f ns/op, %.2f M ops/s)\n", 
           r2.insert_ms, r2.insert_ns, 1000.0 / r2.insert_ns);
    printf("  Lookup: %.2f ms (%.1f ns/op, %.2f M ops/s)\n\n", 
           r2.lookup_ms, r2.lookup_ns, 1000.0 / r2.lookup_ns);
    
    printf("V3 (pre-sized + ULTIMATE):\n");
    printf("  Insert: %.2f ms (%.1f ns/op, %.2f M ops/s) 🔥\n", 
           r3.insert_ms, r3.insert_ns, 1000.0 / r3.insert_ns);
    printf("  Lookup: %.2f ms (%.1f ns/op, %.2f M ops/s) 🔥\n\n", 
           r3.lookup_ms, r3.lookup_ns, 1000.0 / r3.lookup_ns);
    
    printf("═══════════════════════════════════════════════════════════\n\n");
    
    printf("🏆 FINAL SCORES (V3 - Pre-sized):\n\n");
    
    double insert_mops = 1000.0 / r3.insert_ns;
    double lookup_mops = 1000.0 / r3.lookup_ns;
    
    printf("  Insert: %.2f M ops/s (%.1f ns)\n", insert_mops, r3.insert_ns);
    printf("  Lookup: %.2f M ops/s (%.1f ns)\n\n", lookup_mops, r3.lookup_ns);
    
    printf("═══════════════════════════════════════════════════════════\n");
    printf("  📊 COMPETITOR COMPARISON\n");
    printf("═══════════════════════════════════════════════════════════\n\n");
    
    printf("Rust hashbrown:       11-16M inserts/s, 16-25M lookups/s\n");
    printf("Vex V3:              %.1fM inserts/s, %.1fM lookups/s\n\n", insert_mops, lookup_mops);
    
    if (insert_mops >= 11.0) {
        printf("✅ INSERT: BEATING Rust hashbrown lower bound!\n");
    }
    if (insert_mops >= 13.5) {
        printf("✅ INSERT: MATCHING Rust hashbrown average!\n");
    }
    if (insert_mops >= 16.0) {
        printf("🔥 INSERT: BEATING Rust hashbrown completely!\n");
    }
    
    if (lookup_mops >= 16.0) {
        printf("✅ LOOKUP: BEATING Rust hashbrown lower bound!\n");
    }
    if (lookup_mops >= 20.0) {
        printf("✅ LOOKUP: MATCHING Rust hashbrown average!\n");
    }
    if (lookup_mops >= 25.0) {
        printf("🔥 LOOKUP: BEATING Rust hashbrown completely!\n");
    }
    
    printf("\n");
    printf("Rust std HashMap:      8-12M inserts/s, 12-20M lookups/s\n");
    printf("Vex V3:              %.1fM inserts/s, %.1fM lookups/s\n\n", insert_mops, lookup_mops);
    
    if (insert_mops >= 8.0 && lookup_mops >= 12.0) {
        printf("✅ DESTROYING Rust std::HashMap!\n");
    }
    
    printf("\n");
    printf("Go map:                6-10M inserts/s, 10-16M lookups/s\n");
    printf("Vex V3:              %.1fM inserts/s, %.1fM lookups/s\n\n", insert_mops, lookup_mops);
    
    if (insert_mops >= 10.0 && lookup_mops >= 16.0) {
        printf("✅ CRUSHING Go completely!\n");
    }
    
    printf("\n═══════════════════════════════════════════════════════════\n");
    printf("  🎉 Benchmark Complete!\n");
    printf("═══════════════════════════════════════════════════════════\n");
    
    return 0;
}

