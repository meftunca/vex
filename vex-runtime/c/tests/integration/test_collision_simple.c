#include <stdio.h>
#include <string.h>
#include "vex.h"

int main(void) {
    VexMap m;
    if (!vex_map_new(&m, 16)) {
        fprintf(stderr, "map_new failed\n");
        return 1;
    }
    
    printf("Testing collision handling with 100 similar keys...\n");
    
    int vals[100];
    for (int i = 0; i < 100; i++) {
        char key[64];
        snprintf(key, sizeof(key), "prefix_collision_key_%d", i);
        vals[i] = i * 13 + 7;
        
        if (!vex_map_insert(&m, key, &vals[i])) {
            fprintf(stderr, "Insert failed at iteration %d\n", i);
            return 1;
        }
        
        if (i % 10 == 0) {
            printf("  Inserted %d items, map length: %zu\n", i + 1, vex_map_len(&m));
        }
    }
    
    printf("All 100 items inserted successfully!\n");
    printf("Final map length: %zu\n", vex_map_len(&m));
    
    vex_map_free(&m);
    return 0;
}
