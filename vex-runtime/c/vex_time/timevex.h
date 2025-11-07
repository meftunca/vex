// timevex.h - A C date/time runtime for VEX (Go-like API)
// Public Domain / CC0 - use at your own risk.
// Platform: POSIX-y (Linux/macOS). Windows needs small tweaks (see README).

#ifndef TIMEVEX_H
#define TIMEVEX_H

#include <stdint.h>
#include <stddef.h>
#include <time.h>

#ifdef __cplusplus
extern "C" {
#endif

// ---------------------------- Types ---------------------------------

// Duration is nanoseconds, like Go.
typedef int64_t tvx_duration;

// Weekday matches Go's constants (Sunday==0).
typedef enum {
    TVX_SUNDAY = 0, TVX_MONDAY, TVX_TUESDAY, TVX_WEDNESDAY, TVX_THURSDAY, TVX_FRIDAY, TVX_SATURDAY
} tvx_weekday;

// A "location" is an IANA TZ name, e.g. "Europe/Istanbul".
// Internally we keep a copy of the string. Conversions are done by
// temporarily setting TZ (process-global) behind a mutex, performing
// localtime_r/gmtime_r or timegm(), and restoring the prior TZ.
typedef struct {
    char name[64]; // IANA name or "Local" or "UTC"
} tvx_location;

// Time is split into wall clock (Unix seconds + nanos) and an optional
// monotonic stamp (for stable durations). If mono_ns<0, monotonic is absent.
typedef struct {
    int64_t unix_sec;  // seconds since 1970-01-01T00:00:00Z
    int32_t nsec;      // 0..999,999,999
    int64_t mono_ns;   // monotonic reference in ns (CLOCK_MONOTONIC), or -1
    tvx_location loc;  // display/formatting location (UTC by default)
} tvx_time;

// A one-shot timer and periodic ticker (like Go's time.Timer/Ticker).
typedef struct tvx_timer tvx_timer;
typedef struct tvx_ticker tvx_ticker;

// Callback signature for timers/tickers.
typedef void (*tvx_callback)(void *user);

// ------------------------ Duration constants ------------------------
#define TVX_NANOSECOND   ((tvx_duration)1)
#define TVX_MICROSECOND  ((tvx_duration)1000)
#define TVX_MILLISECOND  ((tvx_duration)1000000)
#define TVX_SECOND       ((tvx_duration)1000000000LL)
#define TVX_MINUTE       (60 * TVX_SECOND)
#define TVX_HOUR         (60 * TVX_MINUTE)

// --------------------------- Locations ------------------------------
tvx_location tvx_UTC(void);
tvx_location tvx_Local(void); // Uses system localtime
tvx_location tvx_FixedZone(const char *name, int32_t offset_seconds); // like Go: "UTC+03:00"
// Create location from IANA name, e.g. "Europe/Istanbul". Returns 0 on success.
int tvx_LoadLocation(const char *iana, tvx_location *out);

// ----------------------------- Time ---------------------------------
tvx_time tvx_Now(void);             // wall + mono
tvx_time tvx_NowIn(tvx_location loc);
tvx_time tvx_Unix(int64_t sec, int64_t nsec, tvx_location loc);
int64_t  tvx_UnixSeconds(tvx_time t);
int64_t  tvx_UnixNano(tvx_time t);

tvx_time tvx_UTCTo(tvx_time t, tvx_location loc);
tvx_time tvx_In(tvx_time t, tvx_location loc); // convert representational location

// Comparisons
int tvx_Before(tvx_time a, tvx_time b);
int tvx_After(tvx_time a, tvx_time b);
int tvx_Equal(tvx_time a, tvx_time b);

// Arithmetic
tvx_time tvx_Add(tvx_time t, tvx_duration d);
tvx_time tvx_AddDate(tvx_time t, int years, int months, int days);
tvx_duration tvx_Sub(tvx_time a, tvx_time b); // a-b (ns), uses wall clock
tvx_duration tvx_Since(tvx_time t); // Now()-t
tvx_duration tvx_Until(tvx_time t); // t-Now()

// Floor/ceil-like operations
tvx_time tvx_Truncate(tvx_time t, tvx_duration d);
tvx_time tvx_Round(tvx_time t, tvx_duration d);

// Calendar fields (in given location)
int tvx_Year(tvx_time t);
int tvx_Month(tvx_time t); // 1..12
int tvx_Day(tvx_time t);   // 1..31
int tvx_Hour(tvx_time t);  // 0..23
int tvx_Minute(tvx_time t);// 0..59
int tvx_Second(tvx_time t);// 0..60 (leap second treated as 60)
int tvx_Nanosecond(tvx_time t); // 0..999,999,999
tvx_weekday tvx_Weekday(tvx_time t);
int tvx_YearDay(tvx_time t); // 1..366
void tvx_ISOWeek(tvx_time t, int *iso_year, int *iso_week);

// Parsing & formatting
// Go-style duration parser: "72h3m0.5s", "1h", "250ms", "-2m3s", "1.5h"
int tvx_ParseDuration(const char *s, tvx_duration *out);
int tvx_FormatDuration(tvx_duration d, char *buf, size_t buflen);

// RFC3339 / RFC3339Nano
int tvx_ParseRFC3339(const char *s, tvx_time *out);
int tvx_FormatRFC3339(tvx_time t, int nano, char *buf, size_t buflen);

// Sleep
void tvx_Sleep(tvx_duration d);

// Timers & Tickers
// tvx_NewTimer fires once after 'd' and invokes cb(user) on a worker thread.
tvx_timer* tvx_NewTimer(tvx_duration d, tvx_callback cb, void *user);
// Reset changes the timer to fire after d from now. Returns 1 if the timer
// was active, 0 if it had already fired or been stopped.
int tvx_TimerReset(tvx_timer *tmr, tvx_duration d);
// Stop prevents the timer from firing; returns 1 if stopped, 0 if already idle.
int tvx_TimerStop(tvx_timer *tmr);
// Free resources (safe when stopped or after fired)
void tvx_TimerFree(tvx_timer *tmr);

// Ticker calls cb(user) every period 'd' (>=1ms).
tvx_ticker* tvx_NewTicker(tvx_duration period_ns, tvx_callback cb, void *user);
// Reset ticker period; next tick scheduled from now.
int tvx_TickerReset(tvx_ticker *tk, tvx_duration period_ns);
// Stop the ticker.
int tvx_TickerStop(tvx_ticker *tk);
void tvx_TickerFree(tvx_ticker *tk);

// Utilities
// Monotonic now (ns since unspecified start). Returns >=0.
int64_t tvx_MonotonicNow(void);

// Error strings for nonzero error codes returned by parsers.
const char* tvx_StrError(int code);

// ----------------------------- Misc ---------------------------------
// Return last OS error as string (thread-local static buffer).
const char* tvx_LastOSError(void);

#ifdef __cplusplus
}
#endif

#endif // TIMEVEX_H
