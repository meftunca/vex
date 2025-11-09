#include <stdio.h>
#include <string.h>

int main() {
    const char* layout = "2006-01-02T15:04:05.999999999Z07:00";
    const char* l = layout;
    
    // Find .999999999
    while (*l && !(l[0] == '.' && l[1] == '9')) l++;
    
    printf("At .999999999: [%s]\n", l);
    printf("l[0]=%c, l[1]=%c, l[2]=%c, l[3]=%c, l[4]=%c, l[5]=%c, l[6]=%c, l[7]=%c, l[8]=%c, l[9]=%c, l[10]=%c\n",
           l[0], l[1], l[2], l[3], l[4], l[5], l[6], l[7], l[8], l[9], l[10]);
    
    if (l[2] == '9' && l[3] == '9' && l[4] == '9' && l[5] == '9' && 
        l[6] == '9' && l[7] == '9' && l[8] == '9' && l[9] == '9' && l[10] == '9') {
        printf("Match! l += 10\n");
        l += 10;
        printf("Now l points to: [%s]\n", l);
        printf("l[0]=%c, l[1]=%c, l[2]=%c, l[3]=%c, l[4]=%c\n", l[0], l[1], l[2], l[3], l[4]);
        
        // Check what happens next
        if (l[0] == 'Z' && l[1] == '0' && l[2] == '7' && l[3] == ':') {
            printf("✓ Z07:00 match found\n");
        } else {
            printf("✗ Z07:00 NOT matched\n");
        }
    }
    
    return 0;
}
