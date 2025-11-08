# Vex OpenSSL ABI Wrapper

Zero-copy, high-performance OpenSSL wrapper for Vex standard library crypto operations.

## Overview

This library provides a clean C API wrapping OpenSSL 1.1.0+ functionality for use in Vex's standard library. It covers modern cryptographic primitives with a focus on safety, performance, and ease of use.

## Features

### ✅ Implemented

#### AEAD (Authenticated Encryption)

- **AES-GCM**: AES-128-GCM, AES-256-GCM
- **ChaCha20-Poly1305**: Modern, fast authenticated encryption
- Tag-based authentication (16-byte tags)

#### Symmetric Encryption

- **AES-CBC**: AES-128-CBC, AES-256-CBC
- **AES-CTR**: AES-128-CTR, AES-256-CTR
- PKCS#7 padding for CBC mode

#### Hash Functions

- **SHA-2**: SHA-256, SHA-384, SHA-512
- **SHA-3**: SHA3-256, SHA3-512 (OpenSSL 3.x)
- **Legacy**: MD5, SHA-1 (for compatibility with older systems)

#### Message Authentication

- **HMAC**: HMAC-SHA256, HMAC-SHA512
- Variable-length keys

#### Key Derivation

- **HKDF**: HKDF-SHA256, HKDF-SHA512
- **PBKDF2**: Password-based KDF with configurable iterations

#### Elliptic Curve (Modern)

- **X25519**: ECDH key agreement
- **Ed25519**: EdDSA signatures (64-byte signatures)

#### Elliptic Curve (NIST)

- **ECDSA**: P-256, P-384, P-521 signing/verification
- **ECDH**: P-256, P-384, P-521 key agreement
- DER-encoded keys

#### RSA

- **Key Generation**: 2048, 3072, 4096 bits
- **Signing**: RSA-PSS with SHA-256/SHA-512
- **Encryption**: RSA-OAEP padding
- DER-encoded keys

#### Random Number Generation

- **CSPRNG**: Cryptographically secure random bytes via OpenSSL's RAND_bytes

#### TLS/SSL

- **Context**: Client/server TLS contexts
- **Handshake**: TLS 1.2+ handshake
- **I/O**: Non-blocking read/write
- **ALPN**: Application-Layer Protocol Negotiation
- **Certificate**: X.509 certificate loading and verification

#### X.509 Certificates

- **Parsing**: Extract subject, issuer, serial number, validity dates
- **Chain Verification**: Verify certificate chains against CA certificates
- **Generation**: Create self-signed certificates
- **CSR**: Generate Certificate Signing Requests

#### Key Management

- **PEM/DER Conversion**: Convert between PEM and DER formats
- **AES Key Wrap**: RFC 3394 key wrapping for secure key storage/transport
- **Memory Management**: Efficient key allocation and cleanup

#### Security Utilities

- **Constant-Time Operations**: Timing-attack resistant comparisons
- **Secure Memory**: Constant-time selection and comparison primitives

## Building

### Prerequisites

```bash
# macOS
brew install openssl@3

# Ubuntu/Debian
apt-get install libssl-dev

# Fedora/RHEL
dnf install openssl-devel
```

### Build

```bash
make clean
make
```

### Test

```bash
make test
```

## API Reference

### Error Handling

```c
const char* vex_crypto_last_error(void);
const char* vex_tls_last_error(void);
```

All functions return `0` on success, `-1` on error. Use error functions to get human-readable error messages.

### AEAD Encryption

```c
// Encrypt with authentication
int vex_aead_seal(const char* aead_name,              // "AES-256-GCM", "CHACHA20-POLY1305"
                  const uint8_t* key, size_t key_len,
                  const uint8_t* nonce, size_t nonce_len,
                  const uint8_t* ad, size_t ad_len,    // Additional authenticated data
                  const uint8_t* plaintext, size_t pt_len,
                  uint8_t* out_cipher, size_t* inout_ct_len,
                  size_t tag_len);                     // Usually 16

// Decrypt and verify
int vex_aead_open(const char* aead_name,
                  const uint8_t* key, size_t key_len,
                  const uint8_t* nonce, size_t nonce_len,
                  const uint8_t* ad, size_t ad_len,
                  const uint8_t* ciphertext, size_t ct_len,
                  uint8_t* out_plain, size_t* inout_pt_len,
                  size_t tag_len);
```

### Symmetric Encryption

```c
int vex_cipher_encrypt(const char* cipher_name,        // "AES-256-CBC", "AES-128-CTR"
                       const uint8_t* key, size_t key_len,
                       const uint8_t* iv, size_t iv_len,
                       const uint8_t* plaintext, size_t pt_len,
                       uint8_t* out_cipher, size_t* out_ct_len);

int vex_cipher_decrypt(const char* cipher_name,
                       const uint8_t* key, size_t key_len,
                       const uint8_t* iv, size_t iv_len,
                       const uint8_t* ciphertext, size_t ct_len,
                       uint8_t* out_plain, size_t* out_pt_len);
```

### Hashing

```c
int vex_hash(const char* algo,                        // "SHA-256", "SHA-512", "SHA3-256"
             const uint8_t* msg, size_t len,
             uint8_t* out_digest, size_t* inout_len);

// Legacy hash functions
int vex_md5(const uint8_t* msg, size_t len,
            uint8_t* out_digest, size_t* out_len);      // 16-byte output

int vex_sha1(const uint8_t* msg, size_t len,
             uint8_t* out_digest, size_t* out_len);     // 20-byte output

int vex_sha384(const uint8_t* msg, size_t len,
               uint8_t* out_digest, size_t* out_len);   // 48-byte output
```

### HMAC

```c
int vex_hmac(const char* algo,                        // "SHA-256", "SHA-512"
             const uint8_t* key, size_t key_len,
             const uint8_t* msg, size_t msg_len,
             uint8_t* out_mac, size_t* out_mac_len);
```

### Key Derivation

```c
// HKDF
int vex_hkdf(const char* algo,                        // "HKDF-SHA256", "HKDF-SHA512"
             const uint8_t* ikm, size_t ikm_len,
             const uint8_t* salt, size_t salt_len,
             const uint8_t* info, size_t info_len,
             uint8_t* out_okm, size_t okm_len);

// PBKDF2
int vex_pbkdf2(const char* algo,                      // "SHA-256", "SHA-512"
               const uint8_t* password, size_t pw_len,
               const uint8_t* salt, size_t salt_len,
               int iterations,                        // Recommended: 100,000+
               uint8_t* out_key, size_t key_len);
```

### X25519 (ECDH)

```c
int vex_x25519_public_from_private(uint8_t pub[32], const uint8_t priv[32]);
int vex_x25519(uint8_t shared[32], const uint8_t priv[32], const uint8_t peer_pub[32]);
```

### Ed25519 (Signatures)

```c
int vex_ed25519_sign(uint8_t sig[64], const uint8_t* msg, size_t len, const uint8_t sk[64]);
int vex_ed25519_verify(const uint8_t sig[64], const uint8_t* msg, size_t len, const uint8_t pk[32]);
```

### RSA

```c
// Generate keypair (returns DER-encoded keys)
int vex_rsa_generate_keypair(int bits,                // 2048, 3072, 4096
                             uint8_t** out_public_der, size_t* pub_len,
                             uint8_t** out_private_der, size_t* priv_len);

// Sign and verify
int vex_rsa_sign(const char* hash_algo,               // "SHA-256", "SHA-512"
                 const uint8_t* msg, size_t msg_len,
                 const uint8_t* private_key_der, size_t priv_len,
                 uint8_t* out_sig, size_t* sig_len);

int vex_rsa_verify(const char* hash_algo,
                   const uint8_t* msg, size_t msg_len,
                   const uint8_t* sig, size_t sig_len,
                   const uint8_t* public_key_der, size_t pub_len);

// Encrypt and decrypt (RSA-OAEP)
int vex_rsa_encrypt(const uint8_t* plaintext, size_t pt_len,
                    const uint8_t* public_key_der, size_t pub_len,
                    uint8_t* out_cipher, size_t* ct_len);

int vex_rsa_decrypt(const uint8_t* ciphertext, size_t ct_len,
                    const uint8_t* private_key_der, size_t priv_len,
                    uint8_t* out_plain, size_t* pt_len);
```

### ECDSA (NIST Curves)

```c
// Generate keypair (returns DER-encoded keys)
int vex_ecdsa_generate_keypair(const char* curve,     // "P-256", "P-384", "P-521"
                               uint8_t** out_public_der, size_t* pub_len,
                               uint8_t** out_private_der, size_t* priv_len);

// Sign and verify
int vex_ecdsa_sign(const char* curve, const char* hash_algo,
                   const uint8_t* msg, size_t msg_len,
                   const uint8_t* private_key_der, size_t priv_len,
                   uint8_t* out_sig, size_t* sig_len);

int vex_ecdsa_verify(const char* curve, const char* hash_algo,
                     const uint8_t* msg, size_t msg_len,
                     const uint8_t* sig, size_t sig_len,
                     const uint8_t* public_key_der, size_t pub_len);
```

### ECDH (NIST Curves)

```c
int vex_ecdh(const char* curve,                       // "P-256", "P-384", "P-521"
             const uint8_t* private_key_der, size_t priv_len,
             const uint8_t* peer_public_der, size_t peer_len,
             uint8_t* out_shared, size_t* shared_len);
```

### Random Number Generation

```c
int vex_random_bytes(uint8_t* buf, size_t len);
```

### X.509 Certificates

```c
// Certificate information structure
typedef struct {
    char subject[256];
    char issuer[256];
    char serial[64];
    int64_t not_before;      // Unix timestamp
    int64_t not_after;       // Unix timestamp
    int is_ca;
    int key_usage;
} VexX509Info;

// Parse certificate
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
```

### PEM/DER Conversion

```c
// Convert PEM to DER
int vex_pem_to_der(const uint8_t* pem, size_t pem_len,
                   uint8_t** out_der, size_t* der_len);

// Convert DER to PEM
int vex_der_to_pem(const char* label,     // e.g., "CERTIFICATE", "PRIVATE KEY"
                   const uint8_t* der, size_t der_len,
                   uint8_t** out_pem, size_t* pem_len);

// Free allocated memory
void vex_crypto_free(void* ptr);
```

### Constant-Time Operations

```c
// Constant-time memory comparison (returns 0 if equal, non-zero otherwise)
int vex_constant_time_compare(const uint8_t* a, const uint8_t* b, size_t len);

// Constant-time select (select=1 -> copy a, select=0 -> copy b)
void vex_constant_time_select(uint8_t* out, const uint8_t* a, const uint8_t* b,
                               size_t len, int select);
```

### AES Key Wrap (RFC 3394)

```c
// Wrap a key with a key-encryption-key (KEK)
int vex_aes_key_wrap(const uint8_t* kek, size_t kek_len,
                     const uint8_t* plaintext_key, size_t pt_len,
                     uint8_t* out_wrapped, size_t* out_len);

// Unwrap a key
int vex_aes_key_unwrap(const uint8_t* kek, size_t kek_len,
                       const uint8_t* wrapped_key, size_t wrapped_len,
                       uint8_t* out_plaintext, size_t* out_len);
```

### TLS

```c
// Context creation
typedef struct {
    int is_server;
    int verify_peer;
    const char* ca_bundle_path;
    const char* server_name;
    const char* alpn_csv;                              // e.g., "h2,http/1.1"
    const char* cert_pem;
    const char* key_pem;
} VexTlsConfig;

VexTlsCtx* vex_tls_ctx_create(const VexTlsConfig* cfg);
void vex_tls_ctx_destroy(VexTlsCtx* ctx);

// Connection management
VexTlsConn* vex_tls_conn_wrap_fd(VexTlsCtx* ctx, int fd);
void vex_tls_conn_destroy(VexTlsConn* c);

// I/O operations
VexTlsStatus vex_tls_handshake(VexTlsConn* c);
VexTlsStatus vex_tls_read(VexTlsConn* c, uint8_t* buf, size_t cap, size_t* out_n);
VexTlsStatus vex_tls_write(VexTlsConn* c, const uint8_t* buf, size_t len, size_t* out_n);
int vex_tls_shutdown(VexTlsConn* c);

// Status codes
typedef enum {
    VEX_TLS_OK = 0,
    VEX_TLS_WANT_READ = 1,
    VEX_TLS_WANT_WRITE = 2,
    VEX_TLS_ERR = -1
} VexTlsStatus;
```

## Usage Examples

See `tests/test_comprehensive.c` for complete working examples of all features.

## Performance Notes

- **Zero-copy**: Where possible, operations are done in-place
- **Thread-local errors**: Error messages are thread-safe via TLS
- **OpenSSL 3.x**: Fully supports latest OpenSSL with provider architecture
- **Backward compatible**: Works with OpenSSL 1.1.0+ (some features require 1.1.1+)

## Security Considerations

- **Random seeding**: OpenSSL's RNG is properly seeded on modern systems
- **Memory**: Caller is responsible for zeroing sensitive data
- **DER keys**: RSA/ECDSA keys are DER-encoded (not PEM) for efficiency
- **AEAD**: Prefer AEAD modes (GCM, ChaCha20-Poly1305) over CBC for new applications
- **Key sizes**: Use at least RSA-2048, ECDSA P-256, or Ed25519
- **Timing attacks**: Use constant-time operations for sensitive comparisons
- **Legacy algorithms**: MD5 and SHA-1 are provided for compatibility only - avoid for new security-critical applications
- **Key wrapping**: Use AES Key Wrap for secure key storage and transport

## Integration with Vex

This library will be used by:

- `vex-libs/std/crypto` - High-level crypto API
- `vex-libs/std/tls` - TLS client/server
- `vex-libs/std/jwt` - JWT signing/verification
- `vex-libs/std/password` - Password hashing
- `vex-libs/std/x509` - Certificate management
- `vex-libs/std/keystore` - Secure key storage

## License

MIT License - See main Vex repository for details.

## Version

**v3.0.0** - OpenSSL 3.0+ Only, Simplified & Optimized (November 2025)

### Changelog

#### v3.0.0 (November 2025)

- 🔥 **BREAKING**: Now requires OpenSSL 3.0+ minimum
- ✅ Removed 88 lines of version-checking code (-7.4% codebase)
- ✅ Cleaner, more maintainable codebase
- ✅ All tests passing with OpenSSL 3.x

#### v2.0.0 (November 2025)

- ✅ Added X.509 certificate operations (parse, verify, generate, CSR)
- ✅ Added PEM/DER conversion helpers
- ✅ Added legacy hash functions (MD5, SHA-1, SHA-384)
- ✅ Added constant-time operations for timing attack prevention
- ✅ Added AES Key Wrap (RFC 3394)
- ✅ Comprehensive test suite with 11 test cases

#### v1.0.0 (January 2025)

- ✅ Initial release with AEAD, symmetric encryption, hashing, HMAC, KDF
- ✅ RSA, ECDSA, ECDH, X25519, Ed25519
- ✅ TLS/SSL support
- ✅ Random number generation

## Supported Platforms

- macOS (x86_64, ARM64)
- Linux (x86_64, ARM64)
- BSD (x86_64)
- Windows (MSVC, MinGW) - via OpenSSL binaries

## Requirements

- **OpenSSL >= 3.0.0** (required)
- C11 compiler
- POSIX or Windows platform

## Why OpenSSL 3.0+ Only?

This library requires OpenSSL 3.0+ for several reasons:

1. **Cleaner Codebase**: Eliminated 88 lines of version-checking preprocessor guards
2. **Modern API**: Uses latest OpenSSL APIs without legacy fallbacks
3. **Better Security**: OpenSSL 3.x includes security improvements and better defaults
4. **Simplified Maintenance**: No need to support multiple API versions
5. **Future-Proof**: OpenSSL 1.1.1 reaches EOL in September 2023

**Before**: 1196 lines with version guards  
**After**: 1108 lines, clean and simple

All major distributions (Ubuntu 22.04+, Debian 12+, RHEL 9+, macOS with Homebrew) ship with OpenSSL 3.x.
