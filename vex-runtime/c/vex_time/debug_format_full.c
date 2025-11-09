#include "include/vex_time.h"
#include "include/vex_time_layout.h"
#include <stdio.h>
#include <string.h>

int main() {
    VexTime t;
    char buf[256];
    memset(buf, 0, sizeof(buf));
    
    if (vt_parse_layout("2024-11-07T12:34:56.123456789Z", VEX_LAYOUT_RFC3339NANO, NULL, &t) != 0) {
        printf("Parse failed\n");
        return 1;
    }
    
    printf("Parsed nsec: %d\n", t.wall.nsec);
    
    int len = vt_format_layout(t, VEX_LAYOUT_RFC3339NANO, buf, sizeof(buf));
    if (len < 0) {
        printf("Format failed\n");
        return 1;
    }
    
    printf("Formatted length: %d\n", len);
    printf("Formatted: [%s]\n", buf);
    printf("Buffer contents (hex): ");
    for (int i = 0; i < len + 10; i++) {
        printf("%02x ", (unsigned char)buf[i]);
    }
    printf("\n");
    
    return 0;
}
