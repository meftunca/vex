/*
 * Go-style Layout Formatter for vex_time
 * 
 * Implements time.Format() equivalent
 */

#include "../../include/vex_time_layout.h"
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

/* Helper: append string */
static int append_str(char** buf, size_t* remain, const char* s) {
    size_t len = strlen(s);
    if (*remain < len + 1) return -1;
    memcpy(*buf, s, len);
    *buf += len;
    *remain -= len;
    return 0;
}

/* Helper: append formatted integer */
static int append_int(char** buf, size_t* remain, int val, int width, char pad) {
    char tmp[32];
    int len = snprintf(tmp, sizeof(tmp), "%0*d", width, val);
    if (len < 0 || len >= (int)sizeof(tmp)) return -1;
    
    /* Replace '0' with pad character if needed */
    if (pad == ' ') {
        for (int i = 0; i < len && tmp[i] == '0'; i++) {
            tmp[i] = ' ';
        }
    }
    
    if (*remain < (size_t)len + 1) return -1;
    memcpy(*buf, tmp, len);
    *buf += len;
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
    
    /* Extract components */
    VexInstant wall = t.wall;
    time_t unix_time = (time_t)wall.unix_sec;
    
    struct tm tm;
#ifdef _WIN32
    if (gmtime_s(&tm, &unix_time) != 0) return -1;
#else
    if (gmtime_r(&unix_time, &tm) == NULL) return -1;
#endif
    
    int year = tm.tm_year + 1900;
    int month = tm.tm_mon + 1;
    int day = tm.tm_mday;
    int hour = tm.tm_hour;
    int minute = tm.tm_min;
    int second = tm.tm_sec;
    int nsec = wall.nsec;
    int weekday = tm.tm_wday;
    
    /* Process layout */
    while (*l && remain > 1) {
        if (strncmp(l, "2006", 4) == 0) {
            /* 4-digit year */
            if (append_int(&buf, &remain, year, 4, '0') != 0) return -1;
            l += 4;
        } else if (strncmp(l, "06", 2) == 0) {
            /* 2-digit year */
            if (append_int(&buf, &remain, year % 100, 2, '0') != 0) return -1;
            l += 2;
        } else if (strncmp(l, "January", 7) == 0) {
            /* Full month name */
            if (append_str(&buf, &remain, month_names[month - 1]) != 0) return -1;
            l += 7;
        } else if (strncmp(l, "Jan", 3) == 0) {
            /* Abbreviated month name */
            if (append_str(&buf, &remain, month_abbr[month - 1]) != 0) return -1;
            l += 3;
        } else if (strncmp(l, "01", 2) == 0) {
            /* 2-digit month */
            if (append_int(&buf, &remain, month, 2, '0') != 0) return -1;
            l += 2;
        } else if (strncmp(l, "_2", 2) == 0) {
            /* Right-justified day */
            if (append_int(&buf, &remain, day, 2, ' ') != 0) return -1;
            l += 2;
        } else if (strncmp(l, "02", 2) == 0) {
            /* 2-digit day */
            if (append_int(&buf, &remain, day, 2, '0') != 0) return -1;
            l += 2;
        } else if (*l == '2') {
            /* 1 or 2-digit day */
            if (day < 10) {
                if (append_int(&buf, &remain, day, 1, '0') != 0) return -1;
            } else {
                if (append_int(&buf, &remain, day, 2, '0') != 0) return -1;
            }
            l++;
        } else if (strncmp(l, "Monday", 6) == 0) {
            /* Full weekday */
            if (append_str(&buf, &remain, weekday_names[weekday]) != 0) return -1;
            l += 6;
        } else if (strncmp(l, "Mon", 3) == 0) {
            /* Abbreviated weekday */
            if (append_str(&buf, &remain, weekday_abbr[weekday]) != 0) return -1;
            l += 3;
        } else if (strncmp(l, "15", 2) == 0) {
            /* 24-hour */
            if (append_int(&buf, &remain, hour, 2, '0') != 0) return -1;
            l += 2;
        } else if (strncmp(l, "03", 2) == 0) {
            /* 12-hour (padded) */
            int h12 = (hour % 12);
            if (h12 == 0) h12 = 12;
            if (append_int(&buf, &remain, h12, 2, '0') != 0) return -1;
            l += 2;
        } else if (*l == '3') {
            /* 12-hour (1 or 2 digit) */
            int h12 = (hour % 12);
            if (h12 == 0) h12 = 12;
            if (h12 < 10) {
                if (append_int(&buf, &remain, h12, 1, '0') != 0) return -1;
            } else {
                if (append_int(&buf, &remain, h12, 2, '0') != 0) return -1;
            }
            l++;
        } else if (strncmp(l, "04", 2) == 0) {
            /* 2-digit minute */
            if (append_int(&buf, &remain, minute, 2, '0') != 0) return -1;
            l += 2;
        } else if (*l == '4') {
            /* 1 or 2-digit minute */
            if (minute < 10) {
                if (append_int(&buf, &remain, minute, 1, '0') != 0) return -1;
            } else {
                if (append_int(&buf, &remain, minute, 2, '0') != 0) return -1;
            }
            l++;
        } else if (strncmp(l, "05", 2) == 0) {
            /* 2-digit second */
            if (append_int(&buf, &remain, second, 2, '0') != 0) return -1;
            l += 2;
        } else if (*l == '5') {
            /* 1 or 2-digit second */
            if (second < 10) {
                if (append_int(&buf, &remain, second, 1, '0') != 0) return -1;
            } else {
                if (append_int(&buf, &remain, second, 2, '0') != 0) return -1;
            }
            l++;
        } else if (strncmp(l, ".999999999", 10) == 0) {
            /* Fractional seconds: 9 digits (trailing zeros omitted) */
            l += 10;
            if (nsec > 0) {
                int tmp_nsec = nsec;
                int digits = 9;
                while (digits > 0 && tmp_nsec % 10 == 0) {
                    tmp_nsec /= 10;
                    digits--;
                }
                char frac[16];
                snprintf(frac, sizeof(frac), ".%0*d", digits, tmp_nsec);
                if (append_str(&buf, &remain, frac) != 0) return -1;
            }
        } else if (strncmp(l, ".000000000", 10) == 0) {
            /* Fractional seconds: 9 digits (trailing zeros included) */
            l += 10;
            char frac[16];
            snprintf(frac, sizeof(frac), ".%09d", nsec);
            if (append_str(&buf, &remain, frac) != 0) return -1;
        } else if (strncmp(l, ".000000", 7) == 0) {
            /* Fractional seconds: 6 digits (microseconds) */
            l += 7;
            char frac[16];
            snprintf(frac, sizeof(frac), ".%06d", nsec / 1000);
            if (append_str(&buf, &remain, frac) != 0) return -1;
        } else if (strncmp(l, ".000", 4) == 0) {
            /* Fractional seconds: 3 digits (milliseconds) */
            l += 4;
            char frac[16];
            snprintf(frac, sizeof(frac), ".%03d", nsec / 1000000);
            if (append_str(&buf, &remain, frac) != 0) return -1;
        } else if (strncmp(l, ".9", 2) == 0) {
            /* Fractional seconds: variable digits (trailing zeros omitted) */
            l += 2;
            if (nsec > 0) {
                int tmp_nsec = nsec;
                int digits = 9;
                while (digits > 0 && tmp_nsec % 10 == 0) {
                    tmp_nsec /= 10;
                    digits--;
                }
                char frac[16];
                snprintf(frac, sizeof(frac), ".%0*d", digits, tmp_nsec);
                if (append_str(&buf, &remain, frac) != 0) return -1;
            }
        } else if (strncmp(l, ".0", 2) == 0) {
            /* Fractional seconds: 1 digit */
            l += 2;
            char frac[16];
            snprintf(frac, sizeof(frac), ".%01d", nsec / 100000000);
            if (append_str(&buf, &remain, frac) != 0) return -1;
        } else if (strncmp(l, "PM", 2) == 0) {
            /* AM/PM (uppercase) */
            if (append_str(&buf, &remain, hour >= 12 ? "PM" : "AM") != 0) return -1;
            l += 2;
        } else if (strncmp(l, "pm", 2) == 0) {
            /* am/pm (lowercase) */
            if (append_str(&buf, &remain, hour >= 12 ? "pm" : "am") != 0) return -1;
            l += 2;
        } else if (strncmp(l, "Z07:00", 6) == 0) {
            /* "Z" or numeric timezone with colon */
            /* For now, always output Z (UTC) */
            if (append_char(&buf, &remain, 'Z') != 0) return -1;
            l += 6;
        } else if (strncmp(l, "Z0700", 5) == 0) {
            /* "Z" or numeric timezone */
            if (append_char(&buf, &remain, 'Z') != 0) return -1;
            l += 5;
        } else if (strncmp(l, "-07:00", 6) == 0) {
            /* Numeric timezone with colon */
            if (append_str(&buf, &remain, "+00:00") != 0) return -1;
            l += 6;
        } else if (strncmp(l, "-0700", 5) == 0) {
            /* Numeric timezone */
            if (append_str(&buf, &remain, "+0000") != 0) return -1;
            l += 5;
        } else if (strncmp(l, "MST", 3) == 0) {
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

