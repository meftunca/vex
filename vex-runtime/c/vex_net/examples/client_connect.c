#include "vex_net.h"
#include <stdio.h>
#include <string.h>

int main(int argc, char** argv){
  const char* host = argc>1?argv[1]:"example.com";
  const char* port = argc>2?argv[2]:"80";
  VexNetLoop loop; vex_net_loop_create(&loop);
  VexDialer d = { .host=host, .port=port, .ipv6_first=1, .stagger_ms=200 };
  int fd = vex_net_dial_tcp(&loop, &d);
  if (fd<0){ fprintf(stderr,"dial failed\n"); return 1; }
  vex_net_register(&loop, fd, VEX_EVT_WRITE|VEX_EVT_READ, (uintptr_t)fd);
  VexEvent ev;
  while (1){
    int n = vex_net_tick(&loop, &ev, 1, 3000);
    if (n<=0) { fprintf(stderr,"timeout\n"); return 2; }
    if ((int)ev.userdata == fd && (ev.events & VEX_EVT_WRITE)) break;
  }
  /* optional: HTTP CONNECT through proxy
     VexDialer d2 = d; d2.http_proxy="127.0.0.1:8080"; 
     int pfd = vex_net_dial_tcp(&loop, &(VexDialer){.host="127.0.0.1",.port="8080"});
     vex_proxy_http_connect(pfd, host, port, 3000); fd=pfd;
  */
  const char* req = "GET / HTTP/1.1\r\nHost: example.com\r\nConnection: close\r\n\r\n";
  vex_net_write(fd, req, strlen(req));
  char buf[4096]; ssize_t r;
  while ((r=vex_net_read(fd, buf, sizeof buf))>0) fwrite(buf,1,(size_t)r,stdout);
  vex_net_close(fd); vex_net_loop_close(&loop);
  return 0;
}
