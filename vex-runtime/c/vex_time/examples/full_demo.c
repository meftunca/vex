#include "vex_time.h"
#include <stdio.h>
#include <string.h>

static void on_tick(void* user, VexTime when){
  (void)user;
  char b[64];
  vt_format_rfc3339_utc(when.wall, b, sizeof b);
  printf("[tick] %s\n", b);
}

int main(void){
  /* TZ dir override (needed on Windows if you ship tzdb) */
  /* vt_tz_set_dir("C:/tzdb"); */

  /* Layout format & parse with names and day-of-year */
  VexInstant utc = vt_instant_from_unix(1730937600, 123456789);
  VexTz* tz = vt_tz_load("Europe/Istanbul");
  char out[128];
  vt_format_go(utc, tz, "Mon, 02 Jan 2006 15:04:05.000 Z07:00 MST (yday=002)", out, sizeof out);
  printf("Format: %s\n", out);

  VexInstant p;
  vt_parse_go("Monday, _2 January 2006 03:04:05 PM -07:00", "Thursday,  7 November 2024 03:04:05 PM +03:00", tz, &p);
  char rfc[64]; vt_format_rfc3339_utc(p, rfc, sizeof rfc);
  printf("Parse->UTC: %s\n", rfc);

  /* Scheduler: strict cancel demo */
  VexTimeSched* s = vt_sched_create();
  VexTicker* tk = vt_ticker_create(s, on_tick, NULL);
  vt_ticker_start(tk, 200*1000*1000LL);
  vt_sleep_ns(750*1000*1000LL);
  vt_ticker_stop(tk); /* strict cancel removes it */
  vt_ticker_destroy(tk);
  vt_sched_destroy(s);

  vt_tz_release(tz);
  return 0;
}
