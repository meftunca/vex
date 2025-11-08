#include "include/vex_time.h"
#include "include/vex_time_layout.h"
#include <stdio.h>

int main() {
    VexTime t;
    const char* value = "12:34:56";
    const char* layout = "15:04:05";
    
    printf("Testing parse:\n");
    printf("  Value:  %s\n", value);
    printf("  Layout: %s\n", layout);
    
    int result = vt_parse_layout(value, layout, NULL, &t);
    
    if (result == 0) {
        printf("  ✅ Parse succeeded!\n");
        printf("  Unix time: %lld\n", (long long)t.wall.unix_sec);
    } else {
        printf("  ❌ Parse failed with error code: %d\n", result);
    }
    
    return result;
}

