#include "vex_time.h"
#include <stdio.h>

static void on_timer(void* user, VexTime when){
  (void)user;
  char out[64];
  vt_format_rfc3339_utc(when.wall, out, sizeof out);
  printf("[timer] fired at %s (mono=%llu ns)\n", out, (unsigned long long)when.mono_ns);
}

static void on_tick(void* user, VexTime when){
  int* cnt = (int*)user;
  char out[64]; vt_format_rfc3339_utc(when.wall, out, sizeof out);
  printf("[tick]  #%d at %s\n", *cnt, out);
  (*cnt)++;
}

int main(void){
  VexTime now; vt_now(&now);
  char iso[64]; vt_format_rfc3339_utc(now.wall, iso, sizeof iso);
  printf("Now: %s (mono=%llu ns)\n", iso, (unsigned long long)now.mono_ns);

  VexDuration d; vt_parse_duration("1.5s", &d);
  printf("Parsed 1.5s -> %lld ns\n", (long long)d);

  VexTimeSched* s = vt_sched_create();

  VexTimer* t = vt_timer_create(s, on_timer, NULL);
  vt_timer_start(t, 700*1000*1000LL); /* 700ms */

  int tick_count = 1;
  VexTicker* tk = vt_ticker_create(s, on_tick, &tick_count);
  vt_ticker_start(tk, 250*1000*1000LL); /* 250ms */

  /* Sleep ~1.6s to observe a few ticks */
  vt_sleep_ns(1600LL*1000*1000LL);

  vt_ticker_stop(tk);
  vt_timer_stop(t);

  vt_ticker_destroy(tk);
  vt_timer_destroy(t);
  vt_sched_destroy(s);
  return 0;
}
