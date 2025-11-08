#include "vex_crypto.h"
#include <openssl/evp.h>
#include <openssl/err.h>
#include <openssl/kdf.h>
#include <openssl/hmac.h>
#include <openssl/rand.h>
#include <openssl/rsa.h>
#include <openssl/pem.h>
#include <openssl/bio.h>
#include <openssl/ec.h>
#include <openssl/obj_mac.h>
#include <openssl/x509.h>
#include <openssl/x509v3.h>
#include <openssl/md5.h>
#include <openssl/sha.h>
#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>

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
  if (strcmp(n, "CHACHA20-POLY1305") == 0) return EVP_chacha20_poly1305();
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
  if (strcmp(n, "SHA3-256") == 0) return EVP_sha3_256();
  if (strcmp(n, "SHA3-512") == 0) return EVP_sha3_512();
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
}

int vex_x25519_public_from_private(uint8_t pub[32], const uint8_t priv[32]){
  EVP_PKEY* pkey = EVP_PKEY_new_raw_private_key(EVP_PKEY_X25519, NULL, priv, 32);
  if (!pkey) { set_err_from_queue(); return -1; }
  size_t publen = 32;
  int ok = EVP_PKEY_get_raw_public_key(pkey, pub, &publen);
  EVP_PKEY_free(pkey);
  if (ok != 1 || publen != 32) { set_err_from_queue(); return -1; }
  return 0;
}

int vex_x25519(uint8_t shared[32], const uint8_t priv[32], const uint8_t peer_pub[32]){
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
}

int vex_ed25519_sign(uint8_t sig[64], const uint8_t* msg, size_t len, const uint8_t sk[64]){
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
}

int vex_ed25519_verify(const uint8_t sig[64], const uint8_t* msg, size_t len, const uint8_t pk[32]){
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
}

// ============================================================================
// SYMMETRIC ENCRYPTION (CBC, CTR modes)
// ============================================================================

static const EVP_CIPHER* cipher_from_name(const char* n){
  if (!n) return NULL;
  if (strcmp(n, "AES-128-CBC") == 0) return EVP_aes_128_cbc();
  if (strcmp(n, "AES-256-CBC") == 0) return EVP_aes_256_cbc();
  if (strcmp(n, "AES-128-CTR") == 0) return EVP_aes_128_ctr();
  if (strcmp(n, "AES-256-CTR") == 0) return EVP_aes_256_ctr();
  return NULL;
}

int vex_cipher_encrypt(const char* cipher_name,
                       const uint8_t* key, size_t key_len,
                       const uint8_t* iv, size_t iv_len,
                       const uint8_t* plaintext, size_t pt_len,
                       uint8_t* out_cipher, size_t* out_ct_len){
  const EVP_CIPHER* ciph = cipher_from_name(cipher_name);
  if (!ciph) { set_err("unsupported cipher"); return -1; }
  
  int expected_key_len = EVP_CIPHER_key_length(ciph);
  int expected_iv_len = EVP_CIPHER_iv_length(ciph);
  
  if ((int)key_len != expected_key_len) { set_err("invalid key length"); return -1; }
  if ((int)iv_len != expected_iv_len) { set_err("invalid iv length"); return -1; }
  
  EVP_CIPHER_CTX* ctx = EVP_CIPHER_CTX_new();
  if (!ctx) { set_err_from_queue(); return -1; }
  
  int ok = 0, len = 0, outl = 0;
  if (EVP_EncryptInit_ex(ctx, ciph, NULL, key, iv) != 1) goto done;
  if (pt_len > 0) {
    if (EVP_EncryptUpdate(ctx, out_cipher, &outl, plaintext, (int)pt_len) != 1) goto done;
    len = outl;
  }
  if (EVP_EncryptFinal_ex(ctx, out_cipher + len, &outl) != 1) goto done;
  len += outl;
  *out_ct_len = (size_t)len;
  ok = 1;
done:
  if (!ok) set_err_from_queue();
  EVP_CIPHER_CTX_free(ctx);
  return ok ? 0 : -1;
}

int vex_cipher_decrypt(const char* cipher_name,
                       const uint8_t* key, size_t key_len,
                       const uint8_t* iv, size_t iv_len,
                       const uint8_t* ciphertext, size_t ct_len,
                       uint8_t* out_plain, size_t* out_pt_len){
  const EVP_CIPHER* ciph = cipher_from_name(cipher_name);
  if (!ciph) { set_err("unsupported cipher"); return -1; }
  
  int expected_key_len = EVP_CIPHER_key_length(ciph);
  int expected_iv_len = EVP_CIPHER_iv_length(ciph);
  
  if ((int)key_len != expected_key_len) { set_err("invalid key length"); return -1; }
  if ((int)iv_len != expected_iv_len) { set_err("invalid iv length"); return -1; }
  
  EVP_CIPHER_CTX* ctx = EVP_CIPHER_CTX_new();
  if (!ctx) { set_err_from_queue(); return -1; }
  
  int ok = 0, len = 0, outl = 0;
  if (EVP_DecryptInit_ex(ctx, ciph, NULL, key, iv) != 1) goto done;
  if (ct_len > 0) {
    if (EVP_DecryptUpdate(ctx, out_plain, &outl, ciphertext, (int)ct_len) != 1) goto done;
    len = outl;
  }
  if (EVP_DecryptFinal_ex(ctx, out_plain + len, &outl) != 1) goto done;
  len += outl;
  *out_pt_len = (size_t)len;
  ok = 1;
done:
  if (!ok) set_err_from_queue();
  EVP_CIPHER_CTX_free(ctx);
  return ok ? 0 : -1;
}

// ============================================================================
// HMAC
// ============================================================================

int vex_hmac(const char* algo,
             const uint8_t* key, size_t key_len,
             const uint8_t* msg, size_t msg_len,
             uint8_t* out_mac, size_t* out_mac_len){
  const EVP_MD* md = md_from_name(algo);
  if (!md) { set_err("unsupported hash for hmac"); return -1; }
  
  unsigned int outl = 0;
  unsigned char* result = HMAC(md, key, (int)key_len, msg, msg_len, out_mac, &outl);
  if (!result) { set_err_from_queue(); return -1; }
  if (out_mac_len) *out_mac_len = outl;
  return 0;
}

// ============================================================================
// PBKDF2
// ============================================================================

int vex_pbkdf2(const char* algo,
               const uint8_t* password, size_t pw_len,
               const uint8_t* salt, size_t salt_len,
               int iterations,
               uint8_t* out_key, size_t key_len){
  const EVP_MD* md = md_from_name(algo);
  if (!md) { set_err("unsupported hash for pbkdf2"); return -1; }
  
  int rc = PKCS5_PBKDF2_HMAC((const char*)password, (int)pw_len,
                             salt, (int)salt_len, iterations,
                             md, (int)key_len, out_key);
  if (rc != 1) { set_err_from_queue(); return -1; }
  return 0;
}

// ============================================================================
// RANDOM NUMBER GENERATION
// ============================================================================

int vex_random_bytes(uint8_t* buf, size_t len){
  if (RAND_bytes(buf, (int)len) != 1) {
    set_err_from_queue();
    return -1;
  }
  return 0;
}

// ============================================================================
// RSA OPERATIONS
// ============================================================================

int vex_rsa_generate_keypair(int bits,
                             uint8_t** out_public_der, size_t* pub_len,
                             uint8_t** out_private_der, size_t* priv_len){
  EVP_PKEY* pkey = NULL;
  EVP_PKEY_CTX* ctx = EVP_PKEY_CTX_new_id(EVP_PKEY_RSA, NULL);
  if (!ctx) { set_err_from_queue(); return -1; }
  
  if (EVP_PKEY_keygen_init(ctx) <= 0) { set_err_from_queue(); EVP_PKEY_CTX_free(ctx); return -1; }
  if (EVP_PKEY_CTX_set_rsa_keygen_bits(ctx, bits) <= 0) { set_err_from_queue(); EVP_PKEY_CTX_free(ctx); return -1; }
  if (EVP_PKEY_keygen(ctx, &pkey) <= 0) { set_err_from_queue(); EVP_PKEY_CTX_free(ctx); return -1; }
  
  EVP_PKEY_CTX_free(ctx);
  
  // Export public key to DER
  int pub_der_len = i2d_PUBKEY(pkey, NULL);
  if (pub_der_len <= 0) { set_err_from_queue(); EVP_PKEY_free(pkey); return -1; }
  *out_public_der = (uint8_t*)malloc((size_t)pub_der_len);
  if (!*out_public_der) { set_err("oom"); EVP_PKEY_free(pkey); return -1; }
  uint8_t* p = *out_public_der;
  i2d_PUBKEY(pkey, &p);
  *pub_len = (size_t)pub_der_len;
  
  // Export private key to DER
  int priv_der_len = i2d_PrivateKey(pkey, NULL);
  if (priv_der_len <= 0) { set_err_from_queue(); free(*out_public_der); EVP_PKEY_free(pkey); return -1; }
  *out_private_der = (uint8_t*)malloc((size_t)priv_der_len);
  if (!*out_private_der) { set_err("oom"); free(*out_public_der); EVP_PKEY_free(pkey); return -1; }
  p = *out_private_der;
  i2d_PrivateKey(pkey, &p);
  *priv_len = (size_t)priv_der_len;
  
  EVP_PKEY_free(pkey);
  return 0;
}

int vex_rsa_sign(const char* hash_algo,
                 const uint8_t* msg, size_t msg_len,
                 const uint8_t* private_key_der, size_t priv_len,
                 uint8_t* out_sig, size_t* sig_len){
  const EVP_MD* md = md_from_name(hash_algo);
  if (!md) { set_err("unsupported hash"); return -1; }
  
  const uint8_t* p = private_key_der;
  EVP_PKEY* pkey = d2i_PrivateKey(EVP_PKEY_RSA, NULL, &p, (long)priv_len);
  if (!pkey) { set_err_from_queue(); return -1; }
  
  EVP_MD_CTX* ctx = EVP_MD_CTX_new();
  if (!ctx) { set_err_from_queue(); EVP_PKEY_free(pkey); return -1; }
  
  int ok = -1;
  size_t slen = *sig_len;
  if (EVP_DigestSignInit(ctx, NULL, md, NULL, pkey) != 1) { set_err_from_queue(); goto done; }
  if (EVP_DigestSign(ctx, out_sig, &slen, msg, msg_len) != 1) { set_err_from_queue(); goto done; }
  *sig_len = slen;
  ok = 0;
done:
  EVP_MD_CTX_free(ctx);
  EVP_PKEY_free(pkey);
  return ok;
}

int vex_rsa_verify(const char* hash_algo,
                   const uint8_t* msg, size_t msg_len,
                   const uint8_t* sig, size_t sig_len,
                   const uint8_t* public_key_der, size_t pub_len){
  const EVP_MD* md = md_from_name(hash_algo);
  if (!md) { set_err("unsupported hash"); return -1; }
  
  const uint8_t* p = public_key_der;
  EVP_PKEY* pkey = d2i_PUBKEY(NULL, &p, (long)pub_len);
  if (!pkey) { set_err_from_queue(); return -1; }
  
  EVP_MD_CTX* ctx = EVP_MD_CTX_new();
  if (!ctx) { set_err_from_queue(); EVP_PKEY_free(pkey); return -1; }
  
  int ok = -1;
  if (EVP_DigestVerifyInit(ctx, NULL, md, NULL, pkey) != 1) { set_err_from_queue(); goto done; }
  if (EVP_DigestVerify(ctx, sig, sig_len, msg, msg_len) != 1) { set_err("signature verification failed"); goto done; }
  ok = 0;
done:
  EVP_MD_CTX_free(ctx);
  EVP_PKEY_free(pkey);
  return ok;
}

int vex_rsa_encrypt(const uint8_t* plaintext, size_t pt_len,
                    const uint8_t* public_key_der, size_t pub_len,
                    uint8_t* out_cipher, size_t* ct_len){
  const uint8_t* p = public_key_der;
  EVP_PKEY* pkey = d2i_PUBKEY(NULL, &p, (long)pub_len);
  if (!pkey) { set_err_from_queue(); return -1; }
  
  EVP_PKEY_CTX* ctx = EVP_PKEY_CTX_new(pkey, NULL);
  if (!ctx) { set_err_from_queue(); EVP_PKEY_free(pkey); return -1; }
  
  int ok = -1;
  if (EVP_PKEY_encrypt_init(ctx) <= 0) { set_err_from_queue(); goto done; }
  if (EVP_PKEY_CTX_set_rsa_padding(ctx, RSA_PKCS1_OAEP_PADDING) <= 0) { set_err_from_queue(); goto done; }
  
  size_t outlen = *ct_len;
  if (EVP_PKEY_encrypt(ctx, out_cipher, &outlen, plaintext, pt_len) <= 0) { set_err_from_queue(); goto done; }
  *ct_len = outlen;
  ok = 0;
done:
  EVP_PKEY_CTX_free(ctx);
  EVP_PKEY_free(pkey);
  return ok;
}

int vex_rsa_decrypt(const uint8_t* ciphertext, size_t ct_len,
                    const uint8_t* private_key_der, size_t priv_len,
                    uint8_t* out_plain, size_t* pt_len){
  const uint8_t* p = private_key_der;
  EVP_PKEY* pkey = d2i_PrivateKey(EVP_PKEY_RSA, NULL, &p, (long)priv_len);
  if (!pkey) { set_err_from_queue(); return -1; }
  
  EVP_PKEY_CTX* ctx = EVP_PKEY_CTX_new(pkey, NULL);
  if (!ctx) { set_err_from_queue(); EVP_PKEY_free(pkey); return -1; }
  
  int ok = -1;
  if (EVP_PKEY_decrypt_init(ctx) <= 0) { set_err_from_queue(); goto done; }
  if (EVP_PKEY_CTX_set_rsa_padding(ctx, RSA_PKCS1_OAEP_PADDING) <= 0) { set_err_from_queue(); goto done; }
  
  size_t outlen = *pt_len;
  if (EVP_PKEY_decrypt(ctx, out_plain, &outlen, ciphertext, ct_len) <= 0) { set_err_from_queue(); goto done; }
  *pt_len = outlen;
  ok = 0;
done:
  EVP_PKEY_CTX_free(ctx);
  EVP_PKEY_free(pkey);
  return ok;
}

// ============================================================================
// ECDSA OPERATIONS
// ============================================================================

static int nid_from_curve(const char* curve){
  if (!curve) return -1;
  if (strcmp(curve, "P-256") == 0 || strcmp(curve, "prime256v1") == 0) return NID_X9_62_prime256v1;
  if (strcmp(curve, "P-384") == 0 || strcmp(curve, "secp384r1") == 0) return NID_secp384r1;
  if (strcmp(curve, "P-521") == 0 || strcmp(curve, "secp521r1") == 0) return NID_secp521r1;
  return -1;
}

int vex_ecdsa_generate_keypair(const char* curve,
                               uint8_t** out_public_der, size_t* pub_len,
                               uint8_t** out_private_der, size_t* priv_len){
  int nid = nid_from_curve(curve);
  if (nid < 0) { set_err("unsupported curve"); return -1; }
  
  EVP_PKEY* pkey = NULL;
  EVP_PKEY_CTX* ctx = EVP_PKEY_CTX_new_id(EVP_PKEY_EC, NULL);
  if (!ctx) { set_err_from_queue(); return -1; }
  
  if (EVP_PKEY_keygen_init(ctx) <= 0) { set_err_from_queue(); EVP_PKEY_CTX_free(ctx); return -1; }
  if (EVP_PKEY_CTX_set_ec_paramgen_curve_nid(ctx, nid) <= 0) { set_err_from_queue(); EVP_PKEY_CTX_free(ctx); return -1; }
  if (EVP_PKEY_keygen(ctx, &pkey) <= 0) { set_err_from_queue(); EVP_PKEY_CTX_free(ctx); return -1; }
  
  EVP_PKEY_CTX_free(ctx);
  
  // Export public key to DER
  int pub_der_len = i2d_PUBKEY(pkey, NULL);
  if (pub_der_len <= 0) { set_err_from_queue(); EVP_PKEY_free(pkey); return -1; }
  *out_public_der = (uint8_t*)malloc((size_t)pub_der_len);
  if (!*out_public_der) { set_err("oom"); EVP_PKEY_free(pkey); return -1; }
  uint8_t* p = *out_public_der;
  i2d_PUBKEY(pkey, &p);
  *pub_len = (size_t)pub_der_len;
  
  // Export private key to DER
  int priv_der_len = i2d_PrivateKey(pkey, NULL);
  if (priv_der_len <= 0) { set_err_from_queue(); free(*out_public_der); EVP_PKEY_free(pkey); return -1; }
  *out_private_der = (uint8_t*)malloc((size_t)priv_der_len);
  if (!*out_private_der) { set_err("oom"); free(*out_public_der); EVP_PKEY_free(pkey); return -1; }
  p = *out_private_der;
  i2d_PrivateKey(pkey, &p);
  *priv_len = (size_t)priv_der_len;
  
  EVP_PKEY_free(pkey);
  return 0;
}

int vex_ecdsa_sign(const char* curve, const char* hash_algo,
                   const uint8_t* msg, size_t msg_len,
                   const uint8_t* private_key_der, size_t priv_len,
                   uint8_t* out_sig, size_t* sig_len){
  (void)curve; // Curve info is in the key itself
  const EVP_MD* md = md_from_name(hash_algo);
  if (!md) { set_err("unsupported hash"); return -1; }
  
  const uint8_t* p = private_key_der;
  EVP_PKEY* pkey = d2i_PrivateKey(EVP_PKEY_EC, NULL, &p, (long)priv_len);
  if (!pkey) { set_err_from_queue(); return -1; }
  
  EVP_MD_CTX* ctx = EVP_MD_CTX_new();
  if (!ctx) { set_err_from_queue(); EVP_PKEY_free(pkey); return -1; }
  
  int ok = -1;
  size_t slen = *sig_len;
  if (EVP_DigestSignInit(ctx, NULL, md, NULL, pkey) != 1) { set_err_from_queue(); goto done; }
  if (EVP_DigestSign(ctx, out_sig, &slen, msg, msg_len) != 1) { set_err_from_queue(); goto done; }
  *sig_len = slen;
  ok = 0;
done:
  EVP_MD_CTX_free(ctx);
  EVP_PKEY_free(pkey);
  return ok;
}

int vex_ecdsa_verify(const char* curve, const char* hash_algo,
                     const uint8_t* msg, size_t msg_len,
                     const uint8_t* sig, size_t sig_len,
                     const uint8_t* public_key_der, size_t pub_len){
  (void)curve;
  const EVP_MD* md = md_from_name(hash_algo);
  if (!md) { set_err("unsupported hash"); return -1; }
  
  const uint8_t* p = public_key_der;
  EVP_PKEY* pkey = d2i_PUBKEY(NULL, &p, (long)pub_len);
  if (!pkey) { set_err_from_queue(); return -1; }
  
  EVP_MD_CTX* ctx = EVP_MD_CTX_new();
  if (!ctx) { set_err_from_queue(); EVP_PKEY_free(pkey); return -1; }
  
  int ok = -1;
  if (EVP_DigestVerifyInit(ctx, NULL, md, NULL, pkey) != 1) { set_err_from_queue(); goto done; }
  if (EVP_DigestVerify(ctx, sig, sig_len, msg, msg_len) != 1) { set_err("signature verification failed"); goto done; }
  ok = 0;
done:
  EVP_MD_CTX_free(ctx);
  EVP_PKEY_free(pkey);
  return ok;
}

// ============================================================================
// ECDH
// ============================================================================

int vex_ecdh(const char* curve,
             const uint8_t* private_key_der, size_t priv_len,
             const uint8_t* peer_public_der, size_t peer_len,
             uint8_t* out_shared, size_t* shared_len){
  (void)curve;
  
  const uint8_t* p = private_key_der;
  EVP_PKEY* priv_key = d2i_PrivateKey(EVP_PKEY_EC, NULL, &p, (long)priv_len);
  if (!priv_key) { set_err_from_queue(); return -1; }
  
  p = peer_public_der;
  EVP_PKEY* peer_key = d2i_PUBKEY(NULL, &p, (long)peer_len);
  if (!peer_key) { set_err_from_queue(); EVP_PKEY_free(priv_key); return -1; }
  
  EVP_PKEY_CTX* ctx = EVP_PKEY_CTX_new(priv_key, NULL);
  if (!ctx) { set_err_from_queue(); EVP_PKEY_free(priv_key); EVP_PKEY_free(peer_key); return -1; }
  
  int ok = -1;
  if (EVP_PKEY_derive_init(ctx) != 1) { set_err_from_queue(); goto done; }
  if (EVP_PKEY_derive_set_peer(ctx, peer_key) != 1) { set_err_from_queue(); goto done; }
  
  size_t slen = *shared_len;
  if (EVP_PKEY_derive(ctx, out_shared, &slen) != 1) { set_err_from_queue(); goto done; }
  *shared_len = slen;
  ok = 0;
done:
  EVP_PKEY_CTX_free(ctx);
  EVP_PKEY_free(priv_key);
  EVP_PKEY_free(peer_key);
  return ok;
}

// ============================================================================
// Certificate Operations
// ============================================================================

int vex_x509_parse(const uint8_t* cert_der, size_t cert_len, VexX509Info* info) {
  if (!cert_der || !info) { set_err("NULL args"); return -1; }
  
  const uint8_t* p = cert_der;
  X509* cert = d2i_X509(NULL, &p, cert_len);
  if (!cert) { set_err_from_queue(); return -1; }
  
  // Extract subject
  X509_NAME* subject = X509_get_subject_name(cert);
  if (subject) {
    X509_NAME_oneline(subject, info->subject, sizeof(info->subject));
  }
  
  // Extract issuer
  X509_NAME* issuer = X509_get_issuer_name(cert);
  if (issuer) {
    X509_NAME_oneline(issuer, info->issuer, sizeof(info->issuer));
  }
  
  // Extract serial number
  ASN1_INTEGER* serial = X509_get_serialNumber(cert);
  if (serial) {
    BIGNUM* bn = ASN1_INTEGER_to_BN(serial, NULL);
    char* hex = BN_bn2hex(bn);
    snprintf(info->serial, sizeof(info->serial), "%s", hex ? hex : "");
    OPENSSL_free(hex);
    BN_free(bn);
  }
  
  // Extract validity dates
  const ASN1_TIME* not_before = X509_get0_notBefore(cert);
  const ASN1_TIME* not_after = X509_get0_notAfter(cert);
  
  struct tm tm_before = {0}, tm_after = {0};
  if (not_before) {
    ASN1_TIME_to_tm(not_before, &tm_before);
    info->not_before = mktime(&tm_before);
  }
  if (not_after) {
    ASN1_TIME_to_tm(not_after, &tm_after);
    info->not_after = mktime(&tm_after);
  }
  
  // Check if CA
  info->is_ca = X509_check_ca(cert);
  
  // Extract key usage
  info->key_usage = X509_get_key_usage(cert);
  
  X509_free(cert);
  return 0;
}

int vex_x509_verify_chain(const uint8_t* cert_der, size_t cert_len,
                          const uint8_t* ca_certs_pem, size_t ca_len) {
  if (!cert_der || !ca_certs_pem) { set_err("NULL args"); return -1; }
  
  const uint8_t* p = cert_der;
  X509* cert = d2i_X509(NULL, &p, cert_len);
  if (!cert) { set_err_from_queue(); return -1; }
  
  // Create certificate store
  X509_STORE* store = X509_STORE_new();
  if (!store) { set_err_from_queue(); X509_free(cert); return -1; }
  
  // Load CA certificates from PEM
  BIO* bio = BIO_new_mem_buf(ca_certs_pem, ca_len);
  if (!bio) { set_err_from_queue(); X509_STORE_free(store); X509_free(cert); return -1; }
  
  X509* ca_cert = NULL;
  while ((ca_cert = PEM_read_bio_X509(bio, NULL, NULL, NULL)) != NULL) {
    X509_STORE_add_cert(store, ca_cert);
    X509_free(ca_cert);
  }
  BIO_free(bio);
  
  // Verify certificate
  X509_STORE_CTX* ctx = X509_STORE_CTX_new();
  if (!ctx) { set_err_from_queue(); X509_STORE_free(store); X509_free(cert); return -1; }
  
  if (X509_STORE_CTX_init(ctx, store, cert, NULL) != 1) {
    set_err_from_queue();
    X509_STORE_CTX_free(ctx);
    X509_STORE_free(store);
    X509_free(cert);
    return -1;
  }
  
  int result = X509_verify_cert(ctx);
  
  X509_STORE_CTX_free(ctx);
  X509_STORE_free(store);
  X509_free(cert);
  
  if (result != 1) {
    set_err("certificate verification failed");
    return -1;
  }
  
  return 0;
}

int vex_x509_generate_self_signed(const char* subject, int days_valid,
                                   uint8_t** out_cert_pem, size_t* cert_len,
                                   uint8_t** out_key_pem, size_t* key_len) {
  if (!subject || !out_cert_pem || !out_key_pem) { set_err("NULL args"); return -1; }
  
  // Generate RSA key pair
  EVP_PKEY* pkey = EVP_PKEY_new();
  EVP_PKEY_CTX* pctx = EVP_PKEY_CTX_new_id(EVP_PKEY_RSA, NULL);
  if (!pctx) { set_err_from_queue(); return -1; }
  
  if (EVP_PKEY_keygen_init(pctx) != 1 ||
      EVP_PKEY_CTX_set_rsa_keygen_bits(pctx, 2048) != 1 ||
      EVP_PKEY_keygen(pctx, &pkey) != 1) {
    set_err_from_queue();
    EVP_PKEY_CTX_free(pctx);
    return -1;
  }
  EVP_PKEY_CTX_free(pctx);
  
  // Create certificate
  X509* cert = X509_new();
  if (!cert) { set_err_from_queue(); EVP_PKEY_free(pkey); return -1; }
  
  // Set version (X509v3)
  X509_set_version(cert, 2);
  
  // Set serial number
  ASN1_INTEGER_set(X509_get_serialNumber(cert), 1);
  
  // Set validity period
  X509_gmtime_adj(X509_get_notBefore(cert), 0);
  X509_gmtime_adj(X509_get_notAfter(cert), (long)60 * 60 * 24 * days_valid);
  
  // Set public key
  X509_set_pubkey(cert, pkey);
  
  // Set subject and issuer (same for self-signed)
  X509_NAME* name = X509_get_subject_name(cert);
  X509_NAME_add_entry_by_txt(name, "CN", MBSTRING_ASC, 
                             (const unsigned char*)subject, -1, -1, 0);
  X509_set_issuer_name(cert, name);
  
  // Sign certificate
  if (X509_sign(cert, pkey, EVP_sha256()) == 0) {
    set_err_from_queue();
    X509_free(cert);
    EVP_PKEY_free(pkey);
    return -1;
  }
  
  // Convert certificate to PEM
  BIO* cert_bio = BIO_new(BIO_s_mem());
  if (!cert_bio || PEM_write_bio_X509(cert_bio, cert) != 1) {
    set_err_from_queue();
    BIO_free(cert_bio);
    X509_free(cert);
    EVP_PKEY_free(pkey);
    return -1;
  }
  
  BUF_MEM* cert_mem;
  BIO_get_mem_ptr(cert_bio, &cert_mem);
  *out_cert_pem = malloc(cert_mem->length);
  memcpy(*out_cert_pem, cert_mem->data, cert_mem->length);
  *cert_len = cert_mem->length;
  BIO_free(cert_bio);
  
  // Convert private key to PEM
  BIO* key_bio = BIO_new(BIO_s_mem());
  if (!key_bio || PEM_write_bio_PrivateKey(key_bio, pkey, NULL, NULL, 0, NULL, NULL) != 1) {
    set_err_from_queue();
    BIO_free(key_bio);
    free(*out_cert_pem);
    X509_free(cert);
    EVP_PKEY_free(pkey);
    return -1;
  }
  
  BUF_MEM* key_mem;
  BIO_get_mem_ptr(key_bio, &key_mem);
  *out_key_pem = malloc(key_mem->length);
  memcpy(*out_key_pem, key_mem->data, key_mem->length);
  *key_len = key_mem->length;
  BIO_free(key_bio);
  
  X509_free(cert);
  EVP_PKEY_free(pkey);
  return 0;
}

int vex_x509_generate_csr(const char* subject,
                          const uint8_t* private_key_pem, size_t key_len,
                          uint8_t** out_csr_pem, size_t* csr_len) {
  if (!subject || !private_key_pem || !out_csr_pem) { set_err("NULL args"); return -1; }
  
  // Load private key
  BIO* key_bio = BIO_new_mem_buf(private_key_pem, key_len);
  if (!key_bio) { set_err_from_queue(); return -1; }
  
  EVP_PKEY* pkey = PEM_read_bio_PrivateKey(key_bio, NULL, NULL, NULL);
  BIO_free(key_bio);
  if (!pkey) { set_err_from_queue(); return -1; }
  
  // Create CSR
  X509_REQ* req = X509_REQ_new();
  if (!req) { set_err_from_queue(); EVP_PKEY_free(pkey); return -1; }
  
  // Set version
  X509_REQ_set_version(req, 0);
  
  // Set subject
  X509_NAME* name = X509_REQ_get_subject_name(req);
  X509_NAME_add_entry_by_txt(name, "CN", MBSTRING_ASC,
                             (const unsigned char*)subject, -1, -1, 0);
  
  // Set public key
  X509_REQ_set_pubkey(req, pkey);
  
  // Sign CSR
  if (X509_REQ_sign(req, pkey, EVP_sha256()) == 0) {
    set_err_from_queue();
    X509_REQ_free(req);
    EVP_PKEY_free(pkey);
    return -1;
  }
  
  // Convert to PEM
  BIO* csr_bio = BIO_new(BIO_s_mem());
  if (!csr_bio || PEM_write_bio_X509_REQ(csr_bio, req) != 1) {
    set_err_from_queue();
    BIO_free(csr_bio);
    X509_REQ_free(req);
    EVP_PKEY_free(pkey);
    return -1;
  }
  
  BUF_MEM* csr_mem;
  BIO_get_mem_ptr(csr_bio, &csr_mem);
  *out_csr_pem = malloc(csr_mem->length);
  memcpy(*out_csr_pem, csr_mem->data, csr_mem->length);
  *csr_len = csr_mem->length;
  BIO_free(csr_bio);
  
  X509_REQ_free(req);
  EVP_PKEY_free(pkey);
  return 0;
}

// ============================================================================
// PEM/DER Conversion Helpers
// ============================================================================

int vex_pem_to_der(const uint8_t* pem, size_t pem_len,
                   uint8_t** out_der, size_t* der_len) {
  if (!pem || !out_der) { set_err("NULL args"); return -1; }
  
  BIO* bio = BIO_new_mem_buf(pem, pem_len);
  if (!bio) { set_err_from_queue(); return -1; }
  
  // Try to read as certificate first
  X509* cert = PEM_read_bio_X509(bio, NULL, NULL, NULL);
  if (cert) {
    int len = i2d_X509(cert, out_der);
    if (len < 0) { set_err_from_queue(); X509_free(cert); BIO_free(bio); return -1; }
    *der_len = len;
    X509_free(cert);
    BIO_free(bio);
    return 0;
  }
  
  // Try to read as private key
  BIO_reset(bio);
  EVP_PKEY* pkey = PEM_read_bio_PrivateKey(bio, NULL, NULL, NULL);
  if (pkey) {
    int len = i2d_PrivateKey(pkey, out_der);
    if (len < 0) { set_err_from_queue(); EVP_PKEY_free(pkey); BIO_free(bio); return -1; }
    *der_len = len;
    EVP_PKEY_free(pkey);
    BIO_free(bio);
    return 0;
  }
  
  BIO_free(bio);
  set_err("not a valid PEM certificate or key");
  return -1;
}

int vex_der_to_pem(const char* label, const uint8_t* der, size_t der_len,
                   uint8_t** out_pem, size_t* pem_len) {
  if (!label || !der || !out_pem) { set_err("NULL args"); return -1; }
  
  BIO* bio = BIO_new(BIO_s_mem());
  if (!bio) { set_err_from_queue(); return -1; }
  
  // Write PEM header
  BIO_printf(bio, "-----BEGIN %s-----\n", label);
  
  // Encode DER to base64
  BIO* b64 = BIO_new(BIO_f_base64());
  bio = BIO_push(b64, bio);
  BIO_write(bio, der, der_len);
  BIO_flush(bio);
  
  // Write PEM footer
  BIO* mem_bio = BIO_pop(b64);
  BIO_free(b64);
  BIO_printf(mem_bio, "-----END %s-----\n", label);
  
  BUF_MEM* mem;
  BIO_get_mem_ptr(mem_bio, &mem);
  *out_pem = malloc(mem->length);
  memcpy(*out_pem, mem->data, mem->length);
  *pem_len = mem->length;
  BIO_free(mem_bio);
  
  return 0;
}

void vex_crypto_free(void* ptr) {
  free(ptr);
}

// ============================================================================
// Additional Hash Functions
// ============================================================================

int vex_md5(const uint8_t* msg, size_t len, uint8_t* out_digest, size_t* out_len) {
  if (!msg || !out_digest || !out_len) { set_err("NULL args"); return -1; }
  if (*out_len < MD5_DIGEST_LENGTH) { set_err("buffer too small"); return -1; }
  
  MD5_CTX ctx;
  MD5_Init(&ctx);
  MD5_Update(&ctx, msg, len);
  MD5_Final(out_digest, &ctx);
  *out_len = MD5_DIGEST_LENGTH;
  
  return 0;
}

int vex_sha1(const uint8_t* msg, size_t len, uint8_t* out_digest, size_t* out_len) {
  if (!msg || !out_digest || !out_len) { set_err("NULL args"); return -1; }
  if (*out_len < SHA_DIGEST_LENGTH) { set_err("buffer too small"); return -1; }
  
  SHA_CTX ctx;
  SHA1_Init(&ctx);
  SHA1_Update(&ctx, msg, len);
  SHA1_Final(out_digest, &ctx);
  *out_len = SHA_DIGEST_LENGTH;
  
  return 0;
}

int vex_sha384(const uint8_t* msg, size_t len, uint8_t* out_digest, size_t* out_len) {
  if (!msg || !out_digest || !out_len) { set_err("NULL args"); return -1; }
  if (*out_len < SHA384_DIGEST_LENGTH) { set_err("buffer too small"); return -1; }
  
  SHA512_CTX ctx;
  SHA384_Init(&ctx);
  SHA384_Update(&ctx, msg, len);
  SHA384_Final(out_digest, &ctx);
  *out_len = SHA384_DIGEST_LENGTH;
  
  return 0;
}

// ============================================================================
// Constant-Time Operations
// ============================================================================

int vex_constant_time_compare(const uint8_t* a, const uint8_t* b, size_t len) {
  if (!a || !b) return -1;
  return CRYPTO_memcmp(a, b, len);
}

void vex_constant_time_select(uint8_t* out, const uint8_t* a, const uint8_t* b,
                               size_t len, int select) {
  if (!out || !a || !b) return;
  
  // Constant-time select: if select is 1, copy a; if 0, copy b
  unsigned char mask = (unsigned char)(-(select & 1));
  for (size_t i = 0; i < len; i++) {
    out[i] = (a[i] & mask) | (b[i] & ~mask);
  }
}

// ============================================================================
// AES Key Wrap (RFC 3394)
// ============================================================================

int vex_aes_key_wrap(const uint8_t* kek, size_t kek_len,
                     const uint8_t* plaintext_key, size_t pt_len,
                     uint8_t* out_wrapped, size_t* out_len) {
  if (!kek || !plaintext_key || !out_wrapped || !out_len) { set_err("NULL args"); return -1; }
  
  // Input key must be multiple of 8 bytes
  if (pt_len % 8 != 0) { set_err("plaintext key must be multiple of 8 bytes"); return -1; }
  
  EVP_CIPHER_CTX* ctx = EVP_CIPHER_CTX_new();
  if (!ctx) { set_err_from_queue(); return -1; }
  
  const EVP_CIPHER* cipher = NULL;
  if (kek_len == 16) cipher = EVP_aes_128_wrap();
  else if (kek_len == 24) cipher = EVP_aes_192_wrap();
  else if (kek_len == 32) cipher = EVP_aes_256_wrap();
  else { set_err("invalid KEK size"); EVP_CIPHER_CTX_free(ctx); return -1; }
  
  if (EVP_EncryptInit_ex(ctx, cipher, NULL, kek, NULL) != 1) {
    set_err_from_queue();
    EVP_CIPHER_CTX_free(ctx);
    return -1;
  }
  
  int len = 0;
  if (EVP_EncryptUpdate(ctx, out_wrapped, &len, plaintext_key, pt_len) != 1) {
    set_err_from_queue();
    EVP_CIPHER_CTX_free(ctx);
    return -1;
  }
  
  int final_len = 0;
  if (EVP_EncryptFinal_ex(ctx, out_wrapped + len, &final_len) != 1) {
    set_err_from_queue();
    EVP_CIPHER_CTX_free(ctx);
    return -1;
  }
  
  *out_len = len + final_len;
  EVP_CIPHER_CTX_free(ctx);
  return 0;
}

int vex_aes_key_unwrap(const uint8_t* kek, size_t kek_len,
                       const uint8_t* wrapped_key, size_t wrapped_len,
                       uint8_t* out_plaintext, size_t* out_len) {
  if (!kek || !wrapped_key || !out_plaintext || !out_len) { set_err("NULL args"); return -1; }
  
  EVP_CIPHER_CTX* ctx = EVP_CIPHER_CTX_new();
  if (!ctx) { set_err_from_queue(); return -1; }
  
  const EVP_CIPHER* cipher = NULL;
  if (kek_len == 16) cipher = EVP_aes_128_wrap();
  else if (kek_len == 24) cipher = EVP_aes_192_wrap();
  else if (kek_len == 32) cipher = EVP_aes_256_wrap();
  else { set_err("invalid KEK size"); EVP_CIPHER_CTX_free(ctx); return -1; }
  
  if (EVP_DecryptInit_ex(ctx, cipher, NULL, kek, NULL) != 1) {
    set_err_from_queue();
    EVP_CIPHER_CTX_free(ctx);
    return -1;
  }
  
  int len = 0;
  if (EVP_DecryptUpdate(ctx, out_plaintext, &len, wrapped_key, wrapped_len) != 1) {
    set_err_from_queue();
    EVP_CIPHER_CTX_free(ctx);
    return -1;
  }
  
  int final_len = 0;
  if (EVP_DecryptFinal_ex(ctx, out_plaintext + len, &final_len) != 1) {
    set_err_from_queue();
    EVP_CIPHER_CTX_free(ctx);
    return -1;
  }
  
  *out_len = len + final_len;
  EVP_CIPHER_CTX_free(ctx);
  return 0;
}
