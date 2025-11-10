/*
 * Fast RFC3339 Parser - SWAR Optimized
 * 
 * Uses SIMD Within A Register (SWAR) technique for extreme performance
 * Target: < 800 ns (3x faster than current)
 */

#include "../../include/vex_time.h"
#include <string.h>
#include <stdint.h>
#include <time.h>

/* Fast epoch calculation using Howard Hinnant's algorithm
 * Reference: http://howardhinnant.github.io/date_algorithms.html
 * Avoids timegm() bottleneck for significant speedup
 */
int64_t fast_epoch_from_date(int year, int month, int day, int hour, int min, int sec) {
    /* Convert to March-based year (March 1 = day 0 of year)
     * This simplifies leap year handling */
    year -= (month <= 2);
    
    /* Calculate era: 400-year cycles since epoch */
    int era = (year >= 0 ? year : year - 399) / 400;
    
    /* Year of era [0, 399] */
    int yoe = year - era * 400;
    
    /* Day of year [0, 365] in March-based calendar */
    /* For months 3-12 (Jan/Feb of next year): month - 3
     * For months 1-2 (Jan/Feb): month + 9 (9=Jan, 10=Feb in previous year) */
    int m_offset = (month > 2) ? (month - 3) : (month + 9);
    int doy = (153 * m_offset + 2) / 5 + day - 1;
    
    /* Day of era [0, 146096] */
    int doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    
    /* Days since Unix epoch (1970-01-01) */
    int64_t days = (int64_t)era * 146097 + doe - 719468;
    
    /* Convert to seconds */
    return days * 86400 + hour * 3600 + min * 60 + sec;
}

/* Fast epoch-to-date conversion (reverse of fast_epoch_from_date)
 * Avoids gmtime_r() overhead - critical for layout format performance
 * Based on Howard Hinnant's date algorithms (civil_from_days)
 */
void fast_date_from_epoch(int64_t epoch_sec, int* year, int* month, int* day, int* hour, int* min, int* sec, int* weekday) {
    /* Extract time components */
    int64_t days = epoch_sec / 86400;
    int64_t rem = epoch_sec % 86400;
    if (rem < 0) {
        rem += 86400;
        days--;
    }
    
    *hour = (int)(rem / 3600);
    rem %= 3600;
    *min = (int)(rem / 60);
    *sec = (int)(rem % 60);
    
    /* Convert days to date (Howard Hinnant's civil_from_days) */
    days += 719468;  /* Shift to 0000-03-01 (proleptic Gregorian calendar) */
    
    /* Era calculation */
    int64_t era = (days >= 0 ? days : days - 146096) / 146097;
    unsigned doe = (unsigned)(days - era * 146097);  /* Day of era [0, 146096] */
    
    /* Year of era [0, 399] */
    unsigned yoe = (doe - doe/1460 + doe/36524 - doe/146096) / 365;
    unsigned doy = doe - (365 * yoe + yoe/4 - yoe/100);  /* Day of year [0, 365] */
    unsigned mp = (5 * doy + 2) / 153;  /* Month [0, 11] */
    
    *day = (int)(doy - (153 * mp + 2) / 5 + 1);  /* Day of month [1, 31] */
    *month = (int)(mp < 10 ? mp + 3 : mp - 9);  /* Month [1, 12] */
    *year = (int)(era * 400 + yoe + (mp <= 1 ? 1 : 0));  /* Year */
    
    /* Calculate weekday (Zeller's congruence - optimized) */
    int y = *year;
    int m = *month;
    int d = *day;
    if (m < 3) {
        m += 12;
        y--;
    }
    int k = y % 100;
    int j = y / 100;
    int h = (d + ((13 * (m + 1)) / 5) + k + (k / 4) + (j / 4) - (2 * j)) % 7;
    *weekday = (h + 6) % 7;  /* 0=Sunday */
}

/* SWAR: Parse 4 ASCII digits in one operation - ULTRA OPTIMIZED */
static inline int swar_parse_4digits(const uint8_t* s) {
    /* Load 4 bytes as uint32 and use SWAR magic */
    uint32_t val;
    memcpy(&val, s, 4);
    
    /* Validate all are digits (0x30-0x39) in one check */
    uint32_t mask = val & 0xF0F0F0F0;
    if (mask != 0x30303030) {
        /* Fallback: not all digits, use slow path */
        return (s[0] - '0') * 1000 + (s[1] - '0') * 100 + (s[2] - '0') * 10 + (s[3] - '0');
    }
    
    /* SWAR: Convert 4 ASCII digits to integer in parallel
     * val has bytes: [d3][d2][d1][d0] (little-endian)
     * Subtract '0' from each byte */
    val = val & 0x0F0F0F0F;  /* Remove ASCII prefix */
    
    /* Combine: d0*1000 + d1*100 + d2*10 + d3 */
    #ifdef __LITTLE_ENDIAN__
    return ((val & 0xFF) * 1000) + (((val >> 8) & 0xFF) * 100) + 
           (((val >> 16) & 0xFF) * 10) + ((val >> 24) & 0xFF);
    #else
    return ((val >> 24) * 1000) + (((val >> 16) & 0xFF) * 100) + 
           (((val >> 8) & 0xFF) * 10) + (val & 0xFF);
    #endif
}

/* SWAR: Parse 2 ASCII digits - OPTIMIZED */
static inline int swar_parse_2digits(const uint8_t* s) {
    /* Ultra-fast: direct computation without branches */
    return ((s[0] & 0x0F) * 10) + (s[1] & 0x0F);
}

/* Fast RFC3339 parser with SWAR */
int vt_parse_rfc3339_fast(const char* s, VexInstant* out) {
    if (!s || !out) return -1;
    
    /* Quick validation: check minimum positions without strlen */
    /* Format: "2024-11-07T12:34:56Z" (20 chars minimum) */
    if (!s[0] || !s[4] || !s[7] || !s[10] || !s[13] || !s[16] || !s[19]) {
        return -1;
    }
    
    /* Validate separators (branch predictor friendly) */
    if (s[4] != '-' || s[7] != '-' || s[10] != 'T' || 
        s[13] != ':' || s[16] != ':') {
        return -1;
    }
    
    /* SWAR: Parse date/time components */
    int year = swar_parse_4digits((const uint8_t*)s);
    int month = swar_parse_2digits((const uint8_t*)(s + 5));
    int day = swar_parse_2digits((const uint8_t*)(s + 8));
    int hour = swar_parse_2digits((const uint8_t*)(s + 11));
    int minute = swar_parse_2digits((const uint8_t*)(s + 14));
    int second = swar_parse_2digits((const uint8_t*)(s + 17));
    
    /* Quick range validation */
    if (year < 1970 || year > 9999 ||
        month < 1 || month > 12 ||
        day < 1 || day > 31 ||
        hour > 23 || minute > 59 || second > 60) {
        return -1;
    }
    
    /* Parse fractional seconds (if present) - SWAR optimized */
    int nsec = 0;
    const char* p = s + 19;
    
    if (*p == '.') {
        p++;
        const uint8_t* up = (const uint8_t*)p;
        
        /* Fast path: parse common case (9 digits) with SWAR */
        if (up[0] >= '0' && up[0] <= '9' &&
            up[1] >= '0' && up[1] <= '9' &&
            up[2] >= '0' && up[2] <= '9') {
            
            /* Parse first 3 digits */
            nsec = (up[0] - '0') * 100000000 +
                   (up[1] - '0') * 10000000 +
                   (up[2] - '0') * 1000000;
            
            /* Check for more digits */
            if (up[3] >= '0' && up[3] <= '9') {
                nsec += (up[3] - '0') * 100000;
                if (up[4] >= '0' && up[4] <= '9') {
                    nsec += (up[4] - '0') * 10000;
                    if (up[5] >= '0' && up[5] <= '9') {
                        nsec += (up[5] - '0') * 1000;
                        if (up[6] >= '0' && up[6] <= '9') {
                            nsec += (up[6] - '0') * 100;
                            if (up[7] >= '0' && up[7] <= '9') {
                                nsec += (up[7] - '0') * 10;
                                if (up[8] >= '0' && up[8] <= '9') {
                                    nsec += (up[8] - '0');
                                    p += 9;
                                    /* Skip any extra digits beyond 9 */
                                    while (*p >= '0' && *p <= '9') p++;
                                } else { p += 8; }
                            } else { p += 7; }
                        } else { p += 6; }
                    } else { p += 5; }
                } else { p += 4; }
            } else { p += 3; }
        } else {
            /* Fallback: short fractional part */
            int digits = 0;
            while (*p >= '0' && *p <= '9' && digits < 9) {
                nsec = nsec * 10 + (*p - '0');
                digits++;
                p++;
            }
            /* Pad to nanoseconds */
            while (digits < 9) {
                nsec *= 10;
                digits++;
            }
            /* Skip remaining */
            while (*p >= '0' && *p <= '9') p++;
        }
    }
    
    /* Parse timezone */
    int tz_offset = 0;
    
    if (*p == 'Z') {
        /* UTC - fast path */
        tz_offset = 0;
    } else if (*p == '+' || *p == '-') {
        int sign = (*p == '-') ? -1 : 1;
        p++;
        
        /* Must have at least 2 digits for hour */
        if (p[0] < '0' || p[0] > '9' || p[1] < '0' || p[1] > '9') {
            return -1;
        }
        
        int tz_hour = swar_parse_2digits((const uint8_t*)p);
        p += 2;
        
        int tz_min = 0;
        if (*p == ':') {
            p++;
            if (p[0] >= '0' && p[0] <= '9' && p[1] >= '0' && p[1] <= '9') {
                tz_min = swar_parse_2digits((const uint8_t*)p);
            }
        } else if (p[0] >= '0' && p[0] <= '9' && p[1] >= '0' && p[1] <= '9') {
            /* HHMM format without colon */
            tz_min = swar_parse_2digits((const uint8_t*)p);
        }
        
        tz_offset = sign * (tz_hour * 3600 + tz_min * 60);
    } else {
        return -1;
    }
    
    /* Convert to Unix timestamp using fast algorithm */
    int64_t unix_time = fast_epoch_from_date(year, month, day, hour, minute, second);
    unix_time -= tz_offset;
    
    out->unix_sec = unix_time;
    out->nsec = (int32_t)nsec;
    out->_pad = 0;
    
    return 0;
}

/* Fast RFC3339 formatter - FULLY OPTIMIZED (no gmtime!) */
int vt_format_rfc3339_utc_fast(VexInstant t, char* buf, size_t buflen) {
    if (!buf || buflen < 21) return -1;
    
    /* Use fast_date_from_epoch (NO gmtime overhead!) */
    int year, month, day, hour, min, sec, weekday;
    fast_date_from_epoch(t.unix_sec, &year, &month, &day, &hour, &min, &sec, &weekday);
    
    /* Fast formatting with lookup tables */
    static const char digits[200] = 
        "00010203040506070809"
        "10111213141516171819"
        "20212223242526272829"
        "30313233343536373839"
        "40414243444546474849"
        "50515253545556575859"
        "60616263646566676869"
        "70717273747576777879"
        "80818283848586878889"
        "90919293949596979899";
    
    /* Year (4 digits) */
    buf[0] = '0' + (year / 1000);
    buf[1] = '0' + ((year / 100) % 10);
    buf[2] = '0' + ((year / 10) % 10);
    buf[3] = '0' + (year % 10);
    buf[4] = '-';
    
    /* Month (2 digits) */
    memcpy(buf + 5, &digits[month * 2], 2);
    buf[7] = '-';
    
    /* Day (2 digits) */
    memcpy(buf + 8, &digits[day * 2], 2);
    buf[10] = 'T';
    
    /* Time */
    memcpy(buf + 11, &digits[hour * 2], 2);
    buf[13] = ':';
    memcpy(buf + 14, &digits[min * 2], 2);
    buf[16] = ':';
    memcpy(buf + 17, &digits[sec * 2], 2);
    
    /* Fractional seconds */
    if (t.nsec && buflen >= 30) {
        buf[19] = '.';
        /* Format 9 digits */
        uint32_t ns = t.nsec;
        for (int i = 28; i >= 20; i--) {
            buf[i] = '0' + (ns % 10);
            ns /= 10;
        }
        buf[29] = 'Z';
        buf[30] = '\0';
    } else {
        buf[19] = 'Z';
        buf[20] = '\0';
    }
    
    return 0;
}

