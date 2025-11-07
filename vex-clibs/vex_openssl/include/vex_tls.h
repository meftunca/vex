#pragma once
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct VexTlsCtx  VexTlsCtx;
typedef struct VexTlsConn VexTlsConn;

typedef enum {
    VEX_TLS_OK = 0,
    VEX_TLS_WANT_READ  = 1,
    VEX_TLS_WANT_WRITE = 2,
    VEX_TLS_ERR = -1
} VexTlsStatus;

typedef struct {
    int   is_server;
    int   verify_peer;
    const char* ca_bundle_path;
    const char* server_name;
    const char* alpn_csv;
    const char* cert_pem;
    const char* key_pem;
} VexTlsConfig;

VexTlsCtx*  vex_tls_ctx_create(const VexTlsConfig* cfg);
void        vex_tls_ctx_destroy(VexTlsCtx* ctx);
VexTlsConn* vex_tls_conn_wrap_fd(VexTlsCtx* ctx, int fd);
void        vex_tls_conn_destroy(VexTlsConn* c);
VexTlsStatus vex_tls_handshake(VexTlsConn* c);
VexTlsStatus vex_tls_read(VexTlsConn* c, uint8_t* buf, size_t cap, size_t* out_n);
VexTlsStatus vex_tls_write(VexTlsConn* c, const uint8_t* buf, size_t len, size_t* out_n);
int          vex_tls_shutdown(VexTlsConn* c);
int          vex_tls_peer_verified(VexTlsConn* c);
const char*  vex_tls_peer_alpn(VexTlsConn* c);
int          vex_tls_get_fd(VexTlsConn* c);
const char*  vex_tls_last_error(void);

#ifdef __cplusplus
}
#endif
