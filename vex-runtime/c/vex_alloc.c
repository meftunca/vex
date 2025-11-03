/**
 * Vex Memory Allocation
 * Wrapper around system malloc (will be static linked with musl)
 */

#include "vex.h"
#include <stdlib.h>

// ============================================================================
// MEMORY ALLOCATION
// ============================================================================

void* vex_malloc(size_t size) {
    return malloc(size);
}

void* vex_calloc(size_t nmemb, size_t size) {
    return calloc(nmemb, size);
}

void* vex_realloc(void* ptr, size_t size) {
    return realloc(ptr, size);
}

void vex_free(void* ptr) {
    free(ptr);
}
