#if defined(__linux__) && defined(VEX_NET_HAVE_URING)
#include "vex_net.h"
#include <liburing.h>
int vex_net_capabilities(void){
  struct io_uring ring; int rc = io_uring_queue_init(2, &ring, 0);
  if (rc==0){ io_uring_queue_exit(&ring); return VEX_CAP_IOURING | VEX_CAP_TIMER; }
  return VEX_CAP_TIMER;
}
#endif
