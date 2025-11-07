#include "timevex.h"
#include <stdio.h>

static void on_tick(void *u){ (void)u; puts("tick"); }

int main(void){
    tvx_time now = tvx_NowIn(tvx_Local());
    char buf[64];
    tvx_FormatRFC3339(now, 1, buf, sizeof(buf));
    printf("Now: %s\n", buf);

    tvx_duration d;
    tvx_ParseDuration("1h15m30.25s", &d);
    tvx_time later = tvx_Add(now, d);
    tvx_FormatRFC3339(later, 1, buf, sizeof(buf));
    printf("Later: %s\n", buf);

    tvx_location ist; tvx_LoadLocation("Europe/Istanbul", &ist);
    tvx_time t = tvx_In(now, ist);
    tvx_FormatRFC3339(t, 1, buf, sizeof(buf));
    printf("Istanbul: %s\n", buf);

    tvx_ticker *tk = tvx_NewTicker(500*TVX_MILLISECOND, on_tick, NULL);
    tvx_Sleep(1600*TVX_MILLISECOND);
    tvx_TickerStop(tk);
    tvx_TickerFree(tk);
    return 0;
}
