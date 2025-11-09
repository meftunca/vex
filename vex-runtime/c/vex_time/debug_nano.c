#include "include/vex_time.h"
#include "include/vex_time_layout.h"
#include <stdio.h>

int main() {
    VexTime t;
    char buf[256];
    
    // Parse the failing input
    if (vt_parse_layout("2024-11-07T12:34:56.123456789Z", VEX_LAYOUT_RFC3339NANO, NULL, &t) != 0) {
        printf("Parse failed\n");
        return 1;
    }
    
    printf("Parsed nsec: %d\n", t.wall.nsec);
    
    // Format it back
    int len = vt_format_layout(t, VEX_LAYOUT_RFC3339NANO, buf, sizeof(buf));
    if (len < 0) {
        printf("Format failed\n");
        return 1;
    }
    
    printf("Formatted: %s\n", buf);
    return 0;
}
