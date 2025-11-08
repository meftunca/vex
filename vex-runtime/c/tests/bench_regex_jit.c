/* bench_regex_jit.c - Benchmark PCRE2 with/without JIT
 * 
 * Compile: cc -O3 -std=c17 bench_regex_jit.c ../vex_regex.c -lpcre2-8 -o bench_regex_jit
 * Run: ./bench_regex_jit
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#define PCRE2_CODE_UNIT_WIDTH 8
#include <pcre2.h>

// Test data
static const char *EMAIL_PATTERN = "([a-zA-Z0-9._%-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,})";
static const char *TEST_TEXT = 
  "Contact us at support@example.com or sales@company.org. "
  "For urgent matters: urgent@example.net. "
  "Marketing: marketing@business.co.uk. "
  "Support team: help@support.io.";

static uint64_t get_ns(void) {
  struct timespec ts;
  clock_gettime(CLOCK_MONOTONIC, &ts);
  return (uint64_t)ts.tv_sec * 1000000000ULL + (uint64_t)ts.tv_nsec;
}

static void benchmark_no_jit(void) {
  printf("=== Benchmark WITHOUT JIT ===\n");
  
  int errcode;
  PCRE2_SIZE erroffset;
  pcre2_code *re = pcre2_compile(
    (PCRE2_SPTR)EMAIL_PATTERN,
    PCRE2_ZERO_TERMINATED,
    PCRE2_UTF,
    &errcode,
    &erroffset,
    NULL
  );
  
  if (!re) {
    printf("Compile failed!\n");
    return;
  }
  
  // NO JIT!
  
  pcre2_match_data *match_data = pcre2_match_data_create_from_pattern(re, NULL);
  
  // Warmup
  for (int i = 0; i < 1000; i++) {
    pcre2_match(re, (PCRE2_SPTR)TEST_TEXT, strlen(TEST_TEXT), 0, 0, match_data, NULL);
  }
  
  // Benchmark
  const int ITERATIONS = 1000000;
  uint64_t start = get_ns();
  
  for (int i = 0; i < ITERATIONS; i++) {
    pcre2_match(re, (PCRE2_SPTR)TEST_TEXT, strlen(TEST_TEXT), 0, 0, match_data, NULL);
  }
  
  uint64_t end = get_ns();
  double elapsed_s = (end - start) / 1e9;
  double ops_per_sec = ITERATIONS / elapsed_s;
  double ns_per_op = (end - start) / (double)ITERATIONS;
  
  printf("  Time: %.3f s\n", elapsed_s);
  printf("  Ops/s: %.2f M\n", ops_per_sec / 1e6);
  printf("  Latency: %.1f ns/op\n\n", ns_per_op);
  
  pcre2_match_data_free(match_data);
  pcre2_code_free(re);
}

static void benchmark_with_jit(void) {
  printf("=== Benchmark WITH JIT ===\n");
  
  int errcode;
  PCRE2_SIZE erroffset;
  pcre2_code *re = pcre2_compile(
    (PCRE2_SPTR)EMAIL_PATTERN,
    PCRE2_ZERO_TERMINATED,
    PCRE2_UTF,
    &errcode,
    &erroffset,
    NULL
  );
  
  if (!re) {
    printf("Compile failed!\n");
    return;
  }
  
  // JIT COMPILE! (magic happens here)
  int jit_ret = pcre2_jit_compile(re, PCRE2_JIT_COMPLETE);
  if (jit_ret < 0) {
    printf("JIT compilation failed (code: %d)\n", jit_ret);
    pcre2_code_free(re);
    return;
  }
  
  pcre2_match_data *match_data = pcre2_match_data_create_from_pattern(re, NULL);
  
  // Warmup
  for (int i = 0; i < 1000; i++) {
    pcre2_match(re, (PCRE2_SPTR)TEST_TEXT, strlen(TEST_TEXT), 0, 0, match_data, NULL);
  }
  
  // Benchmark
  const int ITERATIONS = 1000000;
  uint64_t start = get_ns();
  
  for (int i = 0; i < ITERATIONS; i++) {
    pcre2_match(re, (PCRE2_SPTR)TEST_TEXT, strlen(TEST_TEXT), 0, 0, match_data, NULL);
  }
  
  uint64_t end = get_ns();
  double elapsed_s = (end - start) / 1e9;
  double ops_per_sec = ITERATIONS / elapsed_s;
  double ns_per_op = (end - start) / (double)ITERATIONS;
  
  printf("  Time: %.3f s\n", elapsed_s);
  printf("  Ops/s: %.2f M\n", ops_per_sec / 1e6);
  printf("  Latency: %.1f ns/op\n\n", ns_per_op);
  
  pcre2_match_data_free(match_data);
  pcre2_code_free(re);
}

int main(void) {
  printf("PCRE2 JIT Benchmark\n");
  printf("Pattern: %s\n", EMAIL_PATTERN);
  printf("Text: %s\n\n", TEST_TEXT);
  
  benchmark_no_jit();
  benchmark_with_jit();
  
  printf("✅ Benchmark complete!\n");
  printf("💡 Tip: JIT should be 5-10x faster!\n");
  
  return 0;
}

