// swisstable_single.c
// Single-file SwissTable-like hash map with SIMD group scanning and portable fallbacks.
// Production-hardened:
//  - Correct probe termination: stop only on EMPTY (not DELETED)
//  - Tombstone-preferred insert within group
//  - 7/8 load factor growth with safe backstop rehash (prevents rare infinite probe)
//  - Safer round_pow2, overflow/alloc checks, OOM hygiene
//  - FoldHash64 (block folding + strong avalanche) as drop-in hasher
//
// Build examples:
//   x86 AVX2:   cc -O3 -mavx2 -o demo main.c swisstable_single.c
//   x86 SSE2:   cc -O3 -msse2 -o demo main.c swisstable_single.c
//   AArch64:    cc -O3        -o demo main.c swisstable_single.c
//
// Notes:
//  - Keys are `const char*` (null-terminated). Hash: FoldHash64 (replaceable).
//  - No delete() here; DELETED kept for possible future use.
//  - Thread-safety: none.
//  - License: CC0 / Public Domain.

#include "vex.h"
#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>
#include <string.h>
#include <stdlib.h>
#include <limits.h>

// vex.h already includes vex_macros.h with:
// - VEX_STATIC_ASSERT, VEX_LIKELY, VEX_UNLIKELY, VEX_RESTRICT

// ===== Internal types =====
typedef struct
{
    uint64_t hash;
    const char *key; // not owned
    void *value;
} Entry;

typedef struct
{
    uint8_t *ctrl;   // control bytes (capacity + GROUP_PAD)
    Entry *entries;  // slots
    size_t capacity; // power-of-two, >= GROUP_SIZE
    size_t len;      // number of live entries
    size_t max_load; // growth threshold (capacity - capacity/8)
} SwissMap;

// ===== Tuning & control bytes =====
#define GROUP_SIZE 16u
#define GROUP_PAD GROUP_SIZE

#define EMPTY 0x80u
#define DELETED 0xFEu
#define H2_MASK 0x7Fu

_Static_assert(GROUP_SIZE == 16, "GROUP_SIZE must be 16");
_Static_assert((EMPTY & 0x80u) == 0x80u, "EMPTY must have high bit set");
_Static_assert((DELETED & 0x80u) == 0x80u, "DELETED must have high bit set");

static inline uint8_t h2(uint64_t h) { return (uint8_t)((h >> 7) & H2_MASK); }

// ===== FoldHash64 (drop-in replacement for FNV-1a) =====
#include <string.h> // memcpy

static inline uint64_t vex_bswap64(uint64_t x)
{
#if defined(__GNUC__) || defined(__clang__)
    return __builtin_bswap64(x);
#elif defined(_MSC_VER)
    return _byteswap_uint64(x);
#else
    return ((x & 0x00000000000000FFull) << 56) |
           ((x & 0x000000000000FF00ull) << 40) |
           ((x & 0x0000000000FF0000ull) << 24) |
           ((x & 0x00000000FF000000ull) << 8) |
           ((x & 0x000000FF00000000ull) >> 8) |
           ((x & 0x0000FF0000000000ull) >> 24) |
           ((x & 0x00FF000000000000ull) >> 40) |
           ((x & 0xFF00000000000000ull) >> 56);
#endif
}
static inline uint64_t vex_load64_le(const void *p)
{
    uint64_t v;
    memcpy(&v, p, sizeof(v));
#if defined(__BYTE_ORDER__) && (__BYTE_ORDER__ == __ORDER_BIG_ENDIAN__)
    v = vex_bswap64(v);
#endif
    return v;
}

// ===== wyhash (public domain, fastest hash for short strings) =====
// https://github.com/wangyi-fudan/wyhash
// ===== wyhash mix (MSVC dostu) =====
#if defined(_MSC_VER) && !defined(__clang__)
#include <intrin.h>
static inline uint64_t _wymix(uint64_t A, uint64_t B)
{
    unsigned __int64 hi, lo = _umul128(A, B, &hi);
    return (uint64_t)(hi ^ lo);
}
#else
static inline uint64_t _wymix(uint64_t A, uint64_t B)
{
    __uint128_t r = ((__uint128_t)A) * B;
    return (uint64_t)(r ^ (r >> 64));
}
#endif
static inline uint64_t _wyr8(const uint8_t *p)
{
    uint64_t v;
    memcpy(&v, p, 8);
    return v;
}
// dosyanın başlarına (statik yardımcılar // simd yardımcılarından önce olabilir)
static inline bool fast_eq_prefix8_safe(const char *a, const char *b)
{
    // maksimum 8 byte karşılaştır; bir yerde '\0' görürsek orada biter.
    for (int i = 0; i < 8; ++i)
    {
        unsigned char ca = (unsigned char)a[i];
        unsigned char cb = (unsigned char)b[i];
        if (ca != cb)
            return false;
        if (ca == 0)
            return true; // ikisi de 0 ise buraya zaten gelmiştik
    }
    return true;
}
static inline uint64_t _wyr4(const uint8_t *p)
{
    uint32_t v;
    memcpy(&v, p, 4);
    return v;
}

static inline uint64_t _wyr3(const uint8_t *p, size_t k)
{
    return (((uint64_t)p[0]) << 16) | (((uint64_t)p[k >> 1]) << 8) | p[k - 1];
}

static inline uint64_t wyhash64(const void *key, size_t len, uint64_t seed)
{
    const uint8_t *p = (const uint8_t *)key;
    seed ^= 0xa0761d6478bd642full;
    uint64_t a, b;

    if (VEX_LIKELY(len <= 16))
    {
        if (VEX_LIKELY(len >= 4))
        {
            a = (_wyr4(p) << 32) | _wyr4(p + ((len >> 3) << 2));
            b = (_wyr4(p + len - 4) << 32) | _wyr4(p + len - 4 - ((len >> 3) << 2));
        }
        else if (VEX_LIKELY(len > 0))
        {
            a = _wyr3(p, len);
            b = 0;
        }
        else
            a = b = 0;
    }
    else
    {
        size_t i = len;
        if (VEX_UNLIKELY(i > 48))
        {
            uint64_t see1 = seed, see2 = seed;
            do
            {
                seed = _wymix(_wyr8(p) ^ 0x2d358dccaa6c78a5ull, _wyr8(p + 8) ^ seed);
                see1 = _wymix(_wyr8(p + 16) ^ 0x8bb84b93962eacc9ull, _wyr8(p + 24) ^ see1);
                see2 = _wymix(_wyr8(p + 32) ^ 0x4b33a62ed433d4a3ull, _wyr8(p + 40) ^ see2);
                p += 48;
                i -= 48;
            } while (VEX_LIKELY(i > 48));
            seed ^= see1 ^ see2;
        }
        while (VEX_UNLIKELY(i > 16))
        {
            seed = _wymix(_wyr8(p) ^ 0x2d358dccaa6c78a5ull, _wyr8(p + 8) ^ seed);
            p += 16;
            i -= 16;
        }
        a = _wyr8(p + i - 16);
        b = _wyr8(p + i - 8);
    }
    return _wymix(0x2d358dccaa6c78a5ull ^ len, _wymix(a ^ 0x2d358dccaa6c78a5ull, b ^ seed));
}

// Fast string hash wrapper (single-pass strlen + hash)
static inline uint64_t hash64_str(const char *s)
{
    // Compute strlen inline
    const char *p = s;
    while (*p)
        p++;
    size_t len = p - s;

    return wyhash64((const void *)s, len, 0);
}

// next power of two >= n, clamped, minimum GROUP_SIZE, and align to GROUP_SIZE
static inline size_t round_pow2(size_t n)
{
    if (n < GROUP_SIZE)
        return GROUP_SIZE;
    if (n > (SIZE_MAX >> 1))
        return (size_t)1 << (8 * sizeof(size_t) - 1); // clamp
    n--;
    n |= n >> 1;
    n |= n >> 2;
    n |= n >> 4;
    n |= n >> 8;
    n |= n >> 16;
#if SIZE_MAX > 0xFFFFFFFFu
    n |= n >> 32;
#endif
    n++;
    size_t rem = n % GROUP_SIZE;
    if (rem)
        n += GROUP_SIZE - rem;
    return n;
}

static inline size_t bucket_start(uint64_t h, size_t cap)
{
    // Return group-aligned starting index
    size_t slot = (size_t)h & (cap - 1);
    return (slot / GROUP_SIZE) * GROUP_SIZE; // Align to group boundary
}

// ===== SIMD Interface (16-byte group ops) + fallbacks =====
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

// ---- x86 path (SSE2 baseline 16B) ----
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

// ---- NEON movemask helper (optimized for ARM64) ----
static inline uint32_t neon_movemask_u8(uint8x16_t input)
{
    // ARM doesn't have native movemask, so we extract high bits manually
    // Method: Use vshrn to collect bits, then extract to scalar

    // Shift right by 7 to get high bit in LSB position
    uint8x16_t highbits = vshrq_n_u8(input, 7);

    // Pack pairs of bytes: 16 bytes -> 8 uint16_t
    uint8x8_t low = vget_low_u8(highbits);
    uint8x8_t high = vget_high_u8(highbits);

    // Build 16-bit result manually from each byte
    uint16_t result = 0;
    result |= ((uint16_t)vget_lane_u8(low, 0)) << 0;
    result |= ((uint16_t)vget_lane_u8(low, 1)) << 1;
    result |= ((uint16_t)vget_lane_u8(low, 2)) << 2;
    result |= ((uint16_t)vget_lane_u8(low, 3)) << 3;
    result |= ((uint16_t)vget_lane_u8(low, 4)) << 4;
    result |= ((uint16_t)vget_lane_u8(low, 5)) << 5;
    result |= ((uint16_t)vget_lane_u8(low, 6)) << 6;
    result |= ((uint16_t)vget_lane_u8(low, 7)) << 7;
    result |= ((uint16_t)vget_lane_u8(high, 0)) << 8;
    result |= ((uint16_t)vget_lane_u8(high, 1)) << 9;
    result |= ((uint16_t)vget_lane_u8(high, 2)) << 10;
    result |= ((uint16_t)vget_lane_u8(high, 3)) << 11;
    result |= ((uint16_t)vget_lane_u8(high, 4)) << 12;
    result |= ((uint16_t)vget_lane_u8(high, 5)) << 13;
    result |= ((uint16_t)vget_lane_u8(high, 6)) << 14;
    result |= ((uint16_t)vget_lane_u8(high, 7)) << 15;

    return (uint32_t)result;
}

// ---- NEON path (optimized with proper movemask) ----
static inline uint32_t simd_group_match_eq_neon(const uint8_t *p, uint8_t byte)
{
#if SIMD_NEON
    uint8x16_t v = vld1q_u8(p);
    uint8x16_t key = vdupq_n_u8(byte);
    uint8x16_t eq = vceqq_u8(v, key);
    return neon_movemask_u8(eq);
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
    return neon_movemask_u8(orv);
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
    for (int i = 0; i < (int)GROUP_SIZE; ++i)
        m |= (uint32_t)(p[i] == byte) << i;
    return m;
}
static inline uint32_t simd_group_match_any2_scalar(const uint8_t *p, uint8_t a, uint8_t b)
{
    uint32_t m = 0;
    for (int i = 0; i < (int)GROUP_SIZE; ++i)
    {
        uint8_t c = p[i];
        m |= (uint32_t)((c == a) || (c == b)) << i;
    }
    return m;
}

// ---- Unified API ----
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
static bool vex_swiss_remove_internal(SwissMap *map, const char *key);
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
            if (!vex_swiss_insert_internal(&nm, map->entries[i].key, map->entries[i].value))
            {
                vex_swiss_free_internal(&nm);
                return false;
            }
        }
    }
    free(map->ctrl);
    free(map->entries);
    *map = nm;
    return true;
}

// ===== Internal API =====
static bool vex_swiss_init_internal(SwissMap *map, size_t initial_capacity)
{
    if (!map)
        return false;

    size_t cap = round_pow2(initial_capacity ? initial_capacity : GROUP_SIZE);
    if (cap < GROUP_SIZE)
        cap = GROUP_SIZE;

    // Overflow guard for ctrl allocation (cap + GROUP_PAD)
    if (cap > SIZE_MAX - GROUP_PAD)
        return false;

    map->capacity = cap;
    map->len = 0;
    map->max_load = cap - (cap >> 3); // 7/8

    map->entries = (Entry *)calloc(cap, sizeof(Entry));
    map->ctrl = (uint8_t *)malloc(cap + GROUP_PAD);

    if (!map->entries || !map->ctrl)
    {
        free(map->entries);
        free(map->ctrl);
        map->entries = NULL;
        map->ctrl = NULL;
        map->capacity = map->len = map->max_load = 0;
        return false;
    }
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
    map->capacity = 0;
    map->len = 0;
    map->max_load = 0;
}

// ===== Public API glue =====

bool vex_map_new(VexMap *map, size_t initial_capacity) { return vex_swiss_init_internal((SwissMap *)map, initial_capacity); }
bool vex_map_insert(VexMap *map, const char *key, void *value) { return vex_swiss_insert_internal((SwissMap *)map, key, value); }
void *vex_map_get(const VexMap *map, const char *key) { return vex_swiss_get_internal((const SwissMap *)map, key); }
bool vex_map_remove(VexMap *map, const char *key) { return vex_swiss_remove_internal((SwissMap *)map, key); }
size_t vex_map_len(const VexMap *map) { return ((const SwissMap *)map)->len; }
void vex_map_free(VexMap *map) { vex_swiss_free_internal((SwissMap *)map); }

VexMap *vex_map_create(size_t initial_capacity)
{
    VexMap *map = (VexMap *)malloc(sizeof(VexMap));
    if (!map)
        return NULL;
    if (!vex_map_new(map, initial_capacity))
    {
        free(map);
        return NULL;
    }
    return map;
}

// ===== Insert or update (grouped probing with backstop) =====
static bool vex_swiss_insert_internal(SwissMap *map, const char *key, void *value)
{
    if (VEX_UNLIKELY(!map || !key))
        return false;

    // Growth check: keep load <= 7/8
    if (VEX_UNLIKELY((map->len + 1) > map->max_load))
    {
        size_t new_cap = (map->capacity < (SIZE_MAX >> 1)) ? (map->capacity << 1) : map->capacity;
        if (new_cap == map->capacity || !vex_swiss_rehash(map, new_cap))
            return false;
    }

    const uint64_t h = hash64_str(key);
    const uint8_t fp = h2(h);

    // Backstop loop: if we ever scan full table without finding a slot, grow and retry
    for (;;)
    {
        const size_t cap = map->capacity;
        size_t i = bucket_start(h, cap);
        size_t scanned = 0;

        for (;;)
        {
            const uint8_t *gptr = map->ctrl + (i & (cap - 1));

            // 1) Update if key already exists in this group
            uint32_t match = simd_group_match_eq(gptr, fp);
            while (match)
            {
                int off = first_bit(match);
                size_t idx = (i + (size_t)off) & (cap - 1);

                // Fast 8-byte prefix check before full strcmp
                const char *k = map->entries[idx].key;
                uint64_t a, b;
                memcpy(&a, k, 8);
                memcpy(&b, key, 8);

                // if (map->entries[idx].hash == h && a == b && strcmp(k, key) == 0)
                // {
                if (map->entries[idx].hash == h &&
                    fast_eq_prefix8_safe(k, key) &&
                    strcmp(k, key) == 0)
                {
                    // hit / update
                    map->entries[idx].value = value;
                    return true;
                }
                match &= (match - 1);
            }

            // 2) Prefer DELETED; otherwise EMPTY
            uint32_t dels = simd_group_match_eq(gptr, DELETED);
            uint32_t empties = simd_group_match_eq(gptr, EMPTY);
            uint32_t target = dels ? dels : empties;

            if (target)
            {
                int off = first_bit(target);
                size_t idx = (i + (size_t)off) & (cap - 1);
                map->ctrl[idx] = fp;
                map->entries[idx].hash = h;
                map->entries[idx].key = key; // caller owns memory
                map->entries[idx].value = value;
                map->len++;
                return true;
            }

            // 3) Next group
            i += GROUP_SIZE;
            scanned += GROUP_SIZE;

            // Prefetch next group (hide L1 miss latency)
            if (VEX_LIKELY(scanned < cap))
            {
                size_t next = (i + GROUP_SIZE) & (cap - 1);
                __builtin_prefetch((const void *)(map->ctrl + next), 0, 1);
                __builtin_prefetch((const void *)(map->entries + next), 0, 1);
            }

            // Backstop: scanned a full table worth of bytes, no slot found
            if (VEX_UNLIKELY(scanned >= cap))
                break;
        }

        // Grow and retry (rare pathological clustering)
        size_t new_cap = (map->capacity < (SIZE_MAX >> 1)) ? (map->capacity << 1) : map->capacity;
        if (new_cap == map->capacity || !vex_swiss_rehash(map, new_cap))
            return false;
        // loop retries in larger table
    }
}

// ===== Lookup =====
static void *vex_swiss_get_internal(const SwissMap *map, const char *key)
{
    if (VEX_UNLIKELY(!map || !key || map->len == 0))
        return NULL;

    const size_t cap = map->capacity;
    const uint64_t h = hash64_str(key);
    const uint8_t fp = h2(h);
    size_t i = bucket_start(h, cap);
    size_t scanned = 0;

    for (;;)
    {
        const uint8_t *gptr = map->ctrl + (i & (cap - 1));

        // check matching fingerprints
        uint32_t match = simd_group_match_eq(gptr, fp);
        while (match)
        {
            int off = first_bit(match);
            size_t idx = (i + (size_t)off) & (cap - 1);

            // Fast 8-byte prefix check before full strcmp
            const char *k = map->entries[idx].key;
            uint64_t a, b;
            memcpy(&a, k, 8);
            memcpy(&b, key, 8);

            if (map->entries[idx].hash == h && a == b && strcmp(k, key) == 0)
                return map->entries[idx].value;
            match &= (match - 1);
        }

        // Early exit ONLY if we see EMPTY in this group (DELETED does not terminate)
        uint32_t empties = simd_group_match_eq(gptr, EMPTY);
        if (empties)
            return NULL;

        i += GROUP_SIZE;
        scanned += GROUP_SIZE;

        // Prefetch next group (critical for lookup performance)
        if (VEX_LIKELY(scanned <= cap))
        {
            size_t next = (i + GROUP_SIZE) & (cap - 1);
            __builtin_prefetch((const void *)(map->ctrl + next), 0, 0);
            __builtin_prefetch((const void *)(map->entries + next), 0, 0);
        }

        if (VEX_UNLIKELY(scanned > cap))
            return NULL; // safety belt
    }
}

// ===== Legacy wrappers for old tests (optional) =====
bool vex_swiss_insert(SwissMap *map, const char *key, void *value) { return vex_swiss_insert_internal(map, key, value); }
void *vex_swiss_get(const SwissMap *map, const char *key) { return vex_swiss_get_internal(map, key); }
bool vex_swiss_init(SwissMap *map, size_t initial_capacity) { return vex_swiss_init_internal(map, initial_capacity); }
void vex_swiss_free(SwissMap *map) { vex_swiss_free_internal(map); }

// ===== Remove/Delete =====
static bool vex_swiss_remove_internal(SwissMap *map, const char *key)
{
    if (VEX_UNLIKELY(!map || !key || map->len == 0))
        return false;

    const size_t cap = map->capacity;
    const uint64_t h = hash64_str(key);
    const uint8_t fp = h2(h);
    size_t i = bucket_start(h, cap);
    size_t scanned = 0;

    for (;;)
    {
        const uint8_t *gptr = map->ctrl + (i & (cap - 1));

        // Check matching fingerprints
        uint32_t match = simd_group_match_eq(gptr, fp);
        while (match)
        {
            int off = first_bit(match);
            size_t idx = (i + (size_t)off) & (cap - 1);

            // Fast 8-byte prefix check before full strcmp
            const char *k = map->entries[idx].key;
            if (map->entries[idx].hash == h &&
                fast_eq_prefix8_safe(k, key) &&
                strcmp(k, key) == 0)
            {
                // Found it - mark as DELETED
                map->ctrl[idx] = DELETED;
                map->entries[idx].key = NULL;
                map->entries[idx].value = NULL;
                map->entries[idx].hash = 0;
                map->len--;
                return true;
            }
            match &= (match - 1);
        }

        // Early exit ONLY if we see EMPTY in this group
        uint32_t empties = simd_group_match_eq(gptr, EMPTY);
        if (empties)
            return false; // not found

        // Next group
        i += GROUP_SIZE;
        scanned += GROUP_SIZE;

        // Safety: prevent infinite loop
        if (VEX_UNLIKELY(scanned >= cap))
            return false;
    }
}

bool vex_swiss_remove(SwissMap *map, const char *key) { return vex_swiss_remove_internal(map, key); }
