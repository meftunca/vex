/* example_tests.c - Comprehensive examples of vex_testing.c features
 * Demonstrates: basic tests, subtests, fixtures, benchmarks, parallel tests,
 * property-based testing, and fuzzing.
 *
 * Build: cc -O3 -std=c17 -I.. example_tests.c ../vex_testing.c -o example_tests -pthread
 * Run:   ./example_tests
 * Parallel: VEX_PARALLEL=4 ./example_tests
 * Reporter: VEX_REPORTER=tap ./example_tests
 */

#include "vex_testing.c"
#include <string.h>

/* ==========================================
 * Example 1: Basic Tests
 * ========================================== */

VEX_TEST(test_basic_assertions)
{
  VEX_TLOG("Testing basic assertions");
  VEX_ASSERT(1 + 1 == 2);
  VEX_ASSERT(strcmp("hello", "hello") == 0);
}

VEX_TEST(test_with_subtests)
{
  VEX_TLOG("Testing with subtests");

  // Note: VEX_SUBTEST uses nested functions (GNU C extension)
  // For C17 portability, use manual subtests:

  {
    VEX_TLOG("Subtest: addition");
    VEX_ASSERT(2 + 3 == 5);
    VEX_ASSERT(-5 + 5 == 0);
  }

  {
    VEX_TLOG("Subtest: multiplication");
    VEX_ASSERT(2 * 3 == 6);
    VEX_ASSERT(-2 * 3 == -6);
  }

  {
    VEX_TLOG("Subtest: division");
    VEX_ASSERT(10 / 2 == 5);
    VEX_ASSERT(7 / 2 == 3); // Integer division
  }
}

VEX_TEST(test_skip_example)
{
  VEX_SKIP("This test is intentionally skipped");
  VEX_ASSERT(false); // Never reached
}

/* ==========================================
 * Example 2: Fixtures
 * ========================================== */

static int global_counter = 0;
static void *test_buffer = NULL;

static void setup_all(void)
{
  printf("[FIXTURE] setup_all() called\n");
  global_counter = 100;
}

static void teardown_all(void)
{
  printf("[FIXTURE] teardown_all() called\n");
  global_counter = 0;
}

static void setup_each(void)
{
  printf("[FIXTURE] setup_each() called\n");
  test_buffer = vex_malloc(1024);
}

static void teardown_each(void)
{
  printf("[FIXTURE] teardown_each() called\n");
  if (test_buffer)
  {
    vex_free(test_buffer);
    test_buffer = NULL;
  }
}

VEX_TEST(test_with_fixtures_1)
{
  VEX_ASSERT(global_counter == 100);
  VEX_ASSERT(test_buffer != NULL);
}

VEX_TEST(test_with_fixtures_2)
{
  VEX_ASSERT(global_counter == 100);
  VEX_ASSERT(test_buffer != NULL);
}

/* ==========================================
 * Example 3: Benchmarks
 * ========================================== */

// Simple string copy benchmark
static void bench_strcpy(void *ctx)
{
  size_t n = *(size_t *)ctx;
  char *src = (char *)vex_malloc(n);
  char *dst = (char *)vex_malloc(n);

  memset(src, 'A', n);
  src[n - 1] = '\0';

  vex_bench_reset_timer();
  vex_bench_start_timer();

  strcpy(dst, src);

  vex_bench_stop_timer();
  vex_bench_set_bytes(n);

  vex_free(src);
  vex_free(dst);
}

// Matrix multiplication benchmark
typedef struct
{
  double *A, *B, *C;
  size_t n;
} MatMulCtx;

static void bench_matmul(void *p)
{
  MatMulCtx *ctx = (MatMulCtx *)p;
  size_t n = ctx->n;

  vex_bench_start_timer();

  for (size_t i = 0; i < n; i++)
  {
    for (size_t j = 0; j < n; j++)
    {
      double sum = 0.0;
      for (size_t k = 0; k < n; k++)
      {
        sum += ctx->A[i * n + k] * ctx->B[k * n + j];
      }
      ctx->C[i * n + j] = sum;
    }
  }

  vex_bench_stop_timer();
  vex_bench_set_bytes(3 * n * n * sizeof(double)); // A + B + C
}

/* ==========================================
 * Example 4: Property-Based Testing
 * ========================================== */

// Property: reverse(reverse(x)) == x
static int my_reverse(int *arr, size_t n)
{
  for (size_t i = 0; i < n / 2; i++)
  {
    int tmp = arr[i];
    arr[i] = arr[n - 1 - i];
    arr[n - 1 - i] = tmp;
  }
  return 0;
}

VEX_PROPERTY(test_reverse_involution, 100, {
  // Generate random array
  vex_vec_t vec = vex_gen_vec_i64(&prop_ctx, 0, 20, -1000, 1000);

  // Make copy
  int64_t *original = (int64_t *)vex_malloc(vec.len * sizeof(int64_t));
  memcpy(original, vec.data, vec.len * sizeof(int64_t));

  // Reverse twice
  my_reverse((int *)vec.data, vec.len);
  my_reverse((int *)vec.data, vec.len);

  // Check invariant: reverse(reverse(x)) == x
  bool equal = memcmp(original, vec.data, vec.len * sizeof(int64_t)) == 0;
  VEX_PROP_ASSERT(&prop_ctx, equal, "reverse(reverse(x)) != x");

  vex_free(original);
  vex_vec_free(&vec);
})

// Property: sorted array stays sorted
static void bubble_sort(int64_t *arr, size_t n)
{
  for (size_t i = 0; i < n; i++)
  {
    for (size_t j = 0; j < n - 1 - i; j++)
    {
      if (arr[j] > arr[j + 1])
      {
        int64_t tmp = arr[j];
        arr[j] = arr[j + 1];
        arr[j + 1] = tmp;
      }
    }
  }
}

VEX_PROPERTY(test_sort_is_sorted, 100, {
  // Generate random array
  vex_vec_t vec = vex_gen_vec_i64(&prop_ctx, 0, 50, -10000, 10000);

  // Sort
  bubble_sort((int64_t *)vec.data, vec.len);

  // Check property: arr[i] <= arr[i+1]
  for (size_t i = 0; i < vec.len - 1; i++)
  {
    int64_t *arr = (int64_t *)vec.data;
    VEX_PROP_ASSERT(&prop_ctx, arr[i] <= arr[i + 1],
                    "Array not sorted at index %zu: %lld > %lld", i, (long long)arr[i], (long long)arr[i + 1]);
  }

  vex_vec_free(&vec);
})

/* ==========================================
 * Example 5: Fuzzing
 * ========================================== */

#ifdef VEX_FUZZ_TARGET

// Fuzz target: Parse integer from string
int vex_fuzz_test(const uint8_t *data, size_t size)
{
  if (size < 1 || size > 128)
  {
    return 0; // Skip
  }

  // Create null-terminated string
  char buf[129];
  memcpy(buf, data, size);
  buf[size] = '\0';

  // Try to parse as integer
  char *endptr;
  long val = strtol(buf, &endptr, 10);
  (void)val; // Unused

  // No crash = success
  return 0;
}

#endif // VEX_FUZZ_TARGET

/* ==========================================
 * Main: Run All Tests
 * ========================================== */

int main(void)
{
  // Test suite
  const vex_test_case tests[] = {
      VEX_TEST_ENTRY(test_basic_assertions),
      VEX_TEST_ENTRY(test_with_subtests),
      VEX_TEST_ENTRY(test_skip_example),
      VEX_TEST_ENTRY(test_with_fixtures_1),
      VEX_TEST_ENTRY(test_with_fixtures_2),
      VEX_TEST_ENTRY(test_reverse_involution),
      VEX_TEST_ENTRY(test_sort_is_sorted),
  };

  // Fixtures
  vex_fixture fx = vex_fixture_full(setup_all, teardown_all, setup_each, teardown_each);

  // Check for parallel mode
  const char *parallel_env = getenv("VEX_PARALLEL");
  int n_threads = 0;
  if (parallel_env)
  {
    n_threads = atoi(parallel_env);
  }

  int failed;
  if (n_threads > 0)
  {
    printf("Running tests in parallel with %d threads...\n", n_threads);
    failed = vex_run_tests_parallel("example_suite", tests,
                                    sizeof(tests) / sizeof(tests[0]), &fx, n_threads);
  }
  else
  {
    failed = vex_run_tests_with("example_suite", tests,
                                sizeof(tests) / sizeof(tests[0]), &fx);
  }

  if (failed > 0)
  {
    return 1;
  }

  printf("\n✅ All tests completed successfully!\n");
  printf("Note: Benchmark and fuzzing examples are available separately.\n");
  return 0;
}
