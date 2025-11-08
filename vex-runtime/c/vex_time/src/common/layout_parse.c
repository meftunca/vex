/*
 * Go-style Layout Parser for vex_time
 * 
 * Implements time.Parse() equivalent
 */

#include "../../include/vex_time_layout.h"
#include "../fast_parse.h"  /* For fast_epoch_from_date */
#include <string.h>
#include <ctype.h>
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

/* Helper: parse integer (1 or 2 digits) */
static int parse_int_1or2(const char** s, int* out) {
    if (!isdigit((unsigned char)**s)) return -1;
    *out = **s - '0';
    (*s)++;
    if (isdigit((unsigned char)**s)) {
        *out = *out * 10 + (**s - '0');
        (*s)++;
    }
    return 0;
}

/* Helper: parse integer (exactly n digits) */
static int parse_int_n(const char** s, int n, int* out) {
    *out = 0;
    for (int i = 0; i < n; i++) {
        if (!isdigit((unsigned char)**s)) return -1;
        *out = *out * 10 + (**s - '0');
        (*s)++;
    }
    return 0;
}

/* Helper: parse month name */
static int parse_month_name(const char** s, int* month, int full) {
    const char** names = full ? month_names : month_abbr;
    int len = full ? 0 : 3;
    
    for (int i = 0; i < 12; i++) {
        int name_len = full ? (int)strlen(names[i]) : len;
        if (strncmp(*s, names[i], name_len) == 0) {
            *month = i + 1;
            *s += name_len;
            return 0;
        }
    }
    return -1;
}

/* Helper: parse weekday name */
static int parse_weekday_name(const char** s, int full) {
    const char** names = full ? weekday_names : weekday_abbr;
    int len = full ? 0 : 3;
    
    for (int i = 0; i < 7; i++) {
        int name_len = full ? (int)strlen(names[i]) : len;
        if (strncmp(*s, names[i], name_len) == 0) {
            *s += name_len;
            return 0; /* We don't validate weekday */
        }
    }
    return -1;
}

/* Helper: parse timezone offset */
static int parse_tz_offset(const char** s, int with_colon, int* offset) {
    if (**s != '+' && **s != '-') return -1;
    int sign = (**s == '-') ? -1 : 1;
    (*s)++;
    
    int hour, min = 0;
    if (parse_int_n(s, 2, &hour) != 0) return -1;
    
    if (with_colon) {
        if (**s != ':') return -1;
        (*s)++;
    }
    
    if (parse_int_n(s, 2, &min) != 0) return -1;
    
    *offset = sign * (hour * 3600 + min * 60);
    return 0;
}

/* Helper: skip whitespace in value */
static void skip_ws(const char** s) {
    while (**s == ' ' || **s == '\t') (*s)++;
}

/* Main layout parser */
int vt_parse_layout(const char* value, const char* layout, VexTz* tz, VexTime* out) {
    if (!value || !layout || !out) return -1;
    
    const char* v = value;
    const char* l = layout;
    
    /* Parsed components */
    int year = 0, month = 0, day = 0;
    int hour = 0, minute = 0, second = 0, nsec = 0;
    int tz_offset = 0;
    int has_year = 0, has_month = 0, has_day = 0;
    int has_time = 0;
    int is_pm = 0, is_12h = 0;
    
    while (*l) {
        /* Check for layout components */
        if (strncmp(l, "2006", 4) == 0) {
            /* 4-digit year */
            if (parse_int_n(&v, 4, &year) != 0) return -1;
            has_year = 1;
            l += 4;
        } else if (strncmp(l, "06", 2) == 0) {
            /* 2-digit year */
            int yy;
            if (parse_int_n(&v, 2, &yy) != 0) return -1;
            year = (yy < 70) ? 2000 + yy : 1900 + yy;
            has_year = 1;
            l += 2;
        } else if (strncmp(l, "January", 7) == 0) {
            /* Full month name */
            if (parse_month_name(&v, &month, 1) != 0) return -1;
            has_month = 1;
            l += 7;
        } else if (strncmp(l, "Jan", 3) == 0) {
            /* Abbreviated month name */
            if (parse_month_name(&v, &month, 0) != 0) return -1;
            has_month = 1;
            l += 3;
        } else if (strncmp(l, "_2", 2) == 0) {
            /* Right-justified day */
            if (*v == ' ') v++;
            if (parse_int_1or2(&v, &day) != 0) return -1;
            has_day = 1;
            l += 2;
        } else if (strncmp(l, "02", 2) == 0) {
            /* 2-digit day (NO FLAG CHECK - let context decide) */
            if (parse_int_n(&v, 2, &day) != 0) return -1;
            has_day = 1;
            l += 2;
        } else if (strncmp(l, "01", 2) == 0) {
            /* 2-digit month (NO FLAG CHECK - let context decide) */
            if (parse_int_n(&v, 2, &month) != 0) return -1;
            has_month = 1;
            l += 2;
        } else if (strncmp(l, "Monday", 6) == 0) {
            /* Full weekday */
            if (parse_weekday_name(&v, 1) != 0) return -1;
            l += 6;
        } else if (strncmp(l, "Mon", 3) == 0) {
            /* Abbreviated weekday */
            if (parse_weekday_name(&v, 0) != 0) return -1;
            l += 3;
        } else if (strncmp(l, "15", 2) == 0) {
            /* 24-hour (MUST come before single digit checks!) */
            if (parse_int_n(&v, 2, &hour) != 0) return -1;
            has_time = 1;
            l += 2;
        } else if (strncmp(l, "03", 2) == 0) {
            /* 12-hour (padded) */
            if (parse_int_n(&v, 2, &hour) != 0) return -1;
            has_time = 1;
            is_12h = 1;
            l += 2;
        } else if (*l == '3') {
            /* 12-hour (1 or 2 digit) */
            if (parse_int_1or2(&v, &hour) != 0) return -1;
            has_time = 1;
            is_12h = 1;
            l++;
        } else if (*l == '2' && !has_day) {
            /* 1 or 2-digit day (AFTER all 2-digit checks) */
            if (parse_int_1or2(&v, &day) != 0) return -1;
            has_day = 1;
            l++;
        } else if (*l == '1' && !has_month) {
            /* 1 or 2-digit month (AFTER all 2-digit checks) */
            if (parse_int_1or2(&v, &month) != 0) return -1;
            has_month = 1;
            l++;
        } else if (strncmp(l, "05", 2) == 0) {
            /* 2-digit second */
            if (parse_int_n(&v, 2, &second) != 0) return -1;
            l += 2;
        } else if (strncmp(l, "04", 2) == 0) {
            /* 2-digit minute */
            if (parse_int_n(&v, 2, &minute) != 0) return -1;
            l += 2;
        } else if (*l == '5' && (l == layout || !isdigit((unsigned char)l[-1]))) {
            /* 1 or 2-digit second (only if not part of larger number like "15") */
            if (parse_int_1or2(&v, &second) != 0) return -1;
            l++;
        } else if (*l == '4' && (l == layout || !isdigit((unsigned char)l[-1]))) {
            /* 1 or 2-digit minute (only if not part of larger number like "04") */
            if (parse_int_1or2(&v, &minute) != 0) return -1;
            l++;
        } else if (strncmp(l, ".999999999", 10) == 0) {
            /* Fractional seconds: 9 digits (trailing zeros omitted) */
            l += 10;
            if (*v == '.') {
                v++;
                int digits = 0;
                while (*v >= '0' && *v <= '9' && digits < 9) {
                    nsec = nsec * 10 + (*v - '0');
                    digits++;
                    v++;
                }
                while (digits < 9) {
                    nsec *= 10;
                    digits++;
                }
            }
        } else if (strncmp(l, ".000000000", 10) == 0) {
            /* Fractional seconds: 9 digits (trailing zeros included) */
            l += 10;
            if (*v == '.') {
                v++;
                for (int i = 0; i < 9; i++) {
                    if (*v >= '0' && *v <= '9') {
                        nsec = nsec * 10 + (*v - '0');
                        v++;
                    } else {
                        return -1;
                    }
                }
            } else {
                return -1;
            }
        } else if (strncmp(l, ".000000", 7) == 0) {
            /* Fractional seconds: 6 digits (microseconds) */
            l += 7;
            if (*v == '.') {
                v++;
                for (int i = 0; i < 6; i++) {
                    if (*v >= '0' && *v <= '9') {
                        nsec = nsec * 10 + (*v - '0');
                        v++;
                    } else {
                        return -1;
                    }
                }
                /* Pad to nanoseconds */
                nsec *= 1000;
            } else {
                return -1;
            }
        } else if (strncmp(l, ".000", 4) == 0) {
            /* Fractional seconds: 3 digits (milliseconds) */
            l += 4;
            if (*v == '.') {
                v++;
                for (int i = 0; i < 3; i++) {
                    if (*v >= '0' && *v <= '9') {
                        nsec = nsec * 10 + (*v - '0');
                        v++;
                    } else {
                        return -1;
                    }
                }
                /* Pad to nanoseconds */
                nsec *= 1000000;
            } else {
                return -1;
            }
        } else if (strncmp(l, ".9", 2) == 0) {
            /* Fractional seconds: variable digits (trailing zeros omitted) */
            l += 2;
            /* Consume all consecutive 9s from layout */
            while (*l == '9') l++;
            if (*v == '.') {
                v++;
                int digits = 0;
                while (*v >= '0' && *v <= '9' && digits < 9) {
                    nsec = nsec * 10 + (*v - '0');
                    digits++;
                    v++;
                }
                while (digits < 9) {
                    nsec *= 10;
                    digits++;
                }
            }
        } else if (strncmp(l, ".0", 2) == 0 && (l[2] < '1' || l[2] > '7')) {
            /* Fractional seconds: 1 digit (only if not followed by 1-7 which could form .01-.07) */
            l += 2;
            if (*v == '.') {
                v++;
                if (*v >= '0' && *v <= '9') {
                    nsec = (*v - '0') * 100000000;
                    v++;
                } else {
                    return -1;
                }
            } else {
                return -1;
            }
        } else if (strncmp(l, "PM", 2) == 0) {
            /* AM/PM (uppercase) */
            if (strncmp(v, "PM", 2) == 0) {
                is_pm = 1;
            } else if (strncmp(v, "AM", 2) != 0) {
                return -1;
            }
            v += 2;
            l += 2;
        } else if (strncmp(l, "pm", 2) == 0) {
            /* am/pm (lowercase) */
            if (strncmp(v, "pm", 2) == 0) {
                is_pm = 1;
            } else if (strncmp(v, "am", 2) != 0) {
                return -1;
            }
            v += 2;
            l += 2;
        } else if (strncmp(l, "Z07:00", 6) == 0) {
            /* "Z" or numeric timezone with colon */
            if (*v == 'Z') {
                v++;
            } else if (parse_tz_offset(&v, 1, &tz_offset) != 0) {
                return -1;
            }
            l += 6;
        } else if (strncmp(l, "Z0700", 5) == 0) {
            /* "Z" or numeric timezone */
            if (*v == 'Z') {
                v++;
            } else if (parse_tz_offset(&v, 0, &tz_offset) != 0) {
                return -1;
            }
            l += 5;
        } else if (strncmp(l, "-07:00", 6) == 0) {
            /* Numeric timezone with colon */
            if (parse_tz_offset(&v, 1, &tz_offset) != 0) return -1;
            l += 6;
        } else if (strncmp(l, "-0700", 5) == 0) {
            /* Numeric timezone */
            if (parse_tz_offset(&v, 0, &tz_offset) != 0) return -1;
            l += 5;
        } else if (strncmp(l, "MST", 3) == 0) {
            /* Timezone abbreviation (skip) */
            while (*v && !isspace((unsigned char)*v) && *v != ')') v++;
            l += 3;
        } else {
            /* Literal character */
            if (*l != *v) return -1;
            l++;
            v++;
        }
    }
    
    /* Convert 12-hour to 24-hour */
    if (is_12h) {
        if (hour == 12) hour = 0;
        if (is_pm) hour += 12;
    }
    
    /* Construct result */
    if (!has_year) year = 1970;
    if (!has_month) month = 1;
    if (!has_day) day = 1;
    
    /* Create VexInstant first */
    VexInstant instant;
    
    /* Convert to Unix timestamp using fast epoch calculation (no timegm overhead) */
    int64_t unix_time = fast_epoch_from_date(year, month, day, hour, minute, second);
    
    instant.unix_sec = unix_time - tz_offset;
    instant.nsec = (int32_t)nsec;
    instant._pad = 0;
    
    /* Create VexTime with UTC instant and current monotonic time */
    out->wall = instant;
    out->mono_ns = vt_monotonic_now_ns();
    
    /* Note: Timezone 'tz' parameter is used for offset calculation only.
     * VexTime doesn't store timezone info - it's always UTC. */
    (void)tz;  /* Suppress unused parameter warning */
    
    return 0;
}

/* Parse instant (UTC only) */
int vt_parse_instant_layout(const char* value, const char* layout, VexInstant* out) {
    VexTime t;
    if (vt_parse_layout(value, layout, NULL, &t) != 0) return -1;
    *out = t.wall;
    return 0;
}

