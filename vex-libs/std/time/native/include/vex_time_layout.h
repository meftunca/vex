/*
 * Go-style Time Layout Support for vex_time
 * 
 * Supports Go's time.Parse() and time.Format() layout strings
 * Reference: https://pkg.go.dev/time#pkg-constants
 */

#ifndef VEX_TIME_LAYOUT_H
#define VEX_TIME_LAYOUT_H

#include "vex_time.h"

#ifdef __cplusplus
extern "C" {
#endif

/* Standard Go layout constants */
#define VEX_LAYOUT_ANSIC       "Mon Jan _2 15:04:05 2006"
#define VEX_LAYOUT_UNIXDATE    "Mon Jan _2 15:04:05 MST 2006"
#define VEX_LAYOUT_RUBYDATE    "Mon Jan 02 15:04:05 -0700 2006"
#define VEX_LAYOUT_RFC822      "02 Jan 06 15:04 MST"
#define VEX_LAYOUT_RFC822Z     "02 Jan 06 15:04 -0700"
#define VEX_LAYOUT_RFC850      "Monday, 02-Jan-06 15:04:05 MST"
#define VEX_LAYOUT_RFC1123     "Mon, 02 Jan 2006 15:04:05 MST"
#define VEX_LAYOUT_RFC1123Z    "Mon, 02 Jan 2006 15:04:05 -0700"
#define VEX_LAYOUT_RFC3339     "2006-01-02T15:04:05Z07:00"
#define VEX_LAYOUT_RFC3339NANO "2006-01-02T15:04:05.999999999Z07:00"
#define VEX_LAYOUT_KITCHEN     "3:04PM"
#define VEX_LAYOUT_STAMP       "Jan _2 15:04:05"
#define VEX_LAYOUT_STAMPMILLI  "Jan _2 15:04:05.000"
#define VEX_LAYOUT_STAMPMICRO  "Jan _2 15:04:05.000000"
#define VEX_LAYOUT_STAMPNANO   "Jan _2 15:04:05.000000000"
#define VEX_LAYOUT_DATETIME    "2006-01-02 15:04:05"
#define VEX_LAYOUT_DATEONLY    "2006-01-02"
#define VEX_LAYOUT_TIMEONLY    "15:04:05"

/*
 * Go Layout Components:
 * 
 * Year:
 *   2006      - 4-digit year
 *   06        - 2-digit year
 * 
 * Month:
 *   01        - 2-digit month (01-12)
 *   1         - 1 or 2-digit month (1-12)
 *   Jan       - abbreviated month name
 *   January   - full month name
 * 
 * Day:
 *   02        - 2-digit day (01-31)
 *   2         - 1 or 2-digit day (1-31)
 *   _2        - right-justified 2-char day ( 1-31)
 * 
 * Weekday:
 *   Mon       - abbreviated weekday
 *   Monday    - full weekday
 * 
 * Hour:
 *   15        - 24-hour (00-23)
 *   03        - 12-hour (01-12)
 *   3         - 1 or 2-digit 12-hour (1-12)
 * 
 * Minute:
 *   04        - 2-digit minute (00-59)
 *   4         - 1 or 2-digit minute (0-59)
 * 
 * Second:
 *   05        - 2-digit second (00-59)
 *   5         - 1 or 2-digit second (0-59)
 * 
 * Fractional seconds:
 *   .0        - .0, .00, ... .000000000 (trailing zeros included)
 *   .9        - .9, .99, ... .999999999 (trailing zeros omitted)
 * 
 * AM/PM:
 *   PM        - "AM" or "PM"
 *   pm        - "am" or "pm"
 * 
 * Timezone:
 *   MST       - timezone abbreviation
 *   -0700     - numeric timezone (±hhmm)
 *   -07:00    - numeric timezone with colon (±hh:mm)
 *   Z0700     - "Z" or numeric timezone (±hhmm)
 *   Z07:00    - "Z" or numeric timezone with colon (±hh:mm)
 */

/* Parse time from string using Go-style layout
 * Returns 0 on success, -1 on error
 * 
 * Example:
 *   VexTime t;
 *   vt_parse_layout("2024-11-07 12:34:56", "2006-01-02 15:04:05", NULL, &t);
 */
int vt_parse_layout(const char* value, const char* layout, VexTz* tz, VexTime* out);

/* Format time to string using Go-style layout
 * Returns number of bytes written (excluding null terminator), or -1 on error
 * 
 * Example:
 *   char buf[64];
 *   vt_format_layout(t, "2006-01-02 15:04:05", buf, sizeof(buf));
 */
int vt_format_layout(VexTime t, const char* layout, char* buf, size_t buflen);

/* Parse VexInstant from string using Go-style layout (UTC only)
 * Returns 0 on success, -1 on error
 */
int vt_parse_instant_layout(const char* value, const char* layout, VexInstant* out);

/* Format VexInstant to string using Go-style layout (UTC only)
 * Returns number of bytes written (excluding null terminator), or -1 on error
 */
int vt_format_instant_layout(VexInstant t, const char* layout, char* buf, size_t buflen);

#ifdef __cplusplus
}
#endif

#endif /* VEX_TIME_LAYOUT_H */

