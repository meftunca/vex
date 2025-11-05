#pragma once
#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

struct Poller;
typedef struct Poller Poller;

typedef enum {
    EVENT_TYPE_NONE     = 0,
    EVENT_TYPE_READABLE = 1,
    EVENT_TYPE_WRITABLE = 2
} EventType;

typedef struct {
    int fd;
    EventType type;
    void* user_data;
} ReadyEvent;

Poller* poller_create();
void poller_destroy(Poller* poller);
int poller_add(Poller* poller, int fd, EventType type, void* user_data);
int poller_remove(Poller* poller, int fd);
int poller_wait(Poller* poller, ReadyEvent* events, int max_events, int timeout_ms);

#ifdef __cplusplus
}
#endif
