#ifndef VEX_TIME_SIMD_RFC3339_H
#define VEX_TIME_SIMD_RFC3339_H

#include "../../include/vex_time.h"

/* SIMD-accelerated RFC3339 parsing */
int vt_parse_rfc3339_simd(const char* s, VexInstant* out);

/* SIMD-accelerated RFC3339 formatting */
int vt_format_rfc3339_utc_simd(VexInstant t, char* buf, size_t buflen);

/* Initialize SIMD function pointers */
void vt_simd_init(void);

#endif /* VEX_TIME_SIMD_RFC3339_H */

