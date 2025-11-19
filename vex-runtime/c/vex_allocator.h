/**
 * Vex Allocator Abstraction Layer
 * Provides pluggable memory allocator support (mimalloc, system)
 * Selection via VEX_ALLOCATOR environment variable at compile time
 */

#ifndef VEX_ALLOCATOR_H
#define VEX_ALLOCATOR_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C"
{
#endif

    /* ========== Allocator Selection ========== */
    /* Set VEX_ALLOCATOR=mimalloc (default) or system */

#if defined(VEX_USE_MIMALLOC)
/* mimalloc: Fast general-purpose allocator (default) */
#include "allocators/mimalloc/include/mimalloc.h"

#define VEX_ALLOC_IMPL(size) mi_malloc(size)
#define VEX_CALLOC_IMPL(count, size) mi_calloc(count, size)
#define VEX_REALLOC_IMPL(ptr, size) mi_realloc(ptr, size)
#define VEX_FREE_IMPL(ptr) mi_free(ptr)
#define vex_alloc_aligned(size, align) mi_malloc_aligned(size, align)
#define vex_free_aligned(ptr) mi_free(ptr)
#define vex_allocator_stats() mi_stats_print(NULL)
#define VEX_ALLOCATOR_NAME "mimalloc"

#else
/* System allocator: Standard libc malloc (minimal binary size) */
#include <stdlib.h>

#define VEX_ALLOC_IMPL(size) malloc(size)
#define VEX_CALLOC_IMPL(count, size) calloc(count, size)
#define VEX_REALLOC_IMPL(ptr, size) realloc(ptr, size)
#define VEX_FREE_IMPL(ptr) free(ptr)

/* Aligned allocation for system allocator */
static inline void *vex_alloc_aligned(size_t size, size_t alignment)
{
    void *ptr = NULL;
#if defined(_WIN32)
    ptr = _aligned_malloc(size, alignment);
#else
    if (posix_memalign(&ptr, alignment, size) != 0)
    {
        ptr = NULL;
    }
#endif
    return ptr;
}

static inline void vex_free_aligned(void *ptr)
{
#if defined(_WIN32)
    _aligned_free(ptr);
#else
    free(ptr);
#endif
}

#define vex_allocator_stats() ((void)0) /* No stats for system allocator */
#define VEX_ALLOCATOR_NAME "system"
#endif

/* ========== Convenience Functions ========== */

/**
 * Allocate memory for a specific type
 * Usage: Point* p = VEX_ALLOC_TYPE(Point);
 */
#define VEX_ALLOC_TYPE(type) ((type *)VEX_ALLOC_IMPL(sizeof(type)))

/**
 * Allocate array of specific type
 * Usage: int* arr = VEX_ALLOC_ARRAY(int, 100);
 */
#define VEX_ALLOC_ARRAY(type, count) ((type *)VEX_CALLOC_IMPL(count, sizeof(type)))

/**
 * Reallocate array with new size
 * Usage: arr = VEX_REALLOC_ARRAY(arr, int, new_count);
 */
#define VEX_REALLOC_ARRAY(ptr, type, count) ((type *)VEX_REALLOC_IMPL(ptr, (count) * sizeof(type)))

    /* Note: vex_strdup is declared in vex.h and implemented in vex_alloc.c */

    /**
     * Get current allocator name at runtime
     */
    static inline const char *vex_allocator_name(void)
    {
        return VEX_ALLOCATOR_NAME;
    }

#ifdef __cplusplus
}
#endif

#endif /* VEX_ALLOCATOR_H */
