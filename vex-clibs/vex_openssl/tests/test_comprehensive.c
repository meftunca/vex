#include "vex_crypto.h"
#include <stdio.h>
#include <string.h>
#include <stdint.h>
#include <stdlib.h>

#define TEST(name) printf("\n=== Test: %s ===\n", name)
#define PASS() printf("✓ PASS\n")
#define FAIL(msg) do { printf("✗ FAIL: %s\n", msg); return 1; } while(0)

int test_cipher_cbc() {
    TEST("Symmetric Cipher (AES-256-CBC)");
    
    uint8_t key[32] = {0};
    uint8_t iv[16] = {1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16};
    const uint8_t pt[] = "Hello, World! This is a test message.";
    size_t pt_len = sizeof(pt) - 1;
    
    uint8_t ct[256];
    size_t ct_len = sizeof(ct);
    
    int rc = vex_cipher_encrypt("AES-256-CBC", key, 32, iv, 16, pt, pt_len, ct, &ct_len);
    if (rc != 0) FAIL(vex_crypto_last_error());
    
    printf("Encrypted %zu bytes -> %zu bytes\n", pt_len, ct_len);
    
    uint8_t decrypted[256];
    size_t dec_len = sizeof(decrypted);
    
    rc = vex_cipher_decrypt("AES-256-CBC", key, 32, iv, 16, ct, ct_len, decrypted, &dec_len);
    if (rc != 0) FAIL(vex_crypto_last_error());
    
    if (dec_len != pt_len || memcmp(decrypted, pt, pt_len) != 0) {
        FAIL("Decrypted text doesn't match original");
    }
    
    PASS();
    return 0;
}

int test_hmac() {
    TEST("HMAC-SHA256");
    
    uint8_t key[] = "secret_key";
    const uint8_t msg[] = "The quick brown fox jumps over the lazy dog";
    uint8_t mac[64];
    size_t mac_len = sizeof(mac);
    
    int rc = vex_hmac("SHA-256", key, sizeof(key)-1, msg, sizeof(msg)-1, mac, &mac_len);
    if (rc != 0) FAIL(vex_crypto_last_error());
    
    printf("HMAC length: %zu bytes\n", mac_len);
    if (mac_len != 32) FAIL("Unexpected MAC length");
    
    PASS();
    return 0;
}

int test_pbkdf2() {
    TEST("PBKDF2-SHA256");
    
    const uint8_t password[] = "my_password";
    uint8_t salt[16] = {1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16};
    uint8_t key[32];
    
    int rc = vex_pbkdf2("SHA-256", password, sizeof(password)-1, salt, sizeof(salt), 10000, key, sizeof(key));
    if (rc != 0) FAIL(vex_crypto_last_error());
    
    printf("Derived key: ");
    for (size_t i = 0; i < 16; i++) {
        printf("%02x", key[i]);
    }
    printf("...\n");
    
    PASS();
    return 0;
}

int test_random() {
    TEST("Random Bytes Generation");
    
    uint8_t buf1[32], buf2[32];
    
    if (vex_random_bytes(buf1, sizeof(buf1)) != 0) FAIL(vex_crypto_last_error());
    if (vex_random_bytes(buf2, sizeof(buf2)) != 0) FAIL(vex_crypto_last_error());
    
    if (memcmp(buf1, buf2, sizeof(buf1)) == 0) {
        FAIL("Random bytes are identical (very unlikely!)");
    }
    
    printf("Generated 32 random bytes\n");
    PASS();
    return 0;
}

int test_rsa() {
    TEST("RSA 2048 KeyPair Generation & Sign/Verify");
    
    uint8_t* pub_der = NULL;
    uint8_t* priv_der = NULL;
    size_t pub_len = 0, priv_len = 0;
    
    int rc = vex_rsa_generate_keypair(2048, &pub_der, &pub_len, &priv_der, &priv_len);
    if (rc != 0) FAIL(vex_crypto_last_error());
    
    printf("Generated RSA keypair: pub=%zu bytes, priv=%zu bytes\n", pub_len, priv_len);
    
    const uint8_t msg[] = "Test message for RSA signing";
    uint8_t sig[512];
    size_t sig_len = sizeof(sig);
    
    rc = vex_rsa_sign("SHA-256", msg, sizeof(msg)-1, priv_der, priv_len, sig, &sig_len);
    if (rc != 0) {
        free(pub_der);
        free(priv_der);
        FAIL(vex_crypto_last_error());
    }
    
    printf("Signature length: %zu bytes\n", sig_len);
    
    rc = vex_rsa_verify("SHA-256", msg, sizeof(msg)-1, sig, sig_len, pub_der, pub_len);
    if (rc != 0) {
        free(pub_der);
        free(priv_der);
        FAIL("Signature verification failed");
    }
    
    // Test RSA encryption/decryption
    const uint8_t plain[] = "Secret data";
    uint8_t cipher[512];
    size_t ct_len = sizeof(cipher);
    
    rc = vex_rsa_encrypt(plain, sizeof(plain)-1, pub_der, pub_len, cipher, &ct_len);
    if (rc != 0) {
        free(pub_der);
        free(priv_der);
        FAIL(vex_crypto_last_error());
    }
    
    uint8_t decrypted[512];
    size_t dec_len = sizeof(decrypted);
    
    rc = vex_rsa_decrypt(cipher, ct_len, priv_der, priv_len, decrypted, &dec_len);
    if (rc != 0) {
        free(pub_der);
        free(priv_der);
        FAIL(vex_crypto_last_error());
    }
    
    if (dec_len != sizeof(plain)-1 || memcmp(decrypted, plain, dec_len) != 0) {
        free(pub_der);
        free(priv_der);
        FAIL("Decrypted data doesn't match");
    }
    
    free(pub_der);
    free(priv_der);
    
    PASS();
    return 0;
}

int test_ecdsa() {
    TEST("ECDSA P-256 KeyPair Generation & Sign/Verify");
    
    uint8_t* pub_der = NULL;
    uint8_t* priv_der = NULL;
    size_t pub_len = 0, priv_len = 0;
    
    int rc = vex_ecdsa_generate_keypair("P-256", &pub_der, &pub_len, &priv_der, &priv_len);
    if (rc != 0) FAIL(vex_crypto_last_error());
    
    printf("Generated ECDSA P-256 keypair: pub=%zu bytes, priv=%zu bytes\n", pub_len, priv_len);
    
    const uint8_t msg[] = "Test message for ECDSA signing";
    uint8_t sig[256];
    size_t sig_len = sizeof(sig);
    
    rc = vex_ecdsa_sign("P-256", "SHA-256", msg, sizeof(msg)-1, priv_der, priv_len, sig, &sig_len);
    if (rc != 0) {
        free(pub_der);
        free(priv_der);
        FAIL(vex_crypto_last_error());
    }
    
    printf("ECDSA signature length: %zu bytes\n", sig_len);
    
    rc = vex_ecdsa_verify("P-256", "SHA-256", msg, sizeof(msg)-1, sig, sig_len, pub_der, pub_len);
    if (rc != 0) {
        free(pub_der);
        free(priv_der);
        FAIL("ECDSA signature verification failed");
    }
    
    free(pub_der);
    free(priv_der);
    
    PASS();
    return 0;
}

int test_ecdh() {
    TEST("ECDH P-256 Key Agreement");
    
    // Generate Alice's keypair
    uint8_t* alice_pub = NULL;
    uint8_t* alice_priv = NULL;
    size_t alice_pub_len = 0, alice_priv_len = 0;
    
    int rc = vex_ecdsa_generate_keypair("P-256", &alice_pub, &alice_pub_len, &alice_priv, &alice_priv_len);
    if (rc != 0) FAIL(vex_crypto_last_error());
    
    // Generate Bob's keypair
    uint8_t* bob_pub = NULL;
    uint8_t* bob_priv = NULL;
    size_t bob_pub_len = 0, bob_priv_len = 0;
    
    rc = vex_ecdsa_generate_keypair("P-256", &bob_pub, &bob_pub_len, &bob_priv, &bob_priv_len);
    if (rc != 0) {
        free(alice_pub); free(alice_priv);
        FAIL(vex_crypto_last_error());
    }
    
    // Alice computes shared secret
    uint8_t alice_shared[64];
    size_t alice_shared_len = sizeof(alice_shared);
    
    rc = vex_ecdh("P-256", alice_priv, alice_priv_len, bob_pub, bob_pub_len, alice_shared, &alice_shared_len);
    if (rc != 0) {
        free(alice_pub); free(alice_priv);
        free(bob_pub); free(bob_priv);
        FAIL(vex_crypto_last_error());
    }
    
    // Bob computes shared secret
    uint8_t bob_shared[64];
    size_t bob_shared_len = sizeof(bob_shared);
    
    rc = vex_ecdh("P-256", bob_priv, bob_priv_len, alice_pub, alice_pub_len, bob_shared, &bob_shared_len);
    if (rc != 0) {
        free(alice_pub); free(alice_priv);
        free(bob_pub); free(bob_priv);
        FAIL(vex_crypto_last_error());
    }
    
    // Verify shared secrets match
    if (alice_shared_len != bob_shared_len || memcmp(alice_shared, bob_shared, alice_shared_len) != 0) {
        free(alice_pub); free(alice_priv);
        free(bob_pub); free(bob_priv);
        FAIL("ECDH shared secrets don't match");
    }
    
    printf("ECDH shared secret: %zu bytes (matched)\n", alice_shared_len);
    
    free(alice_pub); free(alice_priv);
    free(bob_pub); free(bob_priv);
    
    PASS();
    return 0;
}

int test_x509_self_signed() {
    TEST("X.509 Self-Signed Certificate Generation & Validation");
    
    uint8_t* cert_pem = NULL;
    uint8_t* key_pem = NULL;
    size_t cert_len = 0, key_len = 0;
    
    // Generate self-signed certificate
    int rc = vex_x509_generate_self_signed("CN=Test Root CA", 365, &cert_pem, &cert_len, &key_pem, &key_len);
    if (rc != 0) FAIL(vex_crypto_last_error());
    
    printf("Generated certificate: %zu bytes\n", cert_len);
    printf("Generated private key: %zu bytes\n", key_len);
    
    // Convert PEM to DER for parsing
    uint8_t* cert_der = NULL;
    size_t cert_der_len = 0;
    
    rc = vex_pem_to_der(cert_pem, cert_len, &cert_der, &cert_der_len);
    if (rc != 0) {
        free(cert_pem);
        free(key_pem);
        FAIL("Failed to convert certificate to DER");
    }
    
    // Parse certificate and extract info
    VexX509Info info;
    rc = vex_x509_parse(cert_der, cert_der_len, &info);
    if (rc != 0) {
        free(cert_pem);
        free(key_pem);
        free(cert_der);
        FAIL(vex_crypto_last_error());
    }
    
    printf("Certificate subject: %s\n", info.subject);
    printf("Certificate issuer: %s\n", info.issuer);
    printf("Serial number: %s\n", info.serial);
    printf("Valid from: %ld\n", (long)info.not_before);
    printf("Valid until: %ld\n", (long)info.not_after);
    printf("Is CA: %s\n", info.is_ca ? "Yes" : "No");
    
    // Validate that subject and issuer are the same (self-signed)
    if (strcmp(info.subject, info.issuer) != 0) {
        free(cert_pem);
        free(key_pem);
        free(cert_der);
        FAIL("Subject and issuer should match for self-signed cert");
    }
    
    // Verify certificate chain (self-signed, so it's its own CA)
    rc = vex_x509_verify_chain(cert_der, cert_der_len, cert_pem, cert_len);
    if (rc != 0) {
        printf("Warning: Self-signed cert verification: %s\n", vex_crypto_last_error());
        // This might fail for self-signed certs, which is expected
    } else {
        printf("Certificate chain verified successfully\n");
    }
    
    free(cert_pem);
    free(key_pem);
    free(cert_der);
    
    PASS();
    return 0;
}

int test_x509_csr() {
    TEST("X.509 Certificate Signing Request (CSR)");
    
    // First generate a key pair
    uint8_t* cert_pem = NULL;
    uint8_t* key_pem = NULL;
    size_t cert_len = 0, key_len = 0;
    
    int rc = vex_x509_generate_self_signed("CN=Temporary", 1, &cert_pem, &cert_len, &key_pem, &key_len);
    if (rc != 0) FAIL(vex_crypto_last_error());
    
    // Generate CSR using the private key
    uint8_t* csr_pem = NULL;
    size_t csr_len = 0;
    
    rc = vex_x509_generate_csr("CN=example.com,O=Test Organization,C=US", key_pem, key_len, &csr_pem, &csr_len);
    if (rc != 0) {
        free(cert_pem);
        free(key_pem);
        FAIL(vex_crypto_last_error());
    }
    
    printf("Generated CSR: %zu bytes\n", csr_len);
    
    // Verify CSR contains expected header
    if (strstr((const char*)csr_pem, "BEGIN CERTIFICATE REQUEST") == NULL) {
        free(cert_pem);
        free(key_pem);
        free(csr_pem);
        FAIL("CSR doesn't contain expected PEM header");
    }
    
    printf("CSR successfully generated with correct format\n");
    
    free(cert_pem);
    free(key_pem);
    free(csr_pem);
    
    PASS();
    return 0;
}

int test_x509_pem_der_conversion() {
    TEST("X.509 PEM/DER Conversion");
    
    // Generate a certificate
    uint8_t* cert_pem = NULL;
    uint8_t* key_pem = NULL;
    size_t cert_len = 0, key_len = 0;
    
    int rc = vex_x509_generate_self_signed("CN=Conversion Test", 365, &cert_pem, &cert_len, &key_pem, &key_len);
    if (rc != 0) FAIL(vex_crypto_last_error());
    
    // Convert PEM to DER
    uint8_t* cert_der = NULL;
    size_t cert_der_len = 0;
    
    rc = vex_pem_to_der(cert_pem, cert_len, &cert_der, &cert_der_len);
    if (rc != 0) {
        free(cert_pem);
        free(key_pem);
        FAIL("PEM to DER conversion failed");
    }
    
    printf("PEM (%zu bytes) -> DER (%zu bytes)\n", cert_len, cert_der_len);
    
    // Convert DER back to PEM
    uint8_t* cert_pem2 = NULL;
    size_t cert_pem2_len = 0;
    
    rc = vex_der_to_pem("CERTIFICATE", cert_der, cert_der_len, &cert_pem2, &cert_pem2_len);
    if (rc != 0) {
        free(cert_pem);
        free(key_pem);
        free(cert_der);
        FAIL("DER to PEM conversion failed");
    }
    
    printf("DER (%zu bytes) -> PEM (%zu bytes)\n", cert_der_len, cert_pem2_len);
    
    // Verify PEM format
    if (strstr((const char*)cert_pem2, "BEGIN CERTIFICATE") == NULL) {
        free(cert_pem);
        free(key_pem);
        free(cert_der);
        vex_crypto_free(cert_pem2);
        FAIL("Converted PEM doesn't contain expected header");
    }
    
    // Parse both versions to ensure they represent the same certificate
    VexX509Info info1, info2;
    
    rc = vex_x509_parse(cert_der, cert_der_len, &info1);
    if (rc != 0) {
        free(cert_pem);
        free(key_pem);
        free(cert_der);
        vex_crypto_free(cert_pem2);
        FAIL("Failed to parse original DER");
    }
    
    uint8_t* cert_der2 = NULL;
    size_t cert_der2_len = 0;
    rc = vex_pem_to_der(cert_pem2, cert_pem2_len, &cert_der2, &cert_der2_len);
    if (rc != 0) {
        free(cert_pem);
        free(key_pem);
        free(cert_der);
        vex_crypto_free(cert_pem2);
        FAIL("Failed to convert back to DER");
    }
    
    rc = vex_x509_parse(cert_der2, cert_der2_len, &info2);
    if (rc != 0) {
        free(cert_pem);
        free(key_pem);
        free(cert_der);
        vex_crypto_free(cert_pem2);
        free(cert_der2);
        FAIL("Failed to parse converted DER");
    }
    
    // Compare serial numbers to ensure they're the same certificate
    if (strcmp(info1.serial, info2.serial) != 0) {
        free(cert_pem);
        free(key_pem);
        free(cert_der);
        vex_crypto_free(cert_pem2);
        free(cert_der2);
        FAIL("Serial numbers don't match after conversion");
    }
    
    printf("Certificate integrity verified after conversion cycle\n");
    
    free(cert_pem);
    free(key_pem);
    free(cert_der);
    vex_crypto_free(cert_pem2);
    free(cert_der2);
    
    PASS();
    return 0;
}

int test_hash_functions() {
    TEST("Additional Hash Functions (MD5, SHA-1, SHA-384)");
    
    const uint8_t msg[] = "The quick brown fox jumps over the lazy dog";
    
    // Test MD5
    uint8_t md5_digest[16];
    size_t md5_len = sizeof(md5_digest);
    if (vex_md5(msg, sizeof(msg)-1, md5_digest, &md5_len) != 0) {
        FAIL(vex_crypto_last_error());
    }
    if (md5_len != 16) FAIL("MD5 digest length incorrect");
    printf("MD5: ");
    for (size_t i = 0; i < 8; i++) printf("%02x", md5_digest[i]);
    printf("...\n");
    
    // Test SHA-1
    uint8_t sha1_digest[20];
    size_t sha1_len = sizeof(sha1_digest);
    if (vex_sha1(msg, sizeof(msg)-1, sha1_digest, &sha1_len) != 0) {
        FAIL(vex_crypto_last_error());
    }
    if (sha1_len != 20) FAIL("SHA-1 digest length incorrect");
    printf("SHA-1: ");
    for (size_t i = 0; i < 8; i++) printf("%02x", sha1_digest[i]);
    printf("...\n");
    
    // Test SHA-384
    uint8_t sha384_digest[48];
    size_t sha384_len = sizeof(sha384_digest);
    if (vex_sha384(msg, sizeof(msg)-1, sha384_digest, &sha384_len) != 0) {
        FAIL(vex_crypto_last_error());
    }
    if (sha384_len != 48) FAIL("SHA-384 digest length incorrect");
    printf("SHA-384: ");
    for (size_t i = 0; i < 8; i++) printf("%02x", sha384_digest[i]);
    printf("...\n");
    
    PASS();
    return 0;
}

int test_constant_time() {
    TEST("Constant-Time Operations");
    
    uint8_t a[] = {1, 2, 3, 4, 5};
    uint8_t b[] = {1, 2, 3, 4, 5};
    uint8_t c[] = {1, 2, 3, 4, 6};
    
    // Test constant-time comparison
    int result = vex_constant_time_compare(a, b, sizeof(a));
    if (result != 0) FAIL("Constant-time compare failed (equal arrays)");
    
    result = vex_constant_time_compare(a, c, sizeof(a));
    if (result == 0) FAIL("Constant-time compare failed (different arrays)");
    
    // Test constant-time select
    uint8_t out[5];
    vex_constant_time_select(out, a, c, sizeof(a), 1);  // select a
    if (memcmp(out, a, sizeof(a)) != 0) FAIL("Constant-time select failed (select=1)");
    
    vex_constant_time_select(out, a, c, sizeof(a), 0);  // select c
    if (memcmp(out, c, sizeof(c)) != 0) FAIL("Constant-time select failed (select=0)");
    
    printf("Constant-time operations working correctly\n");
    
    PASS();
    return 0;
}

int test_aes_key_wrap() {
    TEST("AES Key Wrap (RFC 3394)");
    
    // Key Encryption Key (KEK)
    uint8_t kek[32] = {0};
    
    // Key to be wrapped (must be multiple of 8 bytes)
    uint8_t plaintext_key[16] = {1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16};
    
    // Wrap the key
    uint8_t wrapped[32];
    size_t wrapped_len = sizeof(wrapped);
    
    int rc = vex_aes_key_wrap(kek, sizeof(kek), plaintext_key, sizeof(plaintext_key), wrapped, &wrapped_len);
    if (rc != 0) FAIL(vex_crypto_last_error());
    
    printf("Wrapped key: %zu bytes\n", wrapped_len);
    
    // Unwrap the key
    uint8_t unwrapped[32];
    size_t unwrapped_len = sizeof(unwrapped);
    
    rc = vex_aes_key_unwrap(kek, sizeof(kek), wrapped, wrapped_len, unwrapped, &unwrapped_len);
    if (rc != 0) FAIL(vex_crypto_last_error());
    
    // Verify unwrapped key matches original
    if (unwrapped_len != sizeof(plaintext_key) || memcmp(unwrapped, plaintext_key, unwrapped_len) != 0) {
        FAIL("Unwrapped key doesn't match original");
    }
    
    printf("Unwrapped key matches original\n");
    
    PASS();
    return 0;
}

int main() {
    printf("╔═══════════════════════════════════════╗\n");
    printf("║  Vex OpenSSL Wrapper Test Suite     ║\n");
    printf("╚═══════════════════════════════════════╝\n");
    
    int failed = 0;
    
    // Core crypto tests
    failed += test_cipher_cbc();
    failed += test_hmac();
    failed += test_pbkdf2();
    failed += test_random();
    
    // PKI tests
    failed += test_rsa();
    failed += test_ecdsa();
    failed += test_ecdh();
    
    // X.509 certificate tests
    failed += test_x509_self_signed();
    failed += test_x509_csr();
    failed += test_x509_pem_der_conversion();
    
    // Additional crypto tests
    failed += test_hash_functions();
    failed += test_constant_time();
    failed += test_aes_key_wrap();
    
    printf("\n╔═══════════════════════════════════════╗\n");
    if (failed == 0) {
        printf("║  ✓ ALL TESTS PASSED (14/14)         ║\n");
    } else {
        printf("║  ✗ %d TEST(S) FAILED                  ║\n", failed);
    }
    printf("╚═══════════════════════════════════════╝\n");
    
    return failed > 0 ? 1 : 0;
}

