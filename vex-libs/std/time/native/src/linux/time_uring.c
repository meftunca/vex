
#if defined(__linux__) && defined(VEX_TIME_HAVE_URING)
#include "vex_time.h"
#include <liburing.h>
#include <pthread.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>

typedef struct Node {
  uint64_t key; /* user_data key */
  int periodic;
  VexDuration period_ns;
  VexTimeCb cb;
  void* user;
  struct __kernel_timespec ts;
} Node;

typedef struct VexTimeSched {
  struct io_uring ring;
  pthread_t th;
  int running;
} VexTimeSched;

static void submit_timeout(VexTimeSched* s, Node* n){
  struct io_uring_sqe* sqe = io_uring_get_sqe(&s->ring);
  if (!sqe) return;
  io_uring_prep_timeout(sqe, &n->ts, 0, 0);
  io_uring_sqe_set_data64(sqe, n->key);
  io_uring_submit(&s->ring);
}

static void* ur_worker(void* arg){
  VexTimeSched* s=(VexTimeSched*)arg;
  while (s->running){
    struct io_uring_cqe* cqe=NULL;
    int rc = io_uring_wait_cqe(&s->ring, &cqe);
    if (rc==0 && cqe){
      uint64_t key = io_uring_cqe_get_data64(cqe);
      Node* n = (Node*)(uintptr_t)key;
      if (n && n->cb){
        VexTime t; vt_now(&t);
        n->cb(n->user, t);
        if (n->periodic && s->running){
          n->ts.tv_sec  = n->period_ns/1000000000LL;
          n->ts.tv_nsec = n->period_ns%1000000000LL;
          submit_timeout(s, n);
        } else {
          free(n);
        }
      }
      io_uring_cqe_seen(&s->ring, cqe);
    }
  }
  return NULL;
}

VexTimeSched* vt_sched_create_uring(void){
  VexTimeSched* s=(VexTimeSched*)calloc(1,sizeof(*s));
  if (!s) return NULL;
  if (io_uring_queue_init(128, &s->ring, 0)!=0){ free(s); return NULL; }
  s->running=1;
  if (pthread_create(&s->th,NULL,ur_worker,s)!=0){ io_uring_queue_exit(&s->ring); free(s); return NULL; }
  return s;
}
void vt_sched_destroy(VexTimeSched* s){
  if (!s) return;
  s->running=0;
  io_uring_queue_exit(&s->ring);
  pthread_join(s->th,NULL);
  free(s);
}

/* timer/ticker objects */
typedef struct VexTimer  { VexTimeSched* s; Node* n; VexTimeCb cb; void* user; } VexTimer;
typedef struct VexTicker { VexTimeSched* s; Node* n; VexTimeCb cb; void* user; } VexTicker;

VexTimer* vt_timer_create(VexTimeSched* s, VexTimeCb cb, void* user){
  VexTimer* t=(VexTimer*)calloc(1,sizeof(*t)); if (!t) return NULL; t->s=s; t->cb=cb; t->user=user; return t;
}
int vt_timer_start(VexTimer* t, VexDuration after_ns){
  if (!t||!t->s) return -1;
  if (t->n) { free(t->n); t->n=NULL; }
  t->n=(Node*)calloc(1,sizeof(Node)); if (!t->n) return -1;
  t->n->key = (uint64_t)(uintptr_t)t->n;
  t->n->periodic=0; t->n->period_ns=0;
  t->n->cb = t->cb; t->n->user = t->user;
  t->n->ts.tv_sec  = after_ns/1000000000LL;
  t->n->ts.tv_nsec = after_ns%1000000000LL;
  submit_timeout(t->s, t->n);
  return 0;
}
int vt_timer_reset(VexTimer* t, VexDuration after_ns){
  if (!t||!t->n) return -1;
  struct io_uring_sqe* sqe = io_uring_get_sqe(&t->s->ring);
  if (!sqe) return -1;
  io_uring_prep_timeout_remove(sqe, t->n->key, 0);
  io_uring_submit(&t->s->ring);
  t->n->ts.tv_sec  = after_ns/1000000000LL;
  t->n->ts.tv_nsec = after_ns%1000000000LL;
  submit_timeout(t->s, t->n);
  return 0;
}
int vt_timer_stop(VexTimer* t){
  if (!t||!t->n) return -1;
  struct io_uring_sqe* sqe = io_uring_get_sqe(&t->s->ring);
  if (!sqe) return -1;
  io_uring_prep_timeout_remove(sqe, t->n->key, 0);
  io_uring_submit(&t->s->ring);
  return 0;
}
void vt_timer_destroy(VexTimer* t){ if (t){ if (t->n) free(t->n); free(t);} }

VexTicker* vt_ticker_create(VexTimeSched* s, VexTimeCb cb, void* user){
  VexTicker* tk=(VexTicker*)calloc(1,sizeof(*tk)); if (!tk) return NULL; tk->s=s; tk->cb=cb; tk->user=user;
  tk->n=(Node*)calloc(1,sizeof(Node)); if (!tk->n){ free(tk); return NULL; }
  tk->n->key=(uint64_t)(uintptr_t)tk->n; tk->n->periodic=1; tk->n->period_ns=0; tk->n->cb=cb; tk->n->user=user;
  return tk;
}
int vt_ticker_start(VexTicker* tk, VexDuration period_ns){
  if (!tk||!tk->s||!tk->n) return -1;
  tk->n->periodic=1; tk->n->period_ns=period_ns;
  tk->n->ts.tv_sec  = period_ns/1000000000LL;
  tk->n->ts.tv_nsec = period_ns%1000000000LL;
  submit_timeout(tk->s, tk->n);
  return 0;
}
int vt_ticker_reset(VexTicker* tk, VexDuration period_ns){
  if (!tk||!tk->n) return -1;
  struct io_uring_sqe* sqe = io_uring_get_sqe(&tk->s->ring);
  if (!sqe) return -1;
  io_uring_prep_timeout_remove(sqe, tk->n->key, 0);
  io_uring_submit(&tk->s->ring);
  tk->n->period_ns=period_ns;
  tk->n->ts.tv_sec  = period_ns/1000000000LL;
  tk->n->ts.tv_nsec = period_ns%1000000000LL;
  submit_timeout(tk->s, tk->n);
  return 0;
}
int vt_ticker_stop(VexTicker* tk){
  if (!tk||!tk->n) return -1;
  struct io_uring_sqe* sqe = io_uring_get_sqe(&tk->s->ring);
  if (!sqe) return -1;
  io_uring_prep_timeout_remove(sqe, tk->n->key, 0);
  io_uring_submit(&tk->s->ring);
  return 0;
}
void vt_ticker_destroy(VexTicker* tk){ if (tk){ if (tk->n) free(tk->n); free(tk);} }

#endif /* linux & VEX_TIME_HAVE_URING */
