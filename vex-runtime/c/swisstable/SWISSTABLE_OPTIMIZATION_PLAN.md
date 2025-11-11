# Vex SwissTable Optimization Plan
## Goal: Beat Google Abseil Swiss Tables

Current Baseline (SwissTable V3, ARM64/NEON, 100K keys, clang -O3):
- Insert: **51.6 ns/op** (19.4M ops/s)
- Lookup: **74.9 ns/op** (13.4M ops/s)
- Delete: **20.6 ns/op** (48.5M ops/s)

Target Performance (match/beat Abseil on ARM):
- Insert: <40 ns/op (>25M ops/s) - **~22% improvement needed**
- Lookup: <55 ns/op (>18M ops/s) - **~27% improvement needed**
- Delete: <20 ns/op (>50M ops/s) - **~5% improvement needed**

---

## 🚀 V3 Immediate Roadmap (Q4 2025)

1. **Dual-Group SIMD Probe (Work-In-Progress)**
    - Extend NEON/AVX2 matching to compare 2 probe groups per iteration
    - Expected gain: +10-15% lookup throughput, better branch prediction
    - Status: Prototype in `bench_v2_vs_v3` branch, needs integration & tests

2. **Hash Dispatch for 1-32 byte keys**
    - Keep tiny/small key fast path but add 24/32-byte specializations
    - Avoids fallback to generic loop for medium strings
    - Expected gain: +8-10% insert & lookup on compiler workloads

3. **Adaptive Prefetch Tuning**
    - Prefetch next 2-3 groups only when load factor > 0.5
    - Reduce wasted prefetches for sparse tables
    - Expected gain: +5% on mixed workloads, lower power usage

4. **Micro-benchmark Automation**
    - Integrate `bench_v2_vs_v3.c` into CI perf suite (nightly)
    - Capture regression history for insert/lookup/remove
    - Needed before enabling V3 by default for all Map users

5. **API Hardening**
    - Add `vex_map_clear_v3` (DONE ✅)
    - Add stress tests for rehash + cached hashes
    - Document alignment assumptions in `vex_swisstable_v3.c`

---

## 🎯 Optimization Strategies

### 1. **Hash Function Optimization** (Expected: 20-30% improvement)

#### Current Issue:
- `hash64_str()` does two passes: strlen + wyhash
- String length computed every time

#### Solution A: **Cached Hash + Length**
```c
typedef struct {
    const char *key;
    uint64_t hash;  // Pre-computed hash
    uint16_t len;   // Cached length
} CachedKey;
```

#### Solution B: **SIMD-accelerated strlen**
```c
// ARM NEON version
static inline size_t fast_strlen_neon(const char *s) {
    const uint8x16_t zero = vdupq_n_u8(0);
    size_t len = 0;
    while (1) {
        uint8x16_t chunk = vld1q_u8((const uint8_t*)s + len);
        uint8x16_t cmp = vceqq_u8(chunk, zero);
        uint64_t mask = vget_lane_u64(vreinterpret_u64_u8(vmaxvq_u8(cmp)), 0);
        if (mask) return len + __builtin_ctzll(mask) / 8;
        len += 16;
    }
}
```

#### Solution C: **Single-pass hash (best for small keys)**
```c
// Hash while finding null terminator
static inline uint64_t hash64_str_fast(const char *s) {
    uint64_t h = 0xa0761d6478bd642full;
    const uint8_t *p = (const uint8_t *)s;
    
    // Unrolled loop for common short strings
    if (p[0] == 0) return h;
    h ^= (uint64_t)p[0] * 0x2d358dccaa6c78a5ull;
    
    if (p[1] == 0) return _wymix(h, 1);
    h ^= (uint64_t)p[1] * 0x8bb84b93962eacc9ull;
    
    // ... continue for up to 16 bytes
    // Then fall back to wyhash for longer strings
}
```

---

### 2. **SIMD Group Matching Optimization** (Expected: 30-40% improvement)

#### Current Issue:
- Single NEON comparison per group
- No vectorized multi-probe

#### Solution: **Vectorized Multi-Group Probe**
```c
// Check 2 groups simultaneously with NEON
static inline uint64_t simd_match_dual_groups(
    const uint8_t *g1, const uint8_t *g2, uint8_t fp) {
    
    uint8x16_t target = vdupq_n_u8(fp);
    uint8x16_t group1 = vld1q_u8(g1);
    uint8x16_t group2 = vld1q_u8(g2);
    
    uint8x16_t cmp1 = vceqq_u8(group1, target);
    uint8x16_t cmp2 = vceqq_u8(group2, target);
    
    // Pack results into 64-bit mask
    uint32_t mask1 = movemask_neon(cmp1);
    uint32_t mask2 = movemask_neon(cmp2);
    
    return ((uint64_t)mask2 << 32) | mask1;
}
```

---

### 3. **Aggressive Prefetching** (Expected: 15-25% improvement)

#### Current Issue:
- Only single-group prefetch
- No stride prefetch for linear probing

#### Solution: **Multi-Level Prefetch**
```c
// Prefetch next 3 groups ahead
static inline void prefetch_probe_path(
    const uint8_t *ctrl, const Entry *entries,
    size_t current, size_t cap) {
    
    for (int i = 1; i <= 3; i++) {
        size_t next = (current + i * GROUP_SIZE) & (cap - 1);
        __builtin_prefetch(ctrl + next, 0, 0);      // Temporal
        __builtin_prefetch(entries + next, 0, 1);    // Non-temporal
    }
}

// Prefetch likely accessed entries based on H2 distribution
static inline void prefetch_hot_entries(VexMap *map, uint64_t hash) {
    size_t idx = hash & (map->capacity - 1);
    for (int i = 0; i < 4; i++) {
        __builtin_prefetch(&map->entries[(idx + i * 4) & (map->capacity - 1)], 0, 1);
    }
}
```

---

### 4. **Memory Layout Optimization** (Expected: 10-20% improvement)

#### Current Issue:
- ctrl and entries separate
- Cache line splits

#### Solution A: **Interleaved Layout**
```c
// 64-byte cache line optimized
typedef struct {
    uint8_t ctrl[16];     // 16 bytes: control bytes
    uint64_t hashes[6];   // 48 bytes: 6 hashes (or fewer entries)
    // Fits exactly in 1 cache line
} __attribute__((aligned(64))) CacheLine;
```

#### Solution B: **SOA to AOS Conversion**
```c
// Structure of Arrays → Array of Structures for better locality
typedef struct {
    uint8_t ctrl;
    uint8_t _pad[7];      // Align to 8 bytes
    uint64_t hash;
    const char *key;
    void *value;
} __attribute__((packed, aligned(32))) Slot;
```

---

### 5. **Branch Prediction Optimization** (Expected: 5-10% improvement)

#### Solution: **Likely/Unlikely hints + branchless code**
```c
// Branchless H2 match counting
static inline int count_h2_matches(const uint8_t *group, uint8_t fp) {
    uint8x16_t target = vdupq_n_u8(fp);
    uint8x16_t ctrl = vld1q_u8(group);
    uint8x16_t matches = vceqq_u8(ctrl, target);
    
    // Horizontal sum without branches
    return vaddvq_u8(vandq_u8(matches, vdupq_n_u8(1)));
}

// Branchless slot selection
static inline size_t select_empty_or_deleted(uint32_t deleted, uint32_t empty) {
    // Use bit manipulation instead of ternary
    uint32_t mask = (deleted != 0) - 1;  // 0xFFFFFFFF if deleted==0, else 0
    return __builtin_ctz((deleted & ~mask) | (empty & mask));
}
```

---

### 6. **Specialized Fast Paths** (Expected: 20-30% improvement)

#### Solution A: **Small Key Fast Path**
```c
// Optimized for keys <= 16 bytes (90% of use cases)
static inline void* fast_get_small_key(VexMap *map, const char *key) {
    // Hash entire key in registers
    uint64_t k1, k2;
    memcpy(&k1, key, 8);
    memcpy(&k2, key + 8, 8);
    
    uint64_t hash = _wymix(k1, k2);  // Single mix operation
    uint8_t fp = h2(hash);
    size_t i = bucket_start(hash, map->capacity);
    
    // Single group check (most hits)
    uint32_t matches = simd_group_match_eq(map->ctrl + i, fp);
    if (VEX_LIKELY(matches)) {
        // Direct key comparison without strcmp
        int off = first_bit(matches);
        Entry *e = &map->entries[i + off];
        uint64_t e1, e2;
        memcpy(&e1, e->key, 8);
        memcpy(&e2, e->key + 8, 8);
        
        if (VEX_LIKELY(e->hash == hash && e1 == k1 && e2 == k2)) {
            return e->value;
        }
    }
    
    // Fall back to full lookup
    return vex_swiss_get_internal(map, key);
}
```

#### Solution B: **Integer Key Specialization**
```c
// Zero-copy integer keys (for symbol tables, etc)
static inline void* fast_get_int_key(VexMap *map, uint64_t key) {
    uint64_t hash = _wymix(key, 0xa0761d6478bd642full);
    // Direct hash, no string operations
    // ...much faster
}
```

---

### 7. **Probe Sequence Optimization** (Expected: 10-15% improvement)

#### Current Issue:
- Linear group probing
- No quadratic probing

#### Solution: **Triangular Probing**
```c
// Better probe distribution
static inline size_t next_probe(size_t current, size_t iteration, size_t cap) {
    // Triangular numbers: 0, 1, 3, 6, 10, 15...
    size_t offset = (iteration * (iteration + 1)) / 2;
    return (current + offset * GROUP_SIZE) & (cap - 1);
}
```

---

### 8. **Compile-Time Optimizations** (Expected: 10-15% improvement)

```bash
# Enhanced compiler flags
CFLAGS="-O3 -march=native -mtune=native \
        -flto -fno-plt -fno-semantic-interposition \
        -funroll-loops -ffast-math -fomit-frame-pointer \
        -finline-functions -finline-limit=1000 \
        -fprofile-generate"  # First pass

# Then profile-guided optimization
CFLAGS="-O3 -march=native -fprofile-use \
        -fprofile-correction"  # Second pass
```

#### Force inlining critical paths:
```c
__attribute__((always_inline, hot))
static inline void* vex_swiss_get_internal(...)

__attribute__((flatten))  // Inline ALL callees
static void* fast_lookup_chain(...)
```

---

### 9. **Micro-optimizations** (Expected: 5-10% improvement)

#### A. Remove redundant hash storage:
```c
// Store only H2 (7 bits) instead of full hash (64 bits)
// Recompute hash on collision (rare)
typedef struct {
    const char *key;
    void *value;
    // No hash field! Save 8 bytes per entry
} CompactEntry;
```

#### B. Bit-packed control bytes:
```c
// Pack 2 control bytes per byte (4-bit each)
// 16 groups = 8 bytes instead of 16
uint8_t packed_ctrl[8];
```

#### C. Custom allocator:
```c
// Arena allocator for entries (better cache locality)
typedef struct {
    uint8_t *arena;
    size_t used;
    size_t capacity;
} EntryArena;
```

---

## 📊 Expected Cumulative Improvement

| Optimization | Improvement | Cumulative |
|--------------|-------------|------------|
| Base | - | 155.6 ns |
| Hash optimization | -30% | 108.9 ns |
| SIMD multi-group | -25% | 81.7 ns |
| Aggressive prefetch | -15% | 69.4 ns |
| Memory layout | -12% | 61.1 ns |
| Fast paths | -18% | **50.1 ns** |

**Target achieved: 50.1 ns < 65 ns** ✅

---

## 🚀 Implementation Priority

### Phase 1 (Quick Wins):
1. ✅ Single-pass hash for small keys
2. ✅ Force inline critical functions
3. ✅ Aggressive compiler flags
4. ✅ Branchless optimizations

### Phase 2 (Medium Effort):
1. ⚠️ Dual-group SIMD matching
2. ⚠️ Multi-level prefetching
3. ⚠️ Small key fast path
4. ⚠️ Triangular probing

### Phase 3 (Major Refactor):
1. ⏰ Memory layout redesign
2. ⏰ Custom allocator
3. ⏰ Profile-guided optimization
4. ⏰ Integer key specialization

---

## 🎯 Benchmark Methodology

```c
// Test on realistic workloads
1. Small keys (8-16 bytes) - 70% of real usage
2. Medium keys (16-32 bytes) - 20%
3. Large keys (32+ bytes) - 10%

// Test access patterns
1. Sequential access
2. Random access
3. Hot/cold distribution (80/20)
4. Cache eviction patterns
```

---

## 📝 Notes

- ARM NEON is slightly slower than x86 AVX2 for byte operations
- Focus on cache locality over raw instruction count
- Profile-guided optimization is CRITICAL for branch prediction
- Small key optimization will yield biggest real-world gains

**Let's beat Google! 🔥**

