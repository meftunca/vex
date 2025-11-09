#ifndef VEX_TIME_H
#define VEX_TIME_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ==== Basic types ==== */
typedef int64_t VexDuration;     /* nanoseconds */
typedef struct {
  int64_t unix_sec;              /* seconds since Unix epoch (UTC) */
  int32_t nsec;                  /* 0..999,999,999 */
  int32_t _pad;
} VexInstant;                     /* wall clock instant (UTC) */

typedef struct {
  VexInstant wall;               /* wall (UTC) */
  uint64_t   mono_ns;            /* monotonic reading in nanoseconds */
} VexTime;                        /* carries wall + monotonic */

/* ==== Now & conversion ==== */
void vt_now(VexTime* out);
uint64_t vt_monotonic_now_ns(void);
VexInstant vt_instant_from_unix(int64_t sec, int32_t nsec);
void vt_instant_to_unix(VexInstant t, int64_t* sec, int32_t* nsec);

/* ==== Duration ==== */
int  vt_parse_duration(const char* s, VexDuration* out_ns);  /* 1h2m3.5s, 250ms, -1.25h, ... */
int  vt_format_duration(VexDuration ns, char* buf, size_t buflen);

/* ==== Arithmetic ==== */
VexTime     vt_add(VexTime t, VexDuration d);
VexDuration vt_sub(VexTime t, VexTime u);
VexDuration vt_since(VexTime t);
VexDuration vt_until(VexTime t);

/* ==== Sleep ==== */
int vt_sleep_ns(VexDuration ns);

/* ==== RFC3339 ==== */
int vt_format_rfc3339_utc(VexInstant t, char* buf, size_t buflen);
int vt_parse_rfc3339(const char* s, VexInstant* out);

/* ==== Time Zone (TZ) ==== */
typedef struct VexTz VexTz;

/* TZDB directory override (POSIX & Windows if tzfiles are available) */
void vt_tz_set_dir(const char* path);

/* Loaders */
VexTz* vt_tz_utc(void);                                        /* singleton UTC */
VexTz* vt_tz_fixed(const char* name, int offset_sec);          /* fixed offset zone */
VexTz* vt_tz_local(void);                                      /* system local zone (best effort; resolves IANA if possible) */
VexTz* vt_tz_load(const char* name);                           /* IANA name via TZif dir */
VexTz* vt_tz_load_from_memory(const char* name, const unsigned char* tzif, size_t len);
void   vt_tz_release(VexTz* tz);

/* Query offset/abbr at instant (UTC input) */
int    vt_tz_offset_at(const VexTz* tz, VexInstant utc, int* offset_sec, const char** abbr);

/* Convert UTC instant to local instant in tz (offset application only) */
VexInstant vt_utc_to_tz(const VexTz* tz, VexInstant utc);

/* ==== Component extraction (Go Time.Date/Clock equivalent) ==== */
/* Extract date components from UTC instant */
void vt_instant_date(VexInstant t, int* year, int* month, int* day, int* hour, int* minute, int* second, int* nsec);

/* Extract clock components from UTC instant */
void vt_instant_clock(VexInstant t, int* hour, int* minute, int* second);

/* Get day of year (1-366) for UTC instant */
int vt_instant_yearday(VexInstant t);

/* Get weekday (0=Sunday, 6=Saturday) for UTC instant */
int vt_instant_weekday(VexInstant t);

/* Get ISO week number (1-53) for UTC instant; year in first param */
int vt_instant_isoweek(VexInstant t, int* iso_year);

/* ==== Comparison operators ==== */
/* Returns: -1 if a < b, 0 if a == b, 1 if a > b */
int vt_instant_compare(VexInstant a, VexInstant b);
int vt_instant_equal(VexInstant a, VexInstant b);   /* Returns: 1 if equal, 0 if not */
int vt_instant_before(VexInstant a, VexInstant b);  /* Returns: 1 if a < b, 0 otherwise */
int vt_instant_after(VexInstant a, VexInstant b);   /* Returns: 1 if a > b, 0 otherwise */

/* ==== Time truncation and rounding ==== */
/* Truncate instant to duration boundary (e.g., truncate to minute removes seconds) */
VexInstant vt_instant_truncate(VexInstant t, VexDuration d);

/* Round instant to nearest duration boundary */
VexInstant vt_instant_round(VexInstant t, VexDuration d);

/* ==== Unix timestamp variants ==== */
/* Returns milliseconds since epoch */
int64_t vt_instant_unix_milli(VexInstant t);

/* Returns microseconds since epoch */
int64_t vt_instant_unix_micro(VexInstant t);

/* ==== Go layout Format/Parse ====
   Supported tokens (full, commonly used set):
   Year:      2006 / 06
   Month:     January / Jan / 01 / 1
   Day:       02 / 2 / _2   (space padded)
   Weekday:   Monday / Mon
   Hour:      15 (24h) / 03 or 3 (12h) + PM/pm
   Minute:    04 / 4
   Second:    05 / 5
   DayOfYear: 002
   Fraction:  .000 / .000000 / .000000000
   Zone:      MST / -0700 / -07:00 / Z07:00 (Z for UTC)
*/
int vt_format_go(VexInstant utc, const VexTz* tz, const char* layout, char* out, size_t outlen);
int vt_parse_go(const char* layout, const char* value, const VexTz* tz, VexInstant* out);

/* ==== Scheduler: timers & tickers ==== */
typedef struct VexTimeSched VexTimeSched;
typedef struct VexTimer     VexTimer;
typedef struct VexTicker    VexTicker;
typedef void (*VexTimeCb)(void* user, VexTime when);

/* Create/destroy a scheduler (threaded heap). */
VexTimeSched* vt_sched_create(void);
void          vt_sched_destroy(VexTimeSched* s);

/* Linux-only (optional): io_uring-based timer scheduler (deterministic cancel via TIMEOUT_REMOVE) */
VexTimeSched* vt_sched_create_uring(void); /* returns NULL if unavailable/not supported */

/* One-shot timer */
VexTimer* vt_timer_create(VexTimeSched* s, VexTimeCb cb, void* user);
int       vt_timer_start(VexTimer* t, VexDuration after_ns);     /* fire once after duration */
int       vt_timer_reset(VexTimer* t, VexDuration after_ns);     /* strict reschedule */
int       vt_timer_stop(VexTimer* t);                            /* strict cancel */
void      vt_timer_destroy(VexTimer* t);

/* Repeating ticker */
VexTicker* vt_ticker_create(VexTimeSched* s, VexTimeCb cb, void* user);
int        vt_ticker_start(VexTicker* tk, VexDuration period_ns);
int        vt_ticker_reset(VexTicker* tk, VexDuration period_ns);
int        vt_ticker_stop(VexTicker* tk);
void       vt_ticker_destroy(VexTicker* tk);

#ifdef __cplusplus
}
#endif
#endif /* VEX_TIME_H */
