#include <stdio.h>
#include <stdint.h>
#include <string.h>

// FoldHash from vex_swisstable.c (simplified)
static inline uint64_t vex_rotl64(uint64_t x, int r) {
    return (x << r) | (x >> (64 - r));
}

static inline uint64_t vex_fmix64(uint64_t k) {
    k ^= k >> 33;
    k *= 0xff51afd7ed558ccdULL;
    k ^= k >> 33;
    k *= 0xc4ceb9fe1a85ec53ULL;
    k ^= k >> 33;
    return k;
}

static inline uint64_t hash64_str(const char *s) {
    const uint8_t *p = (const uint8_t *)s;
    size_t len = strlen(s);
    uint64_t a = 0x243F6A8885A308D3ULL ^ (uint64_t)len;
    uint64_t b = 0x13198A2E03707344ULL;
    uint64_t c = 0xA4093822299F31D0ULL;
    uint64_t d = 0x082EFA98EC4E6C89ULL;
    
    // Simplified - just hash
    uint64_t h = a ^ b ^ c ^ d ^ (uint64_t)len;
    return vex_fmix64(h);
}

static inline uint8_t h2(uint64_t hash) {
    return (uint8_t)(hash >> 57);  // top 7 bits
}

int main() {
    char keys[10][32];
    for (int i = 0; i < 10; i++) {
        snprintf(keys[i], 32, "key_%d", i);
    }
    
    // Check if any keys have same hash & fingerprint
    for (int i = 0; i < 10; i++) {
        uint64_t hi = hash64_str(keys[i]);
        uint8_t fpi = h2(hi);
        for (int j = i+1; j < 10; j++) {
            uint64_t hj = hash64_str(keys[j]);
            uint8_t fpj = h2(hj);
            if (hi == hj) {
                printf("FULL HASH COLLISION: %s vs %s\n", keys[i], keys[j]);
            }
            if (fpi == fpj) {
                printf("FINGERPRINT COLLISION: %s (fp=%02x) vs %s (fp=%02x)\n",
                       keys[i], fpi, keys[j], fpj);
            }
        }
    }
    
    return 0;
}
