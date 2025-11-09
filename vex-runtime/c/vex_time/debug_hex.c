#include "include/vex_time.h"
#include "include/vex_time_layout.h"
#include <stdio.h>
#include <string.h>

int main() {
    VexTime t;
    char buf[256];
    
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
    
    printf("Formatted: %s\n", buf);
    printf("Length: %d\n", len);
    
    // Find the fractional part
    char* dot = strchr(buf, '.');
    if (dot) {
        printf("Fractional part: '%s'\n", dot);
        printf("Hex dump of fractional:\n");
        for (int i = 0; dot[i] && dot[i] != 'Z'; i++) {
            printf("  [%d]: '%c' (0x%02x)\n", i, dot[i], (unsigned char)dot[i]);
        }
    }
    
    return 0;
}
