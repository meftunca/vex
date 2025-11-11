// SwissTable V2 vs V3 micro-benchmark
// Measures insert/lookups/removes on small string keys (typical Vex workload)
// Build example:
//   clang -O3 -march=native bench_v2_vs_v3.c vex_swisstable_v2.c vex_swisstable_v3.c \
//         -I.. -o bench_v2_vs_v3 && ./bench_v2_vs_v3

#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>
#include <string.h>
#include <time.h>

#include "../vex.h"

// Forward declarations for V3 since they are not yet in the public header
bool vex_map_new_v3(VexMap *map, size_t initial_capacity);
bool vex_map_insert_v3(VexMap *map, const char *key, void *value);
void *vex_map_get_v3(const VexMap *map, const char *key);
bool vex_map_remove_v3(VexMap *map, const char *key);
size_t vex_map_len_v3(const VexMap *map);
void vex_map_free_v3(VexMap *map);

#if defined(_POSIX_C_SOURCE) && _POSIX_C_SOURCE >= 199309L
#define HAVE_CLOCK_GETTIME 1
#else
#define HAVE_CLOCK_GETTIME 0
#endif

static double now_sec(void) {
#if HAVE_CLOCK_GETTIME
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (double)ts.tv_sec + (double)ts.tv_nsec * 1e-9;
#else
    return (double)clock() / (double)CLOCKS_PER_SEC;
#endif
}

static uint32_t xorshift32(uint32_t *state) {
    uint32_t x = *state;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    *state = x;
    return x;
}

static char **generate_keys(size_t n, uint32_t seed) {
    char **keys = (char **)malloc(n * sizeof(char *));
    if (!keys) return NULL;

    const char *prefixes[] = {"var", "temp", "result", "value", "item", "node"};
    size_t prefix_count = sizeof(prefixes) / sizeof(prefixes[0]);

    uint32_t st = seed ? seed : 0x12345678u;
    for (size_t i = 0; i < n; ++i) {
        keys[i] = (char *)malloc(32);
        if (!keys[i]) {
            for (size_t j = 0; j < i; ++j) free(keys[j]);
            free(keys);
            return NULL;
        }
        const char *prefix = prefixes[xorshift32(&st) % prefix_count];
        snprintf(keys[i], 32, "%s_%06u", prefix, xorshift32(&st) & 0xFFFFFFu);
    }
    return keys;
}

static void free_keys(char **keys, size_t n) {
    if (!keys) return;
    for (size_t i = 0; i < n; ++i) free(keys[i]);
    free(keys);
}

typedef struct {
    const char *name;
    bool (*map_new)(VexMap *map, size_t initial_capacity);
    bool (*map_insert)(VexMap *map, const char *key, void *value);
    void *(*map_get)(const VexMap *map, const char *key);
    bool (*map_remove)(VexMap *map, const char *key);
    size_t (*map_len)(const VexMap *map);
    void (*map_free)(VexMap *map);
} MapImpl;

static void benchmark_impl(const MapImpl *impl, size_t n) {
    printf("\n==== %s (N=%zu) ====" , impl->name, n);

    VexMap map;
    if (!impl->map_new(&map, n / 2 + 32)) {
        printf("  ❌ map_new failed\n");
        return;
    }

    char **keys = generate_keys(n, 0xBEEFCAFEu);
    uint64_t *values = (uint64_t *)malloc(n * sizeof(uint64_t));
    if (!keys || !values) {
        printf("  ❌ allocation failed\n");
        free_keys(keys, n);
        free(values);
        impl->map_free(&map);
        return;
    }
    for (size_t i = 0; i < n; ++i) values[i] = (uint64_t)i * 97u;

    // INSERT
    double t0 = now_sec();
    for (size_t i = 0; i < n; ++i) {
        if (!impl->map_insert(&map, keys[i], &values[i])) {
            printf("  ❌ insert failed at %zu\n", i);
            break;
        }
    }
    double t1 = now_sec();
    double insert_elapsed = t1 - t0;
    printf("  📥 Insert:  %6.2f M ops/s  (%.1f ns/op)  size=%zu\n",
           (double)n / insert_elapsed / 1e6,
           insert_elapsed * 1e9 / (double)n,
           impl->map_len(&map));

    // LOOKUP (random order)
    uint32_t rng = 0x1234ABCDu;
    size_t hits = 0;
    size_t iterations = n * 2;
    double t2 = now_sec();
    for (size_t i = 0; i < iterations; ++i) {
        size_t idx = xorshift32(&rng) % n;
        uint64_t *val = (uint64_t *)impl->map_get(&map, keys[idx]);
        if (val && *val == values[idx]) hits++;
    }
    double t3 = now_sec();
    double lookup_elapsed = t3 - t2;
    printf("  🔍 Lookup:  %6.2f M ops/s  (%.1f ns/op)  hit-rate=%.2f%%\n",
           (double)iterations / lookup_elapsed / 1e6,
           lookup_elapsed * 1e9 / (double)iterations,
           (double)hits * 100.0 / (double)iterations);

    // REMOVE (sequential)
    double t4 = now_sec();
    size_t removed = 0;
    for (size_t i = 0; i < n; ++i) {
        if (impl->map_remove(&map, keys[i])) removed++;
    }
    double t5 = now_sec();
    double remove_elapsed = t5 - t4;
    printf("  🗑️  Remove: %6.2f M ops/s  (%.1f ns/op)  removed=%zu\n",
           (double)n / remove_elapsed / 1e6,
           remove_elapsed * 1e9 / (double)n,
           removed);
    printf("  📦 Remaining size: %zu\n", impl->map_len(&map));

    free_keys(keys, n);
    free(values);
    impl->map_free(&map);
}

int main(void) {
    const MapImpl impls[] = {
        {
            .name = "SwissTable V2",
            .map_new = vex_map_new_v2,
            .map_insert = vex_map_insert_v2,
            .map_get = vex_map_get_v2,
            .map_remove = vex_map_remove_v2,
            .map_len = vex_map_len_v2,
            .map_free = vex_map_free_v2,
        },
        {
            .name = "SwissTable V3",
            .map_new = vex_map_new_v3,
            .map_insert = vex_map_insert_v3,
            .map_get = vex_map_get_v3,
            .map_remove = vex_map_remove_v3,
            .map_len = vex_map_len_v3,
            .map_free = vex_map_free_v3,
        },
    };

    const size_t test_sizes[] = {50000, 100000, 200000};
    size_t impl_count = sizeof(impls) / sizeof(impls[0]);
    size_t size_count = sizeof(test_sizes) / sizeof(test_sizes[0]);

    printf("============================================================\n");
    printf(" SwissTable V2 vs V3 Benchmark (ARM/x86 portable)\n");
    printf("============================================================\n");

#if defined(__ARM_NEON) || defined(__ARM_NEON__)
    printf(" Platform: ARM64 NEON\n");
#elif defined(__AVX2__)
    printf(" Platform: x86-64 AVX2\n");
#else
    printf(" Platform: Scalar\n");
#endif
    printf("------------------------------------------------------------\n");

    for (size_t s = 0; s < size_count; ++s) {
        size_t n = test_sizes[s];
        for (size_t i = 0; i < impl_count; ++i) {
            benchmark_impl(&impls[i], n);
        }
        printf("------------------------------------------------------------\n");
    }

    printf(" Benchmark complete.\n");
    printf("============================================================\n");
    return 0;
}
