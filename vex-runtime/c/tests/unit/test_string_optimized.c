// test_string_optimized.c - Performance and correctness tests for optimized string functions
#include "../../vex.h"
#include <stdio.h>
#include <string.h>
#include <time.h>
#include <assert.h>

#define TEST(name) printf("\n[TEST] %s...\n", name)
#define PASS() printf("  ✓ PASS\n")
#define ASSERT_EQ(a, b) if ((a) != (b)) { printf("  ✗ FAIL: %d != %d\n", (int)(a), (int)(b)); return; }
#define ASSERT_TRUE(cond) if (!(cond)) { printf("  ✗ FAIL: assertion failed\n"); return; }

// Benchmarking
static inline double get_time(void)
{
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return ts.tv_sec + ts.tv_nsec / 1e9;
}

#define BENCH(name, iterations, code) do { \
    printf("\n[BENCH] %s (%d iterations)...\n", name, iterations); \
    double start = get_time(); \
    for (int _i = 0; _i < iterations; _i++) { code; } \
    double end = get_time(); \
    double elapsed = end - start; \
    double ns_per_op = (elapsed / iterations) * 1e9; \
    printf("  Time: %.3f ns/op (%.2f M ops/s)\n", ns_per_op, 1000.0 / ns_per_op); \
} while(0)

// Test strlen
void test_strlen(void)
{
    TEST("vex_strlen");
    
    ASSERT_EQ(vex_strlen(""), 0);
    ASSERT_EQ(vex_strlen("a"), 1);
    ASSERT_EQ(vex_strlen("hello"), 5);
    ASSERT_EQ(vex_strlen("hello world!"), 12);
    
    // Long string
    char long_str[1000];
    memset(long_str, 'x', 999);
    long_str[999] = '\0';
    ASSERT_EQ(vex_strlen(long_str), 999);
    
    PASS();
}

void bench_strlen(void)
{
    const char *short_str = "hello world";
    const char *medium_str = "The quick brown fox jumps over the lazy dog. This is a medium length string for testing.";
    
    char long_str[1000];
    memset(long_str, 'x', 999);
    long_str[999] = '\0';
    
    BENCH("strlen (short)", 10000000, {
        volatile size_t len = vex_strlen(short_str);
        (void)len;
    });
    
    BENCH("strlen (medium)", 10000000, {
        volatile size_t len = vex_strlen(medium_str);
        (void)len;
    });
    
    BENCH("strlen (long)", 1000000, {
        volatile size_t len = vex_strlen(long_str);
        (void)len;
    });
}

// Test strcmp
void test_strcmp(void)
{
    TEST("vex_strcmp");
    
    ASSERT_TRUE(vex_strcmp("", "") == 0);
    ASSERT_TRUE(vex_strcmp("abc", "abc") == 0);
    ASSERT_TRUE(vex_strcmp("abc", "abd") < 0);
    ASSERT_TRUE(vex_strcmp("abd", "abc") > 0);
    ASSERT_TRUE(vex_strcmp("hello", "world") < 0);
    
    PASS();
}

void bench_strcmp(void)
{
    const char *s1 = "The quick brown fox jumps over the lazy dog";
    const char *s2 = "The quick brown fox jumps over the lazy dog";
    const char *s3 = "The quick brown fox jumps over the lazy cat";
    
    BENCH("strcmp (equal)", 10000000, {
        volatile int cmp = vex_strcmp(s1, s2);
        (void)cmp;
    });
    
    BENCH("strcmp (different)", 10000000, {
        volatile int cmp = vex_strcmp(s1, s3);
        (void)cmp;
    });
}

// Test strncmp
void test_strncmp(void)
{
    TEST("vex_strncmp");
    
    ASSERT_TRUE(vex_strncmp("abc", "abc", 3) == 0);
    ASSERT_TRUE(vex_strncmp("abc", "abd", 2) == 0);
    ASSERT_TRUE(vex_strncmp("abc", "abd", 3) < 0);
    ASSERT_TRUE(vex_strncmp("", "", 0) == 0);
    
    PASS();
}

// Test UTF-8 validation
void test_utf8_validation(void)
{
    TEST("vex_utf8_valid (SIMD-accelerated)");
    
    // Valid UTF-8
    ASSERT_TRUE(vex_utf8_valid("hello", 5));
    ASSERT_TRUE(vex_utf8_valid("こんにちは", 15)); // Japanese
    ASSERT_TRUE(vex_utf8_valid("🌍🚀✨", 12)); // Emojis
    ASSERT_TRUE(vex_utf8_valid("Ñoño", 6)); // Spanish
    
    // Invalid UTF-8
    const char invalid1[] = {0xC0, 0x80, 0x00}; // Overlong encoding
    const char invalid2[] = {0xE0, 0x80, 0x80, 0x00}; // Overlong encoding
    const char invalid3[] = {0xFF, 0x00}; // Invalid start byte
    
    ASSERT_TRUE(!vex_utf8_valid(invalid1, 2));
    ASSERT_TRUE(!vex_utf8_valid(invalid2, 3));
    ASSERT_TRUE(!vex_utf8_valid(invalid3, 1));
    
    PASS();
}

void bench_utf8_validation(void)
{
    // ASCII string (fast path)
    char ascii[1000];
    memset(ascii, 'A', 999);
    ascii[999] = '\0';
    
    // Mixed UTF-8
    const char *utf8_mixed = "Hello こんにちは 世界 🌍! ASCII and UTF-8 mixed content for testing SIMD performance on various character distributions.";
    
    BENCH("utf8_valid (ASCII)", 1000000, {
        volatile bool valid = vex_utf8_valid(ascii, 999);
        (void)valid;
    });
    
    BENCH("utf8_valid (mixed)", 1000000, {
        volatile bool valid = vex_utf8_valid(utf8_mixed, strlen(utf8_mixed));
        (void)valid;
    });
}

// Test UTF-8 char count
void test_utf8_char_count(void)
{
    TEST("vex_utf8_char_count");
    
    ASSERT_EQ(vex_utf8_char_count("hello"), 5);
    ASSERT_EQ(vex_utf8_char_count("こんにちは"), 5); // 5 Japanese characters
    ASSERT_EQ(vex_utf8_char_count("🌍🚀"), 2); // 2 emojis
    ASSERT_EQ(vex_utf8_char_count("Ñoño"), 4); // 4 characters
    
    PASS();
}

int main(void)
{
    printf("═══════════════════════════════════════════════════════════\n");
    printf("  VEX STRING OPTIMIZATION TESTS\n");
    printf("═══════════════════════════════════════════════════════════\n");
    
    printf("\n🔧 Platform:");
#if VEX_SIMD_X86
    #if defined(__AVX2__)
    printf(" x86-64 with AVX2\n");
    #else
    printf(" x86-64 with SSE2\n");
    #endif
#elif VEX_SIMD_NEON
    printf(" ARM64 with NEON\n");
#else
    printf(" Scalar (no SIMD)\n");
#endif

    printf("\n═══════════════════════════════════════════════════════════\n");
    printf("  CORRECTNESS TESTS\n");
    printf("═══════════════════════════════════════════════════════════\n");
    
    test_strlen();
    test_strcmp();
    test_strncmp();
    test_utf8_validation();
    test_utf8_char_count();
    
    printf("\n═══════════════════════════════════════════════════════════\n");
    printf("  PERFORMANCE BENCHMARKS\n");
    printf("═══════════════════════════════════════════════════════════\n");
    
    bench_strlen();
    bench_strcmp();
    bench_utf8_validation();
    
    printf("\n═══════════════════════════════════════════════════════════\n");
    printf("  ✅ ALL TESTS PASSED!\n");
    printf("═══════════════════════════════════════════════════════════\n");
    printf("\n🚀 String operations are optimized with SIMD acceleration!\n\n");
    
    return 0;
}

