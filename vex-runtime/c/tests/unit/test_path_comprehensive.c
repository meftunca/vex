// test_path_comprehensive.c - Comprehensive cross-platform path tests
#include "../../vex.h"
#include <stdio.h>
#include <string.h>
#include <assert.h>

#ifdef _WIN32
#define TEST_SEP "\\"
#define TEST_PATH "C:\\test\\path"
#else
#define TEST_SEP "/"
#define TEST_PATH "/test/path"
#endif

int test_count = 0;
int test_passed = 0;

#define TEST(name) printf("\n[TEST] %s...\n", name); test_count++
#define PASS() do { test_passed++; printf("  ✓ PASS\n"); } while(0)
#define FAIL(msg) do { printf("  ✗ FAIL: %s\n", msg); return; } while(0)
#define ASSERT_STR_EQ(a, b) if (strcmp(a, b) != 0) { printf("  Expected: %s\n  Got: %s\n", b, a); FAIL("String mismatch"); }
#define ASSERT_TRUE(cond) if (!(cond)) FAIL("Assertion failed")
#define ASSERT_FALSE(cond) if (cond) FAIL("Assertion should be false")

// Test separator
void test_separator(void)
{
  TEST("vex_path_separator");
  const char *sep = vex_path_separator();
  ASSERT_STR_EQ(sep, TEST_SEP);
  PASS();
}

// Test normalization
void test_normalize(void)
{
  TEST("vex_path_normalize");

  // Test 1: Remove .
  char *result = vex_path_normalize("./a/./b");
  ASSERT_STR_EQ(result, "a" TEST_SEP "b");
  vex_free(result);

  // Test 2: Resolve ..
  result = vex_path_normalize("a/b/../c");
  ASSERT_STR_EQ(result, "a" TEST_SEP "c");
  vex_free(result);

  // Test 3: Remove duplicate separators
#ifdef _WIN32
  result = vex_path_normalize("a\\\\b\\\\\\c");
#else
  result = vex_path_normalize("a///b////c");
#endif
  ASSERT_STR_EQ(result, "a" TEST_SEP "b" TEST_SEP "c");
  vex_free(result);

  // Test 4: Empty path
  result = vex_path_normalize("");
  ASSERT_STR_EQ(result, ".");
  vex_free(result);

  // Test 5: Just dots
  result = vex_path_normalize(".");
  ASSERT_STR_EQ(result, ".");
  vex_free(result);

  PASS();
}

// Test validation
void test_validation(void)
{
  TEST("vex_path_is_valid");

  ASSERT_TRUE(vex_path_is_valid("/path/to/file"));
  ASSERT_TRUE(vex_path_is_valid("relative/path"));
  ASSERT_FALSE(vex_path_is_valid(NULL));
  ASSERT_FALSE(vex_path_is_valid(""));

  PASS();
}

// Test sanitization
void test_sanitize(void)
{
  TEST("vex_path_sanitize");

  char *result;

#ifdef _WIN32
  // Windows: Replace invalid chars
  result = vex_path_sanitize("file<>name.txt");
  ASSERT_TRUE(strchr(result, '<') == NULL);
  ASSERT_TRUE(strchr(result, '>') == NULL);
  vex_free(result);
#endif

  // Should keep valid paths
  result = vex_path_sanitize("valid_file-name.txt");
  ASSERT_STR_EQ(result, "valid_file-name.txt");
  vex_free(result);

  PASS();
}

// Test path manipulation
void test_manipulation(void)
{
  TEST("vex_path_join");

  char *result = vex_path_join("a", "b");
  ASSERT_STR_EQ(result, "a" TEST_SEP "b");
  vex_free(result);

  // Test trailing/leading separators (both relative)
  result = vex_path_join("a" TEST_SEP, "b");
  ASSERT_STR_EQ(result, "a" TEST_SEP "b");
  vex_free(result);

  PASS();
}

void test_basename_dirname(void)
{
  TEST("vex_path_basename/dirname");

  char *base = vex_path_basename("/path/to/file.txt");
  ASSERT_STR_EQ(base, "file.txt");
  vex_free(base);

  char *dir = vex_path_dirname("/path/to/file.txt");
#ifdef _WIN32
  // Windows may differ
  ASSERT_TRUE(strstr(dir, "path"));
  ASSERT_TRUE(strstr(dir, "to"));
#else
  ASSERT_STR_EQ(dir, "/path/to");
#endif
  vex_free(dir);

  PASS();
}

void test_extension_stem(void)
{
  TEST("vex_path_extension/stem");

  char *ext = vex_path_extension("file.txt");
  ASSERT_STR_EQ(ext, ".txt");
  vex_free(ext);

  ext = vex_path_extension("file.tar.gz");
  ASSERT_STR_EQ(ext, ".gz");
  vex_free(ext);

  char *stem = vex_path_stem("file.txt");
  ASSERT_STR_EQ(stem, "file");
  vex_free(stem);

  stem = vex_path_stem("/path/to/file.tar.gz");
  ASSERT_STR_EQ(stem, "file.tar");
  vex_free(stem);

  PASS();
}

void test_is_absolute(void)
{
  TEST("vex_path_is_absolute");

#ifdef _WIN32
  ASSERT_TRUE(vex_path_is_absolute("C:\\path"));
  ASSERT_TRUE(vex_path_is_absolute("\\\\server\\share"));
  ASSERT_FALSE(vex_path_is_absolute("relative\\path"));
#else
  ASSERT_TRUE(vex_path_is_absolute("/absolute/path"));
  ASSERT_FALSE(vex_path_is_absolute("relative/path"));
  ASSERT_FALSE(vex_path_is_absolute("./path"));
#endif

  PASS();
}

// Test components
void test_components(void)
{
  TEST("vex_path_components");

  VexArray *components = vex_path_components("/a/b/c");
  ASSERT_TRUE(components != NULL);

  size_t len = vex_array_len(components);
#ifdef _WIN32
  // May include drive
  ASSERT_TRUE(len >= 3);
#else
  ASSERT_TRUE(len == 3);

  char **comp = (char **)vex_array_get(components, 0, sizeof(char *));
  ASSERT_STR_EQ(*comp, "a");

  comp = (char **)vex_array_get(components, 1, sizeof(char *));
  ASSERT_STR_EQ(*comp, "b");

  comp = (char **)vex_array_get(components, 2, sizeof(char *));
  ASSERT_STR_EQ(*comp, "c");
#endif

  // Free components
  for (size_t i = 0; i < len; i++)
  {
    char **comp = (char **)vex_array_get(components, i, sizeof(char *));
    vex_free(*comp);
  }

  PASS();
}

// Test comparison
void test_comparison(void)
{
  TEST("vex_path_equals");

  ASSERT_TRUE(vex_path_equals("a/b/c", "a/b/c"));
  ASSERT_TRUE(vex_path_equals("a/./b", "a/b"));
  ASSERT_TRUE(vex_path_equals("a/b/../b", "a/b"));
  ASSERT_FALSE(vex_path_equals("a/b", "a/c"));

  PASS();
}

void test_starts_with(void)
{
  TEST("vex_path_starts_with");

  ASSERT_TRUE(vex_path_starts_with("/a/b/c", "/a"));
  ASSERT_TRUE(vex_path_starts_with("/a/b/c", "/a/b"));
  ASSERT_FALSE(vex_path_starts_with("/a/b/c", "/x"));

  PASS();
}

void test_ends_with(void)
{
  TEST("vex_path_ends_with");

  ASSERT_TRUE(vex_path_ends_with("file.txt", ".txt"));
  ASSERT_TRUE(vex_path_ends_with("/path/to/file", "file"));
  ASSERT_FALSE(vex_path_ends_with("file.txt", ".doc"));

  PASS();
}

// Test directory operations
void test_dir_create_remove(void)
{
  TEST("vex_dir_create/remove");

  const char *test_dir = "test_temp_dir_12345";

  // Create
  ASSERT_TRUE(vex_dir_create(test_dir, 0755));
  ASSERT_TRUE(vex_path_exists(test_dir));
  ASSERT_TRUE(vex_path_is_dir(test_dir));

  // Remove
  ASSERT_TRUE(vex_dir_remove(test_dir));
  ASSERT_FALSE(vex_path_exists(test_dir));

  PASS();
}

void test_dir_create_all(void)
{
  TEST("vex_dir_create_all");

  const char *test_path = "test_a" TEST_SEP "test_b" TEST_SEP "test_c";

  // Create nested
  ASSERT_TRUE(vex_dir_create_all(test_path, 0755));
  ASSERT_TRUE(vex_path_exists(test_path));
  ASSERT_TRUE(vex_path_is_dir(test_path));

  // Cleanup
  ASSERT_TRUE(vex_dir_remove_all("test_a"));
  ASSERT_FALSE(vex_path_exists("test_a"));

  PASS();
}

void test_dir_remove_all(void)
{
  TEST("vex_dir_remove_all");

  // Create nested structure
  vex_dir_create_all("test_rm" TEST_SEP "sub1" TEST_SEP "sub2", 0755);
  vex_file_write_all("test_rm" TEST_SEP "file1.txt", "test", 4);
  vex_file_write_all("test_rm" TEST_SEP "sub1" TEST_SEP "file2.txt", "test", 4);

  // Remove all
  ASSERT_TRUE(vex_dir_remove_all("test_rm"));
  ASSERT_FALSE(vex_path_exists("test_rm"));

  PASS();
}

// Test file operations
void test_file_operations(void)
{
  TEST("vex_file_copy/move");

  const char *src = "test_src.txt";
  const char *dst = "test_dst.txt";
  const char *moved = "test_moved.txt";

  // Create source
  vex_file_write_all(src, "test content", 12);
  ASSERT_TRUE(vex_path_exists(src));
  ASSERT_TRUE(vex_path_is_file(src));

  // Copy
  ASSERT_TRUE(vex_file_copy(src, dst));
  ASSERT_TRUE(vex_path_exists(dst));

  size_t size;
  char *content = vex_file_read_all(dst, &size);
  ASSERT_TRUE(size == 12);
  ASSERT_TRUE(strncmp(content, "test content", 12) == 0);
  vex_free(content);

  // Move
  ASSERT_TRUE(vex_file_move(dst, moved));
  ASSERT_TRUE(vex_path_exists(moved));
  ASSERT_FALSE(vex_path_exists(dst));

  // Cleanup
  vex_file_remove(src);
  vex_file_remove(moved);

  PASS();
}

// Test temp files
void test_temp_operations(void)
{
  TEST("vex_path_temp_file/dir");

  // Temp file
  char *temp_file = vex_path_temp_file("test");
  ASSERT_TRUE(temp_file != NULL);
  ASSERT_TRUE(vex_path_exists(temp_file));
  vex_file_remove(temp_file);
  vex_free(temp_file);

  // Temp dir
  char *temp_dir = vex_path_temp_dir("test");
  ASSERT_TRUE(temp_dir != NULL);
  ASSERT_TRUE(vex_path_exists(temp_dir));
  ASSERT_TRUE(vex_path_is_dir(temp_dir));
  vex_dir_remove(temp_dir);
  vex_free(temp_dir);

  PASS();
}

// Test glob
void test_glob(void)
{
  TEST("vex_path_match_glob");

  ASSERT_TRUE(vex_path_match_glob("file.txt", "*.txt"));
  ASSERT_TRUE(vex_path_match_glob("test.c", "test.?"));
  ASSERT_TRUE(vex_path_match_glob("file123.txt", "file[0-9]*.txt"));
  ASSERT_FALSE(vex_path_match_glob("file.doc", "*.txt"));

  PASS();
}

// Test permissions
void test_permissions(void)
{
  TEST("vex_path_is_readable/writable");

#ifndef _WIN32
  // Create test file
  const char *test_file = "test_perms.txt";
  vex_file_write_all(test_file, "test", 4);

  ASSERT_TRUE(vex_path_is_readable(test_file));
  ASSERT_TRUE(vex_path_is_writable(test_file));

  // Change permissions
  vex_path_set_permissions(test_file, 0444); // Read-only
  ASSERT_TRUE(vex_path_is_readable(test_file));
  ASSERT_FALSE(vex_path_is_writable(test_file));

  // Cleanup
  vex_path_set_permissions(test_file, 0644);
  vex_file_remove(test_file);
#endif

  PASS();
}

// Test metadata
void test_metadata(void)
{
  TEST("vex_path_metadata");

  const char *test_file = "test_meta.txt";
  vex_file_write_all(test_file, "test content here", 17);

  VexPathMetadata *meta = vex_path_metadata(test_file);
  ASSERT_TRUE(meta != NULL);
  ASSERT_TRUE(meta->size == 17);
  ASSERT_TRUE(meta->is_file);
  ASSERT_FALSE(meta->is_dir);
  ASSERT_FALSE(meta->is_symlink);

  vex_free(meta);
  vex_file_remove(test_file);

  PASS();
}

int main(void)
{
  printf("═══════════════════════════════════════════════════════════\n");
  printf("  VEX PATH COMPREHENSIVE TESTS\n");
  printf("═══════════════════════════════════════════════════════════\n");

#ifdef _WIN32
  printf("Platform: Windows\n");
#else
  printf("Platform: Unix/Linux/macOS\n");
#endif
  printf("Separator: %s\n", TEST_SEP);

  // Run tests
  test_separator();
  test_normalize();
  test_validation();
  test_sanitize();
  test_manipulation();
  test_basename_dirname();
  test_extension_stem();
  test_is_absolute();
  test_components();
  test_comparison();
  test_starts_with();
  test_ends_with();
  test_dir_create_remove();
  test_dir_create_all();
  test_dir_remove_all();
  test_file_operations();
  test_temp_operations();
  test_glob();
  test_permissions();
  test_metadata();

  // Summary
  printf("\n═══════════════════════════════════════════════════════════\n");
  printf("  RESULTS\n");
  printf("═══════════════════════════════════════════════════════════\n");
  printf("Total tests: %d\n", test_count);
  printf("Passed: %d\n", test_passed);
  printf("Failed: %d\n", test_count - test_passed);

  if (test_passed == test_count)
  {
    printf("\n✅ ALL TESTS PASSED!\n");
    return 0;
  }
  else
  {
    printf("\n❌ SOME TESTS FAILED\n");
    return 1;
  }
}

