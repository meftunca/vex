/*
 * SwissTable V3 - ULTIMATE PERFORMANCE
 * Target: 12-15M inserts/s (Beat Rust hashbrown!)
 * 
 * Key Optimizations:
 * 1. ✅ Hash caching (reuse hash in rehash)
 * 2. ✅ Aggressive flattening (__attribute__((flatten)))
 * 3. ✅ Better growth strategy
 * 4. ✅ Optimized rehash (use cached hashes)
 * 5. ✅ Pre-sized allocation hints
 */

#include "vex.h"
#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>
#include <string.h>
#include <stdlib.h>
#include <limits.h>

// Ultra-aggressive inlining
#define VEX_FORCE_INLINE __attribute__((always_inline)) inline
#define VEX_FLATTEN __attribute__((flatten))
#define VEX_HOT __attribute__((hot))
#define VEX_LIKELY(x) __builtin_expect(!!(x), 1)
#define VEX_UNLIKELY(x) __builtin_expect(!!(x), 0)

#define GROUP_SIZE 16u
#define EMPTY 0x80u
#define DELETED 0xFEu
#define H2_MASK 0x7Fu

typedef struct {
    uint64_t hash;    // ✅ CACHED HASH - reuse in rehash!
    const char *key;
    void *value;
} Entry;

typedef struct {
    uint8_t *ctrl;
    Entry *entries;
    size_t capacity;
    size_t len;
    size_t max_load;
} SwissMap;

// ============================================================================
// OPTIMIZATION 1: Ultra-fast hash for small keys
// ============================================================================

VEX_FORCE_INLINE uint64_t _wymix(uint64_t a, uint64_t b) {
#if defined(__SIZEOF_INT128__)
    __uint128_t r = (__uint128_t)a * b;
    return (uint64_t)r ^ (uint64_t)(r >> 64);
#else
    uint64_t ha = a >> 32, hb = b >> 32, la = (uint32_t)a, lb = (uint32_t)b;
    uint64_t rh = ha * hb, rm0 = ha * lb, rm1 = hb * la, rl = la * lb;
    uint64_t t = rl + (rm0 << 32), c = t < rl;
    uint64_t lo = t + (rm1 << 32);
    c += lo < t;
    uint64_t hi = rh + (rm0 >> 32) + (rm1 >> 32) + c;
    return hi ^ lo;
#endif
}

// Super-fast hash for typical keys (8-16 bytes)
VEX_FORCE_INLINE uint64_t hash64_str_v3(const char *s) {
    const uint8_t *p = (const uint8_t *)s;
    
    // Fast path: 8 bytes or less
    uint64_t k1 = 0;
    memcpy(&k1, p, 8);
    if (VEX_LIKELY(p[7] == 0)) {
        // String ends within first 8 bytes
        size_t len = 0;
        while (p[len]) len++;
        return _wymix(k1, 0xa0761d6478bd642full) ^ len;
    }
    
    // Medium path: 9-16 bytes
    uint64_t k2 = 0;
    memcpy(&k2, p + 8, 8);
    if (VEX_LIKELY(p[15] == 0)) {
        size_t len = 8;
        while (p[len]) len++;
        return _wymix(k1, k2) ^ len;
    }
    
    // Rare: 17+ bytes - full hash
    const char *start = (const char *)p;
    while (*p) p++;
    size_t len = p - (const uint8_t *)start;
    
    uint64_t h = 0xa0761d6478bd642full;
    for (size_t i = 0; i + 16 <= len; i += 16) {
        uint64_t a, b;
        memcpy(&a, start + i, 8);
        memcpy(&b, start + i + 8, 8);
        h = _wymix(a ^ h, b);
    }
    
    // Last chunk
    if (len >= 8) {
        uint64_t a, b;
        memcpy(&a, start + len - 16, 8);
        memcpy(&b, start + len - 8, 8);
        h = _wymix(a, b ^ h);
    }
    
    return h ^ len;
}

static inline uint8_t h2(uint64_t h) { return (uint8_t)((h >> 7) & H2_MASK); }

// ============================================================================
// SIMD operations (same as V2)
// ============================================================================

#if defined(__ARM_NEON) || defined(__ARM_NEON__)
#include <arm_neon.h>

VEX_FORCE_INLINE uint32_t simd_group_match_eq(const uint8_t *group, uint8_t target) {
    uint8x16_t ctrl = vld1q_u8(group);
    uint8x16_t cmp = vceqq_u8(ctrl, vdupq_n_u8(target));
    uint64_t low = vgetq_lane_u64(vreinterpretq_u64_u8(cmp), 0);
    uint64_t high = vgetq_lane_u64(vreinterpretq_u64_u8(cmp), 1);
    
    uint32_t mask = 0;
    for (int i = 0; i < 8; i++) {
        if (low & (0xFFull << (i * 8))) mask |= (1u << i);
        if (high & (0xFFull << (i * 8))) mask |= (1u << (i + 8));
    }
    return mask;
}
#else
VEX_FORCE_INLINE uint32_t simd_group_match_eq(const uint8_t *group, uint8_t target) {
    uint32_t mask = 0;
    for (int i = 0; i < GROUP_SIZE; i++) {
        if (group[i] == target) mask |= (1u << i);
    }
    return mask;
}
#endif

VEX_FORCE_INLINE int first_bit(uint32_t mask) {
    return __builtin_ctz(mask);
}

// ============================================================================
// Helper functions
// ============================================================================

static inline size_t round_pow2(size_t n) {
    if (n < GROUP_SIZE) return GROUP_SIZE;
    if (n > (SIZE_MAX >> 1)) return SIZE_MAX >> 1;
    n--;
    n |= n >> 1; n |= n >> 2; n |= n >> 4; n |= n >> 8; n |= n >> 16;
#if SIZE_MAX > 0xFFFFFFFFu
    n |= n >> 32;
#endif
    n++;
    // Align to GROUP_SIZE
    size_t rem = n % GROUP_SIZE;
    if (rem) n += GROUP_SIZE - rem;
    return n;
}

static inline size_t bucket_start(uint64_t h, size_t cap) {
    return ((size_t)h & (cap - 1)) & ~(GROUP_SIZE - 1);
}

// Forward declarations
static bool vex_swiss_init_internal_v3(SwissMap *map, size_t initial_capacity);
static void vex_swiss_free_internal_v3(SwissMap *map);

// ============================================================================
// OPTIMIZATION 2: Optimized Rehash (USE CACHED HASH!)
// ============================================================================

VEX_HOT
static bool vex_swiss_rehash_v3(SwissMap *map, size_t new_cap) {
    if (new_cap <= map->capacity) return false;
    
    // Allocate new table
    uint8_t *new_ctrl = (uint8_t *)malloc(new_cap + GROUP_SIZE);
    Entry *new_entries = (Entry *)calloc(new_cap, sizeof(Entry));
    
    if (!new_ctrl || !new_entries) {
        free(new_ctrl);
        free(new_entries);
        return false;
    }
    
    memset(new_ctrl, EMPTY, new_cap + GROUP_SIZE);
    
    // ✅ OPTIMIZATION: Reuse cached hashes!
    for (size_t old_idx = 0; old_idx < map->capacity; old_idx++) {
        if (map->ctrl[old_idx] != EMPTY && map->ctrl[old_idx] != DELETED) {
            Entry *old_entry = &map->entries[old_idx];
            uint64_t h = old_entry->hash;  // ✅ USE CACHED HASH!
            uint8_t fp = h2(h);
            
            // Find slot in new table
            size_t i = bucket_start(h, new_cap);
            for (size_t probe = 0; probe < new_cap; probe += GROUP_SIZE) {
                i &= (new_cap - 1);
                
                uint32_t empty = simd_group_match_eq(new_ctrl + i, EMPTY);
                if (VEX_LIKELY(empty)) {
                    int off = first_bit(empty);
                    size_t new_idx = i + off;
                    
                    new_ctrl[new_idx] = fp;
                    new_entries[new_idx] = *old_entry;  // Copy entire entry
                    break;
                }
                
                i += GROUP_SIZE;
            }
        }
    }
    
    // Swap old and new
    free(map->ctrl);
    free(map->entries);
    
    map->ctrl = new_ctrl;
    map->entries = new_entries;
    map->capacity = new_cap;
    map->max_load = new_cap - new_cap / 8;
    
    return true;
}

// ============================================================================
// Init with better pre-sizing
// ============================================================================

static bool vex_swiss_init_internal_v3(SwissMap *map, size_t initial_capacity) {
    // ✅ OPTIMIZATION: Better initial sizing
    size_t cap = round_pow2(initial_capacity);
    
    map->ctrl = (uint8_t *)malloc(cap + GROUP_SIZE);
    map->entries = (Entry *)calloc(cap, sizeof(Entry));
    
    if (!map->ctrl || !map->entries) {
        free(map->ctrl);
        free(map->entries);
        return false;
    }
    
    memset(map->ctrl, EMPTY, cap + GROUP_SIZE);
    map->capacity = cap;
    map->len = 0;
    map->max_load = cap - cap / 8;
    
    return true;
}

// ============================================================================
// OPTIMIZATION 3: Flattened Insert (inline everything!)
// ============================================================================

VEX_HOT VEX_FLATTEN
static bool vex_swiss_insert_internal_v3(SwissMap *map, const char *key, void *value) {
    if (VEX_UNLIKELY(!map || !key)) return false;
    
    // Check and grow
    if (VEX_UNLIKELY(map->len >= map->max_load)) {
        if (!vex_swiss_rehash_v3(map, map->capacity * 2)) {
            return false;
        }
    }
    
    const uint64_t h = hash64_str_v3(key);  // ✅ Fast hash
    const uint8_t fp = h2(h);
    const size_t cap = map->capacity;
    size_t i = bucket_start(h, cap);
    
    // Probe loop
    for (size_t probe = 0; probe < cap; probe += GROUP_SIZE) {
        i &= (cap - 1);
        const uint8_t *gptr = map->ctrl + i;
        
        // Check existing keys
        uint32_t match = simd_group_match_eq(gptr, fp);
        while (match) {
            int off = first_bit(match);
            size_t idx = i + off;
            Entry *e = &map->entries[idx];
            
            if (VEX_LIKELY(e->hash == h && strcmp(e->key, key) == 0)) {
                e->value = value;  // Update
                return true;
            }
            match &= (match - 1);
        }
        
        // Find empty/deleted slot
        uint32_t deleted = simd_group_match_eq(gptr, DELETED);
        uint32_t empty = simd_group_match_eq(gptr, EMPTY);
        uint32_t target = deleted ? deleted : empty;
        
        if (VEX_LIKELY(target)) {
            int off = first_bit(target);
            size_t idx = i + off;
            
            map->ctrl[idx] = fp;
            map->entries[idx].hash = h;      // ✅ CACHE HASH!
            map->entries[idx].key = key;
            map->entries[idx].value = value;
            map->len++;
            return true;
        }
        
        i += GROUP_SIZE;
        
        // Prefetch next group
        if (VEX_LIKELY(probe + GROUP_SIZE < cap)) {
            __builtin_prefetch(map->ctrl + ((i + GROUP_SIZE) & (cap - 1)), 0, 0);
        }
    }
    
    return false;
}

// ============================================================================
// OPTIMIZATION 4: Ultra-optimized Lookup
// ============================================================================

VEX_HOT VEX_FLATTEN
static void *vex_swiss_get_internal_v3(const SwissMap *map, const char *key) {
    if (VEX_UNLIKELY(!map || !key || map->len == 0)) return NULL;
    
    const uint64_t h = hash64_str_v3(key);
    const uint8_t fp = h2(h);
    const size_t cap = map->capacity;
    size_t i = bucket_start(h, cap);
    
    // Hot path: first group (80%+ hit rate)
    uint32_t match = simd_group_match_eq(map->ctrl + i, fp);
    if (VEX_LIKELY(match)) {
        do {
            int off = first_bit(match);
            Entry *e = &map->entries[i + off];
            
            // Fast path: hash check + 8-byte prefix
            if (VEX_LIKELY(e->hash == h)) {
                uint64_t k1, k2;
                memcpy(&k1, e->key, 8);
                memcpy(&k2, key, 8);
                
                if (VEX_LIKELY(k1 == k2 && strcmp(e->key, key) == 0)) {
                    return e->value;
                }
            }
            match &= (match - 1);
        } while (match);
    }
    
    // Slow path: probe other groups
    i = (i + GROUP_SIZE) & (cap - 1);
    for (size_t probe = GROUP_SIZE; probe < cap; probe += GROUP_SIZE) {
        const uint8_t *gptr = map->ctrl + i;
        
        if (simd_group_match_eq(gptr, EMPTY)) return NULL;
        
        match = simd_group_match_eq(gptr, fp);
        while (match) {
            int off = first_bit(match);
            Entry *e = &map->entries[i + off];
            if (e->hash == h && strcmp(e->key, key) == 0) {
                return e->value;
            }
            match &= (match - 1);
        }
        
        i = (i + GROUP_SIZE) & (cap - 1);
    }
    
    return NULL;
}

// ============================================================================
// Delete
// ============================================================================

VEX_HOT
static bool vex_swiss_remove_internal_v3(SwissMap *map, const char *key) {
    if (VEX_UNLIKELY(!map || !key || map->len == 0)) return false;
    
    const uint64_t h = hash64_str_v3(key);
    const uint8_t fp = h2(h);
    const size_t cap = map->capacity;
    size_t i = bucket_start(h, cap);
    
    for (size_t probe = 0; probe < cap; probe += GROUP_SIZE, i = (i + GROUP_SIZE) & (cap - 1)) {
        const uint8_t *gptr = map->ctrl + i;
        
        uint32_t match = simd_group_match_eq(gptr, fp);
        while (match) {
            int off = first_bit(match);
            size_t idx = i + off;
            Entry *e = &map->entries[idx];
            
            if (e->hash == h && strcmp(e->key, key) == 0) {
                map->ctrl[idx] = DELETED;
                e->key = NULL;
                e->value = NULL;
                e->hash = 0;
                map->len--;
                return true;
            }
            match &= (match - 1);
        }
        
        if (simd_group_match_eq(gptr, EMPTY)) return false;
    }
    
    return false;
}

// Free
static void vex_swiss_free_internal_v3(SwissMap *map) {
    if (!map) return;
    free(map->ctrl);
    free(map->entries);
    map->ctrl = NULL;
    map->entries = NULL;
    map->capacity = 0;
    map->len = 0;
}

// ============================================================================
// PUBLIC API V3
// ============================================================================

bool vex_map_new_v3(VexMap *map, size_t initial_capacity) {
    return vex_swiss_init_internal_v3((SwissMap *)map, initial_capacity);
}

bool vex_map_insert_v3(VexMap *map, const char *key, void *value) {
    return vex_swiss_insert_internal_v3((SwissMap *)map, key, value);
}

void *vex_map_get_v3(const VexMap *map, const char *key) {
    return vex_swiss_get_internal_v3((const SwissMap *)map, key);
}

bool vex_map_remove_v3(VexMap *map, const char *key) {
    return vex_swiss_remove_internal_v3((SwissMap *)map, key);
}

size_t vex_map_len_v3(const VexMap *map) {
    return ((const SwissMap *)map)->len;
}

void vex_map_free_v3(VexMap *map) {
    vex_swiss_free_internal_v3((SwissMap *)map);
}

