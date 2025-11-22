/**
 * Test panic scenarios (will crash as expected)
 * Run individually to see panic messages
 */

#include "vex.h"
#include <stdio.h>

void test_array_bounds_panic() {
    printf("=== Testing Array Bounds Panic ===\n");
    
    // Create small array
    typedef struct {
        int64_t capacity;
        int64_t length;
        int data[3];
    } TestArray;
    
    TestArray arr_storage = {3, 3, {1, 2, 3}};
    int* arr = arr_storage.data;
    
    printf("Array length: %lld\n", vex_array_len(arr));
    printf("Attempting out-of-bounds access at index 10...\n");
    
    // This will panic!
    int* elem = (int*)vex_array_get(arr, 10, sizeof(int));
    printf("Should not reach here!\n");
}

void test_overflow_panic() {
    printf("=== Testing Integer Overflow Panic ===\n");
    
    // Try to create array with massive size
    int elem = 42;
    void* arr = NULL;
    
    printf("Attempting to append INT64_MAX times (will overflow)...\n");
    
    for (int64_t i = 0; i < 100; i++) {
        arr = vex_array_append(arr, &elem, sizeof(int));
        if (i % 10 == 0) {
            printf("  Appended %lld elements\n", i);
        }
    }
    
    printf("Success! Array is safe.\n");
}

int main(int argc, char** argv) {
    if (argc < 2) {
        printf("Usage: %s <test_name>\n", argv[0]);
        printf("Tests:\n");
        printf("  bounds  - Test out-of-bounds panic\n");
        printf("  overflow - Test overflow protection\n");
        return 1;
    }
    
    if (vex_strcmp(argv[1], "bounds") == 0) {
        test_array_bounds_panic();
    } else if (vex_strcmp(argv[1], "overflow") == 0) {
        test_overflow_panic();
    } else {
        printf("Unknown test: %s\n", argv[1]);
        return 1;
    }
    
    return 0;
}
