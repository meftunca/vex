#include <stdlib.h>
#include <stdio.h>
#include "internal.h"

// Always use vex_malloc/vex_free from vex_alloc.c (uses configured allocator)
extern void* vex_malloc(size_t size);
extern void vex_free(void* ptr);

void* xmalloc(size_t n) {
    void* p = vex_malloc(n);
    if (!p) {
        fprintf(stderr, "Out of memory\n");
        abort();
    }
    return p;
}

void xfree(void* p) { 
    vex_free(p); 
}
