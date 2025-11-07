#include "vex_tls.h"
#include "vex_crypto.h"
#include <stdio.h>
#include <string.h>
#include <stdint.h>

int main(){
    uint8_t key[32] = {0};
    uint8_t nonce[12] = {1,2,3,4,5,6,7,8,9,10,11,12};
    const uint8_t pt[] = "hello aead";
    uint8_t ct[256]; size_t ct_len = sizeof(ct);
    int rc = vex_aead_seal("AES-256-GCM", key, 32, nonce, 12, NULL, 0, pt, sizeof(pt)-1, ct, &ct_len, 16);
    if (rc != 0) { printf("seal err: %s\n", vex_crypto_last_error()); return 1; }
    uint8_t out[256]; size_t pt_len = sizeof(out);
    rc = vex_aead_open("AES-256-GCM", key, 32, nonce, 12, NULL, 0, ct, ct_len, out, &pt_len, 16);
    if (rc != 0) { printf("open err: %s\n", vex_crypto_last_error()); return 1; }
    out[pt_len] = 0;
    printf("decrypted: %s\n", out);
    printf("OK\n");
    return 0;
}
