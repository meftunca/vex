#include <stdlib.h>
#include <stdio.h>
#include "internal.h"

// Use vex_malloc if available, fallback to malloc
#ifdef VEX_RUNTIME_INTEGRATED
extern void* vex_malloc(size_t size);
extern void vex_free(void* ptr);
#define XMALLOC vex_malloc
#define XFREE vex_free
#else
#define XMALLOC malloc
#define XFREE free
#endif

void* xmalloc(size_t n) {
    void* p = XMALLOC(n);
    if (!p) {
        fprintf(stderr, "Out of memory\n");
        abort();
    }
    return p;
}
void xfree(void* p) { XFREE(p); }
