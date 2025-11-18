#include "poller.h"
#ifdef _WIN32
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <winsock2.h>
#include <mswsock.h>
#include <stdlib.h>

/* Use vex allocator */
extern void* xmalloc(size_t size);
extern void xfree(void* ptr);

typedef struct Poller {
    HANDLE iocp;
} Poller;

Poller* poller_create() {
    Poller* p = (Poller*)xmalloc(sizeof(Poller));
    if (!p) return NULL;
    p->iocp = CreateIoCompletionPort(INVALID_HANDLE_VALUE, NULL, 0, 0);
    if (!p->iocp) { xfree(p); return NULL; }
    return p;
}

void poller_destroy(Poller* p) {
    if (!p) return;
    CloseHandle(p->iocp);
    xfree(p);
}

int poller_add(Poller* p, int fd, EventType type, void* user_data) {
    // Minimal association: fd should be SOCKET castable
    HANDLE h = (HANDLE)_get_osfhandle(fd);
    if (!CreateIoCompletionPort(h, p->iocp, (ULONG_PTR)user_data, 0)) return -1;
    (void)type;
    return 0;
}

int poller_remove(Poller* p, int fd) {
    (void)p; (void)fd;
    return 0;
}

int poller_wait(Poller* p, ReadyEvent* events, int max_events, int timeout_ms) {
    DWORD bytes;
    ULONG_PTR key;
    LPOVERLAPPED ov;
    DWORD timeout = (timeout_ms < 0) ? INFINITE : (DWORD)timeout_ms;
    int n = 0;
    while (n < max_events) {
        BOOL ok = GetQueuedCompletionStatus(p->iocp, &bytes, &key, &ov, timeout);
        if (!ok && !ov) break;
        events[n].fd = -1;
        events[n].type = EVENT_TYPE_READABLE;
        events[n].user_data = (void*)key;
        ++n;
        timeout = 0; // non-blocking next
    }
    return n;
}
#endif
