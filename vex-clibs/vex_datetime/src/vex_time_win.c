#if defined(_WIN32)
#include "vex_time.h"
#include <windows.h>
#include <process.h>
#include <stdlib.h>

/* ---- Now ---- */
static LARGE_INTEGER qpf = {0};
void vt_now(VexTime* out){
  if (!qpf.QuadPart) QueryPerformanceFrequency(&qpf);
  LARGE_INTEGER qpc; QueryPerformanceCounter(&qpc);
  ULONGLONG mono_ns = (ULONGLONG)(qpc.QuadPart) * 1000000000ULL / (ULONGLONG)qpf.QuadPart;
  /* wall */
  FILETIME ft; GetSystemTimePreciseAsFileTime(&ft);
  ULONGLONG t = (((ULONGLONG)ft.dwHighDateTime<<32)|ft.dwLowDateTime); /* 100ns since 1601 */
  const ULONGLONG EPOCH_DIFF_100NS = 116444736000000000ULL;
  ULONGLONG unix100 = t - EPOCH_DIFF_100NS;
  out->wall.unix_sec = (int64_t)(unix100 / 10000000ULL);
  out->wall.nsec = (int32_t)((unix100 % 10000000ULL) * 100ULL);
  out->wall._pad=0;
  out->mono_ns = mono_ns;
}
uint64_t vt_monotonic_now_ns(void){
  if (!qpf.QuadPart) QueryPerformanceFrequency(&qpf);
  LARGE_INTEGER qpc; QueryPerformanceCounter(&qpc);
  return (ULONGLONG)(qpc.QuadPart) * 1000000000ULL / (ULONGLONG)qpf.QuadPart;
}

/* ---- Sleep ---- */
int vt_sleep_ns(VexDuration ns){
  if (ns<=0) return 0;
  HANDLE h = CreateWaitableTimerEx(NULL, NULL, CREATE_WAITABLE_TIMER_HIGH_RESOLUTION, TIMER_ALL_ACCESS);
  if (!h) return -1;
  LARGE_INTEGER li;
  /* relative time in 100ns ticks, negative */
  LONGLONG rel = -(LONGLONG)(ns/100); if (rel==0) rel=-1;
  li.QuadPart = rel;
  if (!SetWaitableTimer(h, &li, 0, NULL, NULL, FALSE)){ CloseHandle(h); return -1; }
  DWORD rc = WaitForSingleObject(h, INFINITE);
  CloseHandle(h);
  return (rc==WAIT_OBJECT_0)?0:-1;
}

/* ---- Scheduler (single thread, heap) ---- */
typedef struct HeapNode {
  int active;
  int periodic;
  VexDuration period_ns;
  LONGLONG due_ns;
  VexTimeCb cb;
  void* user;
} HeapNode;

typedef struct VexTimeSched {
  HANDLE th;
  CRITICAL_SECTION mu;
  CONDITION_VARIABLE cv;
  int running;
  HeapNode* heap; int heap_sz; int heap_cap;
} VexTimeSched;

static void heap_swap(HeapNode* a, HeapNode* b){ HeapNode t=*a; *a=*b; *b=t; }
static void heap_up(HeapNode* h, int i){
  while (i>0){ int p=(i-1)/2; if (h[p].due_ns<=h[i].due_ns) break; heap_swap(&h[p],&h[i]); i=p; }
}
static void heap_down(HeapNode* h, int n, int i){
  for(;;){
    int l=2*i+1, r=l+1, m=i;
    if (l<n && h[l].due_ns<h[m].due_ns) m=l;
    if (r<n && h[r].due_ns<h[m].due_ns) m=r;
    if (m==i) break; heap_swap(&h[i],&h[m]); i=m;
  }
}
static void heap_push(VexTimeSched* s, const HeapNode* node){
  if (s->heap_sz==s->heap_cap){
    int nc = s->heap_cap? s->heap_cap*2 : 8;
    s->heap = (HeapNode*)realloc(s->heap, nc*sizeof(HeapNode)); s->heap_cap = nc;
  }
  s->heap[s->heap_sz] = *node;
  heap_up(s->heap, s->heap_sz);
  s->heap_sz++;
}
static int heap_pop(VexTimeSched* s, HeapNode* out){
  if (s->heap_sz==0) return 0;
  *out = s->heap[0];
  s->heap[0] = s->heap[--s->heap_sz];
  heap_down(s->heap, s->heap_sz, 0);
  return 1;
}

static unsigned __stdcall worker(void* arg){
  VexTimeSched* s=(VexTimeSched*)arg;
  EnterCriticalSection(&s->mu);
  while (s->running){
    if (s->heap_sz==0){
      SleepConditionVariableCS(&s->cv, &s->mu, INFINITE);
      continue;
    }
    LONGLONG now = (LONGLONG)vt_monotonic_now_ns();
    LONGLONG due = s->heap[0].due_ns;
    if (due>now){
      DWORD ms = (DWORD)((due-now)/1000000LL);
      SleepConditionVariableCS(&s->cv, &s->mu, ms?ms:1);
      continue;
    }
    HeapNode n; heap_pop(s,&n);
    if (!n.active){ continue; }
    LeaveCriticalSection(&s->mu);
    VexTime tnow; vt_now(&tnow);
    if (n.cb) n.cb(n.user, tnow);
    EnterCriticalSection(&s->mu);
    if (n.periodic && n.active){
      n.due_ns = (LONGLONG)vt_monotonic_now_ns() + n.period_ns;
      heap_push(s, &n);
    }
  }
  LeaveCriticalSection(&s->mu);
  return 0;
}

VexTimeSched* vt_sched_create(void){
  VexTimeSched* s = (VexTimeSched*)calloc(1,sizeof(*s));
  if (!s) return NULL;
  InitializeCriticalSection(&s->mu);
  InitializeConditionVariable(&s->cv);
  s->running=1;
  uintptr_t th = _beginthreadex(NULL, 0, worker, s, 0, NULL);
  if (!th){ s->running=0; DeleteCriticalSection(&s->mu); free(s); return NULL; }
  s->th = (HANDLE)th;
  return s;
}
void vt_sched_destroy(VexTimeSched* s){
  if (!s) return;
  EnterCriticalSection(&s->mu);
  s->running=0;
  WakeAllConditionVariable(&s->cv);
  LeaveCriticalSection(&s->mu);
  WaitForSingleObject(s->th, INFINITE);
  CloseHandle(s->th);
  DeleteCriticalSection(&s->mu);
  free(s->heap);
  free(s);
}

typedef struct VexTimer  { VexTimeSched* s; HeapNode node; } VexTimer;
typedef struct VexTicker { VexTimeSched* s; HeapNode node; } VexTicker;

static int sched_insert(VexTimeSched* s, HeapNode n){
  EnterCriticalSection(&s->mu);
  heap_push(s,&n);
  WakeAllConditionVariable(&s->cv);
  LeaveCriticalSection(&s->mu);
  return 0;
}

VexTimer* vt_timer_create(VexTimeSched* s, VexTimeCb cb, void* user){
  if (!s) return NULL;
  VexTimer* t=(VexTimer*)calloc(1,sizeof(*t));
  t->s=s; t->node.cb=cb; t->node.user=user; t->node.periodic=0; t->node.active=0;
  return t;
}
int vt_timer_start(VexTimer* t, VexDuration after_ns){
  if (!t) return -1;
  t->node.active=1; t->node.periodic=0; t->node.period_ns=0;
  t->node.due_ns = (LONGLONG)vt_monotonic_now_ns() + (after_ns<0?0:after_ns);
  return sched_insert(t->s, t->node);
}
int vt_timer_reset(VexTimer* t, VexDuration after_ns){
  if (!t) return -1;
  t->node.active=1; t->node.periodic=0; t->node.period_ns=0;
  t->node.due_ns = (LONGLONG)vt_monotonic_now_ns() + (after_ns<0?0:after_ns);
  return sched_insert(t->s, t->node);
}
int vt_timer_stop(VexTimer* t){ if (!t) return -1; t->node.active=0; return 0; }
void vt_timer_destroy(VexTimer* t){ if (t){ vt_timer_stop(t); free(t);} }

VexTicker* vt_ticker_create(VexTimeSched* s, VexTimeCb cb, void* user){
  if (!s) return NULL;
  VexTicker* tk=(VexTicker*)calloc(1,sizeof(*tk));
  tk->s=s; tk->node.cb=cb; tk->node.user=user; tk->node.periodic=1; tk->node.active=0;
  return tk;
}
int vt_ticker_start(VexTicker* tk, VexDuration period_ns){
  if (!tk || period_ns<=0) return -1;
  tk->node.active=1; tk->node.periodic=1; tk->node.period_ns=period_ns;
  tk->node.due_ns = (LONGLONG)vt_monotonic_now_ns() + period_ns;
  return sched_insert(tk->s, tk->node);
}
int vt_ticker_reset(VexTicker* tk, VexDuration period_ns){
  if (!tk || period_ns<=0) return -1;
  tk->node.active=1; tk->node.periodic=1; tk->node.period_ns=period_ns;
  tk->node.due_ns = (LONGLONG)vt_monotonic_now_ns() + period_ns;
  return sched_insert(tk->s, tk->node);
}
int vt_ticker_stop(VexTicker* tk){ if (!tk) return -1; tk->node.active=0; return 0; }
void vt_ticker_destroy(VexTicker* tk){ if (tk){ vt_ticker_stop(tk); free(tk);} }

#endif /* _WIN32 */
