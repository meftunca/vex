/*
 * vex_time Stress Test
 * 
 * Tests:
 * 1. Duration parsing/formatting throughput
 * 2. RFC3339 parsing/formatting throughput
 * 3. Go-layout formatting with multiple timezones
 * 4. Timer/Ticker stress (many concurrent timers)
 * 5. Memory leak detection
 * 6. Performance metrics
 */

#include "vex_time.h"
#include <stdio.h>
#include <string.h>
#include <time.h>
#include <stdatomic.h>
#include <unistd.h>

#define TEST_ITERATIONS 100000
#define TIMER_COUNT 100
#define TICKER_COUNT 50

// === Timing helpers ===
static uint64_t get_nanos() {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (uint64_t)ts.tv_sec * 1000000000ULL + (uint64_t)ts.tv_nsec;
}

// === Test 1: Duration Parsing/Formatting ===
static int test_duration_throughput(void) {
    printf("\n[Test 1] Duration Parsing/Formatting\n");
    printf("      Operations: %d\n", TEST_ITERATIONS);
    
    const char* test_cases[] = {
        "1h30m45s",
        "500ms",
        "1.5h",
        "250µs",
        "10s",
        "-2h30m",
        "1h2m3s4ms5us6ns"
    };
    int num_cases = sizeof(test_cases) / sizeof(test_cases[0]);
    
    // Parse test
    uint64_t start = get_nanos();
    int parse_errors = 0;
    
    for (int i = 0; i < TEST_ITERATIONS; i++) {
        VexDuration d;
        if (vt_parse_duration(test_cases[i % num_cases], &d) != 0) {
            parse_errors++;
        }
    }
    
    uint64_t parse_duration = get_nanos() - start;
    double parse_ns_per_op = (double)parse_duration / TEST_ITERATIONS;
    
    // Format test
    start = get_nanos();
    int format_errors = 0;
    
    for (int i = 0; i < TEST_ITERATIONS; i++) {
        char buf[64];
        VexDuration d = (VexDuration)(i * 1000000LL); // Various durations
        if (vt_format_duration(d, buf, sizeof(buf)) != 0) {
            format_errors++;
        }
    }
    
    uint64_t format_duration = get_nanos() - start;
    double format_ns_per_op = (double)format_duration / TEST_ITERATIONS;
    
    printf("      Parse:  %.1f ns/op (%.1fM ops/s) - %d errors\n",
           parse_ns_per_op, 1000.0 / parse_ns_per_op, parse_errors);
    printf("      Format: %.1f ns/op (%.1fM ops/s) - %d errors\n",
           format_ns_per_op, 1000.0 / format_ns_per_op, format_errors);
    
    return parse_errors == 0 && format_errors == 0 ? 0 : 1;
}

// === Test 2: RFC3339 Throughput ===
static int test_rfc3339_throughput(void) {
    printf("\n[Test 2] RFC3339 Parsing/Formatting\n");
    printf("      Operations: %d\n", TEST_ITERATIONS);
    
    const char* test_rfc3339[] = {
        "2024-11-07T12:34:56Z",
        "2024-11-07T12:34:56.123456789Z",
        "2024-11-07T15:34:56+03:00",
        "2024-11-07T09:34:56-03:00"
    };
    int num_cases = sizeof(test_rfc3339) / sizeof(test_rfc3339[0]);
    
    // Parse test
    uint64_t start = get_nanos();
    int parse_errors = 0;
    
    for (int i = 0; i < TEST_ITERATIONS; i++) {
        VexInstant inst;
        if (vt_parse_rfc3339(test_rfc3339[i % num_cases], &inst) != 0) {
            parse_errors++;
        }
    }
    
    uint64_t parse_duration = get_nanos() - start;
    double parse_ns_per_op = (double)parse_duration / TEST_ITERATIONS;
    
    // Format test
    start = get_nanos();
    int format_errors = 0;
    VexInstant inst = vt_instant_from_unix(1699360496, 123456789);
    
    for (int i = 0; i < TEST_ITERATIONS; i++) {
        char buf[64];
        if (vt_format_rfc3339_utc(inst, buf, sizeof(buf)) != 0) {
            format_errors++;
        }
    }
    
    uint64_t format_duration = get_nanos() - start;
    double format_ns_per_op = (double)format_duration / TEST_ITERATIONS;
    
    printf("      Parse:  %.1f ns/op (%.1fM ops/s) - %d errors\n",
           parse_ns_per_op, 1000.0 / parse_ns_per_op, parse_errors);
    printf("      Format: %.1f ns/op (%.1fM ops/s) - %d errors\n",
           format_ns_per_op, 1000.0 / format_ns_per_op, format_errors);
    
    return parse_errors == 0 && format_errors == 0 ? 0 : 1;
}

// === Test 3: Timezone Operations ===
static int test_timezone_stress(void) {
    printf("\n[Test 3] Timezone Operations\n");
    printf("      Timezones: UTC, America/New_York, Europe/London, Asia/Tokyo\n");
    printf("      Operations: %d\n", TEST_ITERATIONS / 10);
    
    const char* tz_names[] = {
        "America/New_York",
        "Europe/London",
        "Asia/Tokyo",
        "Australia/Sydney"
    };
    int num_tzs = sizeof(tz_names) / sizeof(tz_names[0]);
    
    // Load timezones
    VexTz* tzs[4];
    for (int i = 0; i < num_tzs; i++) {
        tzs[i] = vt_tz_load(tz_names[i]);
        if (!tzs[i]) {
            printf("      ⚠️  Could not load %s (skipping)\n", tz_names[i]);
        }
    }
    
    // Test timezone operations
    uint64_t start = get_nanos();
    int errors = 0;
    VexInstant inst = vt_instant_from_unix(1699360496, 0);
    
    for (int i = 0; i < TEST_ITERATIONS / 10; i++) {
        for (int j = 0; j < num_tzs; j++) {
            if (!tzs[j]) continue;
            
            char buf[128];
            if (vt_format_go(inst, tzs[j], "Monday, 02 Jan 2006 15:04:05 MST", buf, sizeof(buf)) != 0) {
                errors++;
            }
        }
    }
    
    uint64_t duration = get_nanos() - start;
    double ns_per_op = (double)duration / (TEST_ITERATIONS / 10 * num_tzs);
    
    printf("      Format: %.1f ns/op (%.1fM ops/s) - %d errors\n",
           ns_per_op, 1000.0 / ns_per_op, errors);
    
    // Cleanup
    for (int i = 0; i < num_tzs; i++) {
        if (tzs[i]) vt_tz_release(tzs[i]);
    }
    
    return errors == 0 ? 0 : 1;
}

// === Test 4: Timer/Ticker Stress ===
static atomic_int g_timer_fires = 0;
static atomic_int g_ticker_fires = 0;

static void timer_callback(void* user, VexTime when) {
    (void)user;
    (void)when;
    atomic_fetch_add(&g_timer_fires, 1);
}

static void ticker_callback(void* user, VexTime when) {
    (void)user;
    (void)when;
    atomic_fetch_add(&g_ticker_fires, 1);
}

static int test_timer_stress(void) {
    printf("\n[Test 4] Timer/Ticker Stress\n");
    printf("      Timers: %d (one-shot)\n", TIMER_COUNT);
    printf("      Tickers: %d (periodic, 50ms)\n", TICKER_COUNT);
    
    VexTimeSched* sched = vt_sched_create();
    if (!sched) {
        printf("      ❌ Failed to create scheduler\n");
        return 1;
    }
    
    // Create and start timers
    VexTimer* timers[TIMER_COUNT];
    for (int i = 0; i < TIMER_COUNT; i++) {
        timers[i] = vt_timer_create(sched, timer_callback, NULL);
        vt_timer_start(timers[i], (VexDuration)(10 + i) * 1000 * 1000); // 10-110ms
    }
    
    // Create and start tickers
    VexTicker* tickers[TICKER_COUNT];
    for (int i = 0; i < TICKER_COUNT; i++) {
        tickers[i] = vt_ticker_create(sched, ticker_callback, NULL);
        vt_ticker_start(tickers[i], (VexDuration)50 * 1000 * 1000); // 50ms
    }
    
    printf("      Running for 500ms...\n");
    vt_sleep_ns(500LL * 1000 * 1000); // 500ms
    
    // Stop all tickers
    for (int i = 0; i < TICKER_COUNT; i++) {
        vt_ticker_stop(tickers[i]);
    }
    
    vt_sleep_ns(100LL * 1000 * 1000); // Give time for cleanup
    
    int timer_fires = atomic_load(&g_timer_fires);
    int ticker_fires = atomic_load(&g_ticker_fires);
    
    printf("      Timer fires: %d (expected ~%d)\n", timer_fires, TIMER_COUNT);
    printf("      Ticker fires: %d (expected ~%d)\n", ticker_fires, TICKER_COUNT * 10);
    
    // Cleanup
    for (int i = 0; i < TIMER_COUNT; i++) {
        vt_timer_destroy(timers[i]);
    }
    for (int i = 0; i < TICKER_COUNT; i++) {
        vt_ticker_destroy(tickers[i]);
    }
    vt_sched_destroy(sched);
    
    // Verify reasonable fire counts
    int passed = (timer_fires >= TIMER_COUNT * 0.9) && 
                 (ticker_fires >= TICKER_COUNT * 8) &&
                 (ticker_fires <= TICKER_COUNT * 12);
    
    return passed ? 0 : 1;
}

// === Test 5: vt_now() Performance ===
static int test_now_performance(void) {
    printf("\n[Test 5] vt_now() Performance\n");
    printf("      Operations: %d\n", TEST_ITERATIONS);
    
    uint64_t start = get_nanos();
    
    for (int i = 0; i < TEST_ITERATIONS; i++) {
        VexTime t;
        vt_now(&t);
    }
    
    uint64_t duration = get_nanos() - start;
    double ns_per_op = (double)duration / TEST_ITERATIONS;
    
    printf("      Time: %.1f ns/op (%.1fM ops/s)\n",
           ns_per_op, 1000.0 / ns_per_op);
    
    return 0;
}

// === Test 6: Memory Leak Test ===
static int test_memory_leaks(void) {
    printf("\n[Test 6] Memory Leak Detection\n");
    printf("      Iterations: 1000 (create/destroy cycles)\n");
    
    // Timezone load/release cycles
    for (int i = 0; i < 1000; i++) {
        VexTz* tz = vt_tz_fixed("TEST", 3600);
        if (tz) vt_tz_release(tz);
    }
    
    // Timer create/destroy cycles
    VexTimeSched* sched = vt_sched_create();
    for (int i = 0; i < 1000; i++) {
        VexTimer* t = vt_timer_create(sched, timer_callback, NULL);
        vt_timer_start(t, 1000000000LL);
        vt_timer_stop(t);
        vt_timer_destroy(t);
    }
    vt_sched_destroy(sched);
    
    printf("      ✓ No crashes (use valgrind/leaks for full analysis)\n");
    return 0;
}

// === Main ===
int main(void) {
    printf("═══════════════════════════════════════════════════════════\n");
    printf("  vex_time Stress Test\n");
    printf("═══════════════════════════════════════════════════════════\n");
    
    int failed = 0;
    
    failed += test_duration_throughput();
    failed += test_rfc3339_throughput();
    failed += test_timezone_stress();
    failed += test_timer_stress();
    failed += test_now_performance();
    failed += test_memory_leaks();
    
    printf("\n═══════════════════════════════════════════════════════════\n");
    if (failed == 0) {
        printf("  ✅ ALL TESTS PASSED!\n");
    } else {
        printf("  ❌ %d TEST(S) FAILED\n", failed);
    }
    printf("═══════════════════════════════════════════════════════════\n");
    
    return failed > 0 ? 1 : 0;
}

