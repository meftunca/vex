#ifndef VEX_TIME_FAST_PARSE_H
#define VEX_TIME_FAST_PARSE_H

#include "../../include/vex_time.h"

/* SWAR-optimized RFC3339 parsing */
int vt_parse_rfc3339_fast(const char* s, VexInstant* out);

/* Fast RFC3339 formatting */
int vt_format_rfc3339_utc_fast(VexInstant t, char* buf, size_t buflen);

#endif /* VEX_TIME_FAST_PARSE_H */

