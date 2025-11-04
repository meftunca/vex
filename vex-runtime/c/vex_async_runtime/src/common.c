#include <stdlib.h>
#include <stdio.h>
#include "internal.h"

void* xmalloc(size_t n) {
    void* p = malloc(n);
    if (!p) {
        fprintf(stderr, "Out of memory\n");
        abort();
    }
    return p;
}
void xfree(void* p) { free(p); }
