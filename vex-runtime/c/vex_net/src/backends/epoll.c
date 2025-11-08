#if defined(__linux__)
#ifndef _POSIX_C_SOURCE
#define _POSIX_C_SOURCE 200112L
#endif
#ifndef _GNU_SOURCE
#define _GNU_SOURCE
#endif

#include "vex_net.h"
#include <sys/epoll.h>
#include <sys/timerfd.h>
#include <unistd.h>
#include <string.h>
#include <errno.h>
#include <time.h>

#ifndef VEX_ARRAY_MAX
#define VEX_ARRAY_MAX 4096
#endif

static inline uint32_t ep_mask(uint32_t ev){
  uint32_t m=0;
  if (ev & VEX_EVT_READ)  m |= EPOLLIN;
  if (ev & VEX_EVT_WRITE) m |= EPOLLOUT;
  if (ev & VEX_EVT_HUP)   m |= EPOLLHUP;
  if (ev & VEX_EVT_ERR)   m |= EPOLLERR;
  return m;
}

int vex_net_capabilities(void){
  int caps = VEX_CAP_TIMER;
#ifdef EPOLLEXCLUSIVE
  caps |= VEX_CAP_EPOLLEXCL;
#endif
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
  memset(loop,0,sizeof(*loop));
  loop->fd = epoll_create1(EPOLL_CLOEXEC);
  if (loop->fd < 0) return -1;
  loop->timer_fd = timerfd_create(CLOCK_MONOTONIC, TFD_NONBLOCK|TFD_CLOEXEC);
  if (loop->timer_fd >= 0){
    struct epoll_event ev; memset(&ev,0,sizeof(ev));
    ev.events = EPOLLIN; ev.data.u64 = (uintptr_t)0; /* userdata=0 reserved for timerfd */
    epoll_ctl(loop->fd, EPOLL_CTL_ADD, loop->timer_fd, &ev);
  }
  loop->backend = 1;
  return 0;
}

int vex_net_loop_close(VexNetLoop* loop){
  if (!loop) return -1;
  if (loop->timer_fd >= 0) close(loop->timer_fd);
  if (loop->fd >= 0) close(loop->fd);
  loop->fd = -1;
  return 0;
}

int vex_net_timer_after(VexNetLoop* loop, uint64_t ms, uintptr_t userdata){
  (void)userdata;
  if (!loop || loop->timer_fd<0) return -1;
  struct itimerspec its; memset(&its,0,sizeof(its));
  its.it_value.tv_sec = ms/1000; its.it_value.tv_nsec = (ms%1000)*1000000;
  return timerfd_settime(loop->timer_fd, 0, &its, NULL)==0 ? 1 : -1;
}

int vex_net_register(VexNetLoop* loop, int fd, uint32_t events, uintptr_t userdata){
  struct epoll_event ev; memset(&ev,0,sizeof(ev));
  ev.events = ep_mask(events);
  ev.data.u64 = (uint64_t)userdata;
  return epoll_ctl(loop->fd, EPOLL_CTL_ADD, fd, &ev)==0?0:-1;
}

int vex_net_modify(VexNetLoop* loop, int fd, uint32_t events, uintptr_t userdata){
  struct epoll_event ev; memset(&ev,0,sizeof(ev));
  ev.events = ep_mask(events);
  ev.data.u64 = (uint64_t)userdata;
  return epoll_ctl(loop->fd, EPOLL_CTL_MOD, fd, &ev)==0?0:-1;
}

int vex_net_unregister(VexNetLoop* loop, int fd){
  return epoll_ctl(loop->fd, EPOLL_CTL_DEL, fd, NULL)==0?0:-1;
}

int vex_net_tick(VexNetLoop* loop, VexEvent* out, int capacity, int timeout_ms){
  if (capacity > VEX_ARRAY_MAX) capacity = VEX_ARRAY_MAX;
  struct epoll_event evs[VEX_ARRAY_MAX];
  int n = epoll_wait(loop->fd, evs, capacity, timeout_ms);
  if (n <= 0) return n;
  for (int i=0;i<n;i++){
    uint32_t e=0;
    if (evs[i].events & (EPOLLIN|EPOLLRDHUP)) e |= VEX_EVT_READ;
    if (evs[i].events & EPOLLOUT) e |= VEX_EVT_WRITE;
    if (evs[i].events & EPOLLHUP) e |= VEX_EVT_HUP;
    if (evs[i].events & EPOLLERR) e |= VEX_EVT_ERR;
    out[i].fd = -1; /* track fd via userdata in upper layer */
    out[i].events = e;
    out[i].userdata = (uintptr_t)evs[i].data.u64;
    if (loop->timer_fd >=0 && evs[i].data.u64==0){
      uint64_t x; read(loop->timer_fd, &x, sizeof(x)); /* drain */
    }
  }
  return n;
}
#endif
