#ifndef VEX_TIME_FAST_PARSE_H
#define VEX_TIME_FAST_PARSE_H

#include "../../include/vex_time.h"
#include <stdint.h>

/* Fast epoch calculation (Howard Hinnant algorithm) - avoids timegm() overhead */
int64_t fast_epoch_from_date(int year, int month, int day, int hour, int min, int sec);

/* Fast epoch-to-date conversion (reverse of fast_epoch_from_date) - avoids gmtime_r() overhead */
void fast_date_from_epoch(int64_t epoch_sec, int* year, int* month, int* day, int* hour, int* min, int* sec, int* weekday);

/* SWAR-optimized RFC3339 parsing */
int vt_parse_rfc3339_fast(const char* s, VexInstant* out);

/* Fast RFC3339 formatting */
int vt_format_rfc3339_utc_fast(VexInstant t, char* buf, size_t buflen);

#endif /* VEX_TIME_FAST_PARSE_H */

