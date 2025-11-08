#if defined(__linux__) && defined(VEX_NET_HAVE_URING)
#ifndef _POSIX_C_SOURCE
#define _POSIX_C_SOURCE 200112L
#endif
#ifndef _GNU_SOURCE
#define _GNU_SOURCE
#endif

#include "vex_net.h"
#include <liburing.h>
#include <unistd.h>
#include <string.h>
#include <errno.h>
#include <time.h>
#include <sys/timerfd.h>
#include <stdlib.h>
#include <poll.h>

#ifndef VEX_ARRAY_MAX
#define VEX_ARRAY_MAX 4096
#endif

/* Global io_uring instance (simplified) */
static struct io_uring global_ring;
static int ring_initialized = 0;

int vex_net_capabilities(void){
  struct io_uring ring;
  int rc = io_uring_queue_init(2, &ring, 0);
  if (rc != 0) return VEX_CAP_TIMER;
  
  io_uring_queue_exit(&ring);
  
  int caps = VEX_CAP_IOURING | VEX_CAP_TIMER;
#ifdef UDP_SEGMENT
  caps |= VEX_CAP_UDP_GSO;
#endif
#ifdef SO_ZEROCOPY
  caps |= VEX_CAP_MSG_ZC;
#endif
  return caps;
}

int vex_net_loop_create(VexNetLoop* loop){
  if (!loop) return -1;
  memset(loop, 0, sizeof(*loop));
  
  if (!ring_initialized) {
    int rc = io_uring_queue_init(256, &global_ring, 0);
    if (rc < 0) return -1;
    ring_initialized = 1;
  }
  
  loop->timer_fd = timerfd_create(CLOCK_MONOTONIC, TFD_NONBLOCK | TFD_CLOEXEC);
  if (loop->timer_fd >= 0) {
    struct io_uring_sqe *sqe = io_uring_get_sqe(&global_ring);
    if (sqe) {
      io_uring_prep_poll_add(sqe, loop->timer_fd, POLLIN);
      io_uring_sqe_set_data(sqe, (void*)(uintptr_t)0);
      io_uring_submit(&global_ring);
    }
  }
  
  loop->fd = global_ring.ring_fd;
  loop->backend = 4; /* 4 = io_uring */
  
  return 0;
}

int vex_net_loop_close(VexNetLoop* loop){
  if (!loop) return -1;
  if (loop->timer_fd >= 0) close(loop->timer_fd);
  loop->timer_fd = -1;
  loop->fd = -1;
  return 0;
}

int vex_net_timer_after(VexNetLoop* loop, uint64_t ms, uintptr_t userdata){
  (void)userdata; /* Timer events use userdata=0 */
  if (!loop || loop->timer_fd < 0) return -1;
  
  struct itimerspec its;
  memset(&its, 0, sizeof(its));
  its.it_value.tv_sec = ms / 1000;
  its.it_value.tv_nsec = (ms % 1000) * 1000000L;
  
  return timerfd_settime(loop->timer_fd, 0, &its, NULL);
}

int vex_net_register(VexNetLoop* loop, int fd, uint32_t events, uintptr_t userdata){
  if (!loop || fd < 0) return -1;
  
  struct io_uring_sqe *sqe = io_uring_get_sqe(&global_ring);
  if (!sqe) return -1;
  
  uint32_t poll_mask = 0;
  if (events & VEX_EVT_READ)  poll_mask |= POLLIN;
  if (events & VEX_EVT_WRITE) poll_mask |= POLLOUT;
  if (events & VEX_EVT_HUP)   poll_mask |= POLLHUP;
  if (events & VEX_EVT_ERR)   poll_mask |= POLLERR;
  
  io_uring_prep_poll_add(sqe, fd, poll_mask);
  io_uring_sqe_set_data(sqe, (void*)userdata);
  
  return io_uring_submit(&global_ring);
}

int vex_net_modify(VexNetLoop* loop, int fd, uint32_t events, uintptr_t userdata){
  if (!loop || fd < 0) return -1;
  
  /* io_uring: cancel + re-add */
  struct io_uring_sqe *sqe = io_uring_get_sqe(&global_ring);
  if (!sqe) return -1;
  
  io_uring_prep_poll_remove(sqe, (void*)userdata);
  io_uring_submit(&global_ring);
  
  return vex_net_register(loop, fd, events, userdata);
}

int vex_net_unregister(VexNetLoop* loop, int fd){
  if (!loop || fd < 0) return -1;
  
  /* Cancel all polls for this fd - use fd as key */
  struct io_uring_sqe *sqe = io_uring_get_sqe(&global_ring);
  if (!sqe) return -1;
  
  io_uring_prep_poll_remove(sqe, (void*)(uintptr_t)fd);
  
  return io_uring_submit(&global_ring);
}

int vex_net_tick(VexNetLoop* loop, VexEvent* events, int maxev, int timeout_ms){
  if (!loop || !events || maxev <= 0) return -1;
  
  struct __kernel_timespec ts, *tsp = NULL;
  if (timeout_ms >= 0) {
    ts.tv_sec = timeout_ms / 1000;
    ts.tv_nsec = (timeout_ms % 1000) * 1000000L;
    tsp = &ts;
  }
  
  struct io_uring_cqe *cqe;
  int ret = io_uring_wait_cqe_timeout(&global_ring, &cqe, tsp);
  if (ret < 0) {
    if (ret == -ETIME || ret == -EINTR) return 0;
    return -1;
  }
  
  int count = 0;
  unsigned head;
  io_uring_for_each_cqe(&global_ring, head, cqe) {
    if (count >= maxev) break;
    
    void *userdata = io_uring_cqe_get_data(cqe);
    int res = cqe->res;
    
    /* Check if timer event (userdata = 0) */
    if ((uintptr_t)userdata == 0) {
      uint64_t x;
      (void)read(loop->timer_fd, &x, sizeof(x));
      
      /* Re-arm timer poll */
      struct io_uring_sqe *sqe = io_uring_get_sqe(&global_ring);
      if (sqe) {
        io_uring_prep_poll_add(sqe, loop->timer_fd, POLLIN);
        io_uring_sqe_set_data(sqe, userdata);
      }
      
      io_uring_cqe_seen(&global_ring, cqe);
      continue;
    }
    
    /* Regular fd event */
    if (res < 0) {
      /* Error in poll */
      io_uring_cqe_seen(&global_ring, cqe);
      continue;
    }
    
    uint32_t vex_events = 0;
    if (res & POLLIN)  vex_events |= VEX_EVT_READ;
    if (res & POLLOUT) vex_events |= VEX_EVT_WRITE;
    if (res & POLLHUP) vex_events |= VEX_EVT_HUP;
    if (res & POLLERR) vex_events |= VEX_EVT_ERR;
    
    events[count].fd = -1; /* fd not directly available from cqe */
    events[count].events = vex_events;
    events[count].userdata = (uintptr_t)userdata;
    count++;
    
    io_uring_cqe_seen(&global_ring, cqe);
  }
  
  io_uring_submit(&global_ring);
  
  return count;
}

#endif /* __linux__ && VEX_NET_HAVE_URING */
