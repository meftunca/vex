#ifndef VEX_SIMD_UTILS_H
#define VEX_SIMD_UTILS_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Get the name of the SIMD backend being used
 * Returns: "AVX-512", "AVX2", "SSE2", "ARM NEON", or "SCALAR"
 */
const char* vex_simd_backend(void);

/**
 * Find first occurrence of character in buffer using SIMD acceleration
 * Falls back to scalar if SIMD not available
 * 
 * @param buf Buffer to search
 * @param len Length of buffer
 * @param c   Character to find
 * @return Index of first occurrence, or len if not found
 */
size_t vex_simd_find_char(const char *buf, size_t len, char c);

/**
 * Find first occurrence of either c1 or c2
 */
size_t vex_simd_find_set2(const char *buf, size_t len, char c1, char c2);

/**
 * Find first occurrence of any of c1, c2, c3, c4
 */
size_t vex_simd_find_set4(const char *buf, size_t len, char c1, char c2, char c3, char c4);

/**
 * XOR a buffer with a 4-byte key (for WebSocket masking)
 * @param buf Buffer to modify in-place
 * @param len Length of buffer
 * @param key 4-byte masking key
 */
void vex_simd_xor_stream(uint8_t *buf, size_t len, const uint8_t key[4]);

#ifdef __cplusplus
}
#endif

#endif /* VEX_SIMD_UTILS_H */
