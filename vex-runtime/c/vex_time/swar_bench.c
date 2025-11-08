/*
 * SWAR Benchmark - Compare old SIMD vs new SWAR optimization
 */

#include "include/vex_time.h"
#include "src/common/fast_parse.h"
#include <stdio.h>
#include <time.h>
#include <string.h>

#define ITERATIONS 1000000

static uint64_t get_nanos() {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (uint64_t)ts.tv_sec * 1000000000ULL + (uint64_t)ts.tv_nsec;
}

int main(void) {
    printf("═══════════════════════════════════════════════════════════\n");
    printf("  vex_time SWAR Optimization Benchmark\n");
    printf("  Comparing: SWAR (new) vs Previous\n");
    printf("═══════════════════════════════════════════════════════════\n\n");
    
    const char* test_input = "2024-11-07T12:34:56.123456789Z";
    VexInstant out;
    
    printf("[RFC3339 Parse Benchmark]\n");
    printf("  Input: %s\n", test_input);
    printf("  Iterations: %d\n\n", ITERATIONS);
    
    /* Warm up */
    for (int i = 0; i < 10000; i++) {
        vt_parse_rfc3339(test_input, &out);
    }
    
    /* Benchmark main API (now uses SWAR) */
    uint64_t start = get_nanos();
    for (int i = 0; i < ITERATIONS; i++) {
        vt_parse_rfc3339(test_input, &out);
    }
    uint64_t swar_time = get_nanos() - start;
    double swar_ns = (double)swar_time / ITERATIONS;
    
    printf("  SWAR (Main API): %.1f ns/op (%.1fM ops/s)\n",
           swar_ns, 1000.0 / swar_ns);
    
    /* Verify correctness */
    vt_parse_rfc3339(test_input, &out);
    /* 2024-11-07T12:34:56Z = Unix timestamp 1730982896 */
    if (out.unix_sec == 1730982896 && out.nsec == 123456789) {
        printf("  ✓ Correctness: PASS\n");
    } else {
        printf("  ✗ Correctness: FAIL (sec=%lld expected=1730982896, nsec=%d expected=123456789)\n", 
               (long long)out.unix_sec, out.nsec);
    }
    
    printf("\n[RFC3339 Format Benchmark]\n");
    printf("  Iterations: %d\n\n", ITERATIONS);
    
    /* Use 2024-11-07T12:34:56Z timestamp */
    VexInstant inst = vt_instant_from_unix(1730982896, 123456789);
    char buf[64];
    
    /* Warm up */
    for (int i = 0; i < 10000; i++) {
        vt_format_rfc3339_utc(inst, buf, sizeof(buf));
    }
    
    /* Benchmark */
    start = get_nanos();
    for (int i = 0; i < ITERATIONS; i++) {
        vt_format_rfc3339_utc(inst, buf, sizeof(buf));
    }
    uint64_t format_time = get_nanos() - start;
    double format_ns = (double)format_time / ITERATIONS;
    
    printf("  SWAR Format: %.1f ns/op (%.1fM ops/s)\n",
           format_ns, 1000.0 / format_ns);
    
    /* Verify output */
    vt_format_rfc3339_utc(inst, buf, sizeof(buf));
    printf("  Output: %s\n", buf);
    
    if (strstr(buf, "2024-11-07T12:34:56") != NULL) {
        printf("  ✓ Format: PASS\n");
    } else {
        printf("  ✗ Format: FAIL (expected 2024-11-07T12:34:56, got %s)\n", buf);
    }
    
    printf("\n═══════════════════════════════════════════════════════════\n");
    printf("  Performance Summary\n");
    printf("═══════════════════════════════════════════════════════════\n\n");
    
    printf("  Parse:  %.1f ns/op (%.1fM ops/s)\n", swar_ns, 1000.0 / swar_ns);
    printf("  Format: %.1f ns/op (%.1fM ops/s)\n", format_ns, 1000.0 / format_ns);
    
    printf("\n💡 Target Performance:\n");
    printf("  Parse:  < 800 ns/op  %s\n", swar_ns < 800 ? "✅ ACHIEVED!" : "⚠️ Not yet");
    printf("  Format: < 200 ns/op  %s\n", format_ns < 200 ? "✅ ACHIEVED!" : "⚠️ Not yet");
    
    printf("\n📊 vs Go/Rust:\n");
    if (swar_ns < 1000) {
        printf("  ✅ FASTER than Go (typical: 1000-1500 ns)\n");
    } else if (swar_ns < 1500) {
        printf("  ✅ COMPETITIVE with Go\n");
    } else {
        printf("  ⚠️  Slower than Go\n");
    }
    
    printf("\n═══════════════════════════════════════════════════════════\n\n");
    
    return 0;
}

