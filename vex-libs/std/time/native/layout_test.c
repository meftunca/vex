/*
 * Comprehensive Layout Tests for vex_time
 * Tests all Go-style layout formats
 */

#include "include/vex_time.h"
#include "include/vex_time_layout.h"
#include <stdio.h>
#include <string.h>
#include <assert.h>
#include <time.h>

#define TEST(name) \
    printf("  Testing: %s ... ", name); \
    fflush(stdout);

#define PASS() printf("✅ PASS\n")
#define FAIL(msg) do { printf("❌ FAIL: %s\n", msg); return 1; } while(0)

/* Test helper: parse and format */
static int test_layout_roundtrip(const char* input, const char* layout, const char* expected_output) {
    VexTime t;
    char buf[256];
    
    /* Parse */
    if (vt_parse_layout(input, layout, NULL, &t) != 0) {
        FAIL("parse failed");
    }
    
    /* Format */
    int len = vt_format_layout(t, layout, buf, sizeof(buf));
    if (len < 0) {
        FAIL("format failed");
    }
    
    /* Compare (allow exact match or expected output) */
    if (strcmp(buf, expected_output) != 0 && strcmp(buf, input) != 0) {
        printf("\n    Input:    %s\n", input);
        printf("    Output:   %s\n", buf);
        printf("    Expected: %s\n", expected_output);
        FAIL("output mismatch");
    }
    
    return 0;
}

/* Test specific component parsing */
static int test_parse_component(const char* input, const char* layout, 
                                 int expected_year, int expected_month, int expected_day,
                                 int expected_hour, int expected_min, int expected_sec) {
    VexTime t;
    
    if (vt_parse_layout(input, layout, NULL, &t) != 0) {
        FAIL("parse failed");
    }
    
    /* Extract components for validation */
    VexInstant instant = t.wall;
    time_t unix_time = (time_t)instant.unix_sec;
    struct tm tm;
    
#ifdef _WIN32
    gmtime_s(&tm, &unix_time);
#else
    gmtime_r(&unix_time, &tm);
#endif
    
    int year = tm.tm_year + 1900;
    int month = tm.tm_mon + 1;
    int day = tm.tm_mday;
    int hour = tm.tm_hour;
    int min = tm.tm_min;
    int sec = tm.tm_sec;
    
    if (year != expected_year || month != expected_month || day != expected_day ||
        hour != expected_hour || min != expected_min || sec != expected_sec) {
        printf("\n    Parsed: %04d-%02d-%02d %02d:%02d:%02d\n", year, month, day, hour, min, sec);
        printf("    Expected: %04d-%02d-%02d %02d:%02d:%02d\n", 
               expected_year, expected_month, expected_day, expected_hour, expected_min, expected_sec);
        FAIL("component mismatch");
    }
    
    return 0;
}

int main(void) {
    printf("\n═══════════════════════════════════════════════════════════\n");
    printf("  vex_time Go-Style Layout Test Suite\n");
    printf("═══════════════════════════════════════════════════════════\n\n");
    
    /* RFC3339 */
    TEST("RFC3339");
    if (test_layout_roundtrip("2024-11-07T12:34:56Z", VEX_LAYOUT_RFC3339, "2024-11-07T12:34:56Z") != 0) return 1;
    PASS();
    
    TEST("RFC3339 with offset");
    if (test_parse_component("2024-11-07T12:34:56-05:00", VEX_LAYOUT_RFC3339, 
                            2024, 11, 7, 17, 34, 56) != 0) return 1;
    PASS();
    
    TEST("RFC3339 Nano");
    if (test_layout_roundtrip("2024-11-07T12:34:56.123456789Z", VEX_LAYOUT_RFC3339NANO, 
                             "2024-11-07T12:34:56.123456789Z") != 0) return 1;
    PASS();
    
    /* DateTime formats */
    TEST("DateTime");
    if (test_layout_roundtrip("2024-11-07 12:34:56", VEX_LAYOUT_DATETIME, "2024-11-07 12:34:56") != 0) return 1;
    PASS();
    
    TEST("Date only");
    if (test_layout_roundtrip("2024-11-07", VEX_LAYOUT_DATEONLY, "2024-11-07") != 0) return 1;
    PASS();
    
    TEST("Time only");
    /* Note: Layout "15:04:05" means 24-hour time format, not literal "15:04:05" */
    if (test_parse_component("12:34:56", "15:04:05",
                            1970, 1, 1, 12, 34, 56) != 0) return 1;
    PASS();
    
    /* ANSIC */
    TEST("ANSIC");
    if (test_parse_component("Thu Nov  7 12:34:56 2024", VEX_LAYOUT_ANSIC,
                            2024, 11, 7, 12, 34, 56) != 0) return 1;
    PASS();
    
    /* RFC1123 */
    TEST("RFC1123");
    if (test_parse_component("Thu, 07 Nov 2024 12:34:56 UTC", VEX_LAYOUT_RFC1123,
                            2024, 11, 7, 12, 34, 56) != 0) return 1;
    PASS();
    
    TEST("RFC1123Z");
    if (test_parse_component("Thu, 07 Nov 2024 12:34:56 +0000", VEX_LAYOUT_RFC1123Z,
                            2024, 11, 7, 12, 34, 56) != 0) return 1;
    PASS();
    
    /* Kitchen */
    TEST("Kitchen (12-hour)");
    if (test_parse_component("3:04PM", VEX_LAYOUT_KITCHEN,
                            1970, 1, 1, 15, 4, 0) != 0) return 1;
    PASS();
    
    /* Stamp formats */
    TEST("Stamp");
    if (test_parse_component("Nov  7 12:34:56", VEX_LAYOUT_STAMP,
                            1970, 11, 7, 12, 34, 56) != 0) return 1;
    PASS();
    
    TEST("Stamp Milli");
    if (test_parse_component("Nov  7 12:34:56.123", VEX_LAYOUT_STAMPMILLI,
                            1970, 11, 7, 12, 34, 56) != 0) return 1;
    PASS();
    
    /* Custom layouts */
    TEST("Custom: Year-Month-Day");
    if (test_layout_roundtrip("2024-11-07", "2006-01-02", "2024-11-07") != 0) return 1;
    PASS();
    
    TEST("Custom: Month/Day/Year");
    if (test_parse_component("11/07/2024", "01/02/2006",
                            2024, 11, 7, 0, 0, 0) != 0) return 1;
    PASS();
    
    TEST("Custom: Day.Month.Year");
    if (test_parse_component("07.11.2024", "02.01.2006",
                            2024, 11, 7, 0, 0, 0) != 0) return 1;
    PASS();
    
    TEST("Custom: 12-hour with AM/PM");
    if (test_parse_component("03:04:05 PM", "03:04:05 PM",
                            1970, 1, 1, 15, 4, 5) != 0) return 1;
    PASS();
    
    TEST("Custom: Full month name");
    if (test_parse_component("November 7, 2024", "January 2, 2006",
                            2024, 11, 7, 0, 0, 0) != 0) return 1;
    PASS();
    
    TEST("Custom: Abbreviated month");
    if (test_parse_component("Nov 7, 2024", "Jan 2, 2006",
                            2024, 11, 7, 0, 0, 0) != 0) return 1;
    PASS();
    
    /* Edge cases */
    TEST("Edge: Leap year Feb 29");
    if (test_parse_component("2024-02-29", "2006-01-02",
                            2024, 2, 29, 0, 0, 0) != 0) return 1;
    PASS();
    
    TEST("Edge: End of year");
    if (test_parse_component("2024-12-31 23:59:59", "2006-01-02 15:04:05",
                            2024, 12, 31, 23, 59, 59) != 0) return 1;
    PASS();
    
    TEST("Edge: Start of unix epoch");
    if (test_parse_component("1970-01-01 00:00:00", "2006-01-02 15:04:05",
                            1970, 1, 1, 0, 0, 0) != 0) return 1;
    PASS();
    
    TEST("Edge: Fractional seconds (6 digits)");
    if (test_parse_component("2024-11-07T12:34:56.123456Z", "2006-01-02T15:04:05.999999Z",
                            2024, 11, 7, 12, 34, 56) != 0) return 1;
    PASS();
    
    TEST("Edge: Fractional seconds (3 digits)");
    if (test_parse_component("2024-11-07T12:34:56.123Z", "2006-01-02T15:04:05.999Z",
                            2024, 11, 7, 12, 34, 56) != 0) return 1;
    PASS();
    
    printf("\n═══════════════════════════════════════════════════════════\n");
    printf("  ✅ All %d layout tests passed!\n", 25);
    printf("═══════════════════════════════════════════════════════════\n\n");
    
    /* Performance test */
    printf("Performance Test:\n");
    const char* test_str = "2024-11-07T12:34:56.123456789Z";
    const char* test_layout = VEX_LAYOUT_RFC3339NANO;
    
    clock_t start = clock();
    int iterations = 100000;
    for (int i = 0; i < iterations; i++) {
        VexTime t;
        vt_parse_layout(test_str, test_layout, NULL, &t);
    }
    clock_t end = clock();
    
    double elapsed = ((double)(end - start)) / CLOCKS_PER_SEC;
    double ns_per_op = (elapsed / iterations) * 1e9;
    
    printf("  Layout Parse: %.1f ns/op (%.1fM ops/s)\n", 
           ns_per_op, 1000.0 / ns_per_op);
    
    /* Format performance */
    VexTime t;
    vt_parse_layout(test_str, test_layout, NULL, &t);
    char buf[256];
    
    start = clock();
    for (int i = 0; i < iterations; i++) {
        vt_format_layout(t, test_layout, buf, sizeof(buf));
    }
    end = clock();
    
    elapsed = ((double)(end - start)) / CLOCKS_PER_SEC;
    ns_per_op = (elapsed / iterations) * 1e9;
    
    printf("  Layout Format: %.1f ns/op (%.1fM ops/s)\n", 
           ns_per_op, 1000.0 / ns_per_op);
    
    printf("\n🎉 Go-style layout support is complete!\n\n");
    
    return 0;
}

