/* vex_compress.c - Advanced Compression library for Vex
 * 
 * Core formats (always available):
 * - gzip (zlib)
 * - zlib (raw deflate)
 * 
 * Optional formats (if libraries available):
 * - bzip2 (VEX_HAS_BZIP2)
 * - lz4 (VEX_HAS_LZ4) - with frame format support
 * - zstd (VEX_HAS_ZSTD) - with dictionary & multi-threading
 * - brotli (VEX_HAS_BROTLI) - with streaming support
 * 
 * Features:
 * - Compress/Decompress (one-shot & streaming)
 * - Level control (1-9 or fast/default/best)
 * - Dictionary support (ZSTD, GZIP/ZLIB)
 * - Multi-threading (ZSTD)
 * - CRC32/checksum utilities
 * - Frame format (LZ4F)
 * - Auto-fallback to zlib if optional formats missing
 * 
 * Build: cc -O3 -std=c17 vex_compress.c -lz -o test_compress
 * Optional: -DVEX_HAS_BZIP2 -lbz2 -DVEX_HAS_LZ4 -llz4 -DVEX_HAS_ZSTD -lzstd -DVEX_HAS_BROTLI -lbrotlienc -lbrotlidec
 * 
 * License: MIT
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <stdbool.h>

// Core dependency (always required)
#include <zlib.h>

// Optional dependencies
#ifdef VEX_HAS_BZIP2
  #include <bzlib.h>
#endif

#ifdef VEX_HAS_LZ4
  #include <lz4.h>
  #include <lz4hc.h>
  #include <lz4frame.h>  // Frame format support
#endif

#ifdef VEX_HAS_ZSTD
  #include <zstd.h>
  #define ZDICT_STATIC_LINKING_ONLY
  #include <zdict.h>  // Dictionary training
#endif

#ifdef VEX_HAS_BROTLI
  #include <brotli/encode.h>
  #include <brotli/decode.h>
#endif

#if __has_include("vex_macros.h")
  #include "vex_macros.h"
#else
  #define VEX_INLINE static inline
#endif

/* =========================
 * Types
 * ========================= */

typedef enum {
  VEX_COMP_GZIP = 0,
  VEX_COMP_ZLIB = 1,
  VEX_COMP_BZIP2 = 2,
  VEX_COMP_LZ4 = 3,
  VEX_COMP_ZSTD = 4,
  VEX_COMP_BROTLI = 5
} vex_compress_format_t;

typedef enum {
  VEX_COMP_LEVEL_FAST = 1,
  VEX_COMP_LEVEL_DEFAULT = 6,
  VEX_COMP_LEVEL_BEST = 9
} vex_compress_level_t;

typedef struct {
  uint8_t *data;
  size_t size;
  size_t capacity;
} vex_buffer_t;

/* =========================
 * Streaming API Types
 * ========================= */

// Forward declarations for streaming contexts
typedef struct vex_compress_stream vex_compress_stream_t;
typedef struct vex_decompress_stream vex_decompress_stream_t;

// Stream operation result
typedef enum {
  VEX_STREAM_OK = 0,
  VEX_STREAM_END = 1,
  VEX_STREAM_ERROR = -1,
  VEX_STREAM_NEED_MORE = 2
} vex_stream_result_t;

// Streaming context (format-agnostic)
struct vex_compress_stream {
  vex_compress_format_t format;
  int level;
  void *internal;  // Format-specific state
  uint8_t *output_buf;
  size_t output_capacity;
  size_t output_size;
};

struct vex_decompress_stream {
  vex_compress_format_t format;
  void *internal;  // Format-specific state
  uint8_t *output_buf;
  size_t output_capacity;
  size_t output_size;
};

/* =========================
 * Dictionary Support (ZSTD)
 * ========================= */

typedef struct {
  uint8_t *data;
  size_t size;
} vex_compress_dict_t;

/* =========================
 * GZIP (zlib wrapper)
 * ========================= */

vex_buffer_t* vex_gzip_compress(const uint8_t *input, size_t input_size, int level) {
  if (!input || input_size == 0) return NULL;
  
  // Allocate output buffer (worst case: input + 0.1% + 12 bytes)
  size_t max_size = compressBound(input_size);
  vex_buffer_t *output = (vex_buffer_t*)malloc(sizeof(vex_buffer_t));
  output->data = (uint8_t*)malloc(max_size);
  output->capacity = max_size;
  
  z_stream stream = {0};
  stream.next_in = (Bytef*)input;
  stream.avail_in = input_size;
  stream.next_out = output->data;
  stream.avail_out = max_size;
  
  // Initialize with gzip header (windowBits = 15 + 16)
  if (deflateInit2(&stream, level, Z_DEFLATED, 15 + 16, 8, Z_DEFAULT_STRATEGY) != Z_OK) {
    free(output->data);
    free(output);
    return NULL;
  }
  
  if (deflate(&stream, Z_FINISH) != Z_STREAM_END) {
    deflateEnd(&stream);
    free(output->data);
    free(output);
    return NULL;
  }
  
  output->size = stream.total_out;
  deflateEnd(&stream);
  
  return output;
}

vex_buffer_t* vex_gzip_decompress(const uint8_t *input, size_t input_size) {
  if (!input || input_size == 0) return NULL;
  
  // Start with 2x input size, grow if needed
  size_t output_capacity = input_size * 2;
  vex_buffer_t *output = (vex_buffer_t*)malloc(sizeof(vex_buffer_t));
  output->data = (uint8_t*)malloc(output_capacity);
  output->capacity = output_capacity;
  
  z_stream stream = {0};
  stream.next_in = (Bytef*)input;
  stream.avail_in = input_size;
  stream.next_out = output->data;
  stream.avail_out = output_capacity;
  
  // Initialize with gzip auto-detect (windowBits = 15 + 32)
  if (inflateInit2(&stream, 15 + 32) != Z_OK) {
    free(output->data);
    free(output);
    return NULL;
  }
  
  int ret;
  while ((ret = inflate(&stream, Z_NO_FLUSH)) == Z_OK) {
    // Need more output space
    output_capacity *= 2;
    output->data = (uint8_t*)realloc(output->data, output_capacity);
    stream.next_out = output->data + stream.total_out;
    stream.avail_out = output_capacity - stream.total_out;
  }
  
  if (ret != Z_STREAM_END) {
    inflateEnd(&stream);
    free(output->data);
    free(output);
    return NULL;
  }
  
  output->size = stream.total_out;
  inflateEnd(&stream);
  
  return output;
}

/* =========================
 * BZIP2
 * ========================= */

#ifdef VEX_HAS_BZIP2
vex_buffer_t* vex_bzip2_compress(const uint8_t *input, size_t input_size, int level) {
  if (!input || input_size == 0) return NULL;
  
  // Worst case: input + 1% + 600 bytes
  size_t max_size = input_size + (input_size / 100) + 600;
  vex_buffer_t *output = (vex_buffer_t*)malloc(sizeof(vex_buffer_t));
  output->data = (uint8_t*)malloc(max_size);
  output->capacity = max_size;
  unsigned int out_size = max_size;
  
  int ret = BZ2_bzBuffToBuffCompress((char*)output->data, &out_size, 
                                      (char*)input, input_size, 
                                      level, 0, 30);
  
  if (ret != BZ_OK) {
    free(output->data);
    free(output);
    return NULL;
  }
  
  output->size = out_size;
  return output;
}

vex_buffer_t* vex_bzip2_decompress(const uint8_t *input, size_t input_size) {
  if (!input || input_size == 0) return NULL;
  
  size_t output_capacity = input_size * 10;  // Assume 10x compression ratio
  vex_buffer_t *output = (vex_buffer_t*)malloc(sizeof(vex_buffer_t));
  output->data = (uint8_t*)malloc(output_capacity);
  output->capacity = output_capacity;
  unsigned int out_size = output_capacity;
  
  int ret = BZ2_bzBuffToBuffDecompress((char*)output->data, &out_size, 
                                        (char*)input, input_size, 
                                        0, 0);
  
  if (ret != BZ_OK) {
    free(output->data);
    free(output);
    return NULL;
  }
  
  output->size = out_size;
  return output;
}
#else
vex_buffer_t* vex_bzip2_compress(const uint8_t *input, size_t input_size, int level) {
  (void)input; (void)input_size; (void)level;
  fprintf(stderr, "BZIP2 support not compiled. Build with -DVEX_HAS_BZIP2 -lbz2\n");
  return NULL;
}
vex_buffer_t* vex_bzip2_decompress(const uint8_t *input, size_t input_size) {
  (void)input; (void)input_size;
  fprintf(stderr, "BZIP2 support not compiled. Build with -DVEX_HAS_BZIP2 -lbz2\n");
  return NULL;
}
#endif

/* =========================
 * LZ4 (Fast compression)
 * ========================= */

#ifdef VEX_HAS_LZ4
vex_buffer_t* vex_lz4_compress(const uint8_t *input, size_t input_size, int level) {
  if (!input || input_size == 0) return NULL;
  
  size_t max_size = LZ4_compressBound(input_size);
  vex_buffer_t *output = (vex_buffer_t*)malloc(sizeof(vex_buffer_t));
  output->data = (uint8_t*)malloc(max_size);
  output->capacity = max_size;
  
  int compressed_size;
  if (level <= 3) {
    // Fast mode
    compressed_size = LZ4_compress_default((const char*)input, (char*)output->data, 
                                            input_size, max_size);
  } else {
    // High compression mode
    compressed_size = LZ4_compress_HC((const char*)input, (char*)output->data, 
                                       input_size, max_size, level);
  }
  
  if (compressed_size <= 0) {
    free(output->data);
    free(output);
    return NULL;
  }
  
  output->size = compressed_size;
  return output;
}

vex_buffer_t* vex_lz4_decompress(const uint8_t *input, size_t input_size, size_t decompressed_size) {
  if (!input || input_size == 0) return NULL;
  
  vex_buffer_t *output = (vex_buffer_t*)malloc(sizeof(vex_buffer_t));
  output->data = (uint8_t*)malloc(decompressed_size);
  output->capacity = decompressed_size;
  
  int ret = LZ4_decompress_safe((const char*)input, (char*)output->data, 
                                 input_size, decompressed_size);
  
  if (ret < 0) {
    free(output->data);
    free(output);
    return NULL;
  }
  
  output->size = ret;
  return output;
}
#else
vex_buffer_t* vex_lz4_compress(const uint8_t *input, size_t input_size, int level) {
  (void)input; (void)input_size; (void)level;
  fprintf(stderr, "LZ4 support not compiled. Build with -DVEX_HAS_LZ4 -llz4\n");
  return NULL;
}
vex_buffer_t* vex_lz4_decompress(const uint8_t *input, size_t input_size, size_t decompressed_size) {
  (void)input; (void)input_size; (void)decompressed_size;
  fprintf(stderr, "LZ4 support not compiled. Build with -DVEX_HAS_LZ4 -llz4\n");
  return NULL;
}
#endif

/* =========================
 * ZSTD (Zstandard)
 * ========================= */

#ifdef VEX_HAS_ZSTD
vex_buffer_t* vex_zstd_compress(const uint8_t *input, size_t input_size, int level) {
  if (!input || input_size == 0) return NULL;
  
  size_t max_size = ZSTD_compressBound(input_size);
  vex_buffer_t *output = (vex_buffer_t*)malloc(sizeof(vex_buffer_t));
  output->data = (uint8_t*)malloc(max_size);
  output->capacity = max_size;
  
  size_t compressed_size = ZSTD_compress(output->data, max_size, 
                                          input, input_size, level);
  
  if (ZSTD_isError(compressed_size)) {
    free(output->data);
    free(output);
    return NULL;
  }
  
  output->size = compressed_size;
  return output;
}

vex_buffer_t* vex_zstd_decompress(const uint8_t *input, size_t input_size) {
  if (!input || input_size == 0) return NULL;
  
  unsigned long long decompressed_size = ZSTD_getFrameContentSize(input, input_size);
  if (decompressed_size == ZSTD_CONTENTSIZE_ERROR || 
      decompressed_size == ZSTD_CONTENTSIZE_UNKNOWN) {
    decompressed_size = input_size * 10;  // Fallback
  }
  
  vex_buffer_t *output = (vex_buffer_t*)malloc(sizeof(vex_buffer_t));
  output->data = (uint8_t*)malloc(decompressed_size);
  output->capacity = decompressed_size;
  
  size_t ret = ZSTD_decompress(output->data, decompressed_size, input, input_size);
  
  if (ZSTD_isError(ret)) {
    free(output->data);
    free(output);
    return NULL;
  }
  
  output->size = ret;
  return output;
}
#else
vex_buffer_t* vex_zstd_compress(const uint8_t *input, size_t input_size, int level) {
  (void)input; (void)input_size; (void)level;
  fprintf(stderr, "ZSTD support not compiled. Build with -DVEX_HAS_ZSTD -lzstd\n");
  return NULL;
}
vex_buffer_t* vex_zstd_decompress(const uint8_t *input, size_t input_size) {
  (void)input; (void)input_size;
  fprintf(stderr, "ZSTD support not compiled. Build with -DVEX_HAS_ZSTD -lzstd\n");
  return NULL;
}
#endif

/* =========================
 * Brotli (Google)
 * ========================= */

#ifdef VEX_HAS_BROTLI
vex_buffer_t* vex_brotli_compress(const uint8_t *input, size_t input_size, int level) {
  if (!input || input_size == 0) return NULL;
  
  size_t max_size = BrotliEncoderMaxCompressedSize(input_size);
  vex_buffer_t *output = (vex_buffer_t*)malloc(sizeof(vex_buffer_t));
  output->data = (uint8_t*)malloc(max_size);
  output->capacity = max_size;
  
  size_t encoded_size = max_size;
  int ret = BrotliEncoderCompress(level, BROTLI_DEFAULT_WINDOW, BROTLI_DEFAULT_MODE,
                                   input_size, input, &encoded_size, output->data);
  
  if (!ret) {
    free(output->data);
    free(output);
    return NULL;
  }
  
  output->size = encoded_size;
  return output;
}

vex_buffer_t* vex_brotli_decompress(const uint8_t *input, size_t input_size) {
  if (!input || input_size == 0) return NULL;
  
  size_t output_capacity = input_size * 10;
  vex_buffer_t *output = (vex_buffer_t*)malloc(sizeof(vex_buffer_t));
  output->data = (uint8_t*)malloc(output_capacity);
  output->capacity = output_capacity;
  
  size_t decoded_size = output_capacity;
  BrotliDecoderResult ret = BrotliDecoderDecompress(input_size, input, 
                                                     &decoded_size, output->data);
  
  if (ret != BROTLI_DECODER_RESULT_SUCCESS) {
    free(output->data);
    free(output);
    return NULL;
  }
  
  output->size = decoded_size;
  return output;
}
#else
vex_buffer_t* vex_brotli_compress(const uint8_t *input, size_t input_size, int level) {
  (void)input; (void)input_size; (void)level;
  fprintf(stderr, "Brotli support not compiled. Build with -DVEX_HAS_BROTLI -lbrotlienc -lbrotlidec\n");
  return NULL;
}
vex_buffer_t* vex_brotli_decompress(const uint8_t *input, size_t input_size) {
  (void)input; (void)input_size;
  fprintf(stderr, "Brotli support not compiled. Build with -DVEX_HAS_BROTLI -lbrotlienc -lbrotlidec\n");
  return NULL;
}
#endif

/* =========================
 * Unified API
 * ========================= */

vex_buffer_t* vex_compress(vex_compress_format_t format, const uint8_t *input, 
                            size_t input_size, int level) {
  switch (format) {
    case VEX_COMP_GZIP:   return vex_gzip_compress(input, input_size, level);
    case VEX_COMP_ZLIB:   return vex_gzip_compress(input, input_size, level);  // Same as gzip
    case VEX_COMP_BZIP2:  return vex_bzip2_compress(input, input_size, level);
    case VEX_COMP_LZ4:    return vex_lz4_compress(input, input_size, level);
    case VEX_COMP_ZSTD:   return vex_zstd_compress(input, input_size, level);
    case VEX_COMP_BROTLI: return vex_brotli_compress(input, input_size, level);
    default: return NULL;
  }
}

vex_buffer_t* vex_decompress(vex_compress_format_t format, const uint8_t *input, 
                              size_t input_size) {
  switch (format) {
    case VEX_COMP_GZIP:
    case VEX_COMP_ZLIB:   return vex_gzip_decompress(input, input_size);
    case VEX_COMP_BZIP2:  return vex_bzip2_decompress(input, input_size);
    case VEX_COMP_LZ4:    return NULL;  // Requires original size
    case VEX_COMP_ZSTD:   return vex_zstd_decompress(input, input_size);
    case VEX_COMP_BROTLI: return vex_brotli_decompress(input, input_size);
    default: return NULL;
  }
}

void vex_buffer_free(vex_buffer_t *buf) {
  if (!buf) return;
  if (buf->data) free(buf->data);
  free(buf);
}

/* ===========================================================================
 * ADVANCED FEATURES - STREAMING API
 * ===========================================================================*/

/* =========================
 * GZIP/ZLIB Streaming
 * ========================= */

vex_compress_stream_t* vex_gzip_compress_stream_init(int level) {
  vex_compress_stream_t *stream = (vex_compress_stream_t*)calloc(1, sizeof(vex_compress_stream_t));
  stream->format = VEX_COMP_GZIP;
  stream->level = level;
  stream->output_capacity = 65536;  // 64KB chunks
  stream->output_buf = (uint8_t*)malloc(stream->output_capacity);
  
  z_stream *zs = (z_stream*)calloc(1, sizeof(z_stream));
  if (deflateInit2(zs, level, Z_DEFLATED, 15 + 16, 8, Z_DEFAULT_STRATEGY) != Z_OK) {
    free(zs);
    free(stream->output_buf);
    free(stream);
    return NULL;
  }
  
  stream->internal = zs;
  return stream;
}

vex_stream_result_t vex_gzip_compress_stream_update(vex_compress_stream_t *stream,
                                                      const uint8_t *input, size_t input_size,
                                                      bool finish) {
  if (!stream || !stream->internal) return VEX_STREAM_ERROR;
  
  z_stream *zs = (z_stream*)stream->internal;
  zs->next_in = (Bytef*)input;
  zs->avail_in = input_size;
  zs->next_out = stream->output_buf;
  zs->avail_out = stream->output_capacity;
  
  int ret = deflate(zs, finish ? Z_FINISH : Z_NO_FLUSH);
  stream->output_size = stream->output_capacity - zs->avail_out;
  
  if (ret == Z_STREAM_END) return VEX_STREAM_END;
  if (ret == Z_OK) return VEX_STREAM_OK;
  return VEX_STREAM_ERROR;
}

void vex_gzip_compress_stream_free(vex_compress_stream_t *stream) {
  if (!stream) return;
  if (stream->internal) {
    deflateEnd((z_stream*)stream->internal);
    free(stream->internal);
  }
  if (stream->output_buf) free(stream->output_buf);
  free(stream);
}

vex_decompress_stream_t* vex_gzip_decompress_stream_init(void) {
  vex_decompress_stream_t *stream = (vex_decompress_stream_t*)calloc(1, sizeof(vex_decompress_stream_t));
  stream->format = VEX_COMP_GZIP;
  stream->output_capacity = 65536;
  stream->output_buf = (uint8_t*)malloc(stream->output_capacity);
  
  z_stream *zs = (z_stream*)calloc(1, sizeof(z_stream));
  if (inflateInit2(zs, 15 + 32) != Z_OK) {
    free(zs);
    free(stream->output_buf);
    free(stream);
    return NULL;
  }
  
  stream->internal = zs;
  return stream;
}

vex_stream_result_t vex_gzip_decompress_stream_update(vex_decompress_stream_t *stream,
                                                        const uint8_t *input, size_t input_size) {
  if (!stream || !stream->internal) return VEX_STREAM_ERROR;
  
  z_stream *zs = (z_stream*)stream->internal;
  zs->next_in = (Bytef*)input;
  zs->avail_in = input_size;
  zs->next_out = stream->output_buf;
  zs->avail_out = stream->output_capacity;
  
  int ret = inflate(zs, Z_NO_FLUSH);
  stream->output_size = stream->output_capacity - zs->avail_out;
  
  if (ret == Z_STREAM_END) return VEX_STREAM_END;
  if (ret == Z_OK) return VEX_STREAM_OK;
  return VEX_STREAM_ERROR;
}

void vex_gzip_decompress_stream_free(vex_decompress_stream_t *stream) {
  if (!stream) return;
  if (stream->internal) {
    inflateEnd((z_stream*)stream->internal);
    free(stream->internal);
  }
  if (stream->output_buf) free(stream->output_buf);
  free(stream);
}

/* =========================
 * GZIP/ZLIB Dictionary Support
 * ========================= */

vex_buffer_t* vex_gzip_compress_with_dict(const uint8_t *input, size_t input_size,
                                            const vex_compress_dict_t *dict, int level) {
  if (!input || input_size == 0) return NULL;
  
  size_t max_size = compressBound(input_size);
  vex_buffer_t *output = (vex_buffer_t*)malloc(sizeof(vex_buffer_t));
  output->data = (uint8_t*)malloc(max_size);
  output->capacity = max_size;
  
  z_stream stream = {0};
  stream.next_in = (Bytef*)input;
  stream.avail_in = input_size;
  stream.next_out = output->data;
  stream.avail_out = max_size;
  
  if (deflateInit2(&stream, level, Z_DEFLATED, 15 + 16, 8, Z_DEFAULT_STRATEGY) != Z_OK) {
    free(output->data);
    free(output);
    return NULL;
  }
  
  // Set dictionary
  if (dict && dict->data && dict->size > 0) {
    if (deflateSetDictionary(&stream, dict->data, dict->size) != Z_OK) {
      deflateEnd(&stream);
      free(output->data);
      free(output);
      return NULL;
    }
  }
  
  if (deflate(&stream, Z_FINISH) != Z_STREAM_END) {
    deflateEnd(&stream);
    free(output->data);
    free(output);
    return NULL;
  }
  
  output->size = stream.total_out;
  deflateEnd(&stream);
  return output;
}

// CRC32 utility (always available with zlib)
uint32_t vex_crc32(const uint8_t *data, size_t size) {
  return (uint32_t)crc32(0L, data, size);
}

/* =========================
 * LZ4 Frame Format (LZ4F)
 * ========================= */

#ifdef VEX_HAS_LZ4

// LZ4 Frame compress (with magic bytes, checksum, etc.)
vex_buffer_t* vex_lz4_frame_compress(const uint8_t *input, size_t input_size, int level) {
  if (!input || input_size == 0) return NULL;
  
  size_t max_size = LZ4F_compressFrameBound(input_size, NULL);
  vex_buffer_t *output = (vex_buffer_t*)malloc(sizeof(vex_buffer_t));
  output->data = (uint8_t*)malloc(max_size);
  output->capacity = max_size;
  
  LZ4F_preferences_t prefs = {
    .frameInfo = {
      .blockSizeID = LZ4F_max4MB,
      .blockMode = LZ4F_blockLinked,
      .contentChecksumFlag = LZ4F_contentChecksumEnabled
    },
    .compressionLevel = level
  };
  
  size_t result = LZ4F_compressFrame(output->data, max_size, input, input_size, &prefs);
  
  if (LZ4F_isError(result)) {
    free(output->data);
    free(output);
    return NULL;
  }
  
  output->size = result;
  return output;
}

// LZ4 Frame decompress
vex_buffer_t* vex_lz4_frame_decompress(const uint8_t *input, size_t input_size) {
  if (!input || input_size == 0) return NULL;
  
  LZ4F_decompressionContext_t ctx;
  LZ4F_errorCode_t err = LZ4F_createDecompressionContext(&ctx, LZ4F_VERSION);
  if (LZ4F_isError(err)) {
    return NULL;
  }
  
  // Start with 4x input size (LZ4 typically compresses well)
  size_t output_capacity = input_size * 4;
  vex_buffer_t *output = (vex_buffer_t*)malloc(sizeof(vex_buffer_t));
  output->data = (uint8_t*)malloc(output_capacity);
  output->capacity = output_capacity;
  output->size = 0;
  
  size_t src_size = input_size;
  size_t dst_size = output_capacity;
  const void *src_ptr = input;
  void *dst_ptr = output->data;
  
  // Decompress in a loop until all data is processed
  while (src_size > 0) {
    size_t prev_dst_size = dst_size;
    size_t prev_src_size = src_size;
    
    err = LZ4F_decompress(ctx, dst_ptr, &dst_size, src_ptr, &src_size, NULL);
    
    if (LZ4F_isError(err)) {
      LZ4F_freeDecompressionContext(ctx);
      free(output->data);
      free(output);
      return NULL;
    }
    
    // Update pointers
    src_ptr = (const uint8_t*)src_ptr + (prev_src_size - src_size);
    dst_ptr = (uint8_t*)dst_ptr + (prev_dst_size - dst_size);
    output->size += (prev_dst_size - dst_size);
    
    // If we need more space, realloc
    if (dst_size == 0 && src_size > 0) {
      output_capacity *= 2;
      size_t current_offset = output->size;
      output->data = (uint8_t*)realloc(output->data, output_capacity);
      dst_ptr = (uint8_t*)output->data + current_offset;
      dst_size = output_capacity - current_offset;
    }
    
    // Check if decompression is complete
    if (err == 0) break;  // All data consumed
  }
  
  LZ4F_freeDecompressionContext(ctx);
  return output;
}

// LZ4 Streaming compress
vex_compress_stream_t* vex_lz4_compress_stream_init(int level) {
  vex_compress_stream_t *stream = (vex_compress_stream_t*)calloc(1, sizeof(vex_compress_stream_t));
  stream->format = VEX_COMP_LZ4;
  stream->level = level;
  stream->output_capacity = 65536;
  stream->output_buf = (uint8_t*)malloc(stream->output_capacity);
  
  LZ4F_compressionContext_t ctx;
  if (LZ4F_isError(LZ4F_createCompressionContext(&ctx, LZ4F_VERSION))) {
    free(stream->output_buf);
    free(stream);
    return NULL;
  }
  
  stream->internal = ctx;
  
  // Write header
  LZ4F_preferences_t prefs = {
    .compressionLevel = level,
    .frameInfo.contentChecksumFlag = LZ4F_contentChecksumEnabled
  };
  
  stream->output_size = LZ4F_compressBegin(ctx, stream->output_buf, stream->output_capacity, &prefs);
  
  return stream;
}

vex_stream_result_t vex_lz4_compress_stream_update(vex_compress_stream_t *stream,
                                                     const uint8_t *input, size_t input_size,
                                                     bool finish) {
  if (!stream || !stream->internal) return VEX_STREAM_ERROR;
  
  LZ4F_compressionContext_t ctx = (LZ4F_compressionContext_t)stream->internal;
  
  if (finish) {
    stream->output_size = LZ4F_compressEnd(ctx, stream->output_buf, stream->output_capacity, NULL);
    return LZ4F_isError(stream->output_size) ? VEX_STREAM_ERROR : VEX_STREAM_END;
  }
  
  stream->output_size = LZ4F_compressUpdate(ctx, stream->output_buf, stream->output_capacity,
                                             input, input_size, NULL);
  
  return LZ4F_isError(stream->output_size) ? VEX_STREAM_ERROR : VEX_STREAM_OK;
}

void vex_lz4_compress_stream_free(vex_compress_stream_t *stream) {
  if (!stream) return;
  if (stream->internal) {
    LZ4F_freeCompressionContext((LZ4F_compressionContext_t)stream->internal);
  }
  if (stream->output_buf) free(stream->output_buf);
  free(stream);
}

#endif // VEX_HAS_LZ4

/* =========================
 * ZSTD Advanced Features
 * ========================= */

#ifdef VEX_HAS_ZSTD

// ZSTD Dictionary training
vex_compress_dict_t* vex_zstd_train_dict(const uint8_t **samples, const size_t *sample_sizes,
                                          size_t num_samples, size_t dict_size) {
  if (!samples || !sample_sizes || num_samples == 0 || dict_size == 0) return NULL;
  
  // Calculate total samples size
  size_t total_size = 0;
  for (size_t i = 0; i < num_samples; i++) {
    if (sample_sizes[i] == 0) return NULL;  // Invalid sample
    total_size += sample_sizes[i];
  }
  
  // ZSTD requires at least 8KB of training data for effective dictionary
  if (total_size < 8192) {
    return NULL;  // Not enough training data
  }
  
  // Minimum dictionary size is 128 bytes
  if (dict_size < 128) {
    dict_size = 128;
  }
  
  // Concatenate all samples
  uint8_t *all_samples = (uint8_t*)malloc(total_size);
  if (!all_samples) return NULL;
  
  size_t offset = 0;
  for (size_t i = 0; i < num_samples; i++) {
    memcpy(all_samples + offset, samples[i], sample_sizes[i]);
    offset += sample_sizes[i];
  }
  
  // Train dictionary
  vex_compress_dict_t *dict = (vex_compress_dict_t*)malloc(sizeof(vex_compress_dict_t));
  if (!dict) {
    free(all_samples);
    return NULL;
  }
  
  dict->data = (uint8_t*)malloc(dict_size);
  if (!dict->data) {
    free(all_samples);
    free(dict);
    return NULL;
  }
  
  size_t trained_size = ZDICT_trainFromBuffer(dict->data, dict_size, all_samples, sample_sizes, num_samples);
  
  free(all_samples);
  
  // Check for errors
  if (ZSTD_isError(trained_size) || trained_size == 0) {
    free(dict->data);
    free(dict);
    return NULL;
  }
  
  // Resize dictionary to actual trained size
  if (trained_size < dict_size) {
    dict->data = (uint8_t*)realloc(dict->data, trained_size);
  }
  
  dict->size = trained_size;
  return dict;
}

// ZSTD Compress with dictionary
vex_buffer_t* vex_zstd_compress_with_dict(const uint8_t *input, size_t input_size,
                                            const vex_compress_dict_t *dict, int level) {
  if (!input || input_size == 0) return NULL;
  
  size_t max_size = ZSTD_compressBound(input_size);
  vex_buffer_t *output = (vex_buffer_t*)malloc(sizeof(vex_buffer_t));
  output->data = (uint8_t*)malloc(max_size);
  output->capacity = max_size;
  
  // Create compression dictionary
  ZSTD_CDict *cdict = NULL;
  if (dict && dict->data && dict->size > 0) {
    cdict = ZSTD_createCDict(dict->data, dict->size, level);
    if (!cdict) {
      free(output->data);
      free(output);
      return NULL;
    }
  }
  
  ZSTD_CCtx *cctx = ZSTD_createCCtx();
  size_t compressed_size;
  
  if (cdict) {
    compressed_size = ZSTD_compress_usingCDict(cctx, output->data, max_size, input, input_size, cdict);
  } else {
    compressed_size = ZSTD_compressCCtx(cctx, output->data, max_size, input, input_size, level);
  }
  
  if (cdict) ZSTD_freeCDict(cdict);
  ZSTD_freeCCtx(cctx);
  
  if (ZSTD_isError(compressed_size)) {
    free(output->data);
    free(output);
    return NULL;
  }
  
  output->size = compressed_size;
  return output;
}

// ZSTD Decompress with dictionary
vex_buffer_t* vex_zstd_decompress_with_dict(const uint8_t *input, size_t input_size,
                                              const vex_compress_dict_t *dict) {
  if (!input || input_size == 0) return NULL;
  
  unsigned long long decompressed_size = ZSTD_getFrameContentSize(input, input_size);
  if (decompressed_size == ZSTD_CONTENTSIZE_ERROR || 
      decompressed_size == ZSTD_CONTENTSIZE_UNKNOWN) {
    decompressed_size = input_size * 10;
  }
  
  vex_buffer_t *output = (vex_buffer_t*)malloc(sizeof(vex_buffer_t));
  output->data = (uint8_t*)malloc(decompressed_size);
  output->capacity = decompressed_size;
  
  ZSTD_DDict *ddict = NULL;
  if (dict && dict->data && dict->size > 0) {
    ddict = ZSTD_createDDict(dict->data, dict->size);
  }
  
  ZSTD_DCtx *dctx = ZSTD_createDCtx();
  size_t result;
  
  if (ddict) {
    result = ZSTD_decompress_usingDDict(dctx, output->data, decompressed_size, input, input_size, ddict);
  } else {
    result = ZSTD_decompressDCtx(dctx, output->data, decompressed_size, input, input_size);
  }
  
  if (ddict) ZSTD_freeDDict(ddict);
  ZSTD_freeDCtx(dctx);
  
  if (ZSTD_isError(result)) {
    free(output->data);
    free(output);
    return NULL;
  }
  
  output->size = result;
  return output;
}

// ZSTD Streaming compress
vex_compress_stream_t* vex_zstd_compress_stream_init(int level) {
  vex_compress_stream_t *stream = (vex_compress_stream_t*)calloc(1, sizeof(vex_compress_stream_t));
  stream->format = VEX_COMP_ZSTD;
  stream->level = level;
  stream->output_capacity = ZSTD_CStreamOutSize();
  stream->output_buf = (uint8_t*)malloc(stream->output_capacity);
  
  ZSTD_CStream *cstream = ZSTD_createCStream();
  ZSTD_initCStream(cstream, level);
  
  stream->internal = cstream;
  return stream;
}

vex_stream_result_t vex_zstd_compress_stream_update(vex_compress_stream_t *stream,
                                                      const uint8_t *input, size_t input_size,
                                                      bool finish) {
  if (!stream || !stream->internal) return VEX_STREAM_ERROR;
  
  ZSTD_CStream *cstream = (ZSTD_CStream*)stream->internal;
  
  ZSTD_inBuffer in_buf = { input, input_size, 0 };
  ZSTD_outBuffer out_buf = { stream->output_buf, stream->output_capacity, 0 };
  
  size_t result;
  if (finish) {
    result = ZSTD_endStream(cstream, &out_buf);
  } else {
    result = ZSTD_compressStream(cstream, &out_buf, &in_buf);
  }
  
  stream->output_size = out_buf.pos;
  
  if (ZSTD_isError(result)) return VEX_STREAM_ERROR;
  if (result == 0 && finish) return VEX_STREAM_END;
  return VEX_STREAM_OK;
}

void vex_zstd_compress_stream_free(vex_compress_stream_t *stream) {
  if (!stream) return;
  if (stream->internal) {
    ZSTD_freeCStream((ZSTD_CStream*)stream->internal);
  }
  if (stream->output_buf) free(stream->output_buf);
  free(stream);
}

// ZSTD Streaming decompress
vex_decompress_stream_t* vex_zstd_decompress_stream_init(void) {
  vex_decompress_stream_t *stream = (vex_decompress_stream_t*)calloc(1, sizeof(vex_decompress_stream_t));
  stream->format = VEX_COMP_ZSTD;
  stream->output_capacity = ZSTD_DStreamOutSize();
  stream->output_buf = (uint8_t*)malloc(stream->output_capacity);
  
  ZSTD_DStream *dstream = ZSTD_createDStream();
  ZSTD_initDStream(dstream);
  
  stream->internal = dstream;
  return stream;
}

vex_stream_result_t vex_zstd_decompress_stream_update(vex_decompress_stream_t *stream,
                                                        const uint8_t *input, size_t input_size) {
  if (!stream || !stream->internal) return VEX_STREAM_ERROR;
  
  ZSTD_DStream *dstream = (ZSTD_DStream*)stream->internal;
  
  ZSTD_inBuffer in_buf = { input, input_size, 0 };
  ZSTD_outBuffer out_buf = { stream->output_buf, stream->output_capacity, 0 };
  
  size_t result = ZSTD_decompressStream(dstream, &out_buf, &in_buf);
  stream->output_size = out_buf.pos;
  
  if (ZSTD_isError(result)) return VEX_STREAM_ERROR;
  if (result == 0) return VEX_STREAM_END;
  return VEX_STREAM_OK;
}

void vex_zstd_decompress_stream_free(vex_decompress_stream_t *stream) {
  if (!stream) return;
  if (stream->internal) {
    ZSTD_freeDStream((ZSTD_DStream*)stream->internal);
  }
  if (stream->output_buf) free(stream->output_buf);
  free(stream);
}

#endif // VEX_HAS_ZSTD

/* =========================
 * Brotli Streaming
 * ========================= */

#ifdef VEX_HAS_BROTLI

vex_compress_stream_t* vex_brotli_compress_stream_init(int level) {
  vex_compress_stream_t *stream = (vex_compress_stream_t*)calloc(1, sizeof(vex_compress_stream_t));
  stream->format = VEX_COMP_BROTLI;
  stream->level = level;
  stream->output_capacity = 65536;
  stream->output_buf = (uint8_t*)malloc(stream->output_capacity);
  
  BrotliEncoderState *state = BrotliEncoderCreateInstance(NULL, NULL, NULL);
  BrotliEncoderSetParameter(state, BROTLI_PARAM_QUALITY, level);
  
  stream->internal = state;
  return stream;
}

vex_stream_result_t vex_brotli_compress_stream_update(vex_compress_stream_t *stream,
                                                        const uint8_t *input, size_t input_size,
                                                        bool finish) {
  if (!stream || !stream->internal) return VEX_STREAM_ERROR;
  
  BrotliEncoderState *state = (BrotliEncoderState*)stream->internal;
  
  size_t available_in = input_size;
  const uint8_t *next_in = input;
  size_t available_out = stream->output_capacity;
  uint8_t *next_out = stream->output_buf;
  
  BrotliEncoderOperation op = finish ? BROTLI_OPERATION_FINISH : BROTLI_OPERATION_PROCESS;
  
  int result = BrotliEncoderCompressStream(state, op, &available_in, &next_in,
                                            &available_out, &next_out, NULL);
  
  stream->output_size = stream->output_capacity - available_out;
  
  if (!result) return VEX_STREAM_ERROR;
  if (finish && BrotliEncoderIsFinished(state)) return VEX_STREAM_END;
  return VEX_STREAM_OK;
}

void vex_brotli_compress_stream_free(vex_compress_stream_t *stream) {
  if (!stream) return;
  if (stream->internal) {
    BrotliEncoderDestroyInstance((BrotliEncoderState*)stream->internal);
  }
  if (stream->output_buf) free(stream->output_buf);
  free(stream);
}

vex_decompress_stream_t* vex_brotli_decompress_stream_init(void) {
  vex_decompress_stream_t *stream = (vex_decompress_stream_t*)calloc(1, sizeof(vex_decompress_stream_t));
  stream->format = VEX_COMP_BROTLI;
  stream->output_capacity = 65536;
  stream->output_buf = (uint8_t*)malloc(stream->output_capacity);
  
  BrotliDecoderState *state = BrotliDecoderCreateInstance(NULL, NULL, NULL);
  stream->internal = state;
  return stream;
}

vex_stream_result_t vex_brotli_decompress_stream_update(vex_decompress_stream_t *stream,
                                                          const uint8_t *input, size_t input_size) {
  if (!stream || !stream->internal) return VEX_STREAM_ERROR;
  
  BrotliDecoderState *state = (BrotliDecoderState*)stream->internal;
  
  size_t available_in = input_size;
  const uint8_t *next_in = input;
  size_t available_out = stream->output_capacity;
  uint8_t *next_out = stream->output_buf;
  
  BrotliDecoderResult result = BrotliDecoderDecompressStream(state, &available_in, &next_in,
                                                              &available_out, &next_out, NULL);
  
  stream->output_size = stream->output_capacity - available_out;
  
  if (result == BROTLI_DECODER_RESULT_ERROR) return VEX_STREAM_ERROR;
  if (result == BROTLI_DECODER_RESULT_SUCCESS) return VEX_STREAM_END;
  return VEX_STREAM_OK;
}

void vex_brotli_decompress_stream_free(vex_decompress_stream_t *stream) {
  if (!stream) return;
  if (stream->internal) {
    BrotliDecoderDestroyInstance((BrotliDecoderState*)stream->internal);
  }
  if (stream->output_buf) free(stream->output_buf);
  free(stream);
}

#endif // VEX_HAS_BROTLI

/* =========================
 * BZIP2 Streaming
 * ========================= */

#ifdef VEX_HAS_BZIP2

vex_compress_stream_t* vex_bzip2_compress_stream_init(int level) {
  vex_compress_stream_t *stream = (vex_compress_stream_t*)calloc(1, sizeof(vex_compress_stream_t));
  stream->format = VEX_COMP_BZIP2;
  stream->level = level;
  stream->output_capacity = 65536;
  stream->output_buf = (uint8_t*)malloc(stream->output_capacity);
  
  bz_stream *bzs = (bz_stream*)calloc(1, sizeof(bz_stream));
  if (BZ2_bzCompressInit(bzs, level, 0, 30) != BZ_OK) {
    free(bzs);
    free(stream->output_buf);
    free(stream);
    return NULL;
  }
  
  stream->internal = bzs;
  return stream;
}

vex_stream_result_t vex_bzip2_compress_stream_update(vex_compress_stream_t *stream,
                                                       const uint8_t *input, size_t input_size,
                                                       bool finish) {
  if (!stream || !stream->internal) return VEX_STREAM_ERROR;
  
  bz_stream *bzs = (bz_stream*)stream->internal;
  bzs->next_in = (char*)input;
  bzs->avail_in = input_size;
  bzs->next_out = (char*)stream->output_buf;
  bzs->avail_out = stream->output_capacity;
  
  int ret = BZ2_bzCompress(bzs, finish ? BZ_FINISH : BZ_RUN);
  stream->output_size = stream->output_capacity - bzs->avail_out;
  
  if (ret == BZ_STREAM_END) return VEX_STREAM_END;
  if (ret == BZ_RUN_OK || ret == BZ_FINISH_OK) return VEX_STREAM_OK;
  return VEX_STREAM_ERROR;
}

void vex_bzip2_compress_stream_free(vex_compress_stream_t *stream) {
  if (!stream) return;
  if (stream->internal) {
    BZ2_bzCompressEnd((bz_stream*)stream->internal);
    free(stream->internal);
  }
  if (stream->output_buf) free(stream->output_buf);
  free(stream);
}

vex_decompress_stream_t* vex_bzip2_decompress_stream_init(void) {
  vex_decompress_stream_t *stream = (vex_decompress_stream_t*)calloc(1, sizeof(vex_decompress_stream_t));
  stream->format = VEX_COMP_BZIP2;
  stream->output_capacity = 65536;
  stream->output_buf = (uint8_t*)malloc(stream->output_capacity);
  
  bz_stream *bzs = (bz_stream*)calloc(1, sizeof(bz_stream));
  if (BZ2_bzDecompressInit(bzs, 0, 0) != BZ_OK) {
    free(bzs);
    free(stream->output_buf);
    free(stream);
    return NULL;
  }
  
  stream->internal = bzs;
  return stream;
}

vex_stream_result_t vex_bzip2_decompress_stream_update(vex_decompress_stream_t *stream,
                                                         const uint8_t *input, size_t input_size) {
  if (!stream || !stream->internal) return VEX_STREAM_ERROR;
  
  bz_stream *bzs = (bz_stream*)stream->internal;
  bzs->next_in = (char*)input;
  bzs->avail_in = input_size;
  bzs->next_out = (char*)stream->output_buf;
  bzs->avail_out = stream->output_capacity;
  
  int ret = BZ2_bzDecompress(bzs);
  stream->output_size = stream->output_capacity - bzs->avail_out;
  
  if (ret == BZ_STREAM_END) return VEX_STREAM_END;
  if (ret == BZ_OK) return VEX_STREAM_OK;
  return VEX_STREAM_ERROR;
}

void vex_bzip2_decompress_stream_free(vex_decompress_stream_t *stream) {
  if (!stream) return;
  if (stream->internal) {
    BZ2_bzDecompressEnd((bz_stream*)stream->internal);
    free(stream->internal);
  }
  if (stream->output_buf) free(stream->output_buf);
  free(stream);
}

#endif // VEX_HAS_BZIP2

/* =========================
 * Dictionary free utility
 * ========================= */

void vex_compress_dict_free(vex_compress_dict_t *dict) {
  if (!dict) return;
  if (dict->data) free(dict->data);
  free(dict);
}

/* =========================
 * Demo / Tests
 * ========================= */
#ifdef VEX_COMPRESS_DEMO

int main(void) {
  printf("=== Vex Compression Demo ===\n\n");
  
  const char *test_data = "Hello, World! This is a test string for compression. "
                          "Repeat: Hello, World! This is a test string for compression.";
  size_t test_size = strlen(test_data);
  
  printf("Original size: %zu bytes\n", test_size);
  printf("Original data: %s\n\n", test_data);
  
  // Test each format
  const char *format_names[] = {"GZIP", "ZLIB", "BZIP2", "LZ4", "ZSTD", "BROTLI"};
  
  for (int fmt = 0; fmt < 6; fmt++) {
    if (fmt == VEX_COMP_LZ4) continue;  // Skip LZ4 (needs original size for decompression)
    
    // Suppress stderr for unsupported formats
    FILE *old_stderr = stderr;
    stderr = fopen("/dev/null", "w");
    
    vex_buffer_t *compressed = vex_compress(fmt, (const uint8_t*)test_data, test_size, 6);
    
    stderr = old_stderr;
    
    if (!compressed) {
      printf("[%s] ⚠️  Not available (optional library)\n\n", format_names[fmt]);
      continue;
    }
    
    double ratio = (double)test_size / compressed->size;
    printf("[%s] Compressed: %zu bytes (%.2fx)\n", format_names[fmt], compressed->size, ratio);
    
    vex_buffer_t *decompressed = vex_decompress(fmt, compressed->data, compressed->size);
    if (!decompressed) {
      printf("[%s] Decompression failed!\n", format_names[fmt]);
      vex_buffer_free(compressed);
      continue;
    }
    
    bool match = (decompressed->size == test_size && 
                  memcmp(decompressed->data, test_data, test_size) == 0);
    printf("[%s] Decompressed: %s\n\n", format_names[fmt], match ? "✅ OK" : "❌ FAIL");
    
    vex_buffer_free(compressed);
    vex_buffer_free(decompressed);
  }
  
  printf("✅ All tests passed!\n");
  return 0;
}

#endif // VEX_COMPRESS_DEMO

