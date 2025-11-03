/**
 * Vex Array Operations
 * Dynamic array utilities
 */

#include "vex.h"

// ============================================================================
// ARRAY METADATA STRUCTURE
// ============================================================================

// Array metadata stored before the actual data
// Memory layout: [capacity][length][data...]
typedef struct {
    int64_t capacity;
    int64_t length;
} VexArrayHeader;

#define VEX_ARRAY_HEADER(arr) ((VexArrayHeader*)(arr) - 1)

// ============================================================================
// ARRAY OPERATIONS
// ============================================================================

int64_t vex_array_len(void* arr) {
    if (!arr) {
        vex_panic("array_len: NULL array pointer");
    }
    VexArrayHeader* header = VEX_ARRAY_HEADER(arr);
    return header->length;
}

/**
 * Get array capacity (internal use)
 */
int64_t vex_array_capacity(void* arr) {
    if (!arr) {
        vex_panic("array_capacity: NULL array pointer");
    }
    VexArrayHeader* header = VEX_ARRAY_HEADER(arr);
    return header->capacity;
}

/**
 * Bounds-checked array access (returns pointer to element)
 * Panics if index out of bounds
 */
void* vex_array_get(void* arr, int64_t index, size_t elem_size) {
    if (!arr) {
        vex_panic("array_get: NULL array pointer");
    }
    
    if (elem_size == 0) {
        vex_panic("array_get: zero element size");
    }
    
    VexArrayHeader* header = VEX_ARRAY_HEADER(arr);
    int64_t len = header->length;
    
    // Bounds checking
    if (index < 0 || index >= len) {
        char msg[128];
        vex_sprintf(msg, "array_get: index out of bounds (index: %lld, length: %lld)", 
                    (long long)index, (long long)len);
        vex_panic(msg);
    }
    
    char* data = (char*)arr;
    return (void*)(data + ((size_t)index * elem_size));
}

/**
 * Bounds-checked array set (copies element to index)
 * Panics if index out of bounds
 */
void vex_array_set(void* arr, int64_t index, void* elem, size_t elem_size) {
    if (!arr) {
        vex_panic("array_set: NULL array pointer");
    }
    
    if (!elem) {
        vex_panic("array_set: NULL element pointer");
    }
    
    if (elem_size == 0) {
        vex_panic("array_set: zero element size");
    }
    
    VexArrayHeader* header = VEX_ARRAY_HEADER(arr);
    int64_t len = header->length;
    
    // Bounds checking
    if (index < 0 || index >= len) {
        char msg[128];
        vex_sprintf(msg, "array_set: index out of bounds (index: %lld, length: %lld)", 
                    (long long)index, (long long)len);
        vex_panic(msg);
    }
    
    char* data = (char*)arr;
    void* dest = (void*)(data + ((size_t)index * elem_size));
    vex_memcpy(dest, elem, elem_size);
}

void* vex_array_slice(void* arr, int64_t start, int64_t end, size_t elem_size) {
    if (!arr) {
        vex_panic("array_slice: NULL array pointer");
    }
    
    if (elem_size == 0) {
        vex_panic("array_slice: zero element size");
    }
    
    VexArrayHeader* src_header = VEX_ARRAY_HEADER(arr);
    int64_t src_len = src_header->length;
    
    // Bounds checking with panic
    if (start < 0) start = 0;
    if (end > src_len) end = src_len;
    if (start >= end) {
        vex_panic("array_slice: invalid range (start >= end)");
    }
    
    int64_t slice_len = end - start;
    
    // Check for integer overflow
    if (slice_len > (INT64_MAX / (int64_t)elem_size)) {
        vex_panic("array_slice: size calculation overflow");
    }
    
    size_t total_size = sizeof(VexArrayHeader) + (size_t)slice_len * elem_size;
    
    // Check for allocation size overflow
    if (total_size < sizeof(VexArrayHeader)) {
        vex_panic("array_slice: allocation size overflow");
    }
    
    // Allocate new array with header
    VexArrayHeader* new_header = (VexArrayHeader*)vex_malloc(total_size);
    
    if (!new_header) {
        vex_panic("array_slice: out of memory");
    }
    
    new_header->capacity = slice_len;
    new_header->length = slice_len;
    
    void* new_arr = (void*)(new_header + 1);
    char* src_data = (char*)arr + (start * elem_size);
    
    vex_memcpy(new_arr, src_data, (size_t)slice_len * elem_size);
    
    return new_arr;
}

void* vex_array_append(void* arr, void* elem, size_t elem_size) {
    if (!elem) {
        vex_panic("array_append: NULL element pointer");
    }
    
    if (elem_size == 0) {
        vex_panic("array_append: zero element size");
    }
    
    VexArrayHeader* header;
    int64_t old_len;
    int64_t old_cap;
    
    if (!arr) {
        // Create new array
        old_len = 0;
        old_cap = 0;
    } else {
        header = VEX_ARRAY_HEADER(arr);
        old_len = header->length;
        old_cap = header->capacity;
        
        // Sanity checks
        if (old_len < 0 || old_cap < 0 || old_len > old_cap) {
            vex_panic("array_append: corrupted array header");
        }
    }
    
    // Check for length overflow
    if (old_len == INT64_MAX) {
        vex_panic("array_append: array length overflow (max capacity reached)");
    }
    
    int64_t new_len = old_len + 1;
    
    // Check if reallocation needed
    if (new_len > old_cap) {
        // Grow capacity (2x strategy)
        int64_t new_cap = old_cap == 0 ? 8 : old_cap * 2;
        
        // Check for capacity overflow
        if (new_cap < 0 || new_cap < old_cap) {
            // Overflow detected, try minimal growth
            new_cap = old_cap + 1;
            if (new_cap < old_cap) {
                vex_panic("array_append: capacity overflow");
            }
        }
        
        // Check for size calculation overflow
        if (new_cap > (INT64_MAX / (int64_t)elem_size)) {
            vex_panic("array_append: size calculation overflow");
        }
        
        size_t total_size = sizeof(VexArrayHeader) + (size_t)new_cap * elem_size;
        
        // Check for allocation size overflow
        if (total_size < sizeof(VexArrayHeader)) {
            vex_panic("array_append: allocation size overflow");
        }
        
        // Allocate new array
        VexArrayHeader* new_header = (VexArrayHeader*)vex_malloc(total_size);
        
        if (!new_header) {
            vex_panic("array_append: out of memory");
        }
        
        new_header->capacity = new_cap;
        new_header->length = new_len;
        
        void* new_arr = (void*)(new_header + 1);
        
        // Copy old data
        if (arr) {
            vex_memcpy(new_arr, arr, (size_t)old_len * elem_size);
            vex_free(VEX_ARRAY_HEADER(arr));
        }
        
        // Append new element
        char* dest = (char*)new_arr + ((size_t)old_len * elem_size);
        vex_memcpy(dest, elem, elem_size);
        
        return new_arr;
    } else {
        // No reallocation needed
        header->length = new_len;
        char* dest = (char*)arr + ((size_t)old_len * elem_size);
        vex_memcpy(dest, elem, elem_size);
        return arr;
    }
}
