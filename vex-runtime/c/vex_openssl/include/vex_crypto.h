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

// Symmetric encryption (non-AEAD modes: CBC, CTR)
int vex_cipher_encrypt(const char* cipher_name,
                       const uint8_t* key, size_t key_len,
                       const uint8_t* iv, size_t iv_len,
                       const uint8_t* plaintext, size_t pt_len,
                       uint8_t* out_cipher, size_t* out_ct_len);

int vex_cipher_decrypt(const char* cipher_name,
                       const uint8_t* key, size_t key_len,
                       const uint8_t* iv, size_t iv_len,
                       const uint8_t* ciphertext, size_t ct_len,
                       uint8_t* out_plain, size_t* out_pt_len);

// HMAC (Message Authentication Code)
int vex_hmac(const char* algo,
             const uint8_t* key, size_t key_len,
             const uint8_t* msg, size_t msg_len,
             uint8_t* out_mac, size_t* out_mac_len);

// PBKDF2 (Password-Based Key Derivation)
int vex_pbkdf2(const char* algo,
               const uint8_t* password, size_t pw_len,
               const uint8_t* salt, size_t salt_len,
               int iterations,
               uint8_t* out_key, size_t key_len);

// Random number generation
int vex_random_bytes(uint8_t* buf, size_t len);

// RSA operations
int vex_rsa_generate_keypair(int bits, 
                             uint8_t** out_public_der, size_t* pub_len,
                             uint8_t** out_private_der, size_t* priv_len);

int vex_rsa_sign(const char* hash_algo,
                 const uint8_t* msg, size_t msg_len,
                 const uint8_t* private_key_der, size_t priv_len,
                 uint8_t* out_sig, size_t* sig_len);

int vex_rsa_verify(const char* hash_algo,
                   const uint8_t* msg, size_t msg_len,
                   const uint8_t* sig, size_t sig_len,
                   const uint8_t* public_key_der, size_t pub_len);

int vex_rsa_encrypt(const uint8_t* plaintext, size_t pt_len,
                    const uint8_t* public_key_der, size_t pub_len,
                    uint8_t* out_cipher, size_t* ct_len);

int vex_rsa_decrypt(const uint8_t* ciphertext, size_t ct_len,
                    const uint8_t* private_key_der, size_t priv_len,
                    uint8_t* out_plain, size_t* pt_len);

// ECDSA (P-256, P-384 curves)
int vex_ecdsa_generate_keypair(const char* curve,
                               uint8_t** out_public_der, size_t* pub_len,
                               uint8_t** out_private_der, size_t* priv_len);

int vex_ecdsa_sign(const char* curve, const char* hash_algo,
                   const uint8_t* msg, size_t msg_len,
                   const uint8_t* private_key_der, size_t priv_len,
                   uint8_t* out_sig, size_t* sig_len);

int vex_ecdsa_verify(const char* curve, const char* hash_algo,
                     const uint8_t* msg, size_t msg_len,
                     const uint8_t* sig, size_t sig_len,
                     const uint8_t* public_key_der, size_t pub_len);

// ECDH (P-256, P-384 curves)
int vex_ecdh(const char* curve,
             const uint8_t* private_key_der, size_t priv_len,
             const uint8_t* peer_public_der, size_t peer_len,
             uint8_t* out_shared, size_t* shared_len);

// ============================================================================
// Certificate Operations
// ============================================================================

// Parse X.509 certificate and extract common fields
typedef struct {
    char subject[256];
    char issuer[256];
    char serial[64];
    int64_t not_before;  // Unix timestamp
    int64_t not_after;   // Unix timestamp
    int is_ca;
    int key_usage;
} VexX509Info;

int vex_x509_parse(const uint8_t* cert_der, size_t cert_len, VexX509Info* info);

// Verify certificate chain
int vex_x509_verify_chain(const uint8_t* cert_der, size_t cert_len,
                          const uint8_t* ca_certs_pem, size_t ca_len);

// Generate self-signed certificate
int vex_x509_generate_self_signed(const char* subject,
                                   int days_valid,
                                   uint8_t** out_cert_pem, size_t* cert_len,
                                   uint8_t** out_key_pem, size_t* key_len);

// Generate Certificate Signing Request (CSR)
int vex_x509_generate_csr(const char* subject,
                          const uint8_t* private_key_pem, size_t key_len,
                          uint8_t** out_csr_pem, size_t* csr_len);

// ============================================================================
// PEM/DER Conversion Helpers
// ============================================================================

// Convert PEM to DER format
int vex_pem_to_der(const uint8_t* pem, size_t pem_len,
                   uint8_t** out_der, size_t* der_len);

// Convert DER to PEM format
int vex_der_to_pem(const char* label,  // e.g., "CERTIFICATE", "PRIVATE KEY"
                   const uint8_t* der, size_t der_len,
                   uint8_t** out_pem, size_t* pem_len);

// Free memory allocated by PEM/DER conversion functions
void vex_crypto_free(void* ptr);

// ============================================================================
// Additional Hash Functions
// ============================================================================

// MD5 (legacy, for compatibility)
int vex_md5(const uint8_t* msg, size_t len, uint8_t* out_digest, size_t* out_len);

// SHA-1 (legacy, for compatibility)
int vex_sha1(const uint8_t* msg, size_t len, uint8_t* out_digest, size_t* out_len);

// SHA-384
int vex_sha384(const uint8_t* msg, size_t len, uint8_t* out_digest, size_t* out_len);

// ============================================================================
// Constant-Time Operations (Timing Attack Prevention)
// ============================================================================

// Constant-time memory comparison
int vex_constant_time_compare(const uint8_t* a, const uint8_t* b, size_t len);

// Constant-time memory select (select = 1 -> return a, select = 0 -> return b)
void vex_constant_time_select(uint8_t* out, const uint8_t* a, const uint8_t* b, 
                               size_t len, int select);

// ============================================================================
// AES Key Wrap (RFC 3394)
// ============================================================================

// AES Key Wrap - wrap a key with a key-encryption-key (KEK)
int vex_aes_key_wrap(const uint8_t* kek, size_t kek_len,
                     const uint8_t* plaintext_key, size_t pt_len,
                     uint8_t* out_wrapped, size_t* out_len);

// AES Key Unwrap - unwrap a key with a key-encryption-key (KEK)
int vex_aes_key_unwrap(const uint8_t* kek, size_t kek_len,
                       const uint8_t* wrapped_key, size_t wrapped_len,
                       uint8_t* out_plaintext, size_t* out_len);

#ifdef __cplusplus
}
#endif
