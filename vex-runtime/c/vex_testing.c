/* vex_testing.c (C17-stable + TAP/JUnit + Fixtures)
 * Full-featured testing & benchmarking harness for Vex (single-file C)
 * - Subtests (C17-friendly macro), logging, skip
 * - Fixtures: setup_all/teardown_all, setup_each/teardown_each
 * - Reporters: text (default), TAP v13, JUnit XML (JUnit4-compatible)
 * - Fine timer control (Reset/Start/Stop)
 * - Auto-calibration (Go-like b.N), bytes/op & MB/s throughput
 * - Cross-platform: GCC/Clang/MSVC friendly; x86 rdtscp if available
 *
 * Build: cc -O3 -std=c17 -Wall -Wextra vex_testing.c -o vt_demo
 * Demo:  cc -O3 -std=c17 -DVEX_TESTING_DEMO vex_testing.c -o vt_demo && ./vt_demo
 *
 * License: CC0 / Public Domain.
 */

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <time.h>
#include <errno.h>

// Use vex_macros.h if available (Vex runtime integration)
#if __has_include("vex_macros.h")
  #include "vex_macros.h"
  // vex_macros.h provides:
  // - VEX_OS_LINUX, VEX_OS_MACOS, VEX_OS_WINDOWS
  // - VEX_ARCH_X86, VEX_ARCH_ARM
  // - VEX_SIMD_X86, VEX_SIMD_NEON
  // - VEX_PREFETCH, VEX_BARRIER
  
  // Compatibility aliases for vex_testing.c
  #define VEX_LINUX VEX_OS_LINUX
  #define VEX_X86 VEX_SIMD_X86
#else
  // Standalone mode: Define macros locally
  #if defined(__linux__)
    #define VEX_LINUX 1
  #else
    #define VEX_LINUX 0
  #endif

  #if defined(__has_include)
    #if __has_include(<x86intrin.h>)
      #include <x86intrin.h>
      #define VEX_X86 1
    #elif __has_include(<immintrin.h>)
      #include <immintrin.h>
      #define VEX_X86 1
    #else
      #define VEX_X86 0
    #endif
  #else
    #if defined(__x86_64__) || defined(__i386__)
      #include <x86intrin.h>
      #define VEX_X86 1
    #else
      #define VEX_X86 0
    #endif
  #endif
  
  // Prefetch/Barrier macros (if not provided by vex_macros.h)
  #if !defined(__has_builtin)
    #define __has_builtin(x) 0
  #endif

  #if __has_builtin(__builtin_prefetch) || defined(__GNUC__)
    #define VEX_PREFETCH(p, rw, loc) __builtin_prefetch((p), (rw), (loc))
  #else
    #define VEX_PREFETCH(p, rw, loc) ((void)0)
  #endif

  #if defined(_MSC_VER)
    #define VEX_BARRIER() _ReadWriteBarrier()
  #else
    #define VEX_BARRIER() asm volatile("" ::: "memory")
  #endif
#endif

#if VEX_LINUX
#include <unistd.h>
#include <sched.h>
#include <sys/mman.h>
#include <sys/time.h>
#include <sys/resource.h>
#include <pthread.h>
#elif defined(__APPLE__) || defined(__unix__) || defined(__unix)
#include <unistd.h>
#include <pthread.h>
#endif

// Windows threading
#ifdef _WIN32
#include <windows.h>
#include <process.h>
#endif

/* =========================
 * Config macros (tweakable)
 * ========================= */
#ifndef VEX_TEST_ENABLE_RDTSC
#define VEX_TEST_ENABLE_RDTSC 1
#endif

#ifndef VEX_TEST_ENABLE_AFFINITY
#define VEX_TEST_ENABLE_AFFINITY 1
#endif

#ifndef VEX_TEST_DEFAULT_WARMUP
#define VEX_TEST_DEFAULT_WARMUP 1000ULL
#endif

#ifndef VEX_TEST_MAX_SAMPLES
#define VEX_TEST_MAX_SAMPLES 100000
#endif

#ifndef VEX_TEST_JSON_BUFSZ
#define VEX_TEST_JSON_BUFSZ 65536
#endif

#ifndef VEX_TEST_AUTOTGT_NS
#define VEX_TEST_AUTOTGT_NS 1000000000ULL
#endif

#ifndef VEX_TEST_LOGBUF_SZ
#define VEX_TEST_LOGBUF_SZ 8192
#endif

// Note: VEX_PREFETCH and VEX_BARRIER are now provided by vex_macros.h
// or defined above in standalone mode

/* =========================
 * Low-level time utilities
 * ========================= */
static inline uint64_t vex_monotonic_ns(void)
{
#if defined(CLOCK_MONOTONIC_RAW)
  struct timespec ts;
  clock_gettime(CLOCK_MONOTONIC_RAW, &ts);
  return (uint64_t)ts.tv_sec * 1000000000ull + (uint64_t)ts.tv_nsec;
#elif defined(CLOCK_MONOTONIC)
  struct timespec ts;
  clock_gettime(CLOCK_MONOTONIC, &ts);
  return (uint64_t)ts.tv_sec * 1000000000ull + (uint64_t)ts.tv_nsec;
#else
  struct timespec ts;
  timespec_get(&ts, TIME_UTC);
  return (uint64_t)ts.tv_sec * 1000000000ull + (uint64_t)ts.tv_nsec;
#endif
}

static inline uint64_t vex_read_cycles(void)
{
#if VEX_X86 && VEX_TEST_ENABLE_RDTSC
  unsigned aux;
  return __rdtscp(&aux); // serialize on read
#else
  return vex_monotonic_ns();
#endif
}

/* Memory barrier */
static inline void vex_fence_seqcst(void)
{
#if __has_builtin(__sync_synchronize) || defined(__GNUC__)
  __sync_synchronize();
#else
  VEX_BARRIER();
#endif
}

/* Expect/Assume */
static inline int vex_expect(int x, int expected)
{
#if __has_builtin(__builtin_expect)
  return __builtin_expect(x, expected);
#else
  return x;
#endif
}
static inline void vex_assume(int cond)
{
#if __has_builtin(__builtin_assume)
  __builtin_assume(cond);
#elif __has_builtin(__builtin_unreachable)
  if (!cond)
    __builtin_unreachable();
#else
  (void)cond;
#endif
}

/* Black-box DCE barriers */
static inline void *vex_black_box_ptr(void *p)
{
  VEX_BARRIER();
  return p;
}
static inline uint64_t vex_black_box_u64(uint64_t x)
{
  VEX_BARRIER();
  return x;
}
static inline double vex_black_box_f64(double x)
{
  VEX_BARRIER();
  return x;
}

/* Trap/assert */
static inline void vex_trap(void)
{
#if __has_builtin(__builtin_trap)
  __builtin_trap();
#else
  *(volatile int *)0 = 0;
#endif
}

/* =========================
 * Test API + logging/skip
 * ========================= */
typedef void (*vex_test_fn)(void);
typedef struct
{
  const char *name;
  vex_test_fn fn;
} vex_test_case;

/* Reporter kind */
typedef enum
{
  VEX_REP_TEXT = 0,
  VEX_REP_TAP = 1,
  VEX_REP_JUNIT = 2
} vex_reporter_kind;

static vex_reporter_kind vex_pick_reporter(void)
{
  const char *r = getenv("VEX_REPORTER");
  if (!r || !*r)
    return VEX_REP_TEXT;
  if (!strcmp(r, "tap"))
    return VEX_REP_TAP;
  if (!strcmp(r, "junit"))
    return VEX_REP_JUNIT;
  return VEX_REP_TEXT;
}

/* test-local state (thread-local if available) */
#if defined(__STDC_NO_THREADS__)
static struct
{
  const char *current;
  int errors;
  char *logbuf;
  size_t logcap;
  size_t loglen;
} g_tstate;
#define TLS
#else
#define TLS _Thread_local
static TLS struct
{
  const char *current;
  int errors;
  char *logbuf;
  size_t logcap;
  size_t loglen;
} g_tstate;
#endif

#define VEX_TEST(name) static void name(void)
#define VEX_TEST_ENTRY(name) {#name, name}

/* C17-friendly SUBTEST macro */
#define VEX_CAT_(a, b) a##b
#define VEX_CAT(a, b) VEX_CAT_(a, b)
#define VEX_SUBTEST(title, CODE)                         \
  do                                                     \
  {                                                      \
    static void VEX_CAT(_vex_st_, __LINE__)(void) CODE   \
        vex_subtest(title, VEX_CAT(_vex_st_, __LINE__)); \
  } while (0)

/* Append to log buffer (and also print to stderr for live view) */
static void vex_log_appendf(const char *level, const char *fmt, va_list ap)
{
  /* duplicate print to stderr */
  fprintf(stderr, "[%s] %s: ", level, g_tstate.current ? g_tstate.current : "<test>");
  vfprintf(stderr, fmt, ap);
  fputc('\n', stderr);

  if (!g_tstate.logbuf || g_tstate.logcap == 0)
    return;
  size_t left = (g_tstate.logcap - g_tstate.loglen);
  if (left == 0)
    return;
  int n = snprintf(g_tstate.logbuf + g_tstate.loglen, left, "[%s] ", level);
  if (n < 0)
    return;
  size_t used = (size_t)n;
  if (used >= left)
  {
    g_tstate.loglen += left - 1;
    return;
  }
  left -= used;
  g_tstate.loglen += used;

  n = vsnprintf(g_tstate.logbuf + g_tstate.loglen, left, fmt, ap);
  if (n < 0)
    return;
  used = (size_t)n;
  if (used >= left)
  {
    g_tstate.loglen += left - 1;
    return;
  }
  left -= used;
  g_tstate.loglen += used;

  if (left > 1)
  {
    g_tstate.logbuf[g_tstate.loglen++] = '\n';
    g_tstate.logbuf[g_tstate.loglen] = '\0';
  }
}

#include <stdarg.h>

// Logging macros
#define VEX_TLOG(fmt, ...)                       \
  do                                             \
  {                                              \
    vex_log_raw("LOG", fmt, ##__VA_ARGS__);      \
  } while (0)

#define VEX_TERROR(fmt, ...)                     \
  do                                             \
  {                                              \
    g_tstate.errors++;                           \
    vex_log_raw("ERROR", fmt, ##__VA_ARGS__);    \
  } while (0)

#define VEX_TFAILNOW(fmt, ...)                   \
  do                                             \
  {                                              \
    vex_log_raw("FAIL", fmt, ##__VA_ARGS__);     \
    vex_trap();                                  \
  } while (0)

// Helper for logging with varargs
static void vex_log_raw(const char *level, const char *fmt, ...)
{
  va_list ap;
  va_start(ap, fmt);
  vex_log_appendf(level, fmt, ap);
  va_end(ap);
}
#define VEX_ASSERT(cond)                           \
  do                                               \
  {                                                \
    if (!(cond))                                   \
    {                                              \
      VEX_TFAILNOW("assertion failed: %s", #cond); \
    }                                              \
  } while (0)
#define VEX_SKIP(msg)                                                                                  \
  do                                                                                                   \
  {                                                                                                    \
    fprintf(stdout, "[TEST] %s ... SKIP (%s)\n", g_tstate.current ? g_tstate.current : "<test>", msg); \
    return;                                                                                            \
  } while (0)

/* Fixtures */
typedef struct
{
  void (*setup_all)(void);
  void (*teardown_all)(void);
  void (*setup_each)(void);
  void (*teardown_each)(void);
} vex_fixture;

/* Per-test result for reporters */
typedef struct
{
  const char *name;
  int errors;
  bool skipped;
  char *log; /* owned, may be NULL */
} vex_test_result;

/* Forward decls */
static void vex_subtest(const char *name, vex_test_fn fn);

/* Subtest runner */
static inline void vex_subtest(const char *name, vex_test_fn fn)
{
  const char *prev = g_tstate.current;
  g_tstate.current = name;
  fprintf(stdout, "  [SUBTEST] %s ... ", name);
  fflush(stdout);
  int before_err = g_tstate.errors;
  fn();
  if (g_tstate.errors == before_err)
    fprintf(stdout, "OK\n");
  else
    fprintf(stdout, "FAIL (%d)\n", g_tstate.errors - before_err);
  g_tstate.current = prev;
}

/* ========== Reporters ========== */
static void vex_report_text(const vex_test_result *rs, size_t n)
{
  int failed = 0, skipped = 0;
  fprintf(stdout, "== Summary ==\n");
  for (size_t i = 0; i < n; i++)
  {
    if (rs[i].skipped)
    {
      fprintf(stdout, "[TEST] %s ... SKIP\n", rs[i].name);
      skipped++;
      continue;
    }
    if (rs[i].errors)
    {
      fprintf(stdout, "[TEST] %s ... FAIL (%d)\n", rs[i].name, rs[i].errors);
      failed++;
    }
    else
    {
      fprintf(stdout, "[TEST] %s ... OK\n", rs[i].name);
    }
  }
  fprintf(stdout, "Total: %zu  Failed: %d  Skipped: %d  Passed: %zu\n",
          n, failed, skipped, n - failed - skipped);
}

/* TAP v13 */
static void vex_report_tap(const vex_test_result *rs, size_t n)
{
  fprintf(stdout, "TAP version 13\n");
  fprintf(stdout, "1..%zu\n", n);
  for (size_t i = 0; i < n; i++)
  {
    if (rs[i].skipped)
    {
      fprintf(stdout, "ok %zu - %s # SKIP\n", i + 1, rs[i].name);
      continue;
    }
    if (rs[i].errors == 0)
    {
      fprintf(stdout, "ok %zu - %s\n", i + 1, rs[i].name);
    }
    else
    {
      fprintf(stdout, "not ok %zu - %s\n", i + 1, rs[i].name);
      if (rs[i].log && *rs[i].log)
      {
        /* YAMLish diagnostics block */
        fprintf(stdout, "  ---\n");
        fprintf(stdout, "  log: |\n");
        const char *p = rs[i].log;
        while (*p)
        { /* indent */
          const char *nl = strchr(p, '\n');
          size_t len = nl ? (size_t)(nl - p) : strlen(p);
          fprintf(stdout, "    %.*s\n", (int)len, p);
          if (!nl)
            break;
          p = nl + 1;
        }
        fprintf(stdout, "  ...\n");
      }
    }
  }
}

/* XML escape helper */
static void vex_xml_esc(FILE *f, const char *s)
{
  for (; *s; ++s)
  {
    unsigned char c = (unsigned char)*s;
    switch (c)
    {
    case '&':
      fputs("&amp;", f);
      break;
    case '<':
      fputs("&lt;", f);
      break;
    case '>':
      fputs("&gt;", f);
      break;
    case '"':
      fputs("&quot;", f);
      break;
    case '\'':
      fputs("&apos;", f);
      break;
    default:
      fputc(c, f);
      break;
    }
  }
}

/* JUnit (single testsuite) */
static void vex_report_junit(const char *suite_name, const vex_test_result *rs, size_t n)
{
  int failures = 0, skipped = 0;
  for (size_t i = 0; i < n; i++)
  {
    if (rs[i].skipped)
      skipped++;
    else if (rs[i].errors)
      failures++;
  }
  const char *out = getenv("VEX_JUNIT_FILE");
  FILE *fp = (out && *out) ? fopen(out, "wb") : stdout;
  if (!fp)
    fp = stdout;

  fprintf(fp, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
  fprintf(fp, "<testsuite name=\"");
  vex_xml_esc(fp, suite_name ? suite_name : "vex");
  fprintf(fp, "\" tests=\"%zu\" failures=\"%d\" skipped=\"%d\">\n", n, failures, skipped);

  for (size_t i = 0; i < n; i++)
  {
    fprintf(fp, "  <testcase classname=\"");
    vex_xml_esc(fp, suite_name ? suite_name : "vex");
    fprintf(fp, "\" name=\"");
    vex_xml_esc(fp, rs[i].name);
    fprintf(fp, "\">");
    if (rs[i].skipped)
    {
      fprintf(fp, "<skipped/>");
    }
    else if (rs[i].errors)
    {
      fprintf(fp, "<failure message=\"%d error(s)\">", rs[i].errors);
      if (rs[i].log && *rs[i].log)
        vex_xml_esc(fp, rs[i].log);
      fprintf(fp, "</failure>");
    }
    else
    {
      /* success: nothing */
    }
    fprintf(fp, "</testcase>\n");
  }
  fprintf(fp, "</testsuite>\n");
  if (fp != stdout)
    fclose(fp);
}

/* =========================
 * CPU pinning / priority
 * ========================= */
static inline void vex_pin_to_cpu(int cpu)
{
#if VEX_LINUX && VEX_TEST_ENABLE_AFFINITY
  cpu_set_t set;
  CPU_ZERO(&set);
  CPU_SET(cpu, &set);
  (void)sched_setaffinity(0, sizeof(set), &set);
#else
  (void)cpu;
#endif
}

static inline void vex_set_realtime_hint(void)
{
#if VEX_LINUX
  struct sched_param sp;
  memset(&sp, 0, sizeof(sp));
  sp.sched_priority = 1;
  (void)sched_setscheduler(0, SCHED_FIFO, &sp);
  (void)mlockall(MCL_CURRENT | MCL_FUTURE);
#endif
}

/* =========================
 * Aligned allocation (portable)
 * ========================= */
static void *vex_aligned_alloc(size_t alignment, size_t size)
{
#if defined(_MSC_VER)
  return _aligned_malloc(size, alignment);
#elif defined(_POSIX_VERSION)
  void *p = NULL;
  if (posix_memalign(&p, alignment, size) != 0)
    return NULL;
  return p;
#else
#if defined(__STDC_VERSION__) && __STDC_VERSION__ >= 201112L
  if (size % alignment == 0)
    return aligned_alloc(alignment, size);
#endif
  void *base = malloc(size + alignment - 1 + sizeof(void *));
  if (!base)
    return NULL;
  uintptr_t raw = (uintptr_t)base + sizeof(void *);
  uintptr_t aligned = (raw + (alignment - 1)) & ~(uintptr_t)(alignment - 1);
  ((void **)aligned)[-1] = base;
  return (void *)aligned;
#endif
}
static void vex_aligned_free(void *p)
{
#if defined(_MSC_VER)
  _aligned_free(p);
#elif defined(_POSIX_VERSION)
  free(p);
#else
  if (p)
    free(((void **)p)[-1]);
#endif
}

/* =========================
 * Benchmark API (unchanged)
 * ========================= */
typedef void (*vex_bench_fn)(void *ctx);

typedef struct
{
  bool running;
  uint64_t t0_ns, t_accum_ns;
  uint64_t c0, c_accum;
  uint64_t bytes_per_op;
} vex_bench_timer;

static TLS vex_bench_timer *g_bench_timer = NULL;

typedef struct
{
  const char *name;
  uint64_t iters;
  uint64_t time_ns;
  uint64_t warmup_iters;
  uint64_t warmup_ns;
  int pin_cpu;
  int repeats;
  bool report_json;
  bool auto_calibrate;
  uint64_t bytes_per_op;
} vex_bench_cfg;

typedef struct
{
  double ns_per_op;
  double cycles_per_op;
  double mb_per_s;
  uint64_t iters_done;
  uint64_t elapsed_ns;
  uint64_t elapsed_cycles;
  double min_ns, max_ns, mean_ns, stddev_ns, median_ns, p90_ns, p95_ns, p99_ns;
  int samples;
  const char *name;
} vex_bench_res;

/* Timer control API */
static inline void vex_bench_set_bytes(uint64_t bytes_per_op)
{
  if (g_bench_timer)
    g_bench_timer->bytes_per_op = bytes_per_op;
}
static inline void vex_bench_reset_timer(void)
{
  if (g_bench_timer)
  {
    g_bench_timer->t_accum_ns = 0;
    g_bench_timer->c_accum = 0;
  }
}
static inline void vex_bench_start_timer(void)
{
  if (!g_bench_timer || g_bench_timer->running)
    return;
  g_bench_timer->running = true;
  g_bench_timer->t0_ns = vex_monotonic_ns();
  g_bench_timer->c0 = vex_read_cycles();
}
static inline void vex_bench_stop_timer(void)
{
  if (!g_bench_timer || !g_bench_timer->running)
    return;
  uint64_t t1 = vex_monotonic_ns();
  uint64_t c1 = vex_read_cycles();
  g_bench_timer->t_accum_ns += (t1 - g_bench_timer->t0_ns);
  g_bench_timer->c_accum += (c1 - g_bench_timer->c0);
  g_bench_timer->running = false;
}

/* Stats helpers */
static int cmp_u64(const void *a, const void *b)
{
  uint64_t x = *(const uint64_t *)a, y = *(const uint64_t *)b;
  return (x > y) - (x < y);
}
static inline void vex_stats_from_samples(const uint64_t *arr, int n, vex_bench_res *r)
{
  if (n <= 0)
    return;
  uint64_t *tmp = (uint64_t *)malloc((size_t)n * sizeof(uint64_t));
  if (!tmp)
    return;
  memcpy(tmp, arr, (size_t)n * sizeof(uint64_t));
  qsort(tmp, (size_t)n, sizeof(uint64_t), cmp_u64);
  double sum = 0.0, sum2 = 0.0;
  for (int i = 0; i < n; i++)
  {
    sum += (double)tmp[i];
    sum2 += (double)tmp[i] * (double)tmp[i];
  }
  r->min_ns = (double)tmp[0];
  r->max_ns = (double)tmp[n - 1];
  r->mean_ns = sum / (double)n;
  double var = (sum2 / (double)n) - (r->mean_ns * r->mean_ns);
  r->stddev_ns = var > 0 ? sqrt(var) : 0.0;
  r->median_ns = (n % 2) ? (double)tmp[n / 2] : 0.5 * ((double)tmp[n / 2 - 1] + (double)tmp[n / 2]);
  int p90i = (int)floor(0.90 * (n - 1));
  if (p90i < 0)
    p90i = 0;
  if (p90i >= n)
    p90i = n - 1;
  int p95i = (int)floor(0.95 * (n - 1));
  if (p95i < 0)
    p95i = 0;
  if (p95i >= n)
    p95i = n - 1;
  int p99i = (int)floor(0.99 * (n - 1));
  if (p99i < 0)
    p99i = 0;
  if (p99i >= n)
    p99i = n - 1;
  r->p90_ns = (double)tmp[p90i];
  r->p95_ns = (double)tmp[p95i];
  r->p99_ns = (double)tmp[p99i];
  free(tmp);
  r->samples = n;
}

/* Bench core */
static inline void vex_bench_once(
    vex_bench_fn fn, void *ctx, const vex_bench_cfg *cfg,
    uint64_t *out_elapsed_ns, uint64_t *out_elapsed_cycles, uint64_t *out_iters)
{
  vex_bench_timer timer = (vex_bench_timer){0};
  timer.bytes_per_op = cfg->bytes_per_op;
  g_bench_timer = &timer;

  if (cfg->warmup_ns)
  {
    uint64_t t0 = vex_monotonic_ns();
    while (vex_monotonic_ns() - t0 < cfg->warmup_ns)
      fn(ctx);
  }
  else
  {
    uint64_t w = cfg->warmup_iters ? cfg->warmup_iters : VEX_TEST_DEFAULT_WARMUP;
    for (uint64_t i = 0; i < w; i++)
      fn(ctx);
  }

  uint64_t iters_done = 0;
  if (cfg->iters)
  {
    vex_bench_reset_timer();
    vex_bench_start_timer();
    for (uint64_t i = 0; i < cfg->iters; i++)
      fn(ctx);
    vex_bench_stop_timer();
    iters_done = cfg->iters;
  }
  else if (cfg->time_ns)
  {
    uint64_t start_ns = vex_monotonic_ns();
    vex_bench_reset_timer();
    vex_bench_start_timer();
    do
    {
      fn(ctx);
      iters_done++;
    } while (vex_monotonic_ns() - start_ns < cfg->time_ns);
    vex_bench_stop_timer();
  }
  else
  {
    uint64_t target = 100000000ULL;
    uint64_t start_ns = vex_monotonic_ns();
    vex_bench_reset_timer();
    vex_bench_start_timer();
    do
    {
      fn(ctx);
      iters_done++;
    } while (vex_monotonic_ns() - start_ns < target);
    vex_bench_stop_timer();
  }

  *out_elapsed_ns = timer.t_accum_ns;
  *out_elapsed_cycles = timer.c_accum;
  *out_iters = iters_done;
  g_bench_timer = NULL;
}

static inline uint64_t vex_bench_calibrate_iters(vex_bench_fn fn, void *ctx, uint64_t target_ns)
{
  uint64_t n = 1;
  for (;;)
  {
    uint64_t t_ns = 0, t_cy = 0, it = 0;
    vex_bench_cfg tmp = (vex_bench_cfg){0};
    tmp.iters = n;
    vex_bench_once(fn, ctx, &tmp, &t_ns, &t_cy, &it);
    if (t_ns >= target_ns / 8)
    {
      if (t_ns == 0)
      {
        n *= 10;
        continue;
      }
      double scale = (double)target_ns / (double)t_ns;
      uint64_t nn = (uint64_t)(n * scale);
      if (nn < n + 1)
        nn = n + 1;
      tmp.iters = nn;
      vex_bench_once(fn, ctx, &tmp, &t_ns, &t_cy, &it);
      return nn;
    }
    if (n > (1ULL << 60))
      return n;
    n *= 2;
  }
}

static inline vex_bench_res vex_bench_run(vex_bench_fn fn, void *ctx, vex_bench_cfg cfg)
{
  if (cfg.pin_cpu >= 0)
    vex_pin_to_cpu(cfg.pin_cpu);
  vex_set_realtime_hint();

  if (cfg.auto_calibrate && cfg.iters == 0)
  {
    uint64_t target = cfg.time_ns ? cfg.time_ns : VEX_TEST_AUTOTGT_NS;
    cfg.iters = vex_bench_calibrate_iters(fn, ctx, target);
    cfg.time_ns = 0;
  }

  int reps = cfg.repeats > 0 ? cfg.repeats : 5;
  if (reps > VEX_TEST_MAX_SAMPLES)
    reps = VEX_TEST_MAX_SAMPLES;

  uint64_t *samples_ns = (uint64_t *)malloc((size_t)reps * sizeof(uint64_t));
  uint64_t *samples_cy = (uint64_t *)malloc((size_t)reps * sizeof(uint64_t));
  uint64_t *samples_it = (uint64_t *)malloc((size_t)reps * sizeof(uint64_t));
  if (!samples_ns || !samples_cy || !samples_it)
  {
    fprintf(stderr, "vex_bench_run: OOM\n");
    exit(1);
  }

  for (int r = 0; r < reps; r++)
  {
    uint64_t ns = 0, cy = 0, it = 0;
    vex_bench_once(fn, ctx, &cfg, &ns, &cy, &it);
    samples_ns[r] = ns;
    samples_cy[r] = cy;
    samples_it[r] = it;
  }

  vex_bench_res res = (vex_bench_res){0};
  vex_stats_from_samples(samples_ns, reps, &res);
  double mean_iters = 0.0;
  for (int r = 0; r < reps; r++)
    mean_iters += (double)samples_it[r];
  mean_iters /= (double)reps;

  res.ns_per_op = res.mean_ns / (mean_iters > 0 ? mean_iters : 1.0);
#if VEX_X86 && VEX_TEST_ENABLE_RDTSC
  double mean_cy = 0.0;
  for (int r = 0; r < reps; r++)
    mean_cy += (double)samples_cy[r];
  mean_cy /= (double)reps;
  res.cycles_per_op = mean_cy / (mean_iters > 0 ? mean_iters : 1.0);
  res.elapsed_cycles = (uint64_t)mean_cy;
#else
  res.cycles_per_op = 0.0;
  res.elapsed_cycles = 0;
#endif
  res.elapsed_ns = (uint64_t)res.mean_ns;
  res.iters_done = (uint64_t)mean_iters;
  res.name = cfg.name ? cfg.name : "bench";
  res.samples = reps;

  uint64_t bytes_per_op = cfg.bytes_per_op;
  if (bytes_per_op == 0 && g_bench_timer)
    bytes_per_op = g_bench_timer->bytes_per_op;
  if (bytes_per_op)
  {
    double bps = (double)bytes_per_op * (1e9 / res.ns_per_op);
    res.mb_per_s = bps / 1e6;
  }
  else
  {
    res.mb_per_s = 0.0;
  }

  free(samples_ns);
  free(samples_cy);
  free(samples_it);
  return res;
}

static inline void vex_bench_report_text(const vex_bench_res *r)
{
  printf("[BENCH] %s\n", r->name);
  printf("  ns/op:      %.2f\n", r->ns_per_op);
#if VEX_X86 && VEX_TEST_ENABLE_RDTSC
  printf("  cyc/op:     %.2f\n", r->cycles_per_op);
#endif
  if (r->mb_per_s > 0.0)
    printf("  MB/s:       %.2f\n", r->mb_per_s);
  printf("  elapsed(ns): %lu   iters: %lu   samples: %d\n",
         (unsigned long)r->elapsed_ns, (unsigned long)r->iters_done, r->samples);
  printf("  min/med/mean/max (ns): %.0f / %.0f / %.0f / %.0f\n",
         r->min_ns, r->median_ns, r->mean_ns, r->max_ns);
  printf("  p90/p95/p99 (ns): %.0f / %.0f / %.0f\n", r->p90_ns, r->p95_ns, r->p99_ns);
}

static inline const char *vex_bench_report_json(const vex_bench_res *r, char *buf, size_t bufsz)
{
  int n = snprintf(buf, bufsz,
                   "{"
                   "\"name\":\"%s\","
                   "\"ns_per_op\":%.6f,"
                   "\"cycles_per_op\":%.6f,"
                   "\"mb_per_s\":%.6f,"
                   "\"elapsed_ns\":%lu,"
                   "\"iters\":%lu,"
                   "\"samples\":%d,"
                   "\"min_ns\":%.0f,"
                   "\"median_ns\":%.0f,"
                   "\"mean_ns\":%.0f,"
                   "\"max_ns\":%.0f,"
                   "\"p90_ns\":%.0f,"
                   "\"p95_ns\":%.0f,"
                   "\"p99_ns\":%.0f"
                   "}",
                   r->name,
                   r->ns_per_op, r->cycles_per_op, r->mb_per_s,
                   (unsigned long)r->elapsed_ns, (unsigned long)r->iters_done, r->samples,
                   r->min_ns, r->median_ns, r->mean_ns, r->max_ns,
                   r->p90_ns, r->p95_ns, r->p99_ns);
  if (n < 0 || (size_t)n >= bufsz)
    return NULL;
  return buf;
}

/* =========================
 * Test runner (with fixtures & reporters)
 * ========================= */
static inline int vex_run_tests_with(const char *suite_name,
                                     const vex_test_case *tests, size_t count,
                                     const vex_fixture *fixture_opt)
{
  vex_reporter_kind rep = vex_pick_reporter();
  const char *filter = getenv("VEX_TEST_FILTER");

  if (rep == VEX_REP_TAP)
  {
    /* TAP prints header + plan first */
    size_t planned = 0;
    for (size_t i = 0; i < count; i++)
      if (!filter || !*filter || strstr(tests[i].name, filter))
        planned++;
    fprintf(stdout, "TAP version 13\n");
    fprintf(stdout, "1..%zu\n", planned);
  }
  else
  {
    fprintf(stdout, "== Running %zu tests ==\n", count);
  }

  if (fixture_opt && fixture_opt->setup_all)
    fixture_opt->setup_all();

  vex_test_result *results = (vex_test_result *)calloc(count, sizeof(*results));
  if (!results)
  {
    fprintf(stderr, "OOM\n");
    if (fixture_opt && fixture_opt->teardown_all)
      fixture_opt->teardown_all();
    return 1;
  }

  size_t ran = 0;
  for (size_t i = 0; i < count; i++)
  {
    if (filter && *filter && !strstr(tests[i].name, filter))
    {
      results[i].name = tests[i].name;
      results[i].skipped = true;
      continue;
    }

    if (fixture_opt && fixture_opt->setup_each)
      fixture_opt->setup_each();

    /* prepare per-test log buffer */
    char *logbuf = (char *)malloc(VEX_TEST_LOGBUF_SZ);
    if (logbuf)
    {
      logbuf[0] = '\0';
    }
    g_tstate.logbuf = logbuf;
    g_tstate.logcap = logbuf ? VEX_TEST_LOGBUF_SZ : 0;
    g_tstate.loglen = 0;

    g_tstate.current = tests[i].name;
    g_tstate.errors = 0;

    if (rep == VEX_REP_TEXT)
    {
      fprintf(stdout, "[TEST] %s ... ", tests[i].name);
      fflush(stdout);
    }

    tests[i].fn();

    results[i].name = tests[i].name;
    results[i].errors = g_tstate.errors;
    results[i].log = logbuf;

    if (rep == VEX_REP_TEXT)
    {
      if (g_tstate.errors)
        fprintf(stdout, "FAIL (%d)\n", g_tstate.errors);
      else
        fprintf(stdout, "OK\n");
    }
    else if (rep == VEX_REP_TAP)
    {
      if (g_tstate.errors == 0)
        fprintf(stdout, "ok %zu - %s\n", ++ran, tests[i].name);
      else
        fprintf(stdout, "not ok %zu - %s\n", ++ran, tests[i].name);
    }

    if (fixture_opt && fixture_opt->teardown_each)
      fixture_opt->teardown_each();
  }

  if (fixture_opt && fixture_opt->teardown_all)
    fixture_opt->teardown_all();

  /* suite reporting */
  if (rep == VEX_REP_TEXT)
  {
    vex_report_text(results, count);
  }
  else if (rep == VEX_REP_JUNIT)
  {
    vex_report_junit(suite_name ? suite_name : "vex", results, count);
  } /* TAP per-test zaten yazıldı */

  int failed = 0;
  for (size_t i = 0; i < count; i++)
    if (!results[i].skipped && results[i].errors)
      failed++;

  for (size_t i = 0; i < count; i++)
    free(results[i].log);
  free(results);
  return failed;
}

/* Back-compat shim */
static inline int vex_run_tests(const vex_test_case *tests, size_t count)
{
  return vex_run_tests_with("vex", tests, count, NULL);
}

/* Convenience: fixture factory helpers */
static inline vex_fixture vex_fixture_all(void (*setup_all)(void), void (*teardown_all)(void))
{
  vex_fixture f = {setup_all, teardown_all, NULL, NULL};
  return f;
}
static inline vex_fixture vex_fixture_each(void (*setup_each)(void), void (*teardown_each)(void))
{
  vex_fixture f = {NULL, NULL, setup_each, teardown_each};
  return f;
}
static inline vex_fixture vex_fixture_full(void (*setup_all)(void), void (*teardown_all)(void),
                                           void (*setup_each)(void), void (*teardown_each)(void))
{
  vex_fixture f = {setup_all, teardown_all, setup_each, teardown_each};
  return f;
}

/* =========================
 * Parallel Test Runner
 * ========================= */

// Platform-agnostic thread wrapper
#if defined(_WIN32)
typedef HANDLE vex_thread_t;
typedef DWORD vex_thread_result_t;
#define VEX_THREAD_CALL __stdcall
#else
typedef pthread_t vex_thread_t;
typedef void* vex_thread_result_t;
#define VEX_THREAD_CALL
#endif

// Mutex wrapper
#if defined(_WIN32)
typedef CRITICAL_SECTION vex_mutex_t;
static inline void vex_mutex_init(vex_mutex_t *m) { InitializeCriticalSection(m); }
static inline void vex_mutex_lock(vex_mutex_t *m) { EnterCriticalSection(m); }
static inline void vex_mutex_unlock(vex_mutex_t *m) { LeaveCriticalSection(m); }
static inline void vex_mutex_destroy(vex_mutex_t *m) { DeleteCriticalSection(m); }
#else
typedef pthread_mutex_t vex_mutex_t;
static inline void vex_mutex_init(vex_mutex_t *m) { pthread_mutex_init(m, NULL); }
static inline void vex_mutex_lock(vex_mutex_t *m) { pthread_mutex_lock(m); }
static inline void vex_mutex_unlock(vex_mutex_t *m) { pthread_mutex_unlock(m); }
static inline void vex_mutex_destroy(vex_mutex_t *m) { pthread_mutex_destroy(m); }
#endif

// Parallel test context
typedef struct {
  const vex_test_case *tests;
  size_t n_tests;
  const vex_fixture *fx;
  vex_reporter_kind reporter;
  
  // Shared state (protected by mutex)
  vex_mutex_t mutex;
  size_t next_test_idx;
  int total_failed;
  vex_test_result *results;
  
  // Thread-local results
  int thread_id;
} vex_parallel_ctx;

// Thread worker function
static VEX_THREAD_CALL vex_thread_result_t vex_test_worker(void *arg) {
  vex_parallel_ctx *ctx = (vex_parallel_ctx *)arg;
  
  while (1) {
    // Get next test index (thread-safe)
    vex_mutex_lock(&ctx->mutex);
    size_t idx = ctx->next_test_idx++;
    vex_mutex_unlock(&ctx->mutex);
    
    if (idx >= ctx->n_tests) {
      break; // No more tests
    }
    
    const vex_test_case *tc = &ctx->tests[idx];
    
    // Run setup_each (if exists)
    if (ctx->fx && ctx->fx->setup_each) {
      ctx->fx->setup_each();
    }
    
    // Initialize thread-local test state
    g_tstate.current = tc->name;
    g_tstate.errors = 0;
    g_tstate.logbuf = (char *)malloc(VEX_TEST_LOGBUF_SZ);
    g_tstate.logcap = VEX_TEST_LOGBUF_SZ;
    g_tstate.loglen = 0;
    if (g_tstate.logbuf) {
      g_tstate.logbuf[0] = '\0';
    }
    
    // Run the test
    tc->fn();
    
    // Store result (thread-safe)
    vex_mutex_lock(&ctx->mutex);
    vex_test_result *r = &ctx->results[idx];
    r->name = tc->name;
    r->errors = g_tstate.errors;
    r->skipped = false; // TODO: detect skip
    r->log = g_tstate.logbuf;
    if (r->errors > 0) {
      ctx->total_failed++;
    }
    vex_mutex_unlock(&ctx->mutex);
    
    // Reset thread-local state
    g_tstate.logbuf = NULL;
    g_tstate.logcap = 0;
    g_tstate.loglen = 0;
    
    // Run teardown_each (if exists)
    if (ctx->fx && ctx->fx->teardown_each) {
      ctx->fx->teardown_each();
    }
  }
  
#if defined(_WIN32)
  return 0;
#else
  return NULL;
#endif
}

// Create thread
static inline bool vex_thread_create(vex_thread_t *t, VEX_THREAD_CALL vex_thread_result_t (*fn)(void*), void *arg) {
#if defined(_WIN32)
  *t = CreateThread(NULL, 0, (LPTHREAD_START_ROUTINE)fn, arg, 0, NULL);
  return *t != NULL;
#else
  return pthread_create(t, NULL, fn, arg) == 0;
#endif
}

// Join thread
static inline void vex_thread_join(vex_thread_t *t) {
#if defined(_WIN32)
  WaitForSingleObject(*t, INFINITE);
  CloseHandle(*t);
#else
  pthread_join(*t, NULL);
#endif
}

// Main parallel runner
static int vex_run_tests_parallel(const char *suite_name,
                                   const vex_test_case *tests,
                                   size_t n_tests,
                                   const vex_fixture *fx,
                                   int n_threads) {
  if (n_threads <= 0) {
    // Auto-detect CPU count
#if defined(_WIN32)
    SYSTEM_INFO sysinfo;
    GetSystemInfo(&sysinfo);
    n_threads = (int)sysinfo.dwNumberOfProcessors;
#elif defined(_SC_NPROCESSORS_ONLN)
    n_threads = (int)sysconf(_SC_NPROCESSORS_ONLN);
#else
    n_threads = 4; // Fallback
#endif
  }
  
  // Clamp to reasonable range
  if (n_threads > 64) n_threads = 64;
  if (n_threads < 1) n_threads = 1;
  
  vex_reporter_kind reporter = vex_pick_reporter();
  
  // Allocate results array
  vex_test_result *results = (vex_test_result *)calloc(n_tests, sizeof(vex_test_result));
  if (!results) {
    fprintf(stderr, "Failed to allocate results array\n");
    return -1;
  }
  
  // Initialize context
  vex_parallel_ctx ctx = {
    .tests = tests,
    .n_tests = n_tests,
    .fx = fx,
    .reporter = reporter,
    .next_test_idx = 0,
    .total_failed = 0,
    .results = results,
  };
  vex_mutex_init(&ctx.mutex);
  
  // Run setup_all (if exists)
  if (fx && fx->setup_all) {
    fx->setup_all();
  }
  
  // Print header
  switch (reporter) {
    case VEX_REP_TAP:
      printf("TAP version 13\n1..%zu\n", n_tests);
      break;
    case VEX_REP_JUNIT:
      printf("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
      printf("<testsuites name=\"%s\">\n", suite_name);
      printf("  <testsuite name=\"%s\" tests=\"%zu\">\n", suite_name, n_tests);
      break;
    default:
      printf("[PARALLEL] Running %zu tests with %d threads...\n", n_tests, n_threads);
      break;
  }
  
  // Create threads
  vex_thread_t *threads = (vex_thread_t *)malloc(sizeof(vex_thread_t) * (size_t)n_threads);
  if (!threads) {
    fprintf(stderr, "Failed to allocate threads\n");
    free(results);
    vex_mutex_destroy(&ctx.mutex);
    return -1;
  }
  
  for (int i = 0; i < n_threads; i++) {
    if (!vex_thread_create(&threads[i], vex_test_worker, &ctx)) {
      fprintf(stderr, "Failed to create thread %d\n", i);
      // Join already created threads
      for (int j = 0; j < i; j++) {
        vex_thread_join(&threads[j]);
      }
      free(threads);
      free(results);
      vex_mutex_destroy(&ctx.mutex);
      return -1;
    }
  }
  
  // Wait for all threads
  for (int i = 0; i < n_threads; i++) {
    vex_thread_join(&threads[i]);
  }
  
  // Print results
  for (size_t i = 0; i < n_tests; i++) {
    const vex_test_result *r = &results[i];
    switch (reporter) {
      case VEX_REP_TAP:
        if (r->skipped) {
          printf("ok %zu %s # SKIP\n", i + 1, r->name);
        } else if (r->errors == 0) {
          printf("ok %zu %s\n", i + 1, r->name);
        } else {
          printf("not ok %zu %s\n", i + 1, r->name);
          if (r->log && *r->log) {
            printf("# %s\n", r->log);
          }
        }
        break;
      case VEX_REP_JUNIT:
        printf("    <testcase name=\"%s\">\n", r->name);
        if (r->errors > 0) {
          printf("      <failure message=\"%d error(s)\">", r->errors);
          if (r->log && *r->log) {
            printf("%s", r->log);
          }
          printf("</failure>\n");
        } else if (r->skipped) {
          printf("      <skipped/>\n");
        }
        printf("    </testcase>\n");
        break;
      default:
        if (r->skipped) {
          printf("[TEST] %s ... SKIP\n", r->name);
        } else if (r->errors == 0) {
          printf("[TEST] %s ... OK\n", r->name);
        } else {
          printf("[TEST] %s ... FAIL (%d error(s))\n", r->name, r->errors);
        }
        break;
    }
    
    // Free log buffer
    if (r->log) {
      free(r->log);
    }
  }
  
  // Print footer
  switch (reporter) {
    case VEX_REP_JUNIT:
      printf("  </testsuite>\n</testsuites>\n");
      break;
    default:
      printf("[PARALLEL] Finished: %d/%zu failed\n", ctx.total_failed, n_tests);
      break;
  }
  
  // Run teardown_all (if exists)
  if (fx && fx->teardown_all) {
    fx->teardown_all();
  }
  
  // Cleanup
  free(threads);
  free(results);
  vex_mutex_destroy(&ctx.mutex);
  
  return ctx.total_failed;
}

/* =========================
 * Property-Based Testing (QuickCheck-style)
 * ========================= */

// RNG for property tests (xoroshiro128+)
typedef struct {
  uint64_t s[2];
} vex_prng_t;

static inline uint64_t vex_prng_rotl(uint64_t x, int k) {
  return (x << k) | (x >> (64 - k));
}

static inline uint64_t vex_prng_next(vex_prng_t *rng) {
  uint64_t s0 = rng->s[0];
  uint64_t s1 = rng->s[1];
  uint64_t result = s0 + s1;
  s1 ^= s0;
  rng->s[0] = vex_prng_rotl(s0, 24) ^ s1 ^ (s1 << 16);
  rng->s[1] = vex_prng_rotl(s1, 37);
  return result;
}

static inline void vex_prng_seed(vex_prng_t *rng, uint64_t seed) {
  // SplitMix64 for seeding
  uint64_t z = (seed + 0x9e3779b97f4a7c15ULL);
  z = (z ^ (z >> 30)) * 0xbf58476d1ce4e5b9ULL;
  z = (z ^ (z >> 27)) * 0x94d049bb133111ebULL;
  rng->s[0] = z ^ (z >> 31);
  
  z = (rng->s[0] + 0x9e3779b97f4a7c15ULL);
  z = (z ^ (z >> 30)) * 0xbf58476d1ce4e5b9ULL;
  z = (z ^ (z >> 27)) * 0x94d049bb133111ebULL;
  rng->s[1] = z ^ (z >> 31);
}

// Property test context
typedef struct {
  vex_prng_t rng;
  size_t test_count;
  size_t max_tests;
  size_t shrink_count;
  bool failed;
  char fail_msg[256];
} vex_property_ctx;

// Initialize property context
static inline vex_property_ctx vex_property_init(uint64_t seed, size_t max_tests) {
  vex_property_ctx ctx;
  vex_prng_seed(&ctx.rng, seed);
  ctx.test_count = 0;
  ctx.max_tests = max_tests;
  ctx.shrink_count = 0;
  ctx.failed = false;
  ctx.fail_msg[0] = '\0';
  return ctx;
}

// Random generators
static inline int64_t vex_gen_i64(vex_property_ctx *ctx, int64_t min, int64_t max) {
  uint64_t range = (uint64_t)(max - min + 1);
  uint64_t val = vex_prng_next(&ctx->rng) % range;
  return min + (int64_t)val;
}

static inline double vex_gen_f64(vex_property_ctx *ctx, double min, double max) {
  double t = (double)vex_prng_next(&ctx->rng) / (double)UINT64_MAX;
  return min + t * (max - min);
}

static inline bool vex_gen_bool(vex_property_ctx *ctx) {
  return (vex_prng_next(&ctx->rng) & 1) != 0;
}

// Dynamic array for property testing
typedef struct {
  void *data;
  size_t elem_size;
  size_t len;
  size_t cap;
} vex_vec_t;

static inline vex_vec_t vex_vec_new(size_t elem_size, size_t cap) {
  vex_vec_t v;
  v.elem_size = elem_size;
  v.len = 0;
  v.cap = cap;
  v.data = malloc(elem_size * cap);
  return v;
}

static inline void vex_vec_free(vex_vec_t *v) {
  if (v->data) {
    free(v->data);
    v->data = NULL;
  }
  v->len = 0;
  v->cap = 0;
}

static inline void vex_vec_push(vex_vec_t *v, const void *elem) {
  if (v->len >= v->cap) {
    v->cap = v->cap * 2 + 8;
    v->data = realloc(v->data, v->elem_size * v->cap);
  }
  memcpy((char*)v->data + v->len * v->elem_size, elem, v->elem_size);
  v->len++;
}

static inline void* vex_vec_get(vex_vec_t *v, size_t idx) {
  if (idx >= v->len) return NULL;
  return (char*)v->data + idx * v->elem_size;
}

// Generate random vector of i64
static inline vex_vec_t vex_gen_vec_i64(vex_property_ctx *ctx, size_t min_len, size_t max_len, int64_t min_val, int64_t max_val) {
  size_t len = (size_t)vex_gen_i64(ctx, (int64_t)min_len, (int64_t)max_len);
  vex_vec_t v = vex_vec_new(sizeof(int64_t), len);
  for (size_t i = 0; i < len; i++) {
    int64_t val = vex_gen_i64(ctx, min_val, max_val);
    vex_vec_push(&v, &val);
  }
  return v;
}

// Property test macro
#define VEX_PROPERTY(name, iterations, CODE) \
  VEX_TEST(name) { \
    vex_property_ctx prop_ctx = vex_property_init((uint64_t)time(NULL), iterations); \
    for (size_t _i = 0; _i < iterations; _i++) { \
      prop_ctx.test_count = _i; \
      CODE \
      if (prop_ctx.failed) { \
        VEX_TFAILNOW("Property failed at iteration %zu: %s", _i, prop_ctx.fail_msg); \
      } \
    } \
  }

// Property assertion
#define VEX_PROP_ASSERT(ctx, cond, ...) \
  do { \
    if (!(cond)) { \
      (ctx)->failed = true; \
      snprintf((ctx)->fail_msg, sizeof((ctx)->fail_msg), __VA_ARGS__); \
      return; \
    } \
  } while (0)

/* =========================
 * Fuzzing Hooks (libFuzzer/AFL)
 * ========================= */

// Fuzzer entry point (for libFuzzer)
#ifdef VEX_FUZZ_TARGET

#include <stdint.h>
#include <stddef.h>

// User-defined fuzzer target (must be implemented by user)
extern int vex_fuzz_test(const uint8_t *data, size_t size);

// libFuzzer entry point
int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size) {
  return vex_fuzz_test(data, size);
}

// AFL++ entry point (stdin-based)
#ifdef __AFL_COMPILER
__AFL_FUZZ_INIT();
#endif

#endif // VEX_FUZZ_TARGET

// Fuzzer helper: Extract integer from buffer
static inline int64_t vex_fuzz_consume_i64(const uint8_t **data, size_t *size) {
  if (*size < sizeof(int64_t)) {
    return 0;
  }
  int64_t val;
  memcpy(&val, *data, sizeof(int64_t));
  *data += sizeof(int64_t);
  *size -= sizeof(int64_t);
  return val;
}

// Fuzzer helper: Extract bytes
static inline const uint8_t* vex_fuzz_consume_bytes(const uint8_t **data, size_t *size, size_t n) {
  if (*size < n) {
    return NULL;
  }
  const uint8_t *result = *data;
  *data += n;
  *size -= n;
  return result;
}

// Fuzzer helper: Extract string (null-terminated)
static inline const char* vex_fuzz_consume_str(const uint8_t **data, size_t *size, size_t max_len) {
  size_t len = 0;
  while (len < *size && len < max_len && (*data)[len] != '\0') {
    len++;
  }
  if (len == 0 || len >= *size) {
    return NULL;
  }
  const char *str = (const char*)*data;
  *data += len + 1;
  *size -= len + 1;
  return str;
}

/* =========================
 * Demo / Self-test (opt-in)
 * ========================= */
#ifdef VEX_TESTING_DEMO

/* Tiny PRNG */
static inline uint64_t splitmix64(uint64_t *x)
{
  uint64_t z = (*x += 0x9e3779b97f4a7c15ULL);
  z = (z ^ (z >> 30)) * 0xbf58476d1ce4e5b9ULL;
  z = (z ^ (z >> 27)) * 0x94d049bb133111ebULL;
  return z ^ (z >> 31);
}

/* Fixtures */
static int g_demo_resource = 0;
static void demo_setup_all(void) { g_demo_resource = 42; }
static void demo_teardown_all(void) { g_demo_resource = 0; }
static void demo_setup_each(void) { /* e.g., reset per-test state */ }
static void demo_teardown_each(void) { /* e.g., verify invariants */ }

/* Tests */
VEX_TEST(test_math)
{
  VEX_TLOG("suite resource=%d", g_demo_resource);
  VEX_SUBTEST("add", { int a=2,b=3; VEX_ASSERT(a+b==5); });
  VEX_SUBTEST("mul", { int a=2,b=3; VEX_ASSERT(a*b==6); });
  VEX_SUBTEST("skip-demo", { VEX_TLOG("about to skip"); VEX_SKIP("not applicable"); });
}

VEX_TEST(test_fail_demo)
{
  VEX_TERROR("this is a non-fatal error");
  VEX_ASSERT(1 == 1);
}

/* Benchmark */
typedef struct
{
  double *a, *b, *c;
  size_t n;
} SaxpyCtx;
static void saxpy(void *p)
{
  SaxpyCtx *x = (SaxpyCtx *)p;
  vex_bench_start_timer();
  for (size_t i = 0; i < x->n; i++)
  {
    double ai = vex_black_box_f64(x->a[i]);
    double bi = vex_black_box_f64(x->b[i]);
    x->c[i] = ai * 2.0 + bi;
  }
  vex_bench_stop_timer();
  vex_bench_set_bytes((uint64_t)(3 * sizeof(double)) * x->n);
}

int main(void)
{
  const vex_test_case tests[] = {
      VEX_TEST_ENTRY(test_math),
      VEX_TEST_ENTRY(test_fail_demo),
  };
  vex_fixture fx = vex_fixture_full(demo_setup_all, demo_teardown_all,
                                    demo_setup_each, demo_teardown_each);
  int failed = vex_run_tests_with("vex_demo", tests, sizeof(tests) / sizeof(tests[0]), &fx);
  if (failed)
    return 1;

  /* Bench */
  size_t n = 1u << 16;
  double *a = (double *)vex_aligned_alloc(64, n * sizeof(double));
  double *b = (double *)vex_aligned_alloc(64, n * sizeof(double));
  double *c = (double *)vex_aligned_alloc(64, n * sizeof(double));
  if (!a || !b || !c)
  {
    fprintf(stderr, "alloc failed\n");
    return 2;
  }

  uint64_t seed = 1;
  for (size_t i = 0; i < n; i++)
  {
    a[i] = (double)(splitmix64(&seed) % 1000) / 10.0;
    b[i] = (double)(splitmix64(&seed) % 1000) / 10.0;
  }

  SaxpyCtx ctx = {a, b, c, n};
  vex_bench_cfg cfg = {
      .name = "saxpy",
      .iters = 0,
      .time_ns = 0,
      .warmup_iters = 0,
      .warmup_ns = 20000000,
      .pin_cpu = 0,
      .repeats = 5,
      .report_json = false,
      .auto_calibrate = true,
      .bytes_per_op = 0};
  vex_bench_res r = vex_bench_run(saxpy, &ctx, cfg);
  vex_bench_report_text(&r);

  char json[VEX_TEST_JSON_BUFSZ];
  if (vex_bench_report_json(&r, json, sizeof(json)))
    printf("JSON: %s\n", json);

  vex_aligned_free(a);
  vex_aligned_free(b);
  vex_aligned_free(c);
  return 0;
}
#endif /* VEX_TESTING_DEMO */
