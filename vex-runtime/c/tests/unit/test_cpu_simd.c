// test_cpu_simd.c - Test CPU feature detection and SIMD capabilities
#include "vex.h"
#include <stdio.h>
#include <assert.h>
#include <string.h>

void test_cpu_detection()
{
  printf("\n=== Testing CPU Feature Detection ===\n");

  const VexCpuFeatures *features = vex_cpu_detect();
  assert(features != NULL);
  printf("✓ vex_cpu_detect\n");

  // Vendor
  const char *vendor = vex_cpu_vendor();
  printf("  CPU Vendor: %s\n", vendor);

  // Check individual features
  printf("  SSE2:    %s\n", vex_cpu_has_sse2() ? "YES" : "NO");
  printf("  AVX2:    %s\n", vex_cpu_has_avx2() ? "YES" : "NO");
  printf("  AVX-512: %s\n", vex_cpu_has_avx512() ? "YES" : "NO");
  printf("  NEON:    %s\n", vex_cpu_has_neon() ? "YES" : "NO");

  // Best SIMD level
  VexSimdLevel best = vex_cpu_best_simd();
  printf("  Best SIMD: %s\n", vex_cpu_simd_name(best));
  printf("✓ CPU features detected\n");
}

void test_runtime_info()
{
  printf("\n=== Testing Runtime Info ===\n");

  printf("  Compiler:  %s\n", vex_runtime_compiler());
  printf("  Arch:      %s\n", vex_runtime_arch());
  printf("  SIMD:      %s\n", vex_runtime_build_flags());
  printf("✓ Runtime info\n");
}

void test_strconv()
{
  printf("\n=== Testing String Conversion (SIMD) ===\n");

  // Parse integers
  int64_t i64;
  assert(vex_parse_i64("12345", &i64));
  assert(i64 == 12345);
  printf("✓ vex_parse_i64: %lld\n", i64);

  assert(vex_parse_i64("-9876", &i64));
  assert(i64 == -9876);
  printf("✓ vex_parse_i64 (negative): %lld\n", i64);

  uint64_t u64;
  assert(vex_parse_u64("18446744073709551615", &u64));
  printf("✓ vex_parse_u64 (max): %llu\n", u64);

  // Parse float
  double f64;
  assert(vex_parse_f64("3.14159", &f64));
  assert(f64 > 3.14 && f64 < 3.15);
  printf("✓ vex_parse_f64: %.5f\n", f64);

  assert(vex_parse_f64("1.23e10", &f64));
  printf("✓ vex_parse_f64 (scientific): %.2e\n", f64);

  // Convenience functions
  assert(vex_str_to_i64("42") == 42);
  assert(vex_str_to_u64("100") == 100);
  assert(vex_str_to_f64("2.5") > 2.4 && vex_str_to_f64("2.5") < 2.6);
  printf("✓ vex_str_to_* convenience functions\n");

  // To string
  char *s1 = vex_i64_to_str(-12345);
  assert(strcmp(s1, "-12345") == 0);
  vex_free(s1);
  printf("✓ vex_i64_to_str\n");

  char *s2 = vex_u64_to_str(999);
  assert(strcmp(s2, "999") == 0);
  vex_free(s2);
  printf("✓ vex_u64_to_str\n");

  char *s3 = vex_f64_to_str(3.14);
  printf("✓ vex_f64_to_str: %s\n", s3);
  vex_free(s3);

  // Base conversion
  char *hex = vex_i64_to_str_base(255, 16);
  assert(strcmp(hex, "ff") == 0);
  vex_free(hex);
  printf("✓ vex_i64_to_str_base (hex)\n");

  char *bin = vex_i64_to_str_base(42, 2);
  assert(strcmp(bin, "101010") == 0);
  vex_free(bin);
  printf("✓ vex_i64_to_str_base (binary)\n");
}

void test_url()
{
  printf("\n=== Testing URL Encoding (SIMD) ===\n");

  // URL encoding
  char *encoded = vex_url_encode("Hello World!");
  assert(strcmp(encoded, "Hello+World%21") == 0);
  printf("✓ vex_url_encode: %s\n", encoded);
  vex_free(encoded);

  char *encoded2 = vex_url_encode("user@example.com");
  printf("✓ vex_url_encode (email): %s\n", encoded2);
  vex_free(encoded2);

  // URL decoding
  char *decoded = vex_url_decode("Hello+World%21");
  assert(strcmp(decoded, "Hello World!") == 0);
  printf("✓ vex_url_decode: %s\n", decoded);
  vex_free(decoded);

  // URL parsing
  VexUrl *url = vex_url_parse("https://example.com:8080/path/to/resource?key=value&foo=bar#section");
  assert(url != NULL);
  assert(strcmp(url->scheme, "https") == 0);
  assert(strcmp(url->host, "example.com") == 0);
  assert(url->port == 8080);
  assert(strcmp(url->path, "/path/to/resource") == 0);
  assert(strcmp(url->query, "key=value&foo=bar") == 0);
  assert(strcmp(url->fragment, "section") == 0);
  printf("✓ vex_url_parse:\n");
  printf("  Scheme:   %s\n", url->scheme);
  printf("  Host:     %s\n", url->host);
  printf("  Port:     %d\n", url->port);
  printf("  Path:     %s\n", url->path);
  printf("  Query:    %s\n", url->query);
  printf("  Fragment: %s\n", url->fragment);
  vex_url_free(url);

  // Query parsing
  VexMap *params = vex_url_parse_query("key1=value1&key2=value2&name=Alice");
  assert(params != NULL);
  assert(vex_map_len(params) == 3);
  assert(strcmp((char *)vex_map_get(params, "key1"), "value1") == 0);
  assert(strcmp((char *)vex_map_get(params, "name"), "Alice") == 0);
  printf("✓ vex_url_parse_query: %zu params\n", vex_map_len(params));
  vex_map_free(params);
}

void test_path()
{
  printf("\n=== Testing Path Operations ===\n");

  // Join
  char *joined = vex_path_join("/usr/local", "bin");
  assert(strcmp(joined, "/usr/local/bin") == 0);
  vex_free(joined);
  printf("✓ vex_path_join\n");

  // Dirname/basename
  char *dir = vex_path_dirname("/usr/local/bin/vex");
  assert(strcmp(dir, "/usr/local/bin") == 0);
  vex_free(dir);
  printf("✓ vex_path_dirname\n");

  char *base = vex_path_basename("/usr/local/bin/vex");
  assert(strcmp(base, "vex") == 0);
  vex_free(base);
  printf("✓ vex_path_basename\n");

  // Extension
  char *ext = vex_path_extension("test.txt");
  assert(strcmp(ext, ".txt") == 0);
  vex_free(ext);
  printf("✓ vex_path_extension\n");

  // Absolute path
  assert(vex_path_is_absolute("/usr/bin"));
  assert(!vex_path_is_absolute("relative/path"));
  printf("✓ vex_path_is_absolute\n");

  // Check current directory
  assert(vex_path_is_dir("."));
  printf("✓ vex_path_is_dir\n");

  // Temp file/dir
  char *temp_file = vex_path_temp_file("vex_test");
  assert(temp_file != NULL);
  assert(vex_file_exists(temp_file));
  printf("✓ vex_path_temp_file: %s\n", temp_file);
  vex_file_remove(temp_file);
  vex_free(temp_file);

  char *temp_dir = vex_path_temp_dir("vex_test");
  assert(temp_dir != NULL);
  assert(vex_dir_exists(temp_dir));
  printf("✓ vex_path_temp_dir: %s\n", temp_dir);
  vex_dir_remove(temp_dir);
  vex_free(temp_dir);
}

int main()
{
  printf("╔════════════════════════════════════════╗\n");
  printf("║  CPU, SIMD & Extended Features Test   ║\n");
  printf("╚════════════════════════════════════════╝\n");

  test_cpu_detection();
  test_runtime_info();
  test_strconv();
  test_url();
  test_path();

  printf("\n╔════════════════════════════════════════╗\n");
  printf("║  All Tests Passed! ✅                  ║\n");
  printf("╚════════════════════════════════════════╝\n");

  return 0;
}
