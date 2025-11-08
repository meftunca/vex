#ifndef VEX_TIME_H
#define VEX_TIME_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ---- Basic types ---- */
typedef int64_t VexDuration;     /* nanoseconds */
typedef struct {
  int64_t unix_sec;              /* seconds since Unix epoch (UTC) */
  int32_t nsec;                  /* 0..999,999,999 */
  int32_t _pad;
} VexInstant;                     /* wall clock instant (UTC) */

typedef struct {
  VexInstant wall;               /* wall (UTC) */
  uint64_t   mono_ns;            /* monotonic reading in nanoseconds */
} VexTime;                        /* Go-like: carries wall + monotonic */

/* ---- Now & conversion ---- */
void vt_now(VexTime* out);                             /* fill both wall & monotonic */
uint64_t vt_monotonic_now_ns(void);                    /* only monotonic */
VexInstant vt_instant_from_unix(int64_t sec, int32_t nsec);
void vt_instant_to_unix(VexInstant t, int64_t* sec, int32_t* nsec);

/* ---- Duration helpers ---- */
int    vt_parse_duration(const char* s, VexDuration* out_ns);  /* accepts "1h2m3.5s", "250ms", "-1.25h" */
int    vt_format_duration(VexDuration ns, char* buf, size_t buflen); /* RFC3339-ish unit string */

/* ---- Time arithmetic ---- */
VexTime vt_add(VexTime t, VexDuration d);              /* t + d (both wall & mono) */
VexDuration vt_sub(VexTime t, VexTime u);              /* t - u using monotonic if both have it */
VexDuration vt_since(VexTime t);                       /* now - t */
VexDuration vt_until(VexTime t);                       /* t - now */

/* ---- Sleep (blocking) ---- */
int vt_sleep_ns(VexDuration ns);                       /* 0 on success */

/* ---- RFC3339 (UTC default) ---- */
int vt_format_rfc3339_utc(VexInstant t, char* buf, size_t buflen); /* YYYY-MM-DDTHH:MM:SS[.nnnnnnnnn]Z */
int vt_parse_rfc3339(const char* s, VexInstant* out);  /* supports Z or ±HH:MM offset */

/* ---- Scheduler: timers & tickers (single background thread) ---- */
typedef struct VexTimeSched VexTimeSched;
typedef struct VexTimer     VexTimer;
typedef struct VexTicker    VexTicker;

/* Callback type */
typedef void (*VexTimeCb)(void* user, VexTime when);

/* Create/destroy a scheduler (starts its worker thread) */
VexTimeSched* vt_sched_create(void);
void          vt_sched_destroy(VexTimeSched* s);

/* One-shot timer */
VexTimer* vt_timer_create(VexTimeSched* s, VexTimeCb cb, void* user);
int       vt_timer_start(VexTimer* t, VexDuration after_ns);     /* fire once after duration */
int       vt_timer_reset(VexTimer* t, VexDuration after_ns);     /* reschedule (safe from cb) */
int       vt_timer_stop(VexTimer* t);                            /* cancel if pending */
void      vt_timer_destroy(VexTimer* t);

/* Repeating ticker */
VexTicker* vt_ticker_create(VexTimeSched* s, VexTimeCb cb, void* user);
int        vt_ticker_start(VexTicker* tk, VexDuration period_ns); /* fire periodically */
int        vt_ticker_reset(VexTicker* tk, VexDuration period_ns);
int        vt_ticker_stop(VexTicker* tk);
void       vt_ticker_destroy(VexTicker* tk);

#ifdef __cplusplus
}
#endif
#endif /* VEX_TIME_H */
