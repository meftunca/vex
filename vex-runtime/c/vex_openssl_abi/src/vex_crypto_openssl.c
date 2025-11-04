#include "vex_crypto.h"
#include <openssl/evp.h>
#include <openssl/err.h>
#include <openssl/kdf.h>
#include <string.h>
#include <stdio.h>

#if defined(_MSC_VER)
#define THREAD_LOCAL __declspec(thread)
#else
#define THREAD_LOCAL _Thread_local
#endif

static THREAD_LOCAL char g_cerr[256];
const char* vex_crypto_last_error(void){ return g_cerr; }
static void set_err_from_queue(void){
  unsigned long e = ERR_get_error();
  if (e) ERR_error_string_n(e, g_cerr, sizeof(g_cerr));
  else snprintf(g_cerr, sizeof(g_cerr), "crypto error");
}
static void set_err(const char* s){
  snprintf(g_cerr, sizeof(g_cerr), "%s", s ? s : "crypto error");
}

static const EVP_CIPHER* aead_from_name(const char* n, int* is_aead_mode){
  *is_aead_mode = 1;
  if (!n) return NULL;
  if (strcmp(n, "AES-128-GCM") == 0) return EVP_aes_128_gcm();
  if (strcmp(n, "AES-256-GCM") == 0) return EVP_aes_256_gcm();
#if defined(EVP_chacha20_poly1305)
  if (strcmp(n, "CHACHA20-POLY1305") == 0) return EVP_chacha20_poly1305();
#endif
  *is_aead_mode = 0;
  return NULL;
}

int vex_aead_seal(const char* aead_name,
                  const uint8_t* key, size_t key_len,
                  const uint8_t* nonce, size_t nonce_len,
                  const uint8_t* ad, size_t ad_len,
                  const uint8_t* plaintext, size_t pt_len,
                  uint8_t* out_cipher, size_t* inout_ct_len,
                  size_t tag_len){
  int is_aead = 0;
  const EVP_CIPHER* ciph = aead_from_name(aead_name, &is_aead);
  if (!is_aead || !ciph) { set_err("unsupported aead"); return -1; }
  EVP_CIPHER_CTX* ctx = EVP_CIPHER_CTX_new();
  if (!ctx) { set_err_from_queue(); return -1; }
  int ok = 0, outl = 0, len = 0;
  if (EVP_EncryptInit_ex(ctx, ciph, NULL, NULL, NULL) != 1) goto done;
  if (EVP_CIPHER_CTX_ctrl(ctx, EVP_CTRL_AEAD_SET_IVLEN, (int)nonce_len, NULL) != 1) goto done;
  if (EVP_EncryptInit_ex(ctx, NULL, NULL, key, nonce) != 1) goto done;
  if (ad && ad_len) { if (EVP_EncryptUpdate(ctx, NULL, &outl, ad, (int)ad_len) != 1) goto done; }
  if (pt_len) {
    if (EVP_EncryptUpdate(ctx, out_cipher, &outl, plaintext, (int)pt_len) != 1) goto done;
    len = outl;
  }
  if (EVP_EncryptFinal_ex(ctx, out_cipher + len, &outl) != 1) goto done;
  len += outl;
  if (EVP_CIPHER_CTX_ctrl(ctx, EVP_CTRL_AEAD_GET_TAG, (int)tag_len, out_cipher + len) != 1) goto done;
  len += (int)tag_len;
  *inout_ct_len = (size_t)len;
  ok = 1;
done:
  if (!ok) set_err_from_queue();
  EVP_CIPHER_CTX_free(ctx);
  return ok ? 0 : -1;
}

int vex_aead_open(const char* aead_name,
                  const uint8_t* key, size_t key_len,
                  const uint8_t* nonce, size_t nonce_len,
                  const uint8_t* ad, size_t ad_len,
                  const uint8_t* ciphertext, size_t ct_len,
                  uint8_t* out_plain, size_t* inout_pt_len,
                  size_t tag_len){
  if (ct_len < tag_len) { set_err("ct too short"); return -1; }
  int is_aead = 0;
  const EVP_CIPHER* ciph = aead_from_name(aead_name, &is_aead);
  if (!is_aead || !ciph) { set_err("unsupported aead"); return -1; }
  size_t data_len = ct_len - tag_len;
  const uint8_t* tag = ciphertext + data_len;
  EVP_CIPHER_CTX* ctx = EVP_CIPHER_CTX_new();
  if (!ctx) { set_err_from_queue(); return -1; }
  int ok = 0, outl = 0, len = 0;
  if (EVP_DecryptInit_ex(ctx, ciph, NULL, NULL, NULL) != 1) goto done;
  if (EVP_CIPHER_CTX_ctrl(ctx, EVP_CTRL_AEAD_SET_IVLEN, (int)nonce_len, NULL) != 1) goto done;
  if (EVP_DecryptInit_ex(ctx, NULL, NULL, key, nonce) != 1) goto done;
  if (ad && ad_len) { if (EVP_DecryptUpdate(ctx, NULL, &outl, ad, (int)ad_len) != 1) goto done; }
  if (data_len) {
    if (EVP_DecryptUpdate(ctx, out_plain, &outl, ciphertext, (int)data_len) != 1) goto done;
    len = outl;
  }
  if (EVP_CIPHER_CTX_ctrl(ctx, EVP_CTRL_AEAD_SET_TAG, (int)tag_len, (void*)tag) != 1) goto done;
  if (EVP_DecryptFinal_ex(ctx, out_plain + len, &outl) != 1) { set_err("tag mismatch"); goto done; }
  len += outl;
  *inout_pt_len = (size_t)len;
  ok = 1;
done:
  if (!ok && strcmp(g_cerr, "tag mismatch") != 0) set_err_from_queue();
  EVP_CIPHER_CTX_free(ctx);
  return ok ? 0 : -1;
}

static const EVP_MD* md_from_name(const char* n){
  if (!n) return NULL;
  if (strcmp(n, "SHA-256") == 0) return EVP_sha256();
  if (strcmp(n, "SHA-512") == 0) return EVP_sha512();
#if defined(EVP_sha3_256)
  if (strcmp(n, "SHA3-256") == 0) return EVP_sha3_256();
  if (strcmp(n, "SHA3-512") == 0) return EVP_sha3_512();
#endif
  return NULL;
}

int vex_hash(const char* algo, const uint8_t* msg, size_t len, uint8_t* out_digest, size_t* inout_len){
  const EVP_MD* md = md_from_name(algo);
  if (!md) { set_err("unsupported hash"); return -1; }
  EVP_MD_CTX* ctx = EVP_MD_CTX_new();
  if (!ctx) { set_err_from_queue(); return -1; }
  int ok = 0;
  if (EVP_DigestInit_ex(ctx, md, NULL) != 1) goto done;
  if (EVP_DigestUpdate(ctx, msg, len) != 1) goto done;
  unsigned outl = 0;
  if (EVP_DigestFinal_ex(ctx, out_digest, &outl) != 1) goto done;
  if (inout_len) *inout_len = outl;
  ok = 1;
done:
  if (!ok) set_err_from_queue();
  EVP_MD_CTX_free(ctx);
  return ok ? 0 : -1;
}

int vex_hkdf(const char* algo,
             const uint8_t* ikm, size_t ikm_len,
             const uint8_t* salt, size_t salt_len,
             const uint8_t* info, size_t info_len,
             uint8_t* out_okm, size_t okm_len){
  const EVP_MD* md = NULL;
  if (strcmp(algo, "HKDF-SHA256") == 0) md = EVP_sha256();
  else if (strcmp(algo, "HKDF-SHA512") == 0) md = EVP_sha512();
  else { set_err("unsupported hkdf"); return -1; }

#if OPENSSL_VERSION_NUMBER >= 0x10101000L
  EVP_PKEY_CTX* pctx = EVP_PKEY_CTX_new_id(EVP_PKEY_HKDF, NULL);
  if (!pctx) { set_err_from_queue(); return -1; }
  int ok = 0;
  if (EVP_PKEY_derive_init(pctx) <= 0) goto done;
  if (EVP_PKEY_CTX_set_hkdf_md(pctx, md) <= 0) goto done;
  if (salt && salt_len) if (EVP_PKEY_CTX_set1_hkdf_salt(pctx, salt, (int)salt_len) <= 0) goto done;
  if (EVP_PKEY_CTX_set1_hkdf_key(pctx, ikm, (int)ikm_len) <= 0) goto done;
  if (info && info_len) if (EVP_PKEY_CTX_add1_hkdf_info(pctx, info, (int)info_len) <= 0) goto done;
  size_t outlen = okm_len;
  if (EVP_PKEY_derive(pctx, out_okm, &outlen) <= 0) goto done;
  ok = 1;
done:
  if (!ok) set_err_from_queue();
  EVP_PKEY_CTX_free(pctx);
  return ok ? 0 : -1;
#else
  set_err("HKDF requires OpenSSL >= 1.1.1");
  return -1;
#endif
}

int vex_x25519_public_from_private(uint8_t pub[32], const uint8_t priv[32]){
#if OPENSSL_VERSION_NUMBER >= 0x10101000L
  EVP_PKEY* pkey = EVP_PKEY_new_raw_private_key(EVP_PKEY_X25519, NULL, priv, 32);
  if (!pkey) { set_err_from_queue(); return -1; }
  size_t publen = 32;
  int ok = EVP_PKEY_get_raw_public_key(pkey, pub, &publen);
  EVP_PKEY_free(pkey);
  if (ok != 1 || publen != 32) { set_err_from_queue(); return -1; }
  return 0;
#else
  set_err("X25519 requires OpenSSL >= 1.1.1");
  return -1;
#endif
}

int vex_x25519(uint8_t shared[32], const uint8_t priv[32], const uint8_t peer_pub[32]){
#if OPENSSL_VERSION_NUMBER >= 0x10101000L
  int ok = -1;
  EVP_PKEY* a = EVP_PKEY_new_raw_private_key(EVP_PKEY_X25519, NULL, priv, 32);
  EVP_PKEY* b = EVP_PKEY_new_raw_public_key(EVP_PKEY_X25519, NULL, peer_pub, 32);
  if (!a || !b) { set_err_from_queue(); goto done; }
  EVP_PKEY_CTX* ctx = EVP_PKEY_CTX_new(a, NULL);
  if (!ctx) { set_err_from_queue(); goto done2; }
  size_t slen = 32;
  if (EVP_PKEY_derive_init(ctx) != 1) { set_err_from_queue(); goto done3; }
  if (EVP_PKEY_derive_set_peer(ctx, b) != 1) { set_err_from_queue(); goto done3; }
  if (EVP_PKEY_derive(ctx, shared, &slen) != 1 || slen != 32) { set_err_from_queue(); goto done3; }
  ok = 0;
done3:
  EVP_PKEY_CTX_free(ctx);
done2:
  EVP_PKEY_free(a); EVP_PKEY_free(b);
done:
  return ok;
#else
  set_err("X25519 requires OpenSSL >= 1.1.1");
  return -1;
#endif
}

int vex_ed25519_sign(uint8_t sig[64], const uint8_t* msg, size_t len, const uint8_t sk[64]){
#if OPENSSL_VERSION_NUMBER >= 0x10101000L
  int ok = -1;
  EVP_PKEY* pkey = EVP_PKEY_new_raw_private_key(EVP_PKEY_ED25519, NULL, sk, 64);
  if (!pkey) { set_err_from_queue(); return -1; }
  EVP_MD_CTX* ctx = EVP_MD_CTX_new();
  if (!ctx) { set_err_from_queue(); EVP_PKEY_free(pkey); return -1; }
  size_t slen = 64;
  if (EVP_DigestSignInit(ctx, NULL, NULL, NULL, pkey) != 1) { set_err_from_queue(); goto done; }
  if (EVP_DigestSign(ctx, sig, &slen, msg, len) != 1 || slen != 64) { set_err_from_queue(); goto done; }
  ok = 0;
done:
  EVP_MD_CTX_free(ctx);
  EVP_PKEY_free(pkey);
  return ok;
#else
  set_err("Ed25519 requires OpenSSL >= 1.1.1");
  return -1;
#endif
}

int vex_ed25519_verify(const uint8_t sig[64], const uint8_t* msg, size_t len, const uint8_t pk[32]){
#if OPENSSL_VERSION_NUMBER >= 0x10101000L
  int ok = -1;
  EVP_PKEY* pkey = EVP_PKEY_new_raw_public_key(EVP_PKEY_ED25519, NULL, pk, 32);
  if (!pkey) { set_err_from_queue(); return -1; }
  EVP_MD_CTX* ctx = EVP_MD_CTX_new();
  if (!ctx) { set_err_from_queue(); EVP_PKEY_free(pkey); return -1; }
  if (EVP_DigestVerifyInit(ctx, NULL, NULL, NULL, pkey) != 1) { set_err_from_queue(); goto done; }
  if (EVP_DigestVerify(ctx, sig, 64, msg, len) != 1) { set_err_from_queue(); goto done; }
  ok = 0;
done:
  EVP_MD_CTX_free(ctx);
  EVP_PKEY_free(pkey);
  return ok;
#else
  set_err("Ed25519 requires OpenSSL >= 1.1.1");
  return -1;
#endif
}
