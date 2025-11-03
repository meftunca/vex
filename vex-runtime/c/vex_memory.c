/**
 * Vex Memory Operations
 * Zero-overhead, SIMD-optimizable implementations
 */

#include "vex.h"

// ============================================================================
// MEMORY OPERATIONS
// ============================================================================

void* vex_memcpy(void* dest, const void* src, size_t n) {
    char* d = (char*)dest;
    const char* s = (const char*)src;
    
    // Simple byte-by-byte copy
    // TODO: Optimize with word-aligned copy or SIMD
    while (n--) {
        *d++ = *s++;
    }
    
    return dest;
}

void* vex_memmove(void* dest, const void* src, size_t n) {
    char* d = (char*)dest;
    const char* s = (const char*)src;
    
    // Handle overlapping regions
    if (d < s) {
        // Copy forward
        while (n--) {
            *d++ = *s++;
        }
    } else if (d > s) {
        // Copy backward
        d += n;
        s += n;
        while (n--) {
            *--d = *--s;
        }
    }
    
    return dest;
}

void* vex_memset(void* s, int c, size_t n) {
    unsigned char* p = (unsigned char*)s;
    unsigned char value = (unsigned char)c;
    
    // Simple byte-by-byte set
    // TODO: Optimize with word-aligned set or SIMD
    while (n--) {
        *p++ = value;
    }
    
    return s;
}

int vex_memcmp(const void* s1, const void* s2, size_t n) {
    const unsigned char* p1 = (const unsigned char*)s1;
    const unsigned char* p2 = (const unsigned char*)s2;
    
    while (n--) {
        if (*p1 != *p2) {
            return *p1 - *p2;
        }
        p1++;
        p2++;
    }
    
    return 0;
}
