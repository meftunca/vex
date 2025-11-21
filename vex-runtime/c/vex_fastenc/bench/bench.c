#include "vex_fastenc.h"
#include "../../vex_allocator.h"
#include <stdio.h>
#include <stdlib.h>
#include <time.h>

static uint64_t now_ns(void)
{
#if defined(_WIN32)
  LARGE_INTEGER f, c;
  QueryPerformanceFrequency(&f);
  QueryPerformanceCounter(&c);
  return (uint64_t)((double)c.QuadPart * 1e9 / (double)f.QuadPart);
#else
  struct timespec ts;
  clock_gettime(CLOCK_MONOTONIC, &ts);
  return (uint64_t)ts.tv_sec * 1000000000ULL + ts.tv_nsec;
#endif
}

int main(void)
{
  size_t n = 1 << 20;
  uint8_t *in = (uint8_t *)vex_malloc(n);
  char *out = (char *)vex_malloc(n * 2 + 16);
  vex_os_random(in, n);

  uint64_t t0 = now_ns();
  size_t m = vex_hex_encode(in, n, out, 0);
  uint64_t t1 = now_ns();
  printf("hex encode: %.2f MB/s\n", (double)n / (double)(t1 - t0) * 1e9 / 1e6);

  uint8_t *back = (uint8_t *)vex_malloc(n);
  t0 = now_ns();
  ssize_t k = vex_hex_decode(out, m, back);
  uint64_t t2 = now_ns();
  printf("hex decode: %.2f MB/s (k=%zd)\n", (double)n / (double)(t2 - t0) * 1e9 / 1e6, k);
  vex_free(in);
  vex_free(out);
  vex_free(back);
  return 0;
}
