#include <stdio.h>
#include "vex.h"

int main() {
    VexMap m;
    vex_map_new(&m, 8);
    
    int val0 = 999;
    int val1 = 111;
    
    // Insert with empty string
    if (!vex_map_insert(&m, "", &val0)) {
        printf("Failed to insert empty key\n");
        return 1;
    }
    
    printf("Inserted empty key\n");
    printf("Map length: %zu\n", vex_map_len(&m));
    
    // Try to get it back
    int *p = (int*)vex_map_get(&m, "");
    if (!p) {
        printf("ERROR: Empty key not found!\n");
        return 1;
    }
    
    printf("Found empty key: value = %d (expected 999)\n", *p);
    
    // Insert another key
    vex_map_insert(&m, "key1", &val1);
    printf("Map length after key1: %zu\n", vex_map_len(&m));
    
    // Check empty key again
    p = (int*)vex_map_get(&m, "");
    if (!p) {
        printf("ERROR: Empty key lost after inserting key1!\n");
        return 1;
    }
    
    printf("SUCCESS: Empty key still there: %d\n", *p);
    
    vex_map_free(&m);
    return 0;
}
