#include "vex_net.h"
#include <stdio.h>
#include <string.h>

int main(void){
  VexNetLoop loop;
  if (vex_net_loop_create(&loop)!=0){ fprintf(stderr,"loop create failed\n"); return 1; }
  int s = vex_net_socket_tcp(0);
  vex_net_set_nodelay(s,1);
  if (vex_net_bind(s, "0.0.0.0", 9000, 1, 0, 0)!=0){ fprintf(stderr,"bind failed\n"); return 1; }
  vex_net_listen(s, 128);
  vex_net_register(&loop, s, VEX_EVT_READ, (uintptr_t)s);
  printf("Echo server listening on 0.0.0.0:9000\n");

  VexEvent evs[256];
  char ip[64]; uint16_t port=0;
  for(;;){
    int n = vex_net_tick(&loop, evs, 256, 1000);
    if (n<=0) continue;
    for (int i=0;i<n;i++){
      uintptr_t u = evs[i].userdata;
      if ((int)u == s){
        int c = vex_net_accept(s, ip, sizeof ip, &port);
        if (c>=0){
          vex_net_register(&loop, c, VEX_EVT_READ, (uintptr_t)c);
        }
      } else {
        int c = (int)u;
        if (evs[i].events & (VEX_EVT_HUP|VEX_EVT_ERR)){ vex_net_unregister(&loop,c); vex_net_close(c); continue; }
        if (evs[i].events & VEX_EVT_READ){
          char buf[4096];
          ssize_t r = vex_net_read(c, buf, sizeof buf);
          if (r<=0){ vex_net_unregister(&loop,c); vex_net_close(c); continue; }
          vex_net_write(c, buf, (size_t)r);
        }
      }
    }
  }
  return 0;
}
