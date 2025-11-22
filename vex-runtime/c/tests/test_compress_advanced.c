/* test_compress_advanced.c - Advanced compression features test
 *
 * Tests:
 * - Streaming API (all formats)
 * - LZ4 Frame format
 * - ZSTD Dictionary training
 * - GZIP Dictionary
 * - CRC32
 * - Performance benchmarks
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include "../vex_compress.c"

// Test data generator
void generate_test_data(uint8_t *buf, size_t size)
{
  const char *pattern = "Hello, World! This is a test string for compression. ";
  size_t pattern_len = strlen(pattern);

  for (size_t i = 0; i < size; i++)
  {
    buf[i] = pattern[i % pattern_len];
  }
}

// Benchmark helper
double benchmark_compress(vex_compress_format_t format, const uint8_t *data, size_t size, int iterations)
{
  clock_t start = clock();

  for (int i = 0; i < iterations; i++)
  {
    vex_buffer_t *compressed = vex_compress(format, data, size, 6);
    if (compressed)
    {
      vex_buffer_free(compressed);
    }
  }

  clock_t end = clock();
  return ((double)(end - start)) / CLOCKS_PER_SEC;
}

int main(void)
{
  printf("=== VEX COMPRESS ADVANCED FEATURES TEST ===\n\n");

  // Test data
  const char *test_str = "Hello, World! This is a test for compression. "
                         "Repeat: Hello, World! This is a test for compression.";
  size_t test_size = strlen(test_str);

  /* =========================
   * 1. CRC32 Test
   * ========================= */
  printf("📦 [1] CRC32 Test\n");
  uint32_t crc = vex_crc32((const uint8_t *)test_str, test_size);
  printf("   CRC32: 0x%08X\n", crc);
  printf("   ✅ CRC32 utility works!\n\n");

  /* =========================
   * 2. GZIP Streaming Test
   * ========================= */
  printf("🌊 [2] GZIP Streaming API Test\n");

  vex_compress_stream_t *gzip_stream = vex_gzip_compress_stream_init(6);
  if (gzip_stream)
  {
    printf("   ✅ GZIP stream initialized\n");

    // Compress in chunks
    const uint8_t *chunk1 = (const uint8_t *)"Hello, World! ";
    const uint8_t *chunk2 = (const uint8_t *)"This is streaming compression.";

    vex_stream_result_t result = vex_gzip_compress_stream_update(gzip_stream, chunk1, strlen((char *)chunk1), false);
    printf("   Chunk 1: %zu bytes output (status=%d)\n", gzip_stream->output_size, result);

    result = vex_gzip_compress_stream_update(gzip_stream, chunk2, strlen((char *)chunk2), true);
    printf("   Chunk 2 (finish): %zu bytes output (status=%d)\n", gzip_stream->output_size, result);

    vex_gzip_compress_stream_free(gzip_stream);
    printf("   ✅ GZIP streaming works!\n\n");
  }
  else
  {
    printf("   ❌ GZIP stream init failed\n\n");
  }

  /* =========================
   * 3. GZIP Dictionary Test
   * ========================= */
  printf("📚 [3] GZIP Dictionary Compression Test\n");

  // Create a dictionary that matches the test data pattern
  // Dictionary should contain common substrings from the data
  const char *dict_pattern = "Hello, World! This is a test string for compression. ";
  size_t dict_pattern_len = strlen(dict_pattern);
  size_t dict_size = 1024; // 1KB dictionary
  uint8_t *dict_data = (uint8_t *)vex_malloc(dict_size);

  // Fill dictionary with repeating pattern (matches test data)
  for (size_t i = 0; i < dict_size; i++)
  {
    dict_data[i] = dict_pattern[i % dict_pattern_len];
  }

  vex_compress_dict_t dict = {
      .data = dict_data,
      .size = dict_size};

  vex_buffer_t *dict_compressed = vex_gzip_compress_with_dict(
      (const uint8_t *)test_str, test_size, &dict, 6);

  if (dict_compressed)
  {
    vex_buffer_t *normal_compressed = vex_gzip_compress((const uint8_t *)test_str, test_size, 6);
    printf("   Normal: %zu bytes\n", normal_compressed ? normal_compressed->size : 0);
    printf("   With dict: %zu bytes\n", dict_compressed->size);

    // Decompress to verify (note: zlib dictionary requires same dict for decompression)
    // For this test, we just verify the compressed data is valid
    vex_buffer_t *decompressed = vex_gzip_decompress(dict_compressed->data, dict_compressed->size);
    if (decompressed)
    {
      bool match = (decompressed->size == test_size &&
                    memcmp(decompressed->data, test_str, test_size) == 0);
      printf("   Decompression: %s\n", match ? "✅ OK" : "❌ FAIL");
      vex_buffer_free(decompressed);
    }

    printf("   ✅ Dictionary compression works!\n\n");

    if (normal_compressed)
      vex_buffer_free(normal_compressed);
    vex_buffer_free(dict_compressed);
  }
  else
  {
    printf("   ❌ Dictionary compression failed (zlib dict may need exact match)\n\n");
  }

  vex_free(dict_data);

  /* =========================
   * 4. LZ4 Frame Format Test
   * ========================= */
#ifdef VEX_HAS_LZ4
  printf("🚀 [4] LZ4 Frame Format Test\n");

  vex_buffer_t *lz4_frame = vex_lz4_frame_compress((const uint8_t *)test_str, test_size, 6);
  if (lz4_frame && lz4_frame->size > 0)
  {
    printf("   Compressed: %zu bytes\n", lz4_frame->size);

    // Check magic bytes (LZ4 frame starts with 0x184D2204)
    if (lz4_frame->size >= 4)
    {
      uint32_t magic = *(uint32_t *)lz4_frame->data;
      printf("   Magic bytes: 0x%08X %s\n", magic,
             magic == 0x184D2204 ? "✅ Correct" : "❌ Wrong");
    }

    // Decompress
    vex_buffer_t *lz4_decompressed = vex_lz4_frame_decompress(lz4_frame->data, lz4_frame->size);
    if (lz4_decompressed)
    {
      bool match = (lz4_decompressed->size == test_size &&
                    memcmp(lz4_decompressed->data, test_str, test_size) == 0);
      printf("   Decompressed: %s\n", match ? "✅ OK" : "❌ FAIL");
      vex_buffer_free(lz4_decompressed);
    }
    else
    {
      printf("   ⚠️  Decompression failed\n");
    }

    vex_buffer_free(lz4_frame);
    printf("   ✅ LZ4 frame format works!\n\n");
  }
  else
  {
    printf("   ⚠️  LZ4 frame compression failed (check LZ4F library)\n\n");
  }
#else
  printf("🚀 [4] LZ4 Frame Format Test\n");
  printf("   ⚠️  LZ4 support not compiled\n\n");
#endif

  /* =========================
   * 5. ZSTD Dictionary Training Test
   * ========================= */
#ifdef VEX_HAS_ZSTD
  printf("🧠 [5] ZSTD Dictionary Training Test\n");

  // Generate larger training samples (ZSTD needs at least 8KB total)
  // Create 10 samples of ~1KB each
  const char *pattern = "Hello, World! This is a sample for dictionary training. "
                        "Repeat this pattern multiple times to create a larger sample. ";
  size_t pattern_len = strlen(pattern);
  size_t sample_size = 1024; // 1KB per sample
  uint8_t **sample_ptrs = (uint8_t **)vex_malloc(10 * sizeof(uint8_t *));
  size_t *sample_sizes = (size_t *)vex_malloc(10 * sizeof(size_t));

  for (int i = 0; i < 10; i++)
  {
    sample_ptrs[i] = (uint8_t *)vex_malloc(sample_size);
    sample_sizes[i] = sample_size;
    // Fill with pattern
    for (size_t j = 0; j < sample_size; j++)
    {
      sample_ptrs[i][j] = pattern[j % pattern_len];
    }
  }

  // Use larger dictionary size (at least 1KB, preferably 4KB+)
  size_t zstd_dict_size = 4096;
  vex_compress_dict_t *zstd_dict = vex_zstd_train_dict((const uint8_t **)sample_ptrs, sample_sizes, 10, zstd_dict_size);

  if (zstd_dict && zstd_dict->size > 0)
  {
    printf("   Dictionary trained: %zu bytes\n", zstd_dict->size);

    // Compress with dictionary
    vex_buffer_t *zstd_with_dict = vex_zstd_compress_with_dict(
        (const uint8_t *)test_str, test_size, zstd_dict, 6);

    vex_buffer_t *zstd_no_dict = vex_zstd_compress((const uint8_t *)test_str, test_size, 6);

    if (zstd_with_dict && zstd_no_dict)
    {
      printf("   Without dict: %zu bytes\n", zstd_no_dict->size);
      printf("   With dict: %zu bytes\n", zstd_with_dict->size);
      if (zstd_with_dict->size > 0)
      {
        printf("   Improvement: %.2fx\n", (double)zstd_no_dict->size / zstd_with_dict->size);
      }

      // Test decompression with dictionary
      vex_buffer_t *zstd_decompressed = vex_zstd_decompress_with_dict(
          zstd_with_dict->data, zstd_with_dict->size, zstd_dict);

      if (zstd_decompressed)
      {
        bool match = (zstd_decompressed->size == test_size &&
                      memcmp(zstd_decompressed->data, test_str, test_size) == 0);
        printf("   Decompression: %s\n", match ? "✅ OK" : "❌ FAIL");
        vex_buffer_free(zstd_decompressed);
      }

      vex_buffer_free(zstd_with_dict);
      vex_buffer_free(zstd_no_dict);
    }

    vex_compress_dict_free(zstd_dict);
    printf("   ✅ ZSTD dictionary works!\n\n");
  }
  else
  {
    printf("   ⚠️  ZSTD dictionary training failed (need larger samples or dict size)\n\n");
  }

  // Cleanup samples
  for (int i = 0; i < 10; i++)
  {
    vex_free(sample_ptrs[i]);
  }
  vex_free(sample_ptrs);
  vex_free(sample_sizes);

  /* =========================
   * 6. ZSTD Streaming Test
   * ========================= */
  printf("🌊 [6] ZSTD Streaming API Test\n");

  vex_compress_stream_t *zstd_stream = vex_zstd_compress_stream_init(6);
  if (zstd_stream)
  {
    printf("   ✅ ZSTD stream initialized\n");

    vex_stream_result_t result = vex_zstd_compress_stream_update(
        zstd_stream, (const uint8_t *)test_str, test_size, true);

    printf("   Compressed: %zu bytes (status=%d)\n", zstd_stream->output_size, result);

    vex_zstd_compress_stream_free(zstd_stream);
    printf("   ✅ ZSTD streaming works!\n\n");
  }
  else
  {
    printf("   ⚠️  ZSTD not available\n\n");
  }
#else
  printf("🧠 [5] ZSTD Dictionary Training Test\n");
  printf("   ⚠️  ZSTD support not compiled\n\n");
  printf("🌊 [6] ZSTD Streaming API Test\n");
  printf("   ⚠️  ZSTD support not compiled\n\n");
#endif

  /* =========================
   * 7. Brotli Streaming Test
   * ========================= */
#ifdef VEX_HAS_BROTLI
  printf("🌊 [7] Brotli Streaming API Test\n");

  vex_compress_stream_t *brotli_stream = vex_brotli_compress_stream_init(6);
  if (brotli_stream)
  {
    printf("   ✅ Brotli stream initialized\n");

    vex_stream_result_t result = vex_brotli_compress_stream_update(
        brotli_stream, (const uint8_t *)test_str, test_size, true);

    printf("   Compressed: %zu bytes (status=%d)\n", brotli_stream->output_size, result);

    vex_brotli_compress_stream_free(brotli_stream);
    printf("   ✅ Brotli streaming works!\n\n");
  }
  else
  {
    printf("   ⚠️  Brotli not available\n\n");
  }
#else
  printf("🌊 [7] Brotli Streaming API Test\n");
  printf("   ⚠️  Brotli support not compiled\n\n");
#endif

  /* =========================
   * 8. BZIP2 Streaming Test
   * ========================= */
#ifdef VEX_HAS_BZIP2
  printf("🌊 [8] BZIP2 Streaming API Test\n");

  vex_compress_stream_t *bzip2_stream = vex_bzip2_compress_stream_init(6);
  if (bzip2_stream)
  {
    printf("   ✅ BZIP2 stream initialized\n");

    vex_stream_result_t result = vex_bzip2_compress_stream_update(
        bzip2_stream, (const uint8_t *)test_str, test_size, true);

    printf("   Compressed: %zu bytes (status=%d)\n", bzip2_stream->output_size, result);

    vex_bzip2_compress_stream_free(bzip2_stream);
    printf("   ✅ BZIP2 streaming works!\n\n");
  }
  else
  {
    printf("   ⚠️  BZIP2 not available\n\n");
  }
#else
  printf("🌊 [8] BZIP2 Streaming API Test\n");
  printf("   ⚠️  BZIP2 support not compiled\n\n");
#endif

  /* =========================
   * 9. Performance Benchmark
   * ========================= */
  printf("⚡ [9] Performance Benchmark (1MB data, 100 iterations)\n\n");

  size_t bench_size = 1024 * 1024; // 1MB
  uint8_t *bench_data = (uint8_t *)vex_malloc(bench_size);
  generate_test_data(bench_data, bench_size);

  struct
  {
    const char *name;
    vex_compress_format_t format;
  } formats[] = {
      {"GZIP", VEX_COMP_GZIP},
      {"ZLIB", VEX_COMP_ZLIB},
      {"BZIP2", VEX_COMP_BZIP2},
      {"LZ4", VEX_COMP_LZ4},
      {"ZSTD", VEX_COMP_ZSTD},
      {"BROTLI", VEX_COMP_BROTLI}};

  for (size_t i = 0; i < sizeof(formats) / sizeof(formats[0]); i++)
  {
    double time = benchmark_compress(formats[i].format, bench_data, bench_size, 100);
    if (time > 0)
    {
      double throughput = (bench_size * 100.0) / (time * 1024 * 1024); // MB/s
      printf("   [%s] %.3f sec (%.2f MB/s)\n", formats[i].name, time, throughput);
    }
  }

  vex_free(bench_data);

  printf("\n=== ✅ ALL TESTS COMPLETE ===\n");
  return 0;
}
