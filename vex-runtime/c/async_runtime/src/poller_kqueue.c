#include "poller.h"
#if defined(__APPLE__) || defined(__FreeBSD__)
#include <sys/event.h>
#include <sys/time.h>
#include <stdlib.h>
#include <unistd.h>
#include <stdio.h>
#include <errno.h>

/* Use vex allocator */
extern void *xmalloc(size_t size);
extern void xfree(void *ptr);

typedef struct Poller
{
    int kq;
} Poller;

Poller *poller_create()
{
    Poller *p = (Poller *)xmalloc(sizeof(Poller));
    if (!p)
        return NULL;
    p->kq = kqueue();
    if (p->kq < 0)
    {
        xfree(p);
        return NULL;
    }
    return p;
}

void poller_destroy(Poller *p)
{
    if (!p)
        return;
    close(p->kq);
    xfree(p);
}

int poller_add(Poller *p, int fd, EventType type, void *user_data)
{
    struct kevent ev[2];
    int n = 0;

    // Handle READ filter
    if (type & EVENT_TYPE_READABLE)
    {
        EV_SET(&ev[n++], fd, EVFILT_READ, EV_ADD | EV_ENABLE | EV_CLEAR, 0, 0, user_data);
    }
    else
    {
        EV_SET(&ev[n++], fd, EVFILT_READ, EV_DELETE, 0, 0, NULL);
    }

    // Handle WRITE filter
    if (type & EVENT_TYPE_WRITABLE)
    {
        EV_SET(&ev[n++], fd, EVFILT_WRITE, EV_ADD | EV_ENABLE | EV_CLEAR, 0, 0, user_data);
    }
    else
    {
        EV_SET(&ev[n++], fd, EVFILT_WRITE, EV_DELETE, 0, 0, NULL);
    }

    // Execute all changes in one syscall
    // We ignore errors from EV_DELETE (if filter didn't exist)
    int rc = kevent(p->kq, ev, n, NULL, 0, NULL);
    
    // If kevent returns -1, it's a fatal error (e.g. bad file descriptor)
    // But if it returns >= 0, it might still have EV_ERROR in flags for some events
    // For our purpose, we just return -1 if the syscall failed entirely
    return rc == -1 ? -1 : 0;
}

int poller_remove(Poller *p, int fd)
{
    struct kevent ev[2];
    EV_SET(&ev[0], fd, EVFILT_READ, EV_DELETE, 0, 0, NULL);
    EV_SET(&ev[1], fd, EVFILT_WRITE, EV_DELETE, 0, 0, NULL);
    (void)kevent(p->kq, ev, 2, NULL, 0, NULL);
    return 0;
}

int poller_wait(Poller *p, ReadyEvent *events, int max_events, int timeout_ms)
{
    struct kevent kev[1024];
    if (max_events > 1024)
        max_events = 1024;
    struct timespec ts, *tsp = NULL;
    if (timeout_ms >= 0)
    {
        ts.tv_sec = timeout_ms / 1000;
        ts.tv_nsec = (timeout_ms % 1000) * 1000000;
        tsp = &ts;
    }
    int n = kevent(p->kq, NULL, 0, kev, max_events, tsp);
    if (n <= 0)
        return 0;
    for (int i = 0; i < n; ++i)
    {
        events[i].fd = (int)kev[i].ident;
        events[i].type = (kev[i].filter == EVFILT_READ) ? EVENT_TYPE_READABLE : EVENT_TYPE_WRITABLE;
        events[i].user_data = kev[i].udata;
    }
    return n;
}

int poller_set_timer(Poller *p, uint64_t ms, void *user_data)
{
    struct kevent ev;
    // Use EVFILT_TIMER with NOTE_USECONDS for microsecond precision
    EV_SET(&ev, 0, EVFILT_TIMER, EV_ADD | EV_ONESHOT, NOTE_USECONDS, ms * 1000, user_data);
    return kevent(p->kq, &ev, 1, NULL, 0, NULL) == -1 ? -1 : 0;
}
#endif
