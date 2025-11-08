/**
 * VEX ZERO-COST ABSTRACTIONS
 * 
 * Philosophy: "You don't pay for what you don't use"
 * 
 * Patterns:
 * - Zero-copy: String views, slices, references
 * - Zero-allocation: Stack buffers, arena scopes
 * - Zero-overhead: Inline everything, compile-time dispatch
 * - Zero-runtime: Const evaluation, static assertions
 */

#ifndef VEX_ZERO_H
#define VEX_ZERO_H

#include <stddef.h>
#include <stdint.h>
#include <string.h>
#include <assert.h>

/* ============================================================================
   COMPILER DIRECTIVES (ZERO COST)
   ============================================================================ */

#define VEX_INLINE static inline __attribute__((always_inline))
#define VEX_CONST __attribute__((const))
#define VEX_PURE __attribute__((pure))
#define VEX_HOT __attribute__((hot))
#define VEX_COLD __attribute__((cold))
#define VEX_LIKELY(x) __builtin_expect(!!(x), 1)
#define VEX_UNLIKELY(x) __builtin_expect(!!(x), 0)
#define VEX_RESTRICT __restrict__
#define VEX_NOALIAS __attribute__((malloc))
#define VEX_NONNULL __attribute__((nonnull))

/* Compile-time assertions (zero runtime cost) */
#define VEX_STATIC_ASSERT(cond, msg) _Static_assert(cond, msg)
#define VEX_ASSERT_SIZE(type, size) VEX_STATIC_ASSERT(sizeof(type) == size, #type " size mismatch")

/* ============================================================================
   ZERO-COPY STRING VIEW
   ============================================================================ */

typedef struct {
    const char* data;
    size_t len;
} VexStr;

/* Create string view (zero copy!) */
VEX_INLINE VEX_CONST
VexStr vex_str(const char* data, size_t len) {
    return (VexStr){.data = data, .len = len};
}

/* From C string (zero copy!) */
VEX_INLINE VEX_PURE
VexStr vex_str_from_cstr(const char* s) {
    return vex_str(s, s ? strlen(s) : 0);
}

/* Slice (zero copy!) */
VEX_INLINE VEX_PURE
VexStr vex_str_slice(VexStr s, size_t start, size_t end) {
    if (start >= s.len) return (VexStr){NULL, 0};
    if (end > s.len) end = s.len;
    return vex_str(s.data + start, end - start);
}

/* Compare (zero copy!) */
VEX_INLINE VEX_PURE
int vex_str_eq(VexStr a, VexStr b) {
    return a.len == b.len && memcmp(a.data, b.data, a.len) == 0;
}

/* ============================================================================
   ZERO-COPY SLICE (GENERIC)
   ============================================================================ */

#define VEX_SLICE(T) struct { T* data; size_t len; size_t cap; }

/* Define typed slice */
#define VEX_DEFINE_SLICE(T) \
    typedef VEX_SLICE(T) T##Slice; \
    \
    VEX_INLINE VEX_CONST \
    T##Slice T##_slice(T* data, size_t len) { \
        return (T##Slice){.data = data, .len = len, .cap = len}; \
    } \
    \
    VEX_INLINE VEX_PURE \
    T##Slice T##_slice_sub(T##Slice s, size_t start, size_t end) { \
        if (start >= s.len) return (T##Slice){NULL, 0, 0}; \
        if (end > s.len) end = s.len; \
        return (T##Slice){s.data + start, end - start, end - start}; \
    } \
    \
    VEX_INLINE VEX_PURE \
    T T##_slice_get(T##Slice s, size_t idx) { \
        assert(idx < s.len); \
        return s.data[idx]; \
    }

/* Common slices */
VEX_DEFINE_SLICE(uint8_t)
VEX_DEFINE_SLICE(int32_t)
VEX_DEFINE_SLICE(uint64_t)

/* ============================================================================
   ZERO-ALLOCATION STACK BUFFER
   ============================================================================ */

/* Stack-allocated buffer (compile-time size) */
#define VEX_STACK_BUF(name, size) \
    uint8_t name##_storage[size]; \
    VexBuf name = {.data = name##_storage, .len = 0, .cap = size}

typedef struct {
    uint8_t* data;
    size_t len;
    size_t cap;
} VexBuf;

/* Append to buffer (bounds-checked) */
VEX_INLINE
int vex_buf_append(VexBuf* buf, const void* data, size_t len) {
    if (VEX_UNLIKELY(buf->len + len > buf->cap)) return 0;
    memcpy(buf->data + buf->len, data, len);
    buf->len += len;
    return 1;
}

/* Write formatted (zero-allocation for small strings) */
#define VEX_BUF_PRINTF(buf, fmt, ...) \
    snprintf((char*)(buf)->data + (buf)->len, (buf)->cap - (buf)->len, fmt, ##__VA_ARGS__)

/* ============================================================================
   ZERO-ALLOCATION ARENA SCOPE
   ============================================================================ */

/* Arena scope (auto-reset on exit) */
#define VEX_ARENA_SCOPE(arena) \
    uint8_t* _arena_mark = (arena)->current; \
    for (int _i = 0; _i < 1; _i++, (arena)->current = _arena_mark)

/* Example usage:
 * VEX_ARENA_SCOPE(arena) {
 *     void* temp = vex_arena_alloc(arena, 1024);
 *     // Use temp...
 * }  // Automatically freed here!
 */

/* ============================================================================
   ZERO-OVERHEAD OPTION/RESULT TYPES
   ============================================================================ */

/* Option type (like Rust Option<T>) */
#define VEX_OPTION(T) struct { T value; int has_value; }

#define VEX_SOME(T, val) ((VEX_OPTION(T)){.value = val, .has_value = 1})
#define VEX_NONE(T) ((VEX_OPTION(T)){.has_value = 0})

#define VEX_IS_SOME(opt) ((opt).has_value)
#define VEX_IS_NONE(opt) (!(opt).has_value)
#define VEX_UNWRAP(opt) ((opt).value)

/* Result type (like Rust Result<T, E>) */
#define VEX_RESULT(T, E) struct { union { T ok; E err; }; int is_ok; }

#define VEX_OK(T, E, val) ((VEX_RESULT(T, E)){.ok = val, .is_ok = 1})
#define VEX_ERR(T, E, val) ((VEX_RESULT(T, E)){.err = val, .is_ok = 0})

#define VEX_IS_OK(res) ((res).is_ok)
#define VEX_IS_ERR(res) (!(res).is_ok)
#define VEX_UNWRAP_OK(res) ((res).ok)
#define VEX_UNWRAP_ERR(res) ((res).err)

/* ============================================================================
   ZERO-COST ITERATORS
   ============================================================================ */

/* Range iterator (compile-time) */
#define VEX_RANGE(i, start, end) \
    for (size_t i = (start); i < (end); i++)

/* Slice iterator (zero overhead) */
#define VEX_ITER_SLICE(T, item, slice) \
    for (T* item = (slice).data, *_end = (slice).data + (slice).len; \
         item < _end; item++)

/* Example:
 * uint8_tSlice bytes = ...;
 * VEX_ITER_SLICE(uint8_t, b, bytes) {
 *     // Use *b...
 * }
 */

/* ============================================================================
   ZERO-COPY REFERENCE COUNTING (OPTIONAL)
   ============================================================================ */

#if VEX_ENABLE_REFCOUNT

#include <stdatomic.h>

typedef struct {
    _Atomic uint32_t count;
    void (*dtor)(void*);
    uint8_t data[];
} VexRc;

/* Create reference-counted object */
VEX_INLINE VEX_NOALIAS
void* vex_rc_new(size_t size, void (*dtor)(void*)) {
    VexRc* rc = (VexRc*)malloc(sizeof(VexRc) + size);
    if (!rc) return NULL;
    atomic_store(&rc->count, 1);
    rc->dtor = dtor;
    return rc->data;
}

/* Increment reference (zero-cost if not used) */
VEX_INLINE
void* vex_rc_retain(void* ptr) {
    if (!ptr) return NULL;
    VexRc* rc = (VexRc*)((uint8_t*)ptr - offsetof(VexRc, data));
    atomic_fetch_add(&rc->count, 1);
    return ptr;
}

/* Decrement reference (zero-cost if not used) */
VEX_INLINE
void vex_rc_release(void* ptr) {
    if (!ptr) return;
    VexRc* rc = (VexRc*)((uint8_t*)ptr - offsetof(VexRc, data));
    if (atomic_fetch_sub(&rc->count, 1) == 1) {
        if (rc->dtor) rc->dtor(ptr);
        free(rc);
    }
}

#endif /* VEX_ENABLE_REFCOUNT */

/* ============================================================================
   ZERO-COST DEFER (CLEANUP ON SCOPE EXIT)
   ============================================================================ */

/* Defer execution (like Go defer) */
#define VEX_DEFER_CONCAT(a, b) a##b
#define VEX_DEFER_VARNAME(line) VEX_DEFER_CONCAT(_defer_, line)

#define VEX_DEFER(code) \
    void VEX_DEFER_VARNAME(__LINE__)(void* _) { code; } \
    __attribute__((cleanup(VEX_DEFER_VARNAME(__LINE__)))) int VEX_DEFER_VARNAME(__LINE__)##_var

/* Example:
 * {
 *     FILE* f = fopen("test.txt", "r");
 *     VEX_DEFER({ fclose(f); });
 *     // Use f...
 * }  // fclose called automatically!
 */

/* ============================================================================
   COMPILE-TIME SIZE OPTIMIZATION
   ============================================================================ */

/* Ensure structs are cache-line aligned */
#define VEX_CACHE_ALIGNED __attribute__((aligned(64)))

/* Pack structs (zero padding) */
#define VEX_PACKED __attribute__((packed))

/* Ensure minimum alignment */
#define VEX_ALIGNED(n) __attribute__((aligned(n)))

/* ============================================================================
   ZERO-OVERHEAD ERROR HANDLING
   ============================================================================ */

/* Error type (small, stack-allocated) */
typedef struct {
    int code;
    const char* msg;  /* Static string - no allocation! */
} VexError;

#define VEX_OK_ERR ((VexError){.code = 0, .msg = NULL})
#define VEX_ERR_NEW(c, m) ((VexError){.code = c, .msg = m})

/* Propagate errors (like Rust ?) */
#define VEX_TRY(expr) \
    do { \
        VexError _err = (expr); \
        if (VEX_UNLIKELY(_err.code != 0)) return _err; \
    } while (0)

/* ============================================================================
   PERFORMANCE HINTS
   ============================================================================ */

/* Prefetch data (reduce cache miss latency) */
#define VEX_PREFETCH(addr) __builtin_prefetch(addr, 0, 3)
#define VEX_PREFETCH_WRITE(addr) __builtin_prefetch(addr, 1, 3)

/* Assume branch (help CPU prediction) */
#define VEX_ASSUME(cond) do { if (!(cond)) __builtin_unreachable(); } while (0)

/* No aliasing (help compiler optimize) */
#define VEX_NOALIAS_PTR(T) T* VEX_RESTRICT

/* ============================================================================
   COMPILE-TIME ASSERTIONS FOR ZERO-COST
   ============================================================================ */

/* Verify size classes are optimal */
VEX_ASSERT_SIZE(VexStr, 16);         /* Fits in 2 registers */
VEX_ASSERT_SIZE(VexError, 16);       /* Fits in 2 registers */

#endif /* VEX_ZERO_H */

