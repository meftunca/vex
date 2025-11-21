#include "vex_tls.h"
#include "../../vex.h"
#include <openssl/ssl.h>
#include <openssl/err.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

#if defined(_MSC_VER)
#define THREAD_LOCAL __declspec(thread)
#else
#define THREAD_LOCAL _Thread_local
#endif

typedef struct VexTlsCtx
{
  SSL_CTX *ctx;
  int is_server;
} VexTlsCtx;

typedef struct VexTlsConn
{
  SSL *ssl;
  int fd;
  char *alpn_selected;
} VexTlsConn;

static THREAD_LOCAL char g_err[256];

const char *vex_tls_last_error(void) { return g_err; }
static void set_err_from_queue(void)
{
  unsigned long e = ERR_get_error();
  if (e)
    ERR_error_string_n(e, g_err, sizeof(g_err));
  else
    snprintf(g_err, sizeof(g_err), "tls error");
}
static void set_err(const char *s)
{
  snprintf(g_err, sizeof(g_err), "%s", s ? s : "tls error");
}

static int parse_alpn_csv(const char *csv, unsigned char **out, unsigned *out_len)
{
  if (!csv)
  {
    *out = NULL;
    *out_len = 0;
    return 0;
  }
  size_t n = strlen(csv);
  unsigned char *buf = (unsigned char *)vex_malloc(n + 1 + 16);
  if (!buf)
    return -1;
  unsigned pos = 0;
  const char *p = csv;
  while (*p)
  {
    const char *q = strchr(p, ',');
    size_t len = q ? (size_t)(q - p) : strlen(p);
    if (len > 255)
    {
      vex_free(buf);
      return -1;
    }
    buf[pos++] = (unsigned char)len;
    memcpy(buf + pos, p, len);
    pos += (unsigned)len;
    if (!q)
      break;
    p = q + 1;
  }
  *out = buf;
  *out_len = pos;
  return 0;
}

VexTlsCtx *vex_tls_ctx_create(const VexTlsConfig *cfg)
{
  if (!cfg)
  {
    set_err("NULL cfg");
    return NULL;
  }
  OPENSSL_init_ssl(0, NULL);
  VexTlsCtx *c = (VexTlsCtx *)calloc(1, sizeof(*c));
  if (!c)
  {
    set_err("oom");
    return NULL;
  }
  c->is_server = cfg->is_server;
  c->ctx = SSL_CTX_new(cfg->is_server ? TLS_server_method() : TLS_client_method());
  if (!c->ctx)
  {
    set_err_from_queue();
    vex_free(c);
    return NULL;
  }

  SSL_CTX_set_min_proto_version(c->ctx, TLS1_2_VERSION);

  if (cfg->verify_peer)
  {
    SSL_CTX_set_verify(c->ctx, SSL_VERIFY_PEER, NULL);
    if (cfg->ca_bundle_path)
    {
      if (SSL_CTX_load_verify_locations(c->ctx, cfg->ca_bundle_path, NULL) != 1)
      {
        set_err_from_queue();
        goto fail;
      }
    }
    else
    {
      if (SSL_CTX_set_default_verify_paths(c->ctx) != 1)
      {
        set_err_from_queue();
        goto fail;
      }
    }
  }

  if (cfg->is_server && cfg->cert_pem && cfg->key_pem)
  {
    if (SSL_CTX_use_certificate_chain_file(c->ctx, cfg->cert_pem) != 1)
    {
      set_err_from_queue();
      goto fail;
    }
    if (SSL_CTX_use_PrivateKey_file(c->ctx, cfg->key_pem, SSL_FILETYPE_PEM) != 1)
    {
      set_err_from_queue();
      goto fail;
    }
    if (SSL_CTX_check_private_key(c->ctx) != 1)
    {
      set_err("private key mismatch");
      goto fail;
    }
  }

#ifdef TLSEXT_TYPE_application_layer_protocol_negotiation
  if (cfg->alpn_csv)
  {
    unsigned char *wire = NULL;
    unsigned wlen = 0;
    if (parse_alpn_csv(cfg->alpn_csv, &wire, &wlen) != 0)
    {
      set_err("alpn parse");
      goto fail;
    }
    if (SSL_CTX_set_alpn_protos(c->ctx, wire, wlen) != 0)
    {
      vex_free(wire);
      set_err_from_queue();
      goto fail;
    }
    vex_free(wire);
  }
#endif

  return c;
fail:
  if (c->ctx)
    SSL_CTX_free(c->ctx);
  vex_free(c);
  return NULL;
}

void vex_tls_ctx_destroy(VexTlsCtx *c)
{
  if (!c)
    return;
  SSL_CTX_free(c->ctx);
  vex_free(c);
}

VexTlsConn *vex_tls_conn_wrap_fd(VexTlsCtx *ctx, int fd)
{
  if (!ctx)
  {
    set_err("NULL ctx");
    return NULL;
  }
  VexTlsConn *c = (VexTlsConn *)calloc(1, sizeof(*c));
  if (!c)
  {
    set_err("oom");
    return NULL;
  }
  c->fd = fd;
  c->ssl = SSL_new(ctx->ctx);
  if (!c->ssl)
  {
    set_err_from_queue();
    vex_free(c);
    return NULL;
  }
  SSL_set_fd(c->ssl, fd);
  if (ctx->is_server)
    SSL_set_accept_state(c->ssl);
  else
    SSL_set_connect_state(c->ssl);
  return c;
}

void vex_tls_conn_destroy(VexTlsConn *c)
{
  if (!c)
    return;
  if (c->alpn_selected)
    vex_free(c->alpn_selected);
  if (c->ssl)
    SSL_free(c->ssl);
  vex_free(c);
}

static VexTlsStatus map_err(int r, SSL *s)
{
  if (r > 0)
    return VEX_TLS_OK;
  int e = SSL_get_error(s, r);
  if (e == SSL_ERROR_WANT_READ)
    return VEX_TLS_WANT_READ;
  if (e == SSL_ERROR_WANT_WRITE)
    return VEX_TLS_WANT_WRITE;
  set_err_from_queue();
  return VEX_TLS_ERR;
}

VexTlsStatus vex_tls_handshake(VexTlsConn *c)
{
  int r = SSL_do_handshake(c->ssl);
  if (r > 0)
  {
#ifdef TLSEXT_TYPE_application_layer_protocol_negotiation
    const unsigned char *data = NULL;
    unsigned len = 0;
    SSL_get0_alpn_selected(c->ssl, &data, &len);
    if (len > 0 && !c->alpn_selected)
    {
      c->alpn_selected = (char *)vex_malloc(len + 1);
      if (c->alpn_selected)
      {
        memcpy(c->alpn_selected, data, len);
        c->alpn_selected[len] = '\0';
      }
    }
#endif
    return VEX_TLS_OK;
  }
  return map_err(r, c->ssl);
}

VexTlsStatus vex_tls_read(VexTlsConn *c, uint8_t *buf, size_t cap, size_t *out_n)
{
  size_t n = 0;
#if OPENSSL_VERSION_NUMBER >= 0x10101000L
  int ok = SSL_read_ex(c->ssl, buf, cap, &n);
  if (out_n)
    *out_n = n;
  return map_err(ok ? 1 : -1, c->ssl);
#else
  int r = SSL_read(c->ssl, buf, (int)cap);
  if (r > 0)
  {
    if (out_n)
      *out_n = (size_t)r;
    return VEX_TLS_OK;
  }
  return map_err(r, c->ssl);
#endif
}

VexTlsStatus vex_tls_write(VexTlsConn *c, const uint8_t *buf, size_t len, size_t *out_n)
{
  size_t n = 0;
#if OPENSSL_VERSION_NUMBER >= 0x10101000L
  int ok = SSL_write_ex(c->ssl, buf, len, &n);
  if (out_n)
    *out_n = n;
  return map_err(ok ? 1 : -1, c->ssl);
#else
  int r = SSL_write(c->ssl, buf, (int)len);
  if (r > 0)
  {
    if (out_n)
      *out_n = (size_t)r;
    return VEX_TLS_OK;
  }
  return map_err(r, c->ssl);
#endif
}

int vex_tls_shutdown(VexTlsConn *c)
{
  int r = SSL_shutdown(c->ssl);
  if (r < 0)
  {
    set_err_from_queue();
    return -1;
  }
  return 0;
}

int vex_tls_peer_verified(VexTlsConn *c)
{
  long v = SSL_get_verify_result(c->ssl);
  return (v == X509_V_OK) ? 1 : 0;
}

const char *vex_tls_peer_alpn(VexTlsConn *c)
{
  return c->alpn_selected;
}

int vex_tls_get_fd(VexTlsConn *c)
{
  return c->fd;
}
