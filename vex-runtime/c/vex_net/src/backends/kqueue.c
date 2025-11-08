#if defined(__APPLE__) || defined(__FreeBSD__) || defined(__NetBSD__) || defined(__OpenBSD__)
#include "vex_net.h"
#include <sys/types.h>
#include <sys/event.h>
#include <sys/time.h>
#include <unistd.h>
#include <string.h>

int vex_net_capabilities(void){ return VEX_CAP_KQUEUE | VEX_CAP_TIMER; }

int vex_net_loop_create(VexNetLoop* loop){
  memset(loop,0,sizeof(*loop));
  loop->fd = kqueue();
  loop->timer_fd = -1;
  loop->backend = 2;
  return (loop->fd<0)?-1:0;
}

int vex_net_loop_close(VexNetLoop* loop){
  if (!loop) return -1;
  if (loop->fd>=0) close(loop->fd);
  loop->fd=-1; return 0;
}

int vex_net_timer_after(VexNetLoop* loop, uint64_t ms, uintptr_t userdata){
  struct kevent ev; EV_SET(&ev, (uintptr_t)userdata, EVFILT_TIMER, EV_ADD|EV_ONESHOT, 0, ms, (void*)userdata);
  return kevent(loop->fd, &ev, 1, NULL, 0, NULL)==0?1:-1;
}

static int kev_change(int kq, int fd, uint32_t events, uintptr_t userdata, int op){
  struct kevent ch[2]; int n=0;
  if (events & VEX_EVT_READ)  EV_SET(&ch[n++], fd, EVFILT_READ,  op, 0, 0, (void*)userdata);
  if (events & VEX_EVT_WRITE) EV_SET(&ch[n++], fd, EVFILT_WRITE, op, 0, 0, (void*)userdata);
  return kevent(kq, ch, n, NULL, 0, NULL)==0?0:-1;
}

int vex_net_register(VexNetLoop* loop, int fd, uint32_t events, uintptr_t userdata){
  return kev_change(loop->fd, fd, events, userdata, EV_ADD|EV_ENABLE);
}
int vex_net_modify(VexNetLoop* loop, int fd, uint32_t events, uintptr_t userdata){
  return kev_change(loop->fd, fd, events, userdata, EV_ADD|EV_ENABLE);
}
int vex_net_unregister(VexNetLoop* loop, int fd){
  struct kevent ch[2]; int n=0;
  EV_SET(&ch[n++], fd, EVFILT_READ, EV_DELETE, 0, 0, NULL);
  EV_SET(&ch[n++], fd, EVFILT_WRITE, EV_DELETE, 0, 0, NULL);
  return kevent(loop->fd, ch, n, NULL, 0, NULL)==0?0:-1;
}

int vex_net_tick(VexNetLoop* loop, VexEvent* out, int capacity, int timeout_ms){
  struct timespec ts, *pts=NULL;
  if (timeout_ms>=0){ ts.tv_sec=timeout_ms/1000; ts.tv_nsec=(timeout_ms%1000)*1000000; pts=&ts; }
  if (capacity>1024) capacity=1024;
  struct kevent evs[1024];
  int n = kevent(loop->fd, NULL, 0, evs, capacity, pts);
  if (n<=0) return n;
  for (int i=0;i<n;i++){
    out[i].fd = (int)evs[i].ident; out[i].events=0;
    if (evs[i].filter==EVFILT_READ)  out[i].events|=VEX_EVT_READ;
    if (evs[i].filter==EVFILT_WRITE) out[i].events|=VEX_EVT_WRITE;
    if (evs[i].flags & EV_EOF)       out[i].events|=VEX_EVT_HUP;
    if (evs[i].flags & EV_ERROR)     out[i].events|=VEX_EVT_ERR;
    out[i].userdata = (uintptr_t)evs[i].udata;
  }
  return n;
}
#endif
