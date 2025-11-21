#if !defined(_WIN32)
#include "vex_time.h"
#include "../../vex_allocator.h"
#include <time.h>
#include <pthread.h>
#include <stdlib.h>
#include <string.h>

/* --- Now --- */
void vt_now(VexTime *out)
{
  struct timespec tsw, tsm;
  clock_gettime(CLOCK_REALTIME, &tsw);
  clock_gettime(CLOCK_MONOTONIC, &tsm);
  out->wall.unix_sec = (int64_t)tsw.tv_sec;
  out->wall.nsec = (int32_t)tsw.tv_nsec;
  out->wall._pad = 0;
  out->mono_ns = (uint64_t)tsm.tv_sec * 1000000000ULL + (uint64_t)tsm.tv_nsec;
}
uint64_t vt_monotonic_now_ns(void)
{
  struct timespec t;
  clock_gettime(CLOCK_MONOTONIC, &t);
  return (uint64_t)t.tv_sec * 1000000000ULL + (uint64_t)t.tv_nsec;
}

/* --- Sleep --- */
int vt_sleep_ns(VexDuration ns)
{
  if (ns <= 0)
    return 0;
  struct timespec ts;
  ts.tv_sec = ns / 1000000000LL;
  ts.tv_nsec = ns % 1000000000LL;
  return nanosleep(&ts, NULL);
}

/* --- Scheduler types --- */
typedef struct HeapNode
{
  void *owner; /* points to VexTimer/VexTicker */
  int active;
  int periodic;
  VexDuration period_ns;
  int64_t due_ns;
  VexTimeCb cb;
  void *user;
} HeapNode;

struct VexTimeSched
{
  pthread_t th;
  int running;
  pthread_mutex_t mu;
  pthread_cond_t cv;
  HeapNode *heap;
  int heap_sz;
  int heap_cap;
};

/* heap helpers */
static void heap_swap(HeapNode *a, HeapNode *b)
{
  HeapNode t = *a;
  *a = *b;
  *b = t;
}
static void heap_up(HeapNode *h, int i)
{
  while (i > 0)
  {
    int p = (i - 1) / 2;
    if (h[p].due_ns <= h[i].due_ns)
      break;
    heap_swap(&h[p], &h[i]);
    i = p;
  }
}
static void heap_down(HeapNode *h, int n, int i)
{
  for (;;)
  {
    int l = 2 * i + 1, r = l + 1, m = i;
    if (l < n && h[l].due_ns < h[m].due_ns)
      m = l;
    if (r < n && h[r].due_ns < h[m].due_ns)
      m = r;
    if (m == i)
      break;
    heap_swap(&h[i], &h[m]);
    i = m;
  }
}
static void heap_push(struct VexTimeSched *s, const HeapNode *node)
{
  if (s->heap_sz == s->heap_cap)
  {
    int nc = s->heap_cap ? s->heap_cap * 2 : 8;
    s->heap = (HeapNode *)VEX_REALLOC_IMPL(s->heap, nc * sizeof(HeapNode));
    s->heap_cap = nc;
  }
  s->heap[s->heap_sz] = *node;
  heap_up(s->heap, s->heap_sz);
  s->heap_sz++;
}
static int heap_pop(struct VexTimeSched *s, HeapNode *out)
{
  if (s->heap_sz == 0)
    return 0;
  *out = s->heap[0];
  s->heap[0] = s->heap[--s->heap_sz];
  heap_down(s->heap, s->heap_sz, 0);
  return 1;
}
static int heap_remove_owner(struct VexTimeSched *s, void *owner)
{
  for (int i = 0; i < s->heap_sz; i++)
  {
    if (s->heap[i].owner == owner)
    {
      s->heap[i] = s->heap[--s->heap_sz];
      heap_down(s->heap, s->heap_sz, i);
      return 1;
    }
  }
  return 0;
}

/* worker thread */
static void *worker(void *arg)
{
  struct VexTimeSched *s = (struct VexTimeSched *)arg;
  pthread_mutex_lock(&s->mu);
  while (s->running)
  {
    if (s->heap_sz == 0)
    {
      pthread_cond_wait(&s->cv, &s->mu);
      continue;
    }
    int64_t now = (int64_t)vt_monotonic_now_ns();
    int64_t due = s->heap[0].due_ns;
    if (due > now)
    {
      struct timespec ts;
      ts.tv_sec = (due - now) / 1000000000LL;
      ts.tv_nsec = (due - now) % 1000000000LL;
      pthread_cond_timedwait(&s->cv, &s->mu, &ts);
      continue;
    }
    HeapNode n;
    heap_pop(s, &n);
    if (!n.active)
    {
      continue;
    }
    pthread_mutex_unlock(&s->mu);
    VexTime tnow;
    vt_now(&tnow);
    if (n.cb)
      n.cb(n.user, tnow);
    pthread_mutex_lock(&s->mu);
    if (n.periodic && n.active)
    {
      n.due_ns = (int64_t)vt_monotonic_now_ns() + n.period_ns;
      heap_push(s, &n);
    }
  }
  pthread_mutex_unlock(&s->mu);
  return NULL;
}

/* API */
VexTimeSched *vt_sched_create(void)
{
  struct VexTimeSched *s = (struct VexTimeSched *)VEX_CALLOC_IMPL(1, sizeof(*s));
  if (!s)
    return NULL;
  pthread_mutex_init(&s->mu, NULL);
  pthread_cond_init(&s->cv, NULL);
  s->running = 1;
  if (pthread_create(&s->th, NULL, worker, s) != 0)
  {
    s->running = 0;
    VEX_FREE_IMPL(s);
    return NULL;
  }
  return s;
}
void vt_sched_destroy(VexTimeSched *s)
{
  if (!s)
    return;
  pthread_mutex_lock(&s->mu);
  s->running = 0;
  pthread_cond_broadcast(&s->cv);
  pthread_mutex_unlock(&s->mu);
  pthread_join(s->th, NULL);
  VEX_FREE_IMPL(s->heap);
  pthread_mutex_destroy(&s->mu);
  pthread_cond_destroy(&s->cv);
  VEX_FREE_IMPL(s);
}

struct VexTimer
{
  struct VexTimeSched *s;
  HeapNode node;
};
struct VexTicker
{
  struct VexTimeSched *s;
  HeapNode node;
};

static int sched_insert(struct VexTimeSched *s, HeapNode n)
{
  pthread_mutex_lock(&s->mu);
  heap_push(s, &n);
  pthread_cond_broadcast(&s->cv);
  pthread_mutex_unlock(&s->mu);
  return 0;
}

VexTimer *vt_timer_create(VexTimeSched *s, VexTimeCb cb, void *user)
{
  if (!s)
    return NULL;
  VexTimer *t = (VexTimer *)VEX_CALLOC_IMPL(1, sizeof(*t));
  t->s = s;
  t->node.cb = cb;
  t->node.user = user;
  t->node.periodic = 0;
  t->node.active = 0;
  t->node.owner = t;
  return t;
}
int vt_timer_start(VexTimer *t, VexDuration after_ns)
{
  if (!t)
    return -1;
  t->node.active = 1;
  t->node.periodic = 0;
  t->node.period_ns = 0;
  t->node.owner = t;
  t->node.due_ns = (int64_t)vt_monotonic_now_ns() + (after_ns < 0 ? 0 : after_ns);
  return sched_insert(t->s, t->node);
}
int vt_timer_reset(VexTimer *t, VexDuration after_ns)
{
  if (!t)
    return -1;
  pthread_mutex_lock(&t->s->mu);
  heap_remove_owner(t->s, t);
  pthread_mutex_unlock(&t->s->mu);
  t->node.active = 1;
  t->node.periodic = 0;
  t->node.period_ns = 0;
  t->node.owner = t;
  t->node.due_ns = (int64_t)vt_monotonic_now_ns() + (after_ns < 0 ? 0 : after_ns);
  return sched_insert(t->s, t->node);
}
int vt_timer_stop(VexTimer *t)
{
  if (!t)
    return -1;
  pthread_mutex_lock(&t->s->mu);
  heap_remove_owner(t->s, t);
  t->node.active = 0;
  pthread_mutex_unlock(&t->s->mu);
  return 0;
}
void vt_timer_destroy(VexTimer *t)
{
  if (t)
  {
    vt_timer_stop(t);
    VEX_FREE_IMPL(t);
  }
}

VexTicker *vt_ticker_create(VexTimeSched *s, VexTimeCb cb, void *user)
{
  if (!s)
    return NULL;
  VexTicker *tk = (VexTicker *)VEX_CALLOC_IMPL(1, sizeof(*tk));
  tk->s = s;
  tk->node.cb = cb;
  tk->node.user = user;
  tk->node.periodic = 1;
  tk->node.active = 0;
  tk->node.owner = tk;
  return tk;
}
int vt_ticker_start(VexTicker *tk, VexDuration period_ns)
{
  if (!tk || period_ns <= 0)
    return -1;
  tk->node.active = 1;
  tk->node.periodic = 1;
  tk->node.period_ns = period_ns;
  tk->node.owner = tk;
  tk->node.due_ns = (int64_t)vt_monotonic_now_ns() + period_ns;
  return sched_insert(tk->s, tk->node);
}
int vt_ticker_reset(VexTicker *tk, VexDuration period_ns)
{
  if (!tk || period_ns <= 0)
    return -1;
  pthread_mutex_lock(&tk->s->mu);
  heap_remove_owner(tk->s, tk);
  pthread_mutex_unlock(&tk->s->mu);
  tk->node.active = 1;
  tk->node.periodic = 1;
  tk->node.period_ns = period_ns;
  tk->node.owner = tk;
  tk->node.due_ns = (int64_t)vt_monotonic_now_ns() + period_ns;
  return sched_insert(tk->s, tk->node);
}
int vt_ticker_stop(VexTicker *tk)
{
  if (!tk)
    return -1;
  pthread_mutex_lock(&tk->s->mu);
  heap_remove_owner(tk->s, tk);
  tk->node.active = 0;
  pthread_mutex_unlock(&tk->s->mu);
  return 0;
}
void vt_ticker_destroy(VexTicker *tk)
{
  if (tk)
  {
    vt_ticker_stop(tk);
    VEX_FREE_IMPL(tk);
  }
}

#endif /* !_WIN32 */
