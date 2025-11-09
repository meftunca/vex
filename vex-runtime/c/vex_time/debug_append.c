#include <stdio.h>
#include <string.h>

static int append_int_no_trailing_zeros(char** buf, size_t* remain, int val, int max_digits) {
    if (*remain < 32) return -1;
    if (val == 0) return 0;
    
    printf("DEBUG: val=%d, max_digits=%d\n", val, max_digits);
    
    int tmp = val;
    int digits = max_digits;
    while (digits > 0 && tmp % 10 == 0) {
        tmp /= 10;
        digits--;
    }
    
    printf("DEBUG: After removing trailing zeros: tmp=%d, digits=%d\n", tmp, digits);
    
    char* start = *buf;
    char* end = *buf;
    int count = 0;
    do {
        *end++ = '0' + (tmp % 10);
        tmp /= 10;
        count++;
    } while (tmp > 0);
    *end = '\0';
    
    printf("DEBUG: Wrote %d digits in reverse\n", count);
    
    char* rev_end = end - 1;
    while (start < rev_end) {
        char tmp_c = *start;
        *start = *rev_end;
        *rev_end = tmp_c;
        start++;
        rev_end--;
    }
    
    size_t len = end - *buf;
    printf("DEBUG: Final length=%zu, buffer=[%.*s]\n", len, (int)len, *buf);
    *buf = end;
    *remain -= len;
    return 0;
}

int main() {
    char buf[256] = {0};
    char* p = buf;
    size_t remain = sizeof(buf);
    
    int nsec = 123456789;
    append_int_no_trailing_zeros(&p, &remain, nsec, 9);
    
    printf("Final buffer: [%s]\n", buf);
    return 0;
}
