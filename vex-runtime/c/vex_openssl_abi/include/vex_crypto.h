#pragma once
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

const char* vex_crypto_last_error(void);

int vex_aead_seal(const char* aead_name,
                  const uint8_t* key, size_t key_len,
                  const uint8_t* nonce, size_t nonce_len,
                  const uint8_t* ad, size_t ad_len,
                  const uint8_t* plaintext, size_t pt_len,
                  uint8_t* out_cipher, size_t* inout_ct_len,
                  size_t tag_len);

int vex_aead_open(const char* aead_name,
                  const uint8_t* key, size_t key_len,
                  const uint8_t* nonce, size_t nonce_len,
                  const uint8_t* ad, size_t ad_len,
                  const uint8_t* ciphertext, size_t ct_len,
                  uint8_t* out_plain, size_t* inout_pt_len,
                  size_t tag_len);

int vex_hash(const char* algo,
             const uint8_t* msg, size_t len,
             uint8_t* out_digest, size_t* inout_len);

int vex_hkdf(const char* algo,
             const uint8_t* ikm, size_t ikm_len,
             const uint8_t* salt, size_t salt_len,
             const uint8_t* info, size_t info_len,
             uint8_t* out_okm, size_t okm_len);

int vex_x25519_public_from_private(uint8_t pub[32], const uint8_t priv[32]);
int vex_x25519(uint8_t shared[32], const uint8_t priv[32], const uint8_t peer_pub[32]);

int vex_ed25519_sign(uint8_t sig[64], const uint8_t* msg, size_t len, const uint8_t sk[64]);
int vex_ed25519_verify(const uint8_t sig[64], const uint8_t* msg, size_t len, const uint8_t pk[32]);

#ifdef __cplusplus
}
#endif
