// swisstable_single.c
// Single-file SwissTable-like hash map with SIMD group scanning and portable fallbacks.
// Features:
//  - Control-byte table with 16-byte groups (Swiss/Abseil style)
//  - SIMD interface: AVX2/SSE2 (x86) and NEON (ARM) paths + scalar fallback
//  - Robin-Hood-lite insertion (grouped probing) and simple rehashing
//
// Build examples:
//   x86 AVX2:   cc -O3 -mavx2 -o demo swisstable_single.c
//   x86 SSE2:   cc -O3 -msse2 -o demo swisstable_single.c
//   AArch64:    cc -O3          -o demo swisstable_single.c
//
// Public API (minimal):
//   typedef struct SwissMap SwissMap;
//   bool  vex_swiss_init(SwissMap* m, size_t initial_capacity);
//   bool  vex_swiss_insert(SwissMap* m, const char* key, void* value);
//   void* vex_swiss_get(const SwissMap* m, const char* key);
//   void  vex_swiss_free(SwissMap* m);
//
// Notes:
//  - This is a minimal educational implementation to show the core ideas.
//  - Keys are `const char*` (null-terminated). Hash: FNV-1a 64-bit (replace with xxhash/city for production).
//  - No delete() in this single-file for brevity; DELETED handling is present for future extension.
//  - Thread-safety: none.
//  - License: CC0 / Public Domain.

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>
#include <string.h>
#include <stdlib.h>

// ===== Internal types =====
typedef struct
{
    uint64_t hash;
    const char *key;
    void *value;
} Entry;

typedef struct
{
    uint8_t *ctrl;   // control bytes (capacity + GROUP_PAD)
    Entry *entries;  // slots
    size_t capacity; // power-of-two
    size_t len;      // number of live entries
} SwissMap;

// Public API uses VexMap (defined in vex.h)
// We'll cast between them

// ===== Tuning & control bytes =====
#define GROUP_SIZE 16
#define GROUP_PAD GROUP_SIZE

#define EMPTY 0x80u
#define DELETED 0xFEu
#define H2_MASK 0x7Fu

static inline uint8_t h2(uint64_t h) { return (uint8_t)((h >> 7) & H2_MASK); }

// ===== Simple 64-bit FNV-1a (replace with xxhash/city for better perf) =====
static inline uint64_t hash64_str(const char *s)
{
    uint64_t h = 1469598103934665603ULL;
    while (*s)
    {
        h ^= (uint8_t)*s++;
        h *= 1099511628211ULL;
    }
    return h;
}

static inline size_t round_pow2(size_t n)
{
    size_t p = 8;
    while (p < n)
        p <<= 1;
    return p;
}

static inline size_t bucket_start(uint64_t h, size_t cap)
{
    return (size_t)h & (cap - 1);
}

// ===== SIMD Interface (16-byte group ops) + fallbacks =====
// We expose two helpers that return a 16-bit mask (LSB = first byte in group):
//   uint32_t simd_group_match_eq(const uint8_t* p, uint8_t byte);
//   uint32_t simd_group_match_any2(const uint8_t* p, uint8_t a, uint8_t b);
// They use AVX2/SSE2/NEON when available, otherwise portable scalar code.

#if defined(__x86_64__) || defined(_M_X64) || defined(__i386__)
#include <immintrin.h> // SSE2/AVX2
#define SIMD_X86 1
#else
#define SIMD_X86 0
#endif

#if defined(__ARM_NEON) || defined(__ARM_NEON__) || defined(__aarch64__)
#include <arm_neon.h>
#define SIMD_NEON 1
#else
#define SIMD_NEON 0
#endif

// ---- x86 path (SSE2 baseline for 16B ops; works under AVX2 too) ----
static inline uint32_t simd_group_match_eq_x86(const uint8_t *p, uint8_t byte)
{
#if SIMD_X86
    __m128i v = _mm_loadu_si128((const __m128i *)p);
    __m128i key = _mm_set1_epi8((char)byte);
    __m128i eq = _mm_cmpeq_epi8(v, key);
    return (uint32_t)_mm_movemask_epi8(eq);
#else
    (void)p;
    (void)byte;
    return 0;
#endif
}
static inline uint32_t simd_group_match_any2_x86(const uint8_t *p, uint8_t a, uint8_t b)
{
#if SIMD_X86
    __m128i v = _mm_loadu_si128((const __m128i *)p);
    __m128i va = _mm_set1_epi8((char)a);
    __m128i vb = _mm_set1_epi8((char)b);
    __m128i eqA = _mm_cmpeq_epi8(v, va);
    __m128i eqB = _mm_cmpeq_epi8(v, vb);
    __m128i orv = _mm_or_si128(eqA, eqB);
    return (uint32_t)_mm_movemask_epi8(orv);
#else
    (void)p;
    (void)a;
    (void)b;
    return 0;
#endif
}

// ---- NEON path ----
// We avoid tricky movemask tricks; we still use NEON equality and then build a mask.
static inline uint32_t simd_group_match_eq_neon(const uint8_t *p, uint8_t byte)
{
#if SIMD_NEON
    uint8x16_t v = vld1q_u8(p);
    uint8x16_t key = vdupq_n_u8(byte);
    uint8x16_t eq = vceqq_u8(v, key); // 0xFF or 0x00 per lane
    // Extract to scalar mask:
    uint8_t tmp[16];
    vst1q_u8(tmp, eq);
    uint32_t m = 0;
    for (int i = 0; i < 16; ++i)
        m |= (uint32_t)(tmp[i] >> 7) << i;
    return m;
#else
    (void)p;
    (void)byte;
    return 0;
#endif
}
static inline uint32_t simd_group_match_any2_neon(const uint8_t *p, uint8_t a, uint8_t b)
{
#if SIMD_NEON
    uint8x16_t v = vld1q_u8(p);
    uint8x16_t va = vdupq_n_u8(a);
    uint8x16_t vb = vdupq_n_u8(b);
    uint8x16_t eqA = vceqq_u8(v, va);
    uint8x16_t eqB = vceqq_u8(v, vb);
    uint8x16_t orv = vorrq_u8(eqA, eqB);
    uint8_t tmp[16];
    vst1q_u8(tmp, orv);
    uint32_t m = 0;
    for (int i = 0; i < 16; ++i)
        m |= (uint32_t)(tmp[i] >> 7) << i;
    return m;
#else
    (void)p;
    (void)a;
    (void)b;
    return 0;
#endif
}

// ---- Scalar fallback ----
static inline uint32_t simd_group_match_eq_scalar(const uint8_t *p, uint8_t byte)
{
    uint32_t m = 0;
    for (int i = 0; i < GROUP_SIZE; ++i)
        m |= (uint32_t)(p[i] == byte) << i;
    return m;
}
static inline uint32_t simd_group_match_any2_scalar(const uint8_t *p, uint8_t a, uint8_t b)
{
    uint32_t m = 0;
    for (int i = 0; i < GROUP_SIZE; ++i)
    {
        uint8_t c = p[i];
        m |= (uint32_t)((c == a) || (c == b)) << i;
    }
    return m;
}

// ---- Unified interface selection ----
static inline uint32_t simd_group_match_eq(const uint8_t *p, uint8_t byte)
{
#if SIMD_X86
    return simd_group_match_eq_x86(p, byte);
#elif SIMD_NEON
    return simd_group_match_eq_neon(p, byte);
#else
    return simd_group_match_eq_scalar(p, byte);
#endif
}
static inline uint32_t simd_group_match_empty_or_deleted(const uint8_t *p)
{
#if SIMD_X86
    return simd_group_match_any2_x86(p, EMPTY, DELETED);
#elif SIMD_NEON
    return simd_group_match_any2_neon(p, EMPTY, DELETED);
#else
    return simd_group_match_any2_scalar(p, EMPTY, DELETED);
#endif
}

// First set bit index (LSB-first). Returns -1 if mask==0.
static inline int first_bit(uint32_t mask)
{
#if defined(_MSC_VER)
    unsigned long idx;
    if (_BitScanForward(&idx, mask))
        return (int)idx;
    return -1;
#else
    if (!mask)
        return -1;
    return __builtin_ctz(mask);
#endif
}

// ===== Forward declarations =====
static bool vex_swiss_init_internal(SwissMap *map, size_t initial_capacity);
static bool vex_swiss_insert_internal(SwissMap *map, const char *key, void *value);
static void *vex_swiss_get_internal(const SwissMap *map, const char *key);
static void vex_swiss_free_internal(SwissMap *map);

// ===== Rehash (grow) =====
static bool vex_swiss_rehash(SwissMap *map, size_t new_cap)
{
    SwissMap nm;
    if (!vex_swiss_init_internal(&nm, new_cap))
        return false;
    for (size_t i = 0; i < map->capacity; ++i)
    {
        uint8_t c = map->ctrl[i];
        if (c != EMPTY && c != DELETED)
        {
            vex_swiss_insert_internal(&nm, map->entries[i].key, map->entries[i].value);
        }
    }
    free(map->ctrl);
    free(map->entries);
    *map = nm;
    return true;
}

// ===== Internal API (keep for testing) =====
static bool vex_swiss_init_internal(SwissMap *map, size_t initial_capacity)
{
    size_t cap = round_pow2(initial_capacity ? initial_capacity : 8);
    map->capacity = cap;
    map->len = 0;
    map->entries = (Entry *)calloc(cap, sizeof(Entry));
    map->ctrl = (uint8_t *)malloc(cap + GROUP_PAD);
    if (!map->entries || !map->ctrl)
        return false;
    memset(map->ctrl, EMPTY, cap + GROUP_PAD);
    return true;
}

static void vex_swiss_free_internal(SwissMap *map)
{
    if (!map)
        return;
    free(map->ctrl);
    free(map->entries);
    map->ctrl = NULL;
    map->entries = NULL;
    map->capacity = map->len = 0;
}

// Wrapper for old test code
bool vex_swiss_insert(SwissMap *map, const char *key, void *value);
void *vex_swiss_get(const SwissMap *map, const char *key);

bool vex_swiss_init(SwissMap *map, size_t initial_capacity)
{
    return vex_swiss_init_internal(map, initial_capacity);
}

void vex_swiss_free(SwissMap *map)
{
    vex_swiss_free_internal(map);
}

// ===== Public API =====
#include "vex.h"

bool vex_map_new(VexMap *map, size_t initial_capacity)
{
    return vex_swiss_init_internal((SwissMap *)map, initial_capacity);
}

bool vex_map_insert(VexMap *map, const char *key, void *value)
{
    return vex_swiss_insert_internal((SwissMap *)map, key, value);
}

void *vex_map_get(const VexMap *map, const char *key)
{
    return vex_swiss_get_internal((const SwissMap *)map, key);
}

size_t vex_map_len(const VexMap *map)
{
    return ((const SwissMap *)map)->len;
}

void vex_map_free(VexMap *map)
{
    vex_swiss_free_internal((SwissMap *)map);
}

// Vec-style API: Create new Map on heap (for Vex builtins)
VexMap *vex_map_create(size_t initial_capacity)
{
    VexMap *map = malloc(sizeof(VexMap));
    if (!map)
        return NULL;

    if (!vex_map_new(map, initial_capacity))
    {
        free(map);
        return NULL;
    }
    return map;
}

// Insert or update (Robin-Hood-lite with grouped probing) - internal
static bool vex_swiss_insert_internal(SwissMap *map, const char *key, void *value)
{
    if ((map->len + 1) * 2 > map->capacity)
    {
        if (!vex_swiss_rehash(map, map->capacity * 2))
            return false;
    }
    const size_t cap = map->capacity;
    uint64_t h = hash64_str(key);
    uint8_t fp = h2(h);
    size_t i = bucket_start(h, cap);

    for (;;)
    {
        const uint8_t *gptr = map->ctrl + (i & (cap - 1));
        // 1) probe candidates with same fingerprint in group
        uint32_t match = simd_group_match_eq(gptr, fp);
        while (match)
        {
            int off = first_bit(match);
            size_t idx = (i + (size_t)off) & (cap - 1);
            if (map->entries[idx].hash == h && strcmp(map->entries[idx].key, key) == 0)
            {
                map->entries[idx].value = value;
                return true;
            }
            match &= (match - 1);
        }
        // 2) any empty/deleted slot in this group? insert there
        uint32_t empty = simd_group_match_empty_or_deleted(gptr);
        if (empty)
        {
            int off = first_bit(empty);
            size_t idx = (i + (size_t)off) & (cap - 1);
            map->ctrl[idx] = fp;
            map->entries[idx].hash = h;
            map->entries[idx].key = key; // NOTE: caller must keep key memory alive
            map->entries[idx].value = value;
            map->len++;
            return true;
        }
        // 3) otherwise jump to next group
        i += GROUP_SIZE;
    }
}

// Lookup - internal
static void *vex_swiss_get_internal(const SwissMap *map, const char *key)
{
    if (!map || map->len == 0)
        return NULL;
    const size_t cap = map->capacity;
    uint64_t h = hash64_str(key);
    uint8_t fp = h2(h);
    size_t i = bucket_start(h, cap);
    size_t scanned = 0;

    for (;;)
    {
        const uint8_t *gptr = map->ctrl + (i & (cap - 1));
        uint32_t match = simd_group_match_eq(gptr, fp);
        while (match)
        {
            int off = first_bit(match);
            size_t idx = (i + (size_t)off) & (cap - 1);
            if (map->entries[idx].hash == h && strcmp(map->entries[idx].key, key) == 0)
                return map->entries[idx].value;
            match &= (match - 1);
        }
        // Early exit: seeing EMPTY in group breaks probe chain.
        uint32_t any = simd_group_match_empty_or_deleted(gptr);
        // If group contains at least one EMPTY (not just DELETED), we can stop.
        // For simplicity in this educational version, we stop on any of (EMPTY|DELETED).
        if (any)
            return NULL;

        i += GROUP_SIZE;
        scanned += GROUP_SIZE;
        if (scanned > cap)
            return NULL; // safety
    }
}

// Wrapper implementations for test code
bool vex_swiss_insert(SwissMap *map, const char *key, void *value)
{
    return vex_swiss_insert_internal(map, key, value);
}

void *vex_swiss_get(const SwissMap *map, const char *key)
{
    return vex_swiss_get_internal(map, key);
}

// End of file
