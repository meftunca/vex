// main.c
// Derleme örnekleri (Linux/macOS):
//   cc -O3 -mavx2 -o demo main.c swisstable_single.c
//   cc -O3 -msse2 -o demo main.c swisstable_single.c
//   cc -O3        -o demo main.c swisstable_single.c   # AArch64/NEON ya da scalar
//
// Sanitizer ile:
//   cc -O1 -g -fsanitize=address,undefined -o demo main.c swisstable_single.c
//
// Çalıştır:
//   ./demo

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <time.h>
#include "vex.h"

#if defined(__x86_64__) || defined(_M_X64) || defined(__i386__)
#define HOST_X86 1
#else
#define HOST_X86 0
#endif

#if defined(__ARM_NEON) || defined(__ARM_NEON__) || defined(__aarch64__)
#define HOST_NEON 1
#else
#define HOST_NEON 0
#endif

static double now_sec(void)
{
#if defined(_POSIX_C_SOURCE) && _POSIX_C_SOURCE >= 199309L
  struct timespec ts;
  clock_gettime(CLOCK_MONOTONIC, &ts);
  return (double)ts.tv_sec + (double)ts.tv_nsec * 1e-9;
#else
  return (double)clock() / (double)CLOCKS_PER_SEC;
#endif
}

// deterministik pseudo-random (test tekrar edilebilirliği için)
static uint32_t xorshift32(uint32_t *state)
{
  uint32_t x = *state;
  x ^= x << 13;
  x ^= x >> 17;
  x ^= x << 5;
  *state = x;
  return x;
}

// N uzunlukta benzersiz anahtarlar üret (ascii)
static char **gen_keys(size_t N, uint32_t seed)
{
  char **keys = (char **)malloc(N * sizeof(char *));
  if (!keys)
    return NULL;

  // anahtar uzunluklarını küçük varyasyonlarla dağıt
  uint32_t st = seed ? seed : 0x12345678u;
  for (size_t i = 0; i < N; ++i)
  {
    int len = 8 + (int)(xorshift32(&st) % 17);  // 8..24
    char *s = (char *)malloc((size_t)len + 32); // biraz fazla ayır
    if (!s)
    { // cleanup
      for (size_t j = 0; j < i; ++j)
        free(keys[j]);
      free(keys);
      return NULL;
    }
    // kök + sıra no + rastgele parça
    // (benzersizlik için i’yi gömeriz)
    int written = snprintf(s, (size_t)len + 32, "key_%zu_%08x", i, xorshift32(&st));
    // emniyet: truncate etsek bile null sonlanır
    s[written] = '\0';
    keys[i] = s;
  }
  return keys;
}

static void free_keys(char **keys, size_t N)
{
  if (!keys)
    return;
  for (size_t i = 0; i < N; ++i)
    free(keys[i]);
  free(keys);
}

static int smoke_tests(void)
{
  printf("== Smoke tests ==\n");
  VexMap m;
  if (!vex_map_new(&m, 8))
  {
    fprintf(stderr, "map_new failed\n");
    return 1;
  }

  // basit ekleme & get
  int v1 = 42, v2 = 7, v3 = 99;
  if (!vex_map_insert(&m, "hello", &v1))
    return 2;
  if (!vex_map_insert(&m, "world", &v2))
    return 3;

  // update testi (aynı anahtara yeni değer)
  if (!vex_map_insert(&m, "hello", &v3))
    return 4;

  int *p = (int *)vex_map_get(&m, "hello");
  if (!p || *p != 99)
  {
    fprintf(stderr, "update/get failed: expected 99 got %d\n", p ? *p : -1);
    return 5;
  }
  if (vex_map_get(&m, "nope") != NULL)
  {
    fprintf(stderr, "missing key returned non-NULL\n");
    return 6;
  }

  // boş string ve uzun string
  const char *empty = "";
  int ve = 123;
  if (!vex_map_insert(&m, empty, &ve))
    return 7;
  int *pe = (int *)vex_map_get(&m, "");
  if (!pe || *pe != 123)
  {
    fprintf(stderr, "empty key failed\n");
    return 8;
  }
  char longk[1024];
  memset(longk, 'A', sizeof(longk));
  longk[1023] = '\0';
  int vl = 31415;
  if (!vex_map_insert(&m, longk, &vl))
    return 9;
  int *pl = (int *)vex_map_get(&m, longk);
  if (!pl || *pl != 31415)
  {
    fprintf(stderr, "long key failed\n");
    return 10;
  }

  vex_map_free(&m);
  printf("Smoke OK\n");
  return 0;
}

// Büyük hacimli doğrulama ve basit benchmark
static int bulk_tests(size_t N, size_t initial_cap, uint32_t seed)
{
  printf("== Bulk tests: N=%zu initial_cap=%zu ==\n", N, initial_cap);

  VexMap m;
  if (!vex_map_new(&m, initial_cap))
  {
    fprintf(stderr, "map_new failed\n");
    return 1;
  }

  char **keys = gen_keys(N, seed);
  if (!keys)
  {
    fprintf(stderr, "key generation failed\n");
    vex_map_free(&m);
    return 2;
  }

  // değerler için ayrı storage (pointer güvenliği)
  uint64_t *vals = (uint64_t *)malloc(N * sizeof(uint64_t));
  if (!vals)
  {
    fprintf(stderr, "value buffer alloc failed\n");
    free_keys(keys, N);
    vex_map_free(&m);
    return 3;
  }

  double t0 = now_sec();
  for (size_t i = 0; i < N; ++i)
  {
    vals[i] = (uint64_t)(i ^ 0xA5A5A5A5u);
    if (!vex_map_insert(&m, keys[i], &vals[i]))
    {
      fprintf(stderr, "insert failed at %zu\n", i);
      free(vals);
      free_keys(keys, N);
      vex_map_free(&m);
      return 4;
    }
  }
  double t1 = now_sec();

  // doğrulama
  size_t missing = 0, wrong = 0;
  for (size_t i = 0; i < N; ++i)
  {
    uint64_t *p = (uint64_t *)vex_map_get(&m, keys[i]);
    if (!p)
      missing++;
    else if (*p != vals[i])
      wrong++;
  }
  double t2 = now_sec();

  // bazılarını güncelle
  for (size_t i = 0; i < N; i += 3)
  {
    vals[i] ^= 0xFFFFFFFFFFFFFFFFull;
    if (!vex_map_insert(&m, keys[i], &vals[i]))
    {
      fprintf(stderr, "update failed at %zu\n", i);
      free(vals);
      free_keys(keys, N);
      vex_map_free(&m);
      return 5;
    }
  }

  // güncelleme sonrası doğrulama
  size_t wrong2 = 0;
  for (size_t i = 0; i < N; i += 3)
  {
    uint64_t *p = (uint64_t *)vex_map_get(&m, keys[i]);
    if (!p || *p != vals[i])
      wrong2++;
  }
  double t3 = now_sec();

  printf("Inserted: %zu items in %.3f s (%.0f inserts/s)\n",
         N, t1 - t0, (double)N / (t1 - t0));
  printf("Looked up: %zu items in %.3f s (%.0f lookups/s)\n",
         N, t2 - t1, (double)N / (t2 - t1));
  printf("Updated ~%zu items in %.3f s\n", N / 3, t3 - t2);
  printf("Map length reported: %zu\n", vex_map_len(&m));
  printf("Missing=%zu Wrong=%zu WrongAfterUpdate=%zu\n", missing, wrong, wrong2);

  int rc = 0;
  if (missing || wrong || wrong2)
    rc = 6;

  free(vals);
  free_keys(keys, N);
  vex_map_free(&m);
  return rc;
}

// H2 çakışma olasılığını artırmaya çalışan ufak bir test (grup maçını zorlar)
static int h2_pressure_test(size_t N)
{
  printf("== H2 pressure test (same prefixes) N=%zu ==\n", N);
  VexMap m;
  if (!vex_map_new(&m, 16))
  {
    fprintf(stderr, "map_new failed\n");
    return 1;
  }

  char **keys = (char **)malloc(N * sizeof(char *));
  uint64_t *vals = (uint64_t *)malloc(N * sizeof(uint64_t));
  if (!keys || !vals)
  {
    free(keys);
    free(vals);
    vex_map_free(&m);
    return 2;
  }

  for (size_t i = 0; i < N; ++i)
  {
    // benzer ön ek: h2 çarpışma şansını arttırır
    char buf[64];
    snprintf(buf, sizeof(buf), "prefix_collision_key_%zu", i);
    keys[i] = strdup(buf);
    vals[i] = i * 13u + 7u;
    if (!keys[i] || !vex_map_insert(&m, keys[i], &vals[i]))
    {
      fprintf(stderr, "insert failed (pressure) at %zu\n", i);
      for (size_t j = 0; j <= i; ++j)
        free(keys[j]);
      free(keys);
      free(vals);
      vex_map_free(&m);
      return 3;
    }
  }

  // doğrulama
  size_t bad = 0;
  for (size_t i = 0; i < N; ++i)
  {
    uint64_t *p = (uint64_t *)vex_map_get(&m, keys[i]);
    if (!p || *p != vals[i])
      bad++;
  }
  printf("Pressure test bad=%zu\n", bad);

  for (size_t i = 0; i < N; ++i)
    free(keys[i]);
  free(keys);
  free(vals);
  vex_map_free(&m);
  return bad ? 4 : 0;
}

int main(void)
{
#if HOST_X86
#if defined(__AVX2__)
  printf("[CPU] x86 with AVX2 path available\n");
#else
  printf("[CPU] x86 with SSE2 path\n");
#endif
#elif HOST_NEON
  printf("[CPU] ARM/AArch64 with NEON path\n");
#else
  printf("[CPU] Scalar path\n");
#endif

  int rc = 0;
  rc |= smoke_tests();

  // Rehash’i zorlamak için küçük başlangıç kapasitesiyle büyük set
  rc |= bulk_tests(100000, 8, 0xC0FFEEu);   // 100k
  rc |= bulk_tests(200000, 32, 0xBADC0DEu); // 200k

  // Grup/çarpışma davranışını zorla
  rc |= h2_pressure_test(50000);

  if (rc == 0)
  {
    printf("\nALL TESTS PASSED ✅\n");
  }
  else
  {
    printf("\nTESTS FAILED (rc=%d) ❌\n", rc);
  }
  return rc;
}
