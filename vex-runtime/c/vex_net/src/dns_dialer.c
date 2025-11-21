#ifndef _POSIX_C_SOURCE
#define _POSIX_C_SOURCE 200112L
#endif

#include "vex_net.h"
#include "../../vex.h"
#include <string.h>
#include <stdlib.h>
#include <stdio.h>

#if defined(_WIN32)
#include <winsock2.h>
#include <ws2tcpip.h>
#pragma comment(lib, "Ws2_32.lib")
static int set_nonblock(int fd)
{
  u_long m = 1;
  return ioctlsocket((SOCKET)fd, FIONBIO, &m) == 0 ? 0 : -1;
}
#else
#include <unistd.h>
#include <fcntl.h>
#include <time.h>
#include <pthread.h>
#include <sys/types.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <netdb.h>
#include <errno.h>
static int set_nonblock(int fd)
{
  int f = fcntl(fd, F_GETFL, 0);
  return fcntl(fd, F_SETFL, f | O_NONBLOCK);
}
#endif

static int dial_one(const struct addrinfo *ai, const VexDialer *d)
{
  int fd = (int)socket(ai->ai_family, SOCK_STREAM, 0);
  if (fd < 0)
    return -1;
  set_nonblock(fd);
  /* optional bind */
  if (d->local_ip)
  {
    if (ai->ai_family == AF_INET)
    {
      struct sockaddr_in a;
      memset(&a, 0, sizeof(a));
      a.sin_family = AF_INET;
      a.sin_port = htons(d->local_port);
      if (inet_pton(AF_INET, d->local_ip, &a.sin_addr) == 1)
        bind(fd, (struct sockaddr *)&a, sizeof(a));
    }
    else if (ai->ai_family == AF_INET6)
    {
      struct sockaddr_in6 a;
      memset(&a, 0, sizeof(a));
      a.sin6_family = AF_INET6;
      a.sin6_port = htons(d->local_port);
      if (inet_pton(AF_INET6, d->local_ip, &a.sin6_addr) == 1)
        bind(fd, (struct sockaddr *)&a, sizeof(a));
    }
  }
  int rc = connect(fd, ai->ai_addr, (socklen_t)ai->ai_addrlen);
#if defined(_WIN32)
  int werr = (rc == 0) ? 0 : WSAGetLastError();
  if (rc == 0 || werr == WSAEWOULDBLOCK || werr == WSAEINPROGRESS)
    return fd;
#else
  if (rc == 0 || errno == EINPROGRESS)
    return fd;
#endif
  vex_net_close(fd);
  return -1;
}

/* Simple immediate nonblocking dial: choose preferred family first, then fallback */
int vex_net_dial_tcp(VexNetLoop *loop, const VexDialer *d)
{
  (void)loop;
  struct addrinfo hints;
  memset(&hints, 0, sizeof(hints));
  hints.ai_family = AF_UNSPEC;
  hints.ai_socktype = SOCK_STREAM;
  struct addrinfo *res = NULL;
  if (getaddrinfo(d->host, d->port ? d->port : "80", &hints, &res) != 0 || !res)
    return -1;
  struct addrinfo *v6[128];
  int n6 = 0;
  struct addrinfo *v4[128];
  int n4 = 0;
  for (struct addrinfo *ai = res; ai && (n6 < 128 || n4 < 128); ai = ai->ai_next)
  {
    if (ai->ai_family == AF_INET6 && n6 < 128)
      v6[n6++] = ai;
    else if (ai->ai_family == AF_INET && n4 < 128)
      v4[n4++] = ai;
  }
  int fd = -1;
  if (d->ipv6_first)
  {
    for (int i = 0; i < n6 && fd < 0; i++)
      fd = dial_one(v6[i], d);
    for (int i = 0; i < n4 && fd < 0; i++)
      fd = dial_one(v4[i], d);
  }
  else
  {
    for (int i = 0; i < n4 && fd < 0; i++)
      fd = dial_one(v4[i], d);
    for (int i = 0; i < n6 && fd < 0; i++)
      fd = dial_one(v6[i], d);
  }
  freeaddrinfo(res);
  return fd;
}

/* Background HEv2: spawn a tiny helper thread (POSIX or Windows) that staggers attempts and posts completion via timer/event queue.
   For simplicity, we return 0 on spawn ok, <0 on error. Completion arrives in loop via userdata. */

typedef struct
{
  VexNetLoop *loop;
  VexDialer d;
  uintptr_t udata;
} HEv2Task;

#if defined(_WIN32)
#include <process.h>
static unsigned __stdcall hev2_thread(void *p)
{
  HEv2Task *t = (HEv2Task *)p;
  struct addrinfo hints;
  memset(&hints, 0, sizeof(hints));
  hints.ai_family = AF_UNSPEC;
  hints.ai_socktype = SOCK_STREAM;
  struct addrinfo *res = NULL;
  if (getaddrinfo(t->d.host, t->d.port ? t->d.port : "80", &hints, &res) != 0 || !res)
  {
    PostQueuedCompletionStatus((HANDLE)(intptr_t)t->loop->fd, 0, (ULONG_PTR)t->udata, NULL);
    vex_free(t);
    return 0;
  }
  struct addrinfo *v6[128];
  int n6 = 0;
  struct addrinfo *v4[128];
  int n4 = 0;
  for (struct addrinfo *ai = res; ai && (n6 < 128 || n4 < 128); ai = ai->ai_next)
  {
    if (ai->ai_family == AF_INET6)
      v6[n6++] = ai;
    else if (ai->ai_family == AF_INET)
      v4[n4++] = ai;
  }
  int delay = t->d.stagger_ms > 0 ? t->d.stagger_ms : 250;
  int fd = -1;
  if (t->d.ipv6_first)
  {
    for (int i = 0; i < n6 && fd < 0; i++)
    {
      fd = dial_one(v6[i], &t->d);
      if (fd >= 0)
        break;
      Sleep(delay);
    }
    for (int i = 0; i < n4 && fd < 0; i++)
    {
      fd = dial_one(v4[i], &t->d);
      if (fd >= 0)
        break;
      Sleep(delay);
    }
  }
  else
  {
    for (int i = 0; i < n4 && fd < 0; i++)
    {
      fd = dial_one(v4[i], &t->d);
      if (fd >= 0)
        break;
      Sleep(delay);
    }
    for (int i = 0; i < n6 && fd < 0; i++)
    {
      fd = dial_one(v6[i], &t->d);
      if (fd >= 0)
        break;
      Sleep(delay);
    }
  }
  freeaddrinfo(res);
  /* Signal completion: we encode fd through userdata by requiring Vex to map userdata->pending dial and fetch fd via an API if needed.
     Simpler: Post completion; Vex can poll a shared table keyed by userdata to retrieve fd stored globally.
     For now, just post the userdata; the user retrieves fd via a shared map in higher layer. */
  PostQueuedCompletionStatus((HANDLE)(intptr_t)t->loop->fd, (DWORD)(fd >= 0 ? fd : 0), (ULONG_PTR)t->udata, NULL);
  vex_free(t);
  return 0;
}
#else
#include <pthread.h>
#include <time.h>
static void *hev2_thread(void *p)
{
  HEv2Task *t = (HEv2Task *)p;
  struct addrinfo hints;
  memset(&hints, 0, sizeof(hints));
  hints.ai_family = AF_UNSPEC;
  hints.ai_socktype = SOCK_STREAM;
  struct addrinfo *res = NULL;
  if (getaddrinfo(t->d.host, t->d.port ? t->d.port : "80", &hints, &res) != 0 || !res)
  { /* post timer pulse */
    vex_net_timer_after(t->loop, 1, t->udata);
    vex_free(t);
    return NULL;
  }
  struct addrinfo *v6[128];
  int n6 = 0;
  struct addrinfo *v4[128];
  int n4 = 0;
  for (struct addrinfo *ai = res; ai && (n6 < 128 || n4 < 128); ai = ai->ai_next)
  {
    if (ai->ai_family == AF_INET6)
      v6[n6++] = ai;
    else if (ai->ai_family == AF_INET)
      v4[n4++] = ai;
  }
  int delay = t->d.stagger_ms > 0 ? t->d.stagger_ms : 250;
  int fd = -1;
  struct timespec ts = {.tv_sec = delay / 1000, .tv_nsec = (delay % 1000) * 1000000};
  if (t->d.ipv6_first)
  {
    for (int i = 0; i < n6 && fd < 0; i++)
    {
      fd = dial_one(v6[i], &t->d);
      if (fd >= 0)
        break;
      nanosleep(&ts, NULL);
    }
    for (int i = 0; i < n4 && fd < 0; i++)
    {
      fd = dial_one(v4[i], &t->d);
      if (fd >= 0)
        break;
      nanosleep(&ts, NULL);
    }
  }
  else
  {
    for (int i = 0; i < n4 && fd < 0; i++)
    {
      fd = dial_one(v4[i], &t->d);
      if (fd >= 0)
        break;
      nanosleep(&ts, NULL);
    }
    for (int i = 0; i < n6 && fd < 0; i++)
    {
      fd = dial_one(v6[i], &t->d);
      if (fd >= 0)
        break;
      nanosleep(&ts, NULL);
    }
  }
  freeaddrinfo(res);
  /* pulse loop */
  vex_net_timer_after(t->loop, 1, t->udata);
  vex_free(t);
  return NULL;
}
#endif

int vex_net_hev2_start(VexNetLoop *loop, const VexDialer *d, uintptr_t completion_userdata)
{
  HEv2Task *t = (HEv2Task *)vex_malloc(sizeof(*t));
  if (!t)
    return -1;
  t->loop = loop;
  t->d = *d;
  t->udata = completion_userdata;
#if defined(_WIN32)
  uintptr_t th = _beginthreadex(NULL, 0, hev2_thread, t, 0, NULL);
  if (!th)
  {
    vex_free(t);
    return -1;
  }
  CloseHandle((HANDLE)th);
#else
  pthread_t th;
  if (pthread_create(&th, NULL, hev2_thread, t) != 0)
  {
    vex_free(t);
    return -1;
  }
  pthread_detach(th);
#endif
  return 0;
}
