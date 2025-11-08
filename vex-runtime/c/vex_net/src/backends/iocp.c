#if defined(_WIN32)
#include "vex_net.h"
#include <winsock2.h>
#include <windows.h>
#include <mswsock.h>
#include <ws2tcpip.h>
#pragma comment(lib, "Ws2_32.lib")

static PTP_TIMER g_timer = NULL;
static HANDLE g_cp = NULL;

int vex_net_capabilities(void){ return VEX_CAP_IOCP | VEX_CAP_TIMER; }

static VOID CALLBACK timer_cb(PTP_CALLBACK_INSTANCE inst, PVOID ctx, PTP_TIMER timer){
  (void)inst; (void)timer;
  uintptr_t userdata = (uintptr_t)ctx;
  /* post completion with userdata */
  PostQueuedCompletionStatus(g_cp, 0, (ULONG_PTR)userdata, NULL);
}

int vex_net_loop_create(VexNetLoop* loop){
  WSADATA w; WSAStartup(MAKEWORD(2,2), &w);
  g_cp = CreateIoCompletionPort(INVALID_HANDLE_VALUE, NULL, 0, 0);
  if (!g_cp) return -1;
  g_timer = CreateThreadpoolTimer(timer_cb, (PVOID)0, NULL);
  if (!g_timer) return -1;
  loop->fd = (int)(intptr_t)g_cp; loop->backend = 3; loop->timer_fd = 1;
  return 0;
}

int vex_net_loop_close(VexNetLoop* loop){
  if (!loop) return -1;
  if (g_timer){ SetThreadpoolTimer(g_timer, NULL, 0, 0); WaitForThreadpoolTimerCallbacks(g_timer, TRUE); CloseThreadpoolTimer(g_timer); g_timer=NULL; }
  if (g_cp){ CloseHandle(g_cp); g_cp=NULL; }
  WSACleanup(); loop->fd=-1; return 0;
}

int vex_net_timer_after(VexNetLoop* loop, uint64_t ms, uintptr_t userdata){
  (void)loop;
  if (!g_timer || !g_cp) return -1;
  FILETIME ft; ULONGLONG due = (ULONGLONG)ms * 10000ULL; /* ms to 100ns */
  ULONGLONG now = 0; GetSystemTimeAsFileTime(&ft); now = (((ULONGLONG)ft.dwHighDateTime<<32)|ft.dwLowDateTime);
  ULONGLONG set = now + due;
  ft.dwLowDateTime = (DWORD)set; ft.dwHighDateTime = (DWORD)(set>>32);
  SetThreadpoolTimer(g_timer, &ft, 0, 0);
  /* stash userdata; simple approach: re-create timer with context */
  CloseThreadpoolTimer(g_timer);
  g_timer = CreateThreadpoolTimer(timer_cb, (PVOID)userdata, NULL);
  SetThreadpoolTimer(g_timer, &ft, 0, 0);
  return 1;
}

int vex_net_register(VexNetLoop* loop, int fd, uint32_t events, uintptr_t userdata){
  (void)events;
  HANDLE cp = (HANDLE)(intptr_t)loop->fd;
  HANDLE h = (HANDLE)_get_osfhandle(fd);
  return CreateIoCompletionPort(h, cp, (ULONG_PTR)userdata, 0) ? 0 : -1;
}
int vex_net_modify(VexNetLoop* loop, int fd, uint32_t events, uintptr_t userdata){
  (void)loop; (void)fd; (void)events; (void)userdata; return 0;
}
int vex_net_unregister(VexNetLoop* loop, int fd){ (void)loop; (void)fd; return 0; }

int vex_net_tick(VexNetLoop* loop, VexEvent* out, int capacity, int timeout_ms){
  HANDLE cp = (HANDLE)(intptr_t)loop->fd;
  DWORD ms = (timeout_ms<0)?INFINITE:(DWORD)timeout_ms;
  DWORD bytes=0; ULONG_PTR key=0; LPOVERLAPPED ov=0;
  int n=0;
  while (n<capacity){
    BOOL ok = GetQueuedCompletionStatus(cp, &bytes, &key, &ov, ms);
    if (!ok && !ov) break;
    out[n].fd=-1; out[n].userdata=(uintptr_t)key;
    out[n].events = ok ? (VEX_EVT_READ|VEX_EVT_WRITE) : VEX_EVT_ERR;
    n++; ms=0;
  }
  return n;
}
#endif
