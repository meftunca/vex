#include "include/vex_time_layout.h"
#include <stdio.h>
#include <string.h>

int main() {
    printf("VEX_LAYOUT_RFC3339NANO = '%s'\n", VEX_LAYOUT_RFC3339NANO);
    printf("Length: %zu\n", strlen(VEX_LAYOUT_RFC3339NANO));
    
    // Print fractional part
    const char* frac = strchr(VEX_LAYOUT_RFC3339NANO, '.');
    if (frac) {
        printf("Fractional part: '%s'\n", frac);
    }
    
    return 0;
}
