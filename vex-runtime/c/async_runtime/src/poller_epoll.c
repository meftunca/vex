#include "poller.h"
#ifdef __linux__
#include <sys/epoll.h>
#include <unistd.h>
#include <errno.h>
#include <stdlib.h>
#include <string.h>

/* Use vex allocator */
extern void* xmalloc(size_t size);
extern void xfree(void* ptr);

typedef struct Poller {
    int epfd;
} Poller;

Poller* poller_create() {
    Poller* p = (Poller*)xmalloc(sizeof(Poller));
    if (!p) return NULL;
    p->epfd = epoll_create1(EPOLL_CLOEXEC);
    if (p->epfd < 0) { xfree(p); return NULL; }
    return p;
}

void poller_destroy(Poller* p) {
    if (!p) return;
    close(p->epfd);
    xfree(p);
}

int poller_add(Poller* p, int fd, EventType type, void* user_data) {
    struct epoll_event ev;
    memset(&ev, 0, sizeof(ev));
    if (type & EVENT_TYPE_READABLE) ev.events |= EPOLLIN;
    if (type & EVENT_TYPE_WRITABLE) ev.events |= EPOLLOUT;
    ev.data.ptr = user_data;
    return epoll_ctl(p->epfd, EPOLL_CTL_ADD, fd, &ev);
}

int poller_remove(Poller* p, int fd) {
    return epoll_ctl(p->epfd, EPOLL_CTL_DEL, fd, NULL);
}

int poller_wait(Poller* p, ReadyEvent* events, int max_events, int timeout_ms) {
    struct epoll_event evlist[1024];
    if (max_events > 1024) max_events = 1024;
    int n = epoll_wait(p->epfd, evlist, max_events, timeout_ms);
    if (n <= 0) return 0;
    for (int i = 0; i < n; ++i) {
        events[i].fd = -1; // unknown without EPOLL data fd; user can carry fd elsewhere
        events[i].type = (evlist[i].events & EPOLLIN) ? EVENT_TYPE_READABLE : EVENT_TYPE_WRITABLE;
        events[i].user_data = evlist[i].data.ptr;
    }
    return n;
}
#endif
