#include "runtime.h"
#include <stdio.h>
#include <stdlib.h>
#include <time.h>

typedef struct {
    int id;
    int remaining_ticks;
} SleepPrint;

static CoroStatus sleep_print(WorkerContext* ctx, void* data) {
    SleepPrint* sp = (SleepPrint*)data;
    printf("[coro %d] tick (%d left)\n", sp->id, sp->remaining_ticks);
    if (--sp->remaining_ticks <= 0) {
        free(sp);
        return CORO_STATUS_DONE;
    }
    // simulate async yield without blocking: just yield control
    return CORO_STATUS_RUNNING;
}

int main() {
    Runtime* rt = runtime_create(4);
    runtime_set_tracing(rt, false);

    for (int i = 0; i < 8; ++i) {
        SleepPrint* sp = (SleepPrint*)malloc(sizeof(SleepPrint));
        sp->id = i;
        sp->remaining_ticks = 5 + (i % 3);
        runtime_spawn_global(rt, sleep_print, sp);
    }

    runtime_run(rt);
    runtime_destroy(rt);
    return 0;
}
