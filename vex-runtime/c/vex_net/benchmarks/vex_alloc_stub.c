// Standalone vex_alloc implementation for benchmarks  
// Uses system malloc/free instead of full vex allocator
#include <stdlib.h>

void* vex_malloc(size_t size) {
    return malloc(size);
}

void vex_free(void* ptr) {
    free(ptr);
}
