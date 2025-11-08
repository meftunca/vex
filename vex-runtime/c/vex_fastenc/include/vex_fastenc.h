#ifndef VEX_FASTENC_H
#define VEX_FASTENC_H

#include <stddef.h>
#include <stdint.h>
#include <sys/types.h>  /* for ssize_t */

#ifdef __cplusplus
extern "C" {
#endif

/* ===================== Categories =====================
   encoding/: base16 (hex), base32, base64
   id/:       uuid (v1, v3, v4, v5, v6, v7, v8), parse/format
   crypto/:   md5, sha1 (only for UUID v3/v5), os_random
   util/:     cpu feature detect, sizing helpers
====================================================== */

/* ---------------- Base16 (Hex) ---------------- */
size_t vex_hex_encoded_len(size_t nbytes);                 /* = nbytes*2 */
size_t vex_hex_decoded_len(size_t nchars);                 /* = nchars/2 (if valid) */

/* uppercase: 0 → lowercase 'a'..'f', 1 → uppercase 'A'..'F' */
size_t vex_hex_encode(const uint8_t* src, size_t n, char* dst, int uppercase);
ssize_t vex_hex_decode(const char* src, size_t n, uint8_t* dst); /* returns bytes written or -1 on invalid */

/* ---------------- Base64 (RFC 4648) ---------------- */
typedef enum {
  VEX_B64_STD = 0,      /* A-Z a-z 0-9 + / */
  VEX_B64_URLSAFE = 1   /* A-Z a-z 0-9 - _ */
} vex_b64_alphabet;

typedef struct {
  vex_b64_alphabet alpha;
  int pad;       /* 1 => '=' padding enabled; 0 => unpadded */
  int wrap;      /* 0=no wrap, else column width (e.g., 76 for MIME) */
} vex_b64_cfg;

size_t vex_base64_encoded_len(size_t nbytes, vex_b64_cfg cfg);
size_t vex_base64_max_decoded_len(size_t nchars);

size_t vex_base64_encode(const uint8_t* src, size_t n, char* dst, vex_b64_cfg cfg);
ssize_t vex_base64_decode(const char* src, size_t n, uint8_t* dst, vex_b64_alphabet alpha);

/* ---------------- Base32 (RFC 4648 + Base32hex + Crockford) ---------------- */
typedef enum {
  VEX_B32_RFC = 0,      /* RFC 4648 Base32 */
  VEX_B32_HEX = 1,      /* RFC 4648 Base32hex */
  VEX_B32_CROCKFORD = 2 /* Crockford Base32 (case-insensitive, I/L→1, O→0) */
} vex_b32_alphabet;

typedef struct {
  vex_b32_alphabet alpha;
  int pad;     /* '=' padding if RFC/HEX; Crockford is unpadded */
} vex_b32_cfg;

size_t vex_base32_encoded_len(size_t nbytes, vex_b32_cfg cfg);
size_t vex_base32_max_decoded_len(size_t nchars);

size_t vex_base32_encode(const uint8_t* src, size_t n, char* dst, vex_b32_cfg cfg);
ssize_t vex_base32_decode(const char* src, size_t n, uint8_t* dst, vex_b32_alphabet alpha);

/* ---------------- UUID (v1,v3,v4,v5,v6,v7,v8) ---------------- */
typedef struct { uint8_t bytes[16]; } vex_uuid;

/* Format: canonical 36-chars "8-4-4-4-12" (lowercase hex), null-terminated */
int  vex_uuid_format(char out[37], const vex_uuid* u);
int  vex_uuid_parse(const char* s, vex_uuid* out);

/* Name-based (v3=MD5, v5=SHA1) requires namespace UUID and name buffer */
int  vex_uuid_v1(vex_uuid* out); /* time-based; node = random 48-bit multicast; clockseq random */
int  vex_uuid_v3(vex_uuid* out, const vex_uuid* ns, const void* name, size_t len);
int  vex_uuid_v4(vex_uuid* out); /* random */
int  vex_uuid_v5(vex_uuid* out, const vex_uuid* ns, const void* name, size_t len);
/* v6 (reordered v1), v7 (Unix time based per RFC 9562), v8 (free-form: user provides 16 bytes; version/variant set) */
int  vex_uuid_v6(vex_uuid* out);
int  vex_uuid_v7(vex_uuid* out);
int  vex_uuid_v8(vex_uuid* out, const uint8_t custom[16]);

/* ---------------- Crypto helpers (internal use but exposed) ---------------- */
int  vex_os_random(void* dst, size_t n);     /* CSPRNG: 0 on success */
void vex_md5(const void* data, size_t len, uint8_t out16[16]);
void vex_sha1(const void* data, size_t len, uint8_t out20[20]);

/* ---------------- CPU feature query (runtime) ---------------- */
int vex_cpu_has_avx2(void);
int vex_cpu_has_avx512bw(void);
int vex_cpu_has_neon(void);

#ifdef __cplusplus
}
#endif
#endif /* VEX_FASTENC_H */
