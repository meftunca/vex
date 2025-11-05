// vex_testing.c
// Full-featured testing & benchmarking harness for Vex (single-file C)
// Now with: subtests, logging (non-fatal), skip, fine timer control (Reset/Start/Stop),
// auto-calibration (Go-like b.N), bytes/op & MB/s throughput.
//
// Build: cc -O3 -std=c11 -Wall -Wextra vex_testing.c -o vt_demo
// Demo:  cc -O3 -std=c11 -DVEX_TESTING_DEMO vex_testing.c -o vt_demo && ./vt_demo
//
// License: CC0 / Public Domain. Use at your own risk.

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <time.h>
#include <errno.h>

#if defined(__linux__)
#define VEX_LINUX 1
#else
#define VEX_LINUX 0
#endif

#if defined(__x86_64__) || defined(__i386__)
#include <x86intrin.h>
#define VEX_X86 1
#else
#define VEX_X86 0
#endif

#if VEX_LINUX
#include <unistd.h>
#include <sched.h>
#include <sys/mman.h>
#include <sys/time.h>
#include <sys/resource.h>
#if !defined(__GLIBC__) || (__GLIBC__ * 1000 + __GLIBC_MINOR__) < 2016
// mallinfo2 may be unavailable; we gate it later
#endif
#endif

// =========================
// Config macros (tweakable)
// =========================
#ifndef VEX_TEST_ENABLE_RDTSC
#define VEX_TEST_ENABLE_RDTSC 1 // use rdtscp on x86
#endif

#ifndef VEX_TEST_ENABLE_AFFINITY
#define VEX_TEST_ENABLE_AFFINITY 1 // pin thread to a core (Linux)
#endif

#ifndef VEX_TEST_DEFAULT_ITER
#define VEX_TEST_DEFAULT_ITER 100000ULL
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
#define VEX_TEST_AUTOTGT_NS 1000000000ULL // 1s auto-calibration target
#endif

// =========================
// Low-level time utilities
// =========================
static inline uint64_t vex_monotonic_ns(void)
{
#if defined(CLOCK_MONOTONIC_RAW)
  struct timespec ts;
  clock_gettime(CLOCK_MONOTONIC_RAW, &ts);
  return (uint64_t)ts.tv_sec * 1000000000ull + (uint64_t)ts.tv_nsec;
#else
  struct timespec ts;
  clock_gettime(CLOCK_MONOTONIC, &ts);
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

// Memory barrier / fence
static inline void vex_fence_seqcst(void)
{
  __sync_synchronize();
}

// Expect/Assume
static inline int vex_expect(int x, int expected)
{
#if defined(__has_builtin)
#if __has_builtin(__builtin_expect)
  return __builtin_expect(x, expected);
#endif
#endif
  return x;
}
static inline void vex_assume(int cond)
{
#if defined(__has_builtin)
#if __has_builtin(__builtin_assume)
  __builtin_assume(cond);
#elif __has_builtin(__builtin_unreachable)
  if (!cond)
    __builtin_unreachable();
#endif
#endif
}

// Black-box DCE barriers
static inline void *vex_black_box_ptr(void *p)
{
  asm volatile("" : "+r"(p)::"memory");
  return p;
}
static inline uint64_t vex_black_box_u64(uint64_t x)
{
  asm volatile("" : "+r"(x)::"memory");
  return x;
}
static inline double vex_black_box_f64(double x)
{
  asm volatile("" : "+x"(x)::"memory");
  return x;
}

// Trap/assert
static inline void vex_trap(void)
{
#if defined(__has_builtin)
#if __has_builtin(__builtin_trap)
  __builtin_trap();
#else
  *(volatile int *)0 = 0;
#endif
#else
  *(volatile int *)0 = 0;
#endif
}

// =========================
// Test API + logging/skip
// =========================
typedef void (*vex_test_fn)(void);
typedef struct
{
  const char *name;
  vex_test_fn fn;
} vex_test_case;

// simple test-local state (thread-local if available)
#if defined(__STDC_NO_THREADS__)
static struct
{
  const char *current;
  int errors;
} g_tstate;
#define TLS
#else
#define TLS _Thread_local
static TLS struct
{
  const char *current;
  int errors;
} g_tstate;
#endif

#define VEX_TEST(name) static void name(void)
#define VEX_TEST_ENTRY(name) {#name, name}

#define VEX_TLOG(fmt, ...)                                                                                 \
  do                                                                                                       \
  {                                                                                                        \
    fprintf(stderr, "[LOG] %s: " fmt "\n", g_tstate.current ? g_tstate.current : "<test>", ##__VA_ARGS__); \
  } while (0)
#define VEX_TERROR(fmt, ...)                                                                                 \
  do                                                                                                         \
  {                                                                                                          \
    g_tstate.errors++;                                                                                       \
    fprintf(stderr, "[ERROR] %s: " fmt "\n", g_tstate.current ? g_tstate.current : "<test>", ##__VA_ARGS__); \
  } while (0)
#define VEX_TFAILNOW(fmt, ...)                                                                              \
  do                                                                                                        \
  {                                                                                                         \
    fprintf(stderr, "[FAIL] %s: " fmt "\n", g_tstate.current ? g_tstate.current : "<test>", ##__VA_ARGS__); \
    vex_trap();                                                                                             \
  } while (0)
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

static inline int vex_run_tests(const vex_test_case *tests, size_t count)
{
  int failed = 0;
  fprintf(stdout, "== Running %zu tests ==\n", count);
  for (size_t i = 0; i < count; i++)
  {
    g_tstate.current = tests[i].name;
    g_tstate.errors = 0;
    fprintf(stdout, "[TEST] %s ... ", tests[i].name);
    fflush(stdout);
    tests[i].fn();
    if (g_tstate.errors)
    {
      fprintf(stdout, "FAIL (%d)\n", g_tstate.errors);
      failed += 1;
    }
    else
    {
      fprintf(stdout, "OK\n");
    }
  }
  return failed;
}

// =========================
// CPU pinning / priority
// =========================
static inline void vex_pin_to_cpu(int cpu)
{
#if VEX_LINUX && VEX_TEST_ENABLE_AFFINITY
  cpu_set_t set;
  CPU_ZERO(&set);
  CPU_SET(cpu, &set);
  sched_setaffinity(0, sizeof(set), &set);
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
  sched_setscheduler(0, SCHED_FIFO, &sp);
  mlockall(MCL_CURRENT | MCL_FUTURE);
#endif
}

// =========================
// Benchmark API
// =========================
typedef void (*vex_bench_fn)(void *ctx);

// Timer control state (thread-local)
typedef struct
{
  bool running;
  uint64_t t0_ns, t_accum_ns;
  uint64_t c0, c_accum;
  // bytes/op (can be set by user)
  uint64_t bytes_per_op;
} vex_bench_timer;

static TLS vex_bench_timer *g_bench_timer = NULL;

typedef struct
{
  const char *name;
  uint64_t iters;        // fixed iterations; if zero, auto-calibration or time-bound is used
  uint64_t time_ns;      // time-bound target (if iters==0 and auto_calibrate==false)
  uint64_t warmup_iters; // warmup iterations
  uint64_t warmup_ns;    // or warmup time
  int pin_cpu;           // -1: no pin
  int repeats;           // number of repeated measurements
  bool report_json;
  bool auto_calibrate;   // Go-like b.N calibration to reach ~VEX_TEST_AUTOTGT_NS
  uint64_t bytes_per_op; // for throughput (MB/s) reporting; can be overridden at runtime
} vex_bench_cfg;

typedef struct
{
  double ns_per_op;
  double cycles_per_op;
  double mb_per_s; // decimal MB/s using bytes_per_op
  uint64_t iters_done;
  uint64_t elapsed_ns;
  uint64_t elapsed_cycles;
  // stats across repeats
  double min_ns, max_ns, mean_ns, stddev_ns, median_ns, p90_ns, p95_ns, p99_ns;
  int samples;
  const char *name;
} vex_bench_res;

// --- Timer control API (to be used inside benchmark function) ---
static inline void vex_bench_set_bytes(uint64_t bytes_per_op)
{
  if (g_bench_timer)
    g_bench_timer->bytes_per_op = bytes_per_op;
}
static inline void vex_bench_reset_timer(void)
{
  if (!g_bench_timer)
    return;
  g_bench_timer->t_accum_ns = 0;
  g_bench_timer->c_accum = 0;
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

// Helpers: statistics
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

// Core bench once (with timer controls available to fn)
static inline void vex_bench_once(vex_bench_fn fn, void *ctx, const vex_bench_cfg *cfg,
                                  uint64_t *out_elapsed_ns, uint64_t *out_elapsed_cycles, uint64_t *out_iters)
{
  vex_bench_timer timer = {0};
  timer.bytes_per_op = cfg->bytes_per_op;
  g_bench_timer = &timer;

  // Warmup
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

  // measurement window: either fixed iters or time-bound (default)
  if (cfg->iters)
  {
    // allow user to place Reset/Start/Stop around hot section
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
    // default 100ms
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

// Auto-calibration for iters to reach ~target_ns
static inline uint64_t vex_bench_calibrate_iters(vex_bench_fn fn, void *ctx, uint64_t target_ns)
{
  // Start with 1, double until we get reasonable time
  uint64_t n = 1;
  for (;;)
  {
    uint64_t t_ns = 0, t_cy = 0, it = 0;
    vex_bench_cfg tmp = {0};
    tmp.iters = n;
    vex_bench_once(fn, ctx, &tmp, &t_ns, &t_cy, &it);
    if (t_ns >= target_ns / 8)
    {
      // scale proportionally
      if (t_ns == 0)
      {
        n *= 10;
        continue;
      }
      double scale = (double)target_ns / (double)t_ns;
      uint64_t nn = (uint64_t)(n * scale);
      if (nn < n + 1)
        nn = n + 1;
      // re-run with nn to stabilize
      tmp.iters = nn;
      vex_bench_once(fn, ctx, &tmp, &t_ns, &t_cy, &it);
      return nn;
    }
    if (n > (1ULL << 60))
      return n; // guard
    n *= 2;
  }
}

// Public bench run with repeats + stats
static inline vex_bench_res vex_bench_run(vex_bench_fn fn, void *ctx, vex_bench_cfg cfg)
{
  if (cfg.pin_cpu >= 0)
    vex_pin_to_cpu(cfg.pin_cpu);
  vex_set_realtime_hint();

  if (cfg.auto_calibrate && cfg.iters == 0)
  {
    uint64_t target = cfg.time_ns ? cfg.time_ns : VEX_TEST_AUTOTGT_NS;
    cfg.iters = vex_bench_calibrate_iters(fn, ctx, target);
    cfg.time_ns = 0; // use fixed iterations now
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

  // stats over ns
  vex_bench_res res = {0};
  vex_stats_from_samples(samples_ns, reps, &res);
  double mean_iters = 0.0;
  for (int r = 0; r < reps; r++)
    mean_iters += (double)samples_it[r];
  mean_iters /= (double)reps;

  res.ns_per_op = res.mean_ns / (mean_iters > 0 ? mean_iters : 1.0);
#if VEX_X86 && VEX_TEST_ENABLE_RDTSC
  // cycles from mean of samples
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

  // MB/s: decimal MB (1e6) like Go
  uint64_t bytes_per_op = cfg.bytes_per_op;
  if (bytes_per_op == 0 && g_bench_timer)
    bytes_per_op = g_bench_timer->bytes_per_op;
  if (bytes_per_op == 0)
    bytes_per_op = 0; // nothing to report
  if (bytes_per_op)
  {
    double bps = (double)bytes_per_op * (1e9 / res.ns_per_op); // bytes/sec
    res.mb_per_s = bps / 1e6;                                  // MB/s (decimal)
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

// Reporting
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
  // Minimal JSON (no escapes)
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

// =========================
// Demo / Self-test (opt-in)
// =========================
#ifdef VEX_TESTING_DEMO

// A tiny PRNG for demo
static inline uint64_t splitmix64(uint64_t *x)
{
  uint64_t z = (*x += 0x9e3779b97f4a7c15ULL);
  z = (z ^ (z >> 30)) * 0xbf58476d1ce4e5b9ULL;
  z = (z ^ (z >> 27)) * 0x94d049bb133111ebULL;
  return z ^ (z >> 31);
}

// Example tests with subtests/log/skip
VEX_TEST(test_math)
{
  vex_subtest("add", []
              { int a=2,b=3; VEX_ASSERT(a+b==5); });
  vex_subtest("mul", []
              { int a=2,b=3; VEX_ASSERT(a*b==6); });
  vex_subtest("skip-demo", []
              { VEX_TLOG("about to skip"); VEX_SKIP("not applicable"); });
}

// Example benchmark function
typedef struct
{
  double *a, *b, *c;
  size_t n;
} SaxpyCtx;
static void saxpy(void *p)
{
  SaxpyCtx *x = (SaxpyCtx *)p;
  // exclude allocation/init from timing via timer controls
  vex_bench_start_timer();
  for (size_t i = 0; i < x->n; i++)
  {
    x->c[i] = vex_black_box_f64(x->a[i]) * 2.0 + vex_black_box_f64(x->b[i]);
  }
  vex_bench_stop_timer();
  vex_bench_set_bytes((uint64_t)(3 * sizeof(double)) * x->n); // 3 arrays touched
  // timer remains stopped when function returns
}

int main(void)
{
  // Run tests
  const vex_test_case tests[] = {
      VEX_TEST_ENTRY(test_math),
  };
  vex_run_tests(tests, sizeof(tests) / sizeof(tests[0]));

  // Prepare bench data
  size_t n = 1 << 16;
  double *a = aligned_alloc(64, n * sizeof(double));
  double *b = aligned_alloc(64, n * sizeof(double));
  double *c = aligned_alloc(64, n * sizeof(double));
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
      .warmup_ns = 20000000, // 20ms
      .pin_cpu = 0,
      .repeats = 5,
      .report_json = false,
      .auto_calibrate = true, // find iters ~ 1s
      .bytes_per_op = 0       // set inside fn
  };
  vex_bench_res r = vex_bench_run(saxpy, &ctx, cfg);
  vex_bench_report_text(&r);

  char json[VEX_TEST_JSON_BUFSZ];
  if (vex_bench_report_json(&r, json, sizeof(json)))
  {
    printf("JSON: %s\n", json);
  }

  free(a);
  free(b);
  free(c);
  return 0;
}
#endif // VEX_TESTING_DEMO
