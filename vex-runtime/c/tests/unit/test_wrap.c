#include <stdio.h>
#include <stddef.h>

int main() {
    size_t cap = 256;  // capacity
    size_t i = 240;    // near end
    
    for (int g = 0; g < 5; g++) {
        size_t idx = (i + 15) & (cap - 1);  // last element in group
        printf("Group %d: i=%zu, idx=%zu\n", g, i, idx);
        i += 16;  // GROUP_SIZE
    }
    
    return 0;
}
