#ifndef VEX_NET_H
#define VEX_NET_H
#include <stdint.h>
#include <stddef.h>
#include <sys/types.h>  /* for ssize_t */
#ifdef __cplusplus
extern "C" {
#endif

/* Event flags */
enum { VEX_EVT_READ=1u<<0, VEX_EVT_WRITE=1u<<1, VEX_EVT_HUP=1u<<2, VEX_EVT_ERR=1u<<3 };

/* Backends: 1=epoll, 2=kqueue, 3=iocp, 4=io_uring */
typedef struct { int backend; int fd; int timer_fd; int reserved; } VexNetLoop;
typedef struct { int fd; uint32_t events; uintptr_t userdata; } VexReg;
typedef struct { int fd; uint32_t events; uintptr_t userdata; } VexEvent;

/* Capabilities */
enum {
  VEX_CAP_IOURING   = 1u<<0,
  VEX_CAP_EPOLLEXCL = 1u<<1,
  VEX_CAP_KQUEUE    = 1u<<2,
  VEX_CAP_IOCP      = 1u<<3,
  VEX_CAP_TIMER     = 1u<<4,
  VEX_CAP_UDP_GSO   = 1u<<5,
  VEX_CAP_MSG_ZC    = 1u<<6
};

/* Loop */
int  vex_net_capabilities(void);
int  vex_net_loop_create(VexNetLoop* loop);
int  vex_net_loop_close(VexNetLoop* loop);
int  vex_net_timer_after(VexNetLoop* loop, uint64_t ms, uintptr_t userdata);
int  vex_net_register(VexNetLoop* loop, int fd, uint32_t events, uintptr_t userdata);
int  vex_net_modify(VexNetLoop* loop, int fd, uint32_t events, uintptr_t userdata);
int  vex_net_unregister(VexNetLoop* loop, int fd);
int  vex_net_tick(VexNetLoop* loop, VexEvent* out, int capacity, int timeout_ms);

/* Sockets */
int  vex_net_socket_tcp(int ipv6);
int  vex_net_socket_udp(int ipv6);
int  vex_net_bind(int fd, const char* ip, uint16_t port, int reuseaddr, int reuseport, int ipv6only);
int  vex_net_listen(int fd, int backlog);
int  vex_net_accept(int fd, char* ip_buf, size_t ip_buflen, uint16_t* port);
int  vex_net_connect(int fd, const char* ip, uint16_t port);
int  vex_net_set_nodelay(int fd, int on);
int  vex_net_set_keepalive(int fd, int on, int idle_s, int intvl_s, int cnt);
int  vex_net_set_tos(int fd, int tos);
int  vex_net_set_recvbuf(int fd, int bytes);
int  vex_net_set_sendbuf(int fd, int bytes);
int  vex_net_close(int fd);

/* Linux extras (best-effort) */
int  vex_net_enable_udp_gso(int fd, int gso_size);      /* returns 0 or -1 */
int  vex_net_enable_msg_zerocopy(int fd, int enable);   /* returns 0 or -1 */

/* TLS hook surface (no TLS impl; Vex wraps the fd) */
typedef struct { int fd; } VexRawConn;
static inline VexRawConn vex_raw_from_fd(int fd){ VexRawConn c; c.fd=fd; return c; }
static inline int vex_raw_fd(VexRawConn c){ return c.fd; }

/* Dialer + DNS / Happy Eyeballs v2 helper */
typedef struct {
  const char* host;
  const char* port;     /* default "80" */
  int ipv6_first;       /* prefer v6 first if 1 */
  int stagger_ms;       /* HEv2 family stagger (default 250ms) */
  int per_attempt_to;   /* per attempt timeout ms (default 1000) */
  const char* local_ip; /* optional bind */
  uint16_t    local_port;
  const char* http_proxy; /* "host:port" or NULL */
  const char* socks5_proxy; /* "host:port" or NULL */
} VexDialer;

int vex_net_dial_tcp(VexNetLoop* loop, const VexDialer* d);
/* HEv2 background helper that tries dual-stack with stagger without blocking the caller.
   Returns first connecting fd via user-provided IO callback posted to loop (Linux/BSD) or IOCP queue (Windows). */
int vex_net_hev2_start(VexNetLoop* loop, const VexDialer* d, uintptr_t completion_userdata);

/* I/O */
struct vex_iovec { void* base; size_t len; };
ssize_t vex_net_read(int fd, void* buf, size_t len);
ssize_t vex_net_write(int fd, const void* buf, size_t len);
ssize_t vex_net_readv(int fd, struct vex_iovec* iov, int iovcnt);
ssize_t vex_net_writev(int fd, struct vex_iovec* iov, int iovcnt);

/* Proxy helpers (blocking short steps with timeouts; optional) */
int vex_proxy_http_connect(int fd, const char* host, const char* port, int timeout_ms);
int vex_proxy_socks5_connect(int fd, const char* host, const char* port, int timeout_ms);

#ifdef __cplusplus
}
#endif
#endif
