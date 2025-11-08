// poller_vexnet.c - vex_net adapter for async_runtime
// Replaces platform-specific pollers (kqueue/epoll/io_uring/iocp) with unified vex_net
#include "poller.h"
#include "vex_net.h"
#include <stdlib.h>
#include <string.h>

// Reserved userdata value for timer events (use address as unique marker)
static char timer_marker;
#define TIMER_USERDATA ((uintptr_t)&timer_marker)

typedef struct Poller {
    VexNetLoop loop;
    void* timer_user_data;  // Store timer callback data
} Poller;

Poller* poller_create() {
    Poller* p = (Poller*)malloc(sizeof(Poller));
    if (!p) return NULL;
    
    if (vex_net_loop_create(&p->loop) != 0) {
        free(p);
        return NULL;
    }
    
    p->timer_user_data = NULL;
    
    return p;
}

void poller_destroy(Poller* p) {
    if (!p) return;
    vex_net_loop_close(&p->loop);
    free(p);
}

int poller_add(Poller* p, int fd, EventType type, void* user_data) {
    if (!p) return -1;
    
    uint32_t events = 0;
    if (type & EVENT_TYPE_READABLE) events |= VEX_EVT_READ;
    if (type & EVENT_TYPE_WRITABLE) events |= VEX_EVT_WRITE;
    
    return vex_net_register(&p->loop, fd, events, (uintptr_t)user_data);
}

int poller_remove(Poller* p, int fd) {
    if (!p) return -1;
    return vex_net_unregister(&p->loop, fd);
}

int poller_wait(Poller* p, ReadyEvent* events, int max_events, int timeout_ms) {
    if (!p || !events) return 0;
    
    VexEvent vex_events[1024];
    if (max_events > 1024) max_events = 1024;
    
    int n = vex_net_tick(&p->loop, vex_events, max_events, timeout_ms);
    if (n <= 0) return 0;
    
    // Convert VexEvent -> ReadyEvent
    int out_count = 0;
    for (int i = 0; i < n; i++) {
        // Check if this is a timer event
        if (vex_events[i].userdata == TIMER_USERDATA) {
            events[out_count].fd = -1;  // Timer has no fd
            events[out_count].type = EVENT_TYPE_TIMER;
            events[out_count].user_data = p->timer_user_data;
            out_count++;
            continue;
        }
        
        // Regular fd event
        events[out_count].fd = vex_events[i].fd;
        events[out_count].type = EVENT_TYPE_NONE;
        
        if (vex_events[i].events & VEX_EVT_READ)  
            events[out_count].type |= EVENT_TYPE_READABLE;
        if (vex_events[i].events & VEX_EVT_WRITE) 
            events[out_count].type |= EVENT_TYPE_WRITABLE;
        
        events[out_count].user_data = (void*)vex_events[i].userdata;
        out_count++;
    }
    
    return out_count;
}

int poller_set_timer(Poller* p, uint64_t ms, void* user_data) {
    if (!p) return -1;
    
    p->timer_user_data = user_data;
    return vex_net_timer_after(&p->loop, ms, TIMER_USERDATA);
}

