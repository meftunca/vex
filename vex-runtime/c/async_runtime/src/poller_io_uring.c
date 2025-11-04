#include "poller.h"
#ifdef __linux__
#include <liburing.h>
#include <errno.h>
#include <stdlib.h>
#include <string.h>

#ifndef IORING_OP_POLL_ADD
#warning "io_uring headers may be too old; falling back should be configured in Makefile."
#endif

typedef struct Poller {
    struct io_uring ring;
} Poller;

Poller* poller_create() {
    Poller* p = (Poller*)malloc(sizeof(Poller));
    if (!p) return NULL;
    if (io_uring_queue_init(256, &p->ring, 0) != 0) {
        free(p); return NULL;
    }
    return p;
}

void poller_destroy(Poller* p) {
    if (!p) return;
    io_uring_queue_exit(&p->ring);
    free(p);
}

int poller_add(Poller* p, int fd, EventType type, void* user_data) {
    unsigned poll_mask = 0;
    if (type & EVENT_TYPE_READABLE) poll_mask |= POLLIN;
    if (type & EVENT_TYPE_WRITABLE) poll_mask |= POLLOUT;
    struct io_uring_sqe* sqe = io_uring_get_sqe(&p->ring);
    if (!sqe) return -1;
    io_uring_prep_poll_add(sqe, fd, poll_mask);
    io_uring_sqe_set_data(sqe, user_data);
    return io_uring_submit(&p->ring) >= 0 ? 0 : -1;
}

int poller_remove(Poller* p, int fd) {
    // Best-effort: in this minimal design we use one-shot polls, so remove is a no-op.
    (void)p; (void)fd;
    return 0;
}

int poller_wait(Poller* p, ReadyEvent* events, int max_events, int timeout_ms) {
    int count = 0;
    struct __kernel_timespec ts, *tsp = NULL;
    if (timeout_ms >= 0) {
        ts.tv_sec = timeout_ms / 1000;
        ts.tv_nsec = (timeout_ms % 1000) * 1000000;
        tsp = &ts;
    }
    while (count < max_events) {
        struct io_uring_cqe* cqe = NULL;
        int rc = io_uring_wait_cqe_timeout(&p->ring, &cqe, tsp);
        if (rc < 0) break;
        events[count].fd = -1;
        events[count].type = EVENT_TYPE_READABLE; // best-effort
        events[count].user_data = io_uring_cqe_get_data(cqe);
        io_uring_cqe_seen(&p->ring, cqe);
        ++count;
    }
    return count;
}
#endif
