#include "vex_net.h"
#include <string.h>
#include <stdio.h>
#include <stdlib.h>

#if defined(_WIN32)
#include <winsock2.h>
#include <ws2tcpip.h>
#pragma comment(lib, "Ws2_32.lib")
static int set_deadline(int fd, int ms){ (void)fd;(void)ms; return 0; }
#else
#include <sys/types.h>
#include <sys/socket.h>
#include <sys/select.h>
#include <unistd.h>
#include <errno.h>
static int set_deadline(int fd, int ms){
  struct timeval tv; tv.tv_sec=ms/1000; tv.tv_usec=(ms%1000)*1000;
  setsockopt(fd,SOL_SOCKET,SO_RCVTIMEO,&tv,sizeof(tv));
  setsockopt(fd,SOL_SOCKET,SO_SNDTIMEO,&tv,sizeof(tv));
  return 0;
}
#endif

int vex_proxy_http_connect(int fd, const char* host, const char* port, int timeout_ms){
  char req[512];
  int n = snprintf(req,sizeof(req),"CONNECT %s:%s HTTP/1.1\r\nHost: %s:%s\r\nProxy-Connection: Keep-Alive\r\n\r\n",host,port,host,port);
  set_deadline(fd, timeout_ms>0?timeout_ms:3000);
#if defined(_WIN32)
  if (send((SOCKET)fd, req, n, 0)!=n) return -1;
  char buf[512]; int r = recv((SOCKET)fd, buf, sizeof(buf)-1, 0);
#else
  if (send(fd, req, n, 0)!=n) return -1;
  char buf[512]; int r = (int)recv(fd, buf, sizeof(buf)-1, 0);
#endif
  if (r<=0) return -1; buf[r]=0;
  if (strstr(buf," 200 ") || strstr(buf," 200\r")) return 0;
  return -1;
}

int vex_proxy_socks5_connect(int fd, const char* host, const char* port, int timeout_ms){
  set_deadline(fd, timeout_ms>0?timeout_ms:3000);
  unsigned char hello[3]={0x05,0x01,0x00};
#if defined(_WIN32)
  if (send((SOCKET)fd,(const char*)hello,3,0)!=3) return -1;
  unsigned char r0[2]; if (recv((SOCKET)fd,(char*)r0,2,0)!=2) return -1;
#else
  if (send(fd,hello,3,0)!=3) return -1;
  unsigned char r0[2]; if (recv(fd,r0,2,0)!=2) return -1;
#endif
  if (!(r0[0]==0x05 && r0[1]==0x00)) return -1;
  unsigned char req[512]; size_t off=0;
  req[off++]=0x05; req[off++]=0x01; req[off++]=0x00; /* CONNECT, no auth */
  req[off++]=0x03; /* domain */
  size_t hlen=strlen(host); if (hlen>255) return -1; req[off++]=(unsigned char)hlen; memcpy(&req[off],host,hlen); off+=hlen;
  int p = atoi(port?port:"80"); req[off++]=(unsigned char)((p>>8)&0xFF); req[off++]=(unsigned char)(p&0xFF);
#if defined(_WIN32)
  if (send((SOCKET)fd,(const char*)req,(int)off,0)!=(int)off) return -1;
  unsigned char resp[10+255]; int r = recv((SOCKET)fd,(char*)resp,sizeof(resp),0);
#else
  if (send(fd,req,off,0)!=(int)off) return -1;
  unsigned char resp[10+255]; int r = (int)recv(fd,resp,sizeof(resp),0);
#endif
  if (r<7) return -1;
  if (!(resp[0]==0x05 && resp[1]==0x00)) return -1;
  return 0;
}
