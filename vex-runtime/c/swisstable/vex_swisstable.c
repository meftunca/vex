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

#if defined(__GNUC__) || defined(__clang__)
#define VEX_ALWAYS_INLINE __attribute__((always_inline)) inline
#elif defined(_MSC_VER)
#define VEX_ALWAYS_INLINE __forceinline
#else
#define VEX_ALWAYS_INLINE inline
#endif

// ===== Internal types =====
typedef struct
{
    uint64_t hash;
    const char *key; // not owned
    size_t len;      // <--- ADDED: Cache friendly filtering
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

_Static_assert(GROUP_SIZE == 16 || GROUP_SIZE == 32, "GROUP_SIZE must be 16 or 32");
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
// Optimized key equality check (Phase 1 optimization)
// Eliminates redundant work: checks 8-byte chunks first, then remaining bytes
static inline bool fast_key_eq(const char *a, const char *b, size_t len)
{
    // Fast path: 8+ byte keys (most common case)
    if (VEX_LIKELY(len >= 8))
    {
        // Quick reject: compare first 8 bytes as uint64_t
        uint64_t a_word, b_word;
        memcpy(&a_word, a, 8);
        memcpy(&b_word, b, 8);
        if (a_word != b_word)
            return false;
        
        if (len == 8)
            return true;
        
        // Check remaining bytes (len > 8)
        return memcmp(a + 8, b + 8, len - 8) == 0;
    }
    
    // Medium keys: 4-7 bytes
    if (len >= 4)
    {
        uint32_t a_word, b_word;
        memcpy(&a_word, a, 4);
        memcpy(&b_word, b, 4);
        if (a_word != b_word)
            return false;
        
        if (len == 4)
            return true;
        
        return memcmp(a + 4, b + 4, len - 4) == 0;
    }
    
    // Small keys: 0-3 bytes (fallback)
    return memcmp(a, b, len) == 0;
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



static VEX_ALWAYS_INLINE uint64_t wyhash64(const void *key, size_t len, uint64_t seed)
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
static VEX_ALWAYS_INLINE uint32_t neon_movemask_u8(uint8x16_t input)
{
    // Optimized movemask for AArch64 using ADDV (Horizontal Add)
    // We mask with powers of 2, then sum the bytes.
    // Since the sum of lower 8 lanes fits in 8 bits, and upper 8 lanes fits in 8 bits,
    // we can use pairwise addition or split-and-add.
    
    const uint8_t __attribute__((aligned(16))) powers[16] =
        {1, 2, 4, 8, 16, 32, 64, 128, 1, 2, 4, 8, 16, 32, 64, 128};
    uint8x16_t mask_vec = vld1q_u8(powers);
    uint8x16_t bits = vandq_u8(input, mask_vec);

    // Sum lower 8 bytes and upper 8 bytes separately
    // vpaddq_u8 pairs adjacent bytes: [b0+b1, b2+b3, ..., b14+b15] (16 bytes, but top 8 are garbage/dup?)
    // Actually, simpler:
    
    uint8x8_t low = vget_low_u8(bits);
    uint8x8_t high = vget_high_u8(bits);
    
    // vaddv_u8 sums all elements in 8x8 vector to a scalar
    uint32_t l = vaddv_u8(low);
    uint32_t h = vaddv_u8(high);
    
    return l | (h << 8);
}

// ---- NEON path (optimized with proper movemask) ----
static VEX_ALWAYS_INLINE uint32_t simd_group_match_eq_neon(const uint8_t *p, uint8_t byte)
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
static VEX_ALWAYS_INLINE uint32_t simd_group_match_any2_neon(const uint8_t *p, uint8_t a, uint8_t b)
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
// ---- AVX2 path (32-byte) ----
#if SIMD_AVX2
static inline uint32_t simd_group_match_eq_avx2(const uint8_t *p, uint8_t byte)
{
    __m256i v = _mm256_loadu_si256((const __m256i *)p);
    __m256i key = _mm256_set1_epi8((char)byte);
    __m256i eq = _mm256_cmpeq_epi8(v, key);
    return (uint32_t)_mm256_movemask_epi8(eq);
}
static inline uint32_t simd_group_match_any2_avx2(const uint8_t *p, uint8_t a, uint8_t b)
{
    __m256i v = _mm256_loadu_si256((const __m256i *)p);
    __m256i va = _mm256_set1_epi8((char)a);
    __m256i vb = _mm256_set1_epi8((char)b);
    __m256i eqA = _mm256_cmpeq_epi8(v, va);
    __m256i eqB = _mm256_cmpeq_epi8(v, vb);
    __m256i orv = _mm256_or_si256(eqA, eqB);
    return (uint32_t)_mm256_movemask_epi8(orv);
}
#endif

// ---- Unified API ----
static VEX_ALWAYS_INLINE uint32_t simd_group_match_eq(const uint8_t *p, uint8_t byte)
{
#if SIMD_AVX2
    return simd_group_match_eq_avx2(p, byte);
#elif SIMD_X86
    return simd_group_match_eq_x86(p, byte);
#elif SIMD_NEON
    return simd_group_match_eq_neon(p, byte);
#else
    return simd_group_match_eq_scalar(p, byte);
#endif
}




static VEX_ALWAYS_INLINE uint32_t simd_group_match_empty_or_deleted(const uint8_t *p)
{
#if SIMD_AVX2
    return simd_group_match_any2_avx2(p, EMPTY, DELETED);
#elif SIMD_X86
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
static bool vex_swiss_insert_internal(SwissMap *map, const char *key, size_t len, void *value);
static bool vex_swiss_remove_internal(SwissMap *map, const char *key, size_t len);
static void *vex_swiss_get_internal(const SwissMap *map, const char *key, size_t len);
static void vex_swiss_free_internal(SwissMap *map);

// ===== Fast Insert for Rehash (No duplicate check, no rehash check) =====
static void vex_swiss_insert_move(SwissMap *map, uint64_t h, const char *key, size_t len, void *value)
{
    const size_t cap = map->capacity;
    const uint8_t fp = h2(h);
    size_t i = bucket_start(h, cap);

    for (;;)
    {
        const uint8_t *gptr = map->ctrl + (i & (cap - 1));

        // We only look for EMPTY slots in a fresh map
        uint32_t empties = simd_group_match_eq(gptr, EMPTY);

        if (empties)
        {
            int off = first_bit(empties);
            size_t idx = (i + (size_t)off) & (cap - 1);

            map->ctrl[idx] = fp;
            map->entries[idx].hash = h;
            map->entries[idx].key = key;
            map->entries[idx].len = len;
            map->entries[idx].value = value;
            map->len++;
            return;
        }

        i += GROUP_SIZE;
    }
}

// ---- AVX2 path (32-byte groups) ----
#if defined(__AVX2__)
#define SIMD_AVX2 1
#else
#define SIMD_AVX2 0
#endif

// Unified Group Size
#if SIMD_AVX2
#undef GROUP_SIZE
#define GROUP_SIZE 32u
#endif

// ... (existing SIMD helpers) ...

// Rehash (grow) - Optimized Batch Processing
static bool vex_swiss_rehash(SwissMap *map, size_t new_cap)
{
    SwissMap nm;
    if (!vex_swiss_init_internal(&nm, new_cap))
        return false;

    // Batch process entries
    // We can iterate linearly over the old arrays
    // Since we are rehashing, we don't need to check for duplicates or DELETED
    
    const Entry *old_entries = map->entries;
    const uint8_t *old_ctrl = map->ctrl;
    const size_t old_cap = map->capacity;
    
    for (size_t i = 0; i < old_cap; ++i)
    {
        uint8_t c = old_ctrl[i];
        if (c != EMPTY && c != DELETED)
        {
            // Hot path: direct insert without full probe overhead
            // We know the new map is empty, so we just find the first empty slot
            // in the target group or subsequent groups.
            
            Entry *e = (Entry*)&old_entries[i];
            vex_swiss_insert_move(&nm, e->hash, e->key, e->len, e->value);
        }
    }
    
    // Free old ctrl with correct free function
#if defined(_WIN32)
    _aligned_free(map->ctrl);
#else
    free(map->ctrl);
#endif
    vex_free(map->entries);
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

    map->entries = (Entry *)vex_calloc(cap, sizeof(Entry));

    // 64 byte alignment for SIMD (Cache line size)
    size_t ctrl_size = cap + GROUP_PAD;
    size_t aligned_size = (ctrl_size + 63) & ~63;

#if defined(_WIN32)
    map->ctrl = (uint8_t *)_aligned_malloc(aligned_size, 64);
#else
// C11 aligned_alloc or posix_memalign
#if defined(_ISOC11_SOURCE) || (defined(__STDC_VERSION__) && __STDC_VERSION__ >= 201112L)
    map->ctrl = (uint8_t *)aligned_alloc(64, aligned_size);
#else
    if (posix_memalign((void **)&map->ctrl, 64, aligned_size) != 0)
        map->ctrl = NULL;
#endif
#endif

    if (!map->entries || !map->ctrl)
    {
        vex_free(map->entries);
        if (map->ctrl)
        {
#if defined(_WIN32)
            _aligned_free(map->ctrl);
#else
            free(map->ctrl);
#endif
        }
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
#if defined(_WIN32)
    _aligned_free(map->ctrl);
#else
    free(map->ctrl);
#endif
    vex_free(map->entries);
    map->ctrl = NULL;
    map->entries = NULL;
    map->capacity = 0;
    map->len = 0;
    map->max_load = 0;
}

// ===== Public API glue =====

bool vex_map_new(VexMap *map, size_t initial_capacity) { return vex_swiss_init_internal((SwissMap *)map, initial_capacity); }
bool vex_map_insert(VexMap *map, const char *key, size_t len, void *value) { return vex_swiss_insert_internal((SwissMap *)map, key, len, value); }
void *vex_map_get(const VexMap *map, const char *key, size_t len) { return vex_swiss_get_internal((const SwissMap *)map, key, len); }
bool vex_map_remove(VexMap *map, const char *key, size_t len) { return vex_swiss_remove_internal((SwissMap *)map, key, len); }
size_t vex_map_len(const VexMap *map) { return ((const SwissMap *)map)->len; }
void vex_map_free(VexMap *map) { vex_swiss_free_internal((SwissMap *)map); }

VexMap *vex_map_create(size_t initial_capacity)
{
    VexMap *map = (VexMap *)vex_malloc(sizeof(VexMap));
    if (!map)
        return NULL;
    if (!vex_map_new(map, initial_capacity))
    {
        vex_free(map);
        return NULL;
    }
    return map;
}

// ===== Insert or update (grouped probing with backstop) =====
static bool vex_swiss_insert_internal(SwissMap *map, const char *key, size_t len, void *value)
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

    const uint64_t h = wyhash64(key, len, 0);
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
                Entry *e = &map->entries[idx];

                // 1. Filter: Hash check
                // 2. Filter: Length check (Cache friendly)
                if (e->hash == h && e->len == len)
                {
                    // 3. Optimized key comparison (fast_key_eq eliminates redundant work)
                    if (fast_key_eq(e->key, key, len))
                    {
                        // hit / update
                        e->value = value;
                        return true;
                    }
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
                map->entries[idx].len = len; // Store length
                map->entries[idx].value = value;
                map->len++;
                return true;
            }

            // 3) Next group + Enhanced Prefetching (hide L1 miss latency)
            i += GROUP_SIZE;
            scanned += GROUP_SIZE;

            // Prefetch next group ahead (critical for performance)
            if (VEX_LIKELY(scanned < cap))
            {
                size_t next = (i + GROUP_SIZE) & (cap - 1);
                __builtin_prefetch((const void *)(map->ctrl + next), 0, 1);      // Read, low temporal locality
                __builtin_prefetch((const void *)(map->entries + next), 0, 1);  // Prefetch entries too
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
static void *vex_swiss_get_internal(const SwissMap *map, const char *key, size_t len)
{
    if (VEX_UNLIKELY(!map || !key || map->len == 0))
        return NULL;

    const size_t cap = map->capacity;
    const uint64_t h = wyhash64(key, len, 0);
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
            Entry *e = &map->entries[idx];

            if (e->hash == h && e->len == len)
            {
                if (fast_key_eq(e->key, key, len))
                    return e->value;
            }
            match &= (match - 1);
        }

        // Early exit ONLY if we see EMPTY in this group (DELETED does not terminate)
        uint32_t empties = simd_group_match_eq(gptr, EMPTY);
        if (empties)
            return NULL;

        // Next group + Enhanced Prefetching (critical for lookup performance)
        i += GROUP_SIZE;
        scanned += GROUP_SIZE;

        if (VEX_LIKELY(scanned < cap))
        {
            size_t next = (i + GROUP_SIZE) & (cap - 1);
            __builtin_prefetch((const void *)(map->ctrl + next), 0, 0);      // Read-only, no temporal locality
            __builtin_prefetch((const void *)(map->entries + next), 0, 0);  // Prefetch entries too
        }

        if (VEX_UNLIKELY(scanned > cap))
            return NULL; // safety belt
    }
}

// ===== Legacy wrappers for old tests (optional) =====
bool vex_swiss_insert(SwissMap *map, const char *key, void *value) { return vex_swiss_insert_internal(map, key, strlen(key), value); }
void *vex_swiss_get(const SwissMap *map, const char *key) { return vex_swiss_get_internal(map, key, strlen(key)); }
bool vex_swiss_init(SwissMap *map, size_t initial_capacity) { return vex_swiss_init_internal(map, initial_capacity); }
void vex_swiss_free(SwissMap *map) { vex_swiss_free_internal(map); }

// ===== Remove/Delete =====
static bool vex_swiss_remove_internal(SwissMap *map, const char *key, size_t len)
{
    if (VEX_UNLIKELY(!map || !key || map->len == 0))
        return false;

    const size_t cap = map->capacity;
    const uint64_t h = wyhash64(key, len, 0);
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
            Entry *e = &map->entries[idx];

            if (e->hash == h && e->len == len)
            {
                if (fast_key_eq(e->key, key, len))
                {
                    // Found it. Mark DELETED.
                    map->ctrl[idx] = DELETED;
                    map->len--;
                    return true;
                }
            }
            match &= (match - 1);
        }

        // Early exit ONLY if we see EMPTY in this group
        uint32_t empties = simd_group_match_eq(gptr, EMPTY);
        if (empties)
            return false;

        // Next group + Enhanced Prefetching
        i += GROUP_SIZE;
        scanned += GROUP_SIZE;

        if (VEX_LIKELY(scanned < cap))
        {
            size_t next = (i + GROUP_SIZE) & (cap - 1);
            __builtin_prefetch((const void *)(map->ctrl + next), 0, 1);      // Read, low temporal locality
            __builtin_prefetch((const void *)(map->entries + next), 0, 1);  // Prefetch entries too
        }

        if (VEX_UNLIKELY(scanned > cap))
            return false;
    }
}

// ============================================================================
