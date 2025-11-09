/*
 * Go-style Layout Formatter for vex_time
 * 
 * Implements time.Format() equivalent
 */

#include "../../include/vex_time_layout.h"
#include "fast_parse.h"  /* For fast_date_from_epoch */
#include <string.h>
#include <stdio.h>
#include <time.h>

/* Month names */
static const char* month_names[] = {
    "January", "February", "March", "April", "May", "June",
    "July", "August", "September", "October", "November", "December"
};

static const char* month_abbr[] = {
    "Jan", "Feb", "Mar", "Apr", "May", "Jun",
    "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"
};

/* Weekday names */
static const char* weekday_names[] = {
    "Sunday", "Monday", "Tuesday", "Wednesday",
    "Thursday", "Friday", "Saturday"
};

static const char* weekday_abbr[] = {
    "Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"
};

/* Fast integer-to-string with width and padding (no snprintf) */
static inline char* fast_itoa_width(char* p, int val, int width, char pad) {
    char* start = p;
    if (val == 0) {
        for (int i = 0; i < width; i++) *p++ = pad == ' ' ? ' ' : '0';
        return p;
    }
    
    /* Write digits in reverse */
    char* end = p;
    int digits = 0;
    do {
        *end++ = '0' + (val % 10);
        val /= 10;
        digits++;
    } while (val > 0);
    
    /* Pad with pad character */
    while (digits < width) {
        *end++ = pad;
        digits++;
    }
    *end = '\0';
    
    /* Reverse */
    char* rev_end = end - 1;
    while (start < rev_end) {
        char tmp = *start;
        *start = *rev_end;
        *rev_end = tmp;
        start++;
        rev_end--;
    }
    
    return end;
}

/* Helper: append string (optimized: use known lengths) */
static inline int append_str_fast(char** buf, size_t* remain, const char* s, size_t len) {
    if (*remain < len + 1) return -1;
    memcpy(*buf, s, len);
    *buf += len;
    *remain -= len;
    return 0;
}

/* Helper: append string (with strlen fallback for unknown lengths) */
static int append_str(char** buf, size_t* remain, const char* s) {
    /* Optimize common cases */
    if (s == month_names[0]) return append_str_fast(buf, remain, s, 7);  /* January */
    if (s == month_abbr[0]) return append_str_fast(buf, remain, s, 3);  /* Jan */
    if (s == weekday_names[0]) return append_str_fast(buf, remain, s, 6);  /* Sunday */
    if (s == weekday_abbr[0]) return append_str_fast(buf, remain, s, 3);  /* Sun */
    
    size_t len = strlen(s);
    return append_str_fast(buf, remain, s, len);
}

/* Helper: append formatted integer (optimized: no snprintf) */
static int append_int(char** buf, size_t* remain, int val, int width, char pad) {
    if (*remain < 32) return -1;  /* Safety check */
    char* new_buf = fast_itoa_width(*buf, val, width, pad);
    size_t len = new_buf - *buf;
    *buf = new_buf;
    *remain -= len;
    return 0;
}

/* Helper: append integer without trailing zeros (for fractional seconds) */
static int append_int_no_trailing_zeros(char** buf, size_t* remain, int val, int max_digits) {
    if (*remain < 32) return -1;
    if (val == 0) return 0;  /* Skip if zero */
    
    /* Count trailing zeros */
    int tmp = val;
    int digits = max_digits;
    while (digits > 0 && tmp % 10 == 0) {
        tmp /= 10;
        digits--;
    }
    
    /* Write digits in reverse */
    char* start = *buf;
    char* end = *buf;
    do {
        *end++ = '0' + (tmp % 10);
        tmp /= 10;
    } while (tmp > 0);
    *end = '\0';
    
    /* Reverse */
    char* rev_end = end - 1;
    while (start < rev_end) {
        char tmp_c = *start;
        *start = *rev_end;
        *rev_end = tmp_c;
        start++;
        rev_end--;
    }
    
    size_t len = end - *buf;
    *buf = end;
    *remain -= len;
    return 0;
}

/* Helper: append character */
static int append_char(char** buf, size_t* remain, char c) {
    if (*remain < 2) return -1;
    **buf = c;
    (*buf)++;
    (*remain)--;
    return 0;
}

/* Helper: get weekday from date */
static int get_weekday(int year, int month, int day) {
    /* Zeller's congruence for Gregorian calendar */
    if (month < 3) {
        month += 12;
        year--;
    }
    int q = day;
    int m = month;
    int k = year % 100;
    int j = year / 100;
    int h = (q + ((13 * (m + 1)) / 5) + k + (k / 4) + (j / 4) - (2 * j)) % 7;
    return (h + 6) % 7;  /* Convert to 0=Sunday */
}

/* Main layout formatter */
int vt_format_layout(VexTime t, const char* layout, char* buf, size_t buflen) {
    if (!layout || !buf || buflen == 0) return -1;
    
    char* start = buf;
    size_t remain = buflen;
    const char* l = layout;
    
    /* Extract components using fast epoch-to-date (no gmtime_r overhead!) */
    VexInstant wall = t.wall;
    int64_t unix_time = wall.unix_sec;
    
    int year, month, day, hour, minute, second, weekday;
    fast_date_from_epoch(unix_time, &year, &month, &day, &hour, &minute, &second, &weekday);
    int nsec = wall.nsec;
    
    /* Process layout - optimized: direct character comparison instead of strncmp */
    while (*l && remain > 1) {
        /* Fast path: check first character for branch prediction */
        if (l[0] == '2') {
            if (l[1] == '0' && l[2] == '0' && l[3] == '6') {
                /* 4-digit year */
                if (append_int(&buf, &remain, year, 4, '0') != 0) return -1;
                l += 4;
            } else if (l[1] == '_') {
                /* Right-justified day */
                if (append_int(&buf, &remain, day, 2, ' ') != 0) return -1;
                l += 2;
            } else if (l[1] == '0' || l[1] == '1' || l[1] == '2' || l[1] == '3' || l[1] == '4' || l[1] == '5' || l[1] == '6' || l[1] == '7' || l[1] == '8' || l[1] == '9') {
                /* 2-digit day */
                if (append_int(&buf, &remain, day, 2, '0') != 0) return -1;
                l += 2;
            } else {
                /* 1 or 2-digit day */
                if (day < 10) {
                    if (append_int(&buf, &remain, day, 1, '0') != 0) return -1;
                } else {
                    if (append_int(&buf, &remain, day, 2, '0') != 0) return -1;
                }
                l++;
            }
        } else if (l[0] == '0') {
            if (l[1] == '6') {
                /* 2-digit year */
                if (append_int(&buf, &remain, year % 100, 2, '0') != 0) return -1;
                l += 2;
            } else if (l[1] == '1') {
                /* 2-digit month */
                if (append_int(&buf, &remain, month, 2, '0') != 0) return -1;
                l += 2;
            } else if (l[1] == '2') {
                /* 2-digit day */
                if (append_int(&buf, &remain, day, 2, '0') != 0) return -1;
                l += 2;
            } else if (l[1] == '3') {
                /* 12-hour (padded) */
                int h12 = (hour % 12);
                if (h12 == 0) h12 = 12;
                if (append_int(&buf, &remain, h12, 2, '0') != 0) return -1;
                l += 2;
            } else if (l[1] == '4') {
                /* 2-digit minute */
                if (append_int(&buf, &remain, minute, 2, '0') != 0) return -1;
                l += 2;
            } else if (l[1] == '5') {
                /* 2-digit second */
                if (append_int(&buf, &remain, second, 2, '0') != 0) return -1;
                l += 2;
            } else if (l[1] == '0' && l[2] == '0' && l[3] == '0') {
                if (l[4] == '0' && l[5] == '0' && l[6] == '0' && l[7] == '0' && l[8] == '0' && l[9] == '0') {
                    /* .000000000 - 9 digits */
                    l += 10;
                    if (append_char(&buf, &remain, '.') != 0) return -1;
                    if (append_int(&buf, &remain, nsec, 9, '0') != 0) return -1;
                } else if (l[4] == '0' && l[5] == '0' && l[6] == '\0') {
                    /* .000000 - 6 digits */
                    l += 7;
                    if (append_char(&buf, &remain, '.') != 0) return -1;
                    if (append_int(&buf, &remain, nsec / 1000, 6, '0') != 0) return -1;
                } else {
                    /* .000 - 3 digits */
                    l += 4;
                    if (append_char(&buf, &remain, '.') != 0) return -1;
                    if (append_int(&buf, &remain, nsec / 1000000, 3, '0') != 0) return -1;
                }
            } else {
                /* Literal */
                if (append_char(&buf, &remain, *l) != 0) return -1;
                l++;
            }
        } else if (l[0] == 'J' && l[1] == 'a' && l[2] == 'n') {
            if (l[3] == 'u' && l[4] == 'a' && l[5] == 'r' && l[6] == 'y') {
                /* Full month name */
                if (append_str(&buf, &remain, month_names[month - 1]) != 0) return -1;
                l += 7;
            } else {
                /* Abbreviated month name */
                if (append_str(&buf, &remain, month_abbr[month - 1]) != 0) return -1;
                l += 3;
            }
        } else if (l[0] == 'M' && l[1] == 'o' && l[2] == 'n') {
            if (l[3] == 'd' && l[4] == 'a' && l[5] == 'y') {
                /* Full weekday */
                if (append_str(&buf, &remain, weekday_names[weekday]) != 0) return -1;
                l += 6;
            } else {
                /* Abbreviated weekday */
                if (append_str(&buf, &remain, weekday_abbr[weekday]) != 0) return -1;
                l += 3;
            }
        } else if (l[0] == '1' && l[1] == '5') {
            /* 24-hour */
            if (append_int(&buf, &remain, hour, 2, '0') != 0) return -1;
            l += 2;
        } else if (l[0] == '3') {
            /* 12-hour (1 or 2 digit) */
            int h12 = (hour % 12);
            if (h12 == 0) h12 = 12;
            if (h12 < 10) {
                if (append_int(&buf, &remain, h12, 1, '0') != 0) return -1;
            } else {
                if (append_int(&buf, &remain, h12, 2, '0') != 0) return -1;
            }
            l++;
        } else if (l[0] == '4') {
            /* 1 or 2-digit minute */
            if (minute < 10) {
                if (append_int(&buf, &remain, minute, 1, '0') != 0) return -1;
            } else {
                if (append_int(&buf, &remain, minute, 2, '0') != 0) return -1;
            }
            l++;
        } else if (l[0] == '5') {
            /* 1 or 2-digit second */
            if (second < 10) {
                if (append_int(&buf, &remain, second, 1, '0') != 0) return -1;
            } else {
                if (append_int(&buf, &remain, second, 2, '0') != 0) return -1;
            }
            l++;
        } else if (l[0] == '.' && l[1] == '9') {
            if (l[2] == '9' && l[3] == '9' && l[4] == '9' && l[5] == '9' && l[6] == '9' && l[7] == '9' && l[8] == '9' && l[9] == '9' && l[10] == '9') {
                /* .999999999 - 9 digits (trailing zeros omitted) */
                l += 10;
                if (nsec > 0) {
                    if (append_char(&buf, &remain, '.') != 0) return -1;
                    if (append_int_no_trailing_zeros(&buf, &remain, nsec, 9) != 0) return -1;
                }
            } else {
                /* .9 - variable digits (trailing zeros omitted) */
                l += 2;
                if (nsec > 0) {
                    if (append_char(&buf, &remain, '.') != 0) return -1;
                    if (append_int_no_trailing_zeros(&buf, &remain, nsec, 9) != 0) return -1;
                }
            }
        } else if (l[0] == '.' && l[1] == '0') {
            /* .0 - 1 digit */
            l += 2;
            if (append_char(&buf, &remain, '.') != 0) return -1;
            if (append_int(&buf, &remain, nsec / 100000000, 1, '0') != 0) return -1;
        } else if (l[0] == 'P' && l[1] == 'M') {
            /* AM/PM (uppercase) */
            if (append_str(&buf, &remain, hour >= 12 ? "PM" : "AM") != 0) return -1;
            l += 2;
        } else if (l[0] == 'p' && l[1] == 'm') {
            /* am/pm (lowercase) */
            if (append_str(&buf, &remain, hour >= 12 ? "pm" : "am") != 0) return -1;
            l += 2;
        } else if (l[0] == 'Z' && l[1] == '0' && l[2] == '7' && l[3] == ':') {
            /* "Z" or numeric timezone with colon */
            if (append_char(&buf, &remain, 'Z') != 0) return -1;
            l += 6;
        } else if (l[0] == 'Z' && l[1] == '0' && l[2] == '7' && l[3] == '0' && l[4] == '0') {
            /* "Z" or numeric timezone */
            if (append_char(&buf, &remain, 'Z') != 0) return -1;
            l += 5;
        } else if (l[0] == '-' && l[1] == '0' && l[2] == '7' && l[3] == ':') {
            /* Numeric timezone with colon */
            if (append_str(&buf, &remain, "+00:00") != 0) return -1;
            l += 6;
        } else if (l[0] == '-' && l[1] == '0' && l[2] == '7' && l[3] == '0' && l[4] == '0') {
            /* Numeric timezone */
            if (append_str(&buf, &remain, "+0000") != 0) return -1;
            l += 5;
        } else if (l[0] == 'M' && l[1] == 'S' && l[2] == 'T') {
            /* Timezone abbreviation */
            if (append_str(&buf, &remain, "UTC") != 0) return -1;
            l += 3;
        } else {
            /* Literal character */
            if (append_char(&buf, &remain, *l) != 0) return -1;
            l++;
        }
    }
    
    /* Null terminate */
    if (remain < 1) return -1;
    *buf = '\0';
    
    return (int)(buf - start);
}

/* Format instant (UTC only) */
int vt_format_instant_layout(VexInstant t, const char* layout, char* buf, size_t buflen) {
    VexTime time;
    time.wall = t;
    time.mono_ns = 0;  /* Not used in formatting */
    return vt_format_layout(time, layout, buf, buflen);
}

