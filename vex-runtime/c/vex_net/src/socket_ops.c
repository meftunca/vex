#ifndef _POSIX_C_SOURCE
#define _POSIX_C_SOURCE 200112L
#endif
#if defined(__linux__)
#ifndef _GNU_SOURCE
#define _GNU_SOURCE
#endif
#endif

#include "vex_net.h"
#include <string.h>
#include <errno.h>

#if defined(_WIN32)
#include <winsock2.h>
#include <ws2tcpip.h>
#pragma comment(lib, "Ws2_32.lib")
static int set_nonblock(int fd){ u_long m=1; return ioctlsocket((SOCKET)fd, FIONBIO, &m)==0?0:-1; }
#else
#include <unistd.h>
#include <fcntl.h>
#include <sys/types.h>
#include <sys/socket.h>
#include <sys/uio.h>
#include <netinet/in.h>
#include <netinet/tcp.h>
#include <arpa/inet.h>
#include <netdb.h>
static int set_nonblock(int fd){ int f=fcntl(fd, F_GETFL, 0); return fcntl(fd, F_SETFL, f|O_NONBLOCK); }
#endif

int vex_net_socket_tcp(int ipv6){
  int af = ipv6?AF_INET6:AF_INET;
#if defined(_WIN32)
  SOCKET s = socket(af, SOCK_STREAM, IPPROTO_TCP);
  if (s==INVALID_SOCKET) return -1; 
  set_nonblock((int)s); 
  return (int)s;
#elif defined(__linux__)
  int fd = (int)socket(af, SOCK_STREAM | SOCK_NONBLOCK, 0);
  return fd<0?-1:fd;
#else
  /* BSD/macOS: SOCK_NONBLOCK not available, use fcntl */
  int fd = (int)socket(af, SOCK_STREAM, 0);
  if (fd < 0) return -1;
  set_nonblock(fd);
  return fd;
#endif
}

int vex_net_socket_udp(int ipv6){
  int af = ipv6?AF_INET6:AF_INET;
#if defined(_WIN32)
  SOCKET s = socket(af, SOCK_DGRAM, IPPROTO_UDP);
  if (s==INVALID_SOCKET) return -1; 
  set_nonblock((int)s); 
  return (int)s;
#elif defined(__linux__)
  int fd = (int)socket(af, SOCK_DGRAM | SOCK_NONBLOCK, 0);
  return fd<0?-1:fd;
#else
  /* BSD/macOS: SOCK_NONBLOCK not available, use fcntl */
  int fd = (int)socket(af, SOCK_DGRAM, 0);
  if (fd < 0) return -1;
  set_nonblock(fd);
  return fd;
#endif
}

int vex_net_bind(int fd, const char* ip, uint16_t port, int reuseaddr, int reuseport, int ipv6only){
#if defined(_WIN32)
  (void)reuseport; (void)ipv6only;
#endif
  int rc=0; if (!ip) ip = "0.0.0.0";
#if defined(_WIN32)
  struct addrinfo hints; memset(&hints,0,sizeof(hints)); hints.ai_family=AF_UNSPEC; hints.ai_socktype=0; hints.ai_flags=AI_PASSIVE;
  char portbuf[16]; _snprintf(portbuf, sizeof(portbuf), "%u", (unsigned)port);
  struct addrinfo *ai=NULL; if (getaddrinfo(ip, portbuf, &hints, &ai)!=0) return -1;
  if (reuseaddr){ BOOL on=TRUE; setsockopt((SOCKET)fd,SOL_SOCKET,SO_REUSEADDR,(const char*)&on,sizeof(on)); }
  rc = bind((SOCKET)fd, ai->ai_addr, (int)ai->ai_addrlen);
  freeaddrinfo(ai); return rc==0?0:-1;
#else
  if (strchr(ip,':')) {
    struct sockaddr_in6 a6; memset(&a6,0,sizeof(a6)); a6.sin6_family=AF_INET6; a6.sin6_port=htons(port);
    if (inet_pton(AF_INET6, ip, &a6.sin6_addr)!=1) return -1;
    if (reuseaddr){ int on=1; setsockopt(fd,SOL_SOCKET,SO_REUSEADDR,&on,sizeof(on)); }
#ifdef SO_REUSEPORT
    if (reuseport){ int on=1; setsockopt(fd,SOL_SOCKET,SO_REUSEPORT,&on,sizeof(on)); }
#endif
#ifdef IPV6_V6ONLY
    if (ipv6only){ int on=1; setsockopt(fd,IPPROTO_IPV6,IPV6_V6ONLY,&on,sizeof(on)); }
#endif
    rc = bind(fd,(struct sockaddr*)&a6,sizeof(a6));
  } else {
    struct sockaddr_in a4; memset(&a4,0,sizeof(a4)); a4.sin_family=AF_INET; a4.sin_port=htons(port);
    if (inet_pton(AF_INET, ip, &a4.sin_addr)!=1) return -1;
    if (reuseaddr){ int on=1; setsockopt(fd,SOL_SOCKET,SO_REUSEADDR,&on,sizeof(on)); }
#ifdef SO_REUSEPORT
    if (reuseport){ int on=1; setsockopt(fd,SOL_SOCKET,SO_REUSEPORT,&on,sizeof(on)); }
#endif
    rc = bind(fd,(struct sockaddr*)&a4,sizeof(a4));
  }
  return rc==0?0:-1;
#endif
}

int vex_net_listen(int fd, int backlog){
#if defined(_WIN32)
  return listen((SOCKET)fd, backlog)==0?0:-1;
#else
  return listen(fd, backlog)==0?0:-1;
#endif
}

int vex_net_accept(int fd, char* ip_buf, size_t ip_buflen, uint16_t* port){
  struct sockaddr_storage ss; 
#if defined(_WIN32)
  int slen=(int)sizeof(ss);
  SOCKET c = accept((SOCKET)fd, (struct sockaddr*)&ss, &slen);
  if (c==INVALID_SOCKET) return -1; 
  set_nonblock((int)c);
#elif defined(__linux__)
  socklen_t slen=sizeof(ss);
  int c = accept4(fd, (struct sockaddr*)&ss, &slen, SOCK_NONBLOCK);
  if (c<0) return -1;
#else
  /* BSD/macOS: accept4 not available */
  socklen_t slen=sizeof(ss);
  int c = accept(fd, (struct sockaddr*)&ss, &slen);
  if (c<0) return -1;
  set_nonblock(c);
#endif
  if (ip_buf && ip_buflen>0){
    void* addr=NULL; uint16_t p=0;
    if (ss.ss_family==AF_INET){
      struct sockaddr_in* a=(struct sockaddr_in*)&ss; addr=&a->sin_addr; p=ntohs(a->sin_port);
      inet_ntop(AF_INET, addr, ip_buf, (socklen_t)ip_buflen);
    } else if (ss.ss_family==AF_INET6){
      struct sockaddr_in6* a=(struct sockaddr_in6*)&ss; addr=&a->sin6_addr; p=ntohs(a->sin6_port);
      inet_ntop(AF_INET6, addr, ip_buf, (socklen_t)ip_buflen);
    } else { ip_buf[0]=0; }
    if (port) *port = p;
  }
#if defined(_WIN32)
  return (int)c;
#else
  return c;
#endif
}

int vex_net_connect(int fd, const char* ip, uint16_t port){
#if defined(_WIN32)
  struct addrinfo hints; memset(&hints,0,sizeof(hints)); hints.ai_family=AF_UNSPEC; hints.ai_socktype=SOCK_STREAM;
  char portbuf[16]; _snprintf(portbuf, sizeof(portbuf), "%u", (unsigned)port);
  struct addrinfo *ai=NULL; if (getaddrinfo(ip, portbuf, &hints, &ai)!=0) return -1;
  int rc = connect((SOCKET)fd, ai->ai_addr, (int)ai->ai_addrlen); int err = (rc==0)?0:WSAGetLastError(); freeaddrinfo(ai);
  if (rc==0) return 0; return (err==WSAEWOULDBLOCK || err==WSAEINPROGRESS)?0:-1;
#else
  int rc=-1;
  if (strchr(ip,':')){ struct sockaddr_in6 a6; memset(&a6,0,sizeof(a6)); a6.sin6_family=AF_INET6; a6.sin6_port=htons(port);
    if (inet_pton(AF_INET6, ip, &a6.sin6_addr)!=1) return -1; rc = connect(fd, (struct sockaddr*)&a6, sizeof(a6));
  } else { struct sockaddr_in a4; memset(&a4,0,sizeof(a4)); a4.sin_family=AF_INET; a4.sin_port=htons(port);
    if (inet_pton(AF_INET, ip, &a4.sin_addr)!=1) return -1; rc = connect(fd, (struct sockaddr*)&a4, sizeof(a4)); }
  if (rc==0) return 0; return (errno==EINPROGRESS)?0:-1;
#endif
}

int vex_net_set_nodelay(int fd, int on){
  int v = !!on;
#if defined(_WIN32)
  return setsockopt((SOCKET)fd, IPPROTO_TCP, TCP_NODELAY, (const char*)&v, sizeof(v))==0?0:-1;
#else
  return setsockopt(fd, IPPROTO_TCP, TCP_NODELAY, &v, sizeof(v))==0?0:-1;
#endif
}

int vex_net_set_keepalive(int fd, int on, int idle_s, int intvl_s, int cnt){
  int v = !!on;
#if defined(_WIN32)
  if (setsockopt((SOCKET)fd, SOL_SOCKET, SO_KEEPALIVE, (const char*)&v, sizeof(v))!=0) return -1;
  (void)idle_s; (void)intvl_s; (void)cnt; return 0;
#else
  if (setsockopt(fd, SOL_SOCKET, SO_KEEPALIVE, &v, sizeof(v))!=0) return -1;
#ifdef TCP_KEEPIDLE
  if (idle_s>0) setsockopt(fd, IPPROTO_TCP, TCP_KEEPIDLE, &idle_s, sizeof(idle_s));
#endif
#ifdef TCP_KEEPINTVL
  if (intvl_s>0) setsockopt(fd, IPPROTO_TCP, TCP_KEEPINTVL, &intvl_s, sizeof(intvl_s));
#endif
#ifdef TCP_KEEPCNT
  if (cnt>0) setsockopt(fd, IPPROTO_TCP, TCP_KEEPCNT, &cnt, sizeof(cnt));
#endif
  return 0;
#endif
}

int vex_net_set_tos(int fd, int tos){
#if defined(_WIN32)
  return setsockopt((SOCKET)fd, IPPROTO_IP, IP_TOS, (const char*)&tos, sizeof(tos))==0?0:-1;
#elif defined(__APPLE__)
  /* macOS uses IP_TOS but needs special handling */
  (void)fd; (void)tos;
  return -1;  /* Not supported on macOS in standard way */
#else
  return setsockopt(fd, IPPROTO_IP, IP_TOS, &tos, sizeof(tos))==0?0:-1;
#endif
}

int vex_net_set_recvbuf(int fd, int bytes){
#if defined(_WIN32)
  return setsockopt((SOCKET)fd, SOL_SOCKET, SO_RCVBUF, (const char*)&bytes, sizeof(bytes))==0?0:-1;
#else
  return setsockopt(fd, SOL_SOCKET, SO_RCVBUF, &bytes, sizeof(bytes))==0?0:-1;
#endif
}

int vex_net_set_sendbuf(int fd, int bytes){
#if defined(_WIN32)
  return setsockopt((SOCKET)fd, SOL_SOCKET, SO_SNDBUF, (const char*)&bytes, sizeof(bytes))==0?0:-1;
#else
  return setsockopt(fd, SOL_SOCKET, SO_SNDBUF, &bytes, sizeof(bytes))==0?0:-1;
#endif
}

int vex_net_close(int fd){
#if defined(_WIN32)
  return closesocket((SOCKET)fd)==0?0:-1;
#else
  return close(fd)==0?0:-1;
#endif
}

/* Linux extras */
int vex_net_enable_udp_gso(int fd, int gso_size){
#if defined(__linux__) && defined(UDP_SEGMENT)
  return setsockopt(fd, IPPROTO_UDP, UDP_SEGMENT, &gso_size, sizeof(gso_size));
#else
  (void)fd;(void)gso_size; return -1;
#endif
}

int vex_net_enable_msg_zerocopy(int fd, int enable){
#if defined(__linux__) && defined(SO_ZEROCOPY)
  return setsockopt(fd, SOL_SOCKET, SO_ZEROCOPY, &enable, sizeof(enable));
#else
  (void)fd;(void)enable; return -1;
#endif
}

/* I/O */
ssize_t vex_net_read(int fd, void* buf, size_t len){
#if defined(_WIN32)
  return recv((SOCKET)fd, (char*)buf, (int)len, 0);
#else
  return recv(fd, buf, len, 0);
#endif
}
ssize_t vex_net_write(int fd, const void* buf, size_t len){
#if defined(_WIN32)
  return send((SOCKET)fd, (const char*)buf, (int)len, 0);
#else
  return send(fd, buf, len, 0);
#endif
}
ssize_t vex_net_readv(int fd, struct vex_iovec* iov, int iovcnt){
#if defined(_WIN32)
  (void)fd;(void)iov;(void)iovcnt; return -1;
#else
  struct iovec* v=(struct iovec*)iov; return readv(fd, v, iovcnt);
#endif
}
ssize_t vex_net_writev(int fd, struct vex_iovec* iov, int iovcnt){
#if defined(_WIN32)
  (void)fd;(void)iov;(void)iovcnt; return -1;
#else
  struct iovec* v=(struct iovec*)iov; return writev(fd, v, iovcnt);
#endif
}

