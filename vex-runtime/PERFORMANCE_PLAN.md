# Vex Runtime Performance Optimization Plan

## 📊 Current Baseline

- **C Runtime**: ~900 lines, 16 KB library
- **String ops**: Scalar only (vex_utf8_valid ~300 chars/µs)
- **No HashMap**: Missing `Map<K,V>` builtin type
- **Test suite**: 100% passing ✅

---

## 🚀 Phase 1: SwissTable Integration (HIGH PRIORITY)

### Why First?

1. **New Feature**: Vex doesn't have HashMap yet → users need this
2. **Compiler Benefit**: Symbol tables, type caches, macro storage
3. **Proven**: Google/Rust use it (battle-tested)
4. **Small**: Only adds ~8 KB to runtime

### Implementation Steps

#### 1.1 Add to vex.h

```c
// ===== SwissTable API =====
typedef struct SwissMap SwissMap;

bool   vex_map_new(SwissMap** map, size_t initial_capacity);
bool   vex_map_insert(SwissMap* map, const char* key, void* value);
void*  vex_map_get(const SwissMap* map, const char* key);
bool   vex_map_remove(SwissMap* map, const char* key);
void   vex_map_free(SwissMap* map);
size_t vex_map_len(const SwissMap* map);
```

#### 1.2 Update build.sh

```bash
# Add vex_swisstable.c to build list
SOURCES="vex_string.c vex_memory.c vex_alloc.c vex_io.c vex_array.c vex_error.c vex_swisstable.c"

# SIMD flags for SwissTable
SIMD_FLAGS=""
if [[ $(uname -m) == "x86_64" ]]; then
    SIMD_FLAGS="-mavx2 -msse2"
elif [[ $(uname -m) == "arm64" ]]; then
    SIMD_FLAGS=""  # NEON enabled by default on ARM
fi

clang -S -emit-llvm -O3 $SIMD_FLAGS -fno-builtin ...
```

#### 1.3 Add Tests

```c
// test_map.c
void test_swiss_map() {
    SwissMap* map = NULL;
    assert(vex_map_new(&map, 16));

    // Insert
    vex_map_insert(map, "name", (void*)"Alice");
    vex_map_insert(map, "age", (void*)25);

    // Lookup
    assert(strcmp(vex_map_get(map, "name"), "Alice") == 0);
    assert((intptr_t)vex_map_get(map, "age") == 25);

    // Update
    vex_map_insert(map, "age", (void*)26);
    assert((intptr_t)vex_map_get(map, "age") == 26);

    // Not found
    assert(vex_map_get(map, "unknown") == NULL);

    vex_map_free(map);
    printf("✓ SwissTable tests passed\n");
}
```

#### 1.4 Vex Language Syntax

```vex
// Builtin Map type
let cache: Map<String, i64> = Map.new();
cache.insert("key", 42);

let value = cache.get("key"); // Option<i64>
match value {
    Some(v) => println("Found: {}", v),
    None => println("Not found")
}
```

### Expected Performance

- **Insert**: 70ns/op (1.4x faster than std::unordered_map)
- **Lookup hit**: 35ns/op (1.4x faster)
- **Lookup miss**: 25ns/op (1.8x faster)
- **Memory**: 24 bytes/entry overhead (16B ctrl + 8B padding)

### Size Impact

- **Library size**: 16 KB → 24 KB (+50%)
- **LLVM IR**: 2,052 lines → ~2,800 lines (+36%)

---

## 🌟 Phase 2: SIMD UTF-8 (MEDIUM PRIORITY)

### Why Second?

1. **Optimization**: Current UTF-8 already works (correctness ✅)
2. **Use Cases**: File I/O, web servers, text processing (future features)
3. **Complexity**: Requires conditional compilation, testing matrix

### Integration Strategy

#### Option A: Replace Existing (Recommended)

```c
// vex_string.c - replace vex_utf8_valid()
bool vex_utf8_valid(const char* str, size_t len) {
    // Delegate to SIMD version (with scalar fallback)
    return utf8_validate((const uint8_t*)str, len);
}
```

#### Option B: Dual API (User Choice)

```c
// vex.h
bool vex_utf8_valid(const char* str, size_t len);        // Scalar (current)
bool vex_utf8_valid_fast(const char* str, size_t len);   // SIMD-accelerated
```

```vex
// User can choose
let is_valid = str.is_valid_utf8();       // Scalar (portable)
let is_valid = str.is_valid_utf8_fast();  // SIMD (requires CPU support)
```

### Implementation Steps

#### 2.1 Merge vex_simd_utf.c

```bash
# Rename for consistency
mv vex_simd_utf.c vex_utf8_simd.c

# Update vex.h
// UTF-8 SIMD API
bool vex_utf8_validate_simd(const uint8_t* s, size_t len);
size_t vex_utf8_to_utf16(const uint8_t* src, size_t len, uint16_t* dst);
size_t vex_utf8_to_utf32(const uint8_t* src, size_t len, uint32_t* dst);
```

#### 2.2 Add Benchmarks

```c
// bench_utf8.c
#include <time.h>

void benchmark_utf8() {
    const char* text = "Hello 世界 مرحبا 👋 " /* ... 1MB ... */;
    size_t len = strlen(text);

    // Scalar
    clock_t start = clock();
    for (int i = 0; i < 1000; i++) {
        vex_utf8_valid(text, len);
    }
    double scalar_time = (double)(clock() - start) / CLOCKS_PER_SEC;

    // SIMD
    start = clock();
    for (int i = 0; i < 1000; i++) {
        vex_utf8_validate_simd((const uint8_t*)text, len);
    }
    double simd_time = (double)(clock() - start) / CLOCKS_PER_SEC;

    printf("Scalar: %.3fms, SIMD: %.3fms (%.1fx speedup)\n",
           scalar_time * 1000, simd_time * 1000, scalar_time / simd_time);
}
```

#### 2.3 Testing Matrix

```bash
# Test on different CPUs
make test ARCH=x86_64 SIMD=avx2   # AVX2 path
make test ARCH=x86_64 SIMD=sse2   # SSE2 path
make test ARCH=arm64              # NEON path
make test ARCH=riscv64 SIMD=none  # Scalar fallback
```

### Expected Performance

```
CPU              | Scalar   | SIMD     | Speedup
-----------------|----------|----------|--------
Intel (AVX2)     | 500MB/s  | 10GB/s   | 20x
AMD (AVX2)       | 450MB/s  | 9GB/s    | 20x
ARM (NEON)       | 400MB/s  | 3GB/s    | 7.5x
RISC-V (scalar)  | 350MB/s  | 350MB/s  | 1x
```

### Size Impact

- **Library size**: 24 KB → 30 KB (+25%)
- **LLVM IR**: 2,800 lines → ~3,500 lines (+25%)

---

## 📈 Phase 3: Additional Optimizations (LOW PRIORITY)

### 3.1 SIMD memcpy/memset

- Use `rep movsb` (x86) or NEON (ARM)
- Expected: 2-3x speedup for large copies
- Size: +2 KB

### 3.2 String Interning (using SwissTable)

```c
const char* vex_intern(const char* str);
```

- Deduplicates string literals
- Saves memory in string-heavy programs
- Size: +4 KB

### 3.3 Custom Allocator

```c
typedef struct VexAllocator {
    void* (*alloc)(size_t size);
    void  (*free)(void* ptr);
} VexAllocator;

void vex_set_allocator(VexAllocator* allocator);
```

- Allows jemalloc/mimalloc integration
- Size: +2 KB

---

## 🎯 Recommended Roadmap

### Immediate (This Week)

1. ✅ **Integrate SwissTable**

   - Add to build system
   - Write tests
   - Document API
   - Expected: 1-2 days

2. ✅ **Add Map<K,V> to Vex language**
   - Parser support
   - Type checking
   - Codegen (call vex*map*\*)
   - Expected: 2-3 days

### Short-term (Next 2 Weeks)

3. ⏳ **SIMD UTF-8 (conditional)**
   - Benchmark current scalar performance
   - Only add if bottleneck found
   - Expected: 2-3 days (if needed)

### Long-term (Future)

4. ⏳ **SIMD memory ops**
5. ⏳ **String interning**
6. ⏳ **Custom allocators**

---

## 🔍 Decision Matrix

| Feature          | Impact | Effort | Priority | Add Now? |
| ---------------- | ------ | ------ | -------- | -------- |
| SwissTable       | HIGH   | LOW    | HIGH     | ✅ YES   |
| SIMD UTF-8       | MEDIUM | MEDIUM | MEDIUM   | ⏳ LATER |
| SIMD memcpy      | LOW    | LOW    | LOW      | ❌ NO    |
| String interning | MEDIUM | MEDIUM | LOW      | ❌ NO    |
| Custom allocator | LOW    | HIGH   | LOW      | ❌ NO    |

---

## 💡 Final Recommendation

### Start with SwissTable! 🗺️

**Reasoning:**

1. **User-facing feature**: Vex needs HashMap, users will use this daily
2. **Compiler benefit**: Faster symbol tables, better compile times
3. **Proven tech**: Google/Rust already use it (low risk)
4. **Small cost**: Only 8 KB added to runtime
5. **Clear win**: 1.4-1.8x speedup over std::unordered_map

**SIMD UTF-8 can wait:**

- Current scalar UTF-8 works fine for most cases
- Only needed for high-throughput text processing
- Adds complexity (multi-platform testing)
- Can add later when file I/O is implemented

---

## 📝 Next Steps

```bash
# 1. Test SwissTable standalone
cd vex-runtime/c
clang -O3 -mavx2 -DTEST_SWISS vex_swisstable.c -o test_swiss
./test_swiss

# 2. Integrate into runtime
# - Add to vex.h
# - Update build.sh
# - Add to test.c

# 3. Add to Vex language
# - Parser: Map<K,V> syntax
# - Codegen: vex_map_* calls
# - Examples: examples/map_test.vx
```

Would you like me to start with SwissTable integration? 🚀
