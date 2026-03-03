#include "codec.h"
#include <lz4.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#ifdef __AVX2__
#include <immintrin.h>
#endif

int32_t barq_compress_lz4(const uint8_t *input, size_t in_len, uint8_t *output,
                          size_t out_capacity) {
  int result = LZ4_compress_default((const char *)input, (char *)output,
                                    (int)in_len, (int)out_capacity);
  return (result <= 0) ? -1 : result;
}

int32_t barq_decompress_lz4(const uint8_t *input, size_t in_len,
                            uint8_t *output, size_t out_capacity) {
  int result = LZ4_decompress_safe((const char *)input, (char *)output,
                                   (int)in_len, (int)out_capacity);
  return (result < 0) ? -1 : result;
}

size_t barq_estimate_lz4_bound(size_t in_len) {
  return (size_t)LZ4_compressBound((int)in_len);
}

void barq_delta_encode_f32(const float *src, float *dst, size_t len) {
  if (len == 0)
    return;

  dst[0] = src[0];

#ifdef __AVX2__
  size_t i = 1;
  /* Process 8 floats at a time using AVX2. */
  for (; i + 8 <= len; i += 8) {
    __m256 cur = _mm256_loadu_ps(src + i);
    __m256 prev = _mm256_loadu_ps(src + i - 1);
    __m256 diff = _mm256_sub_ps(cur, prev);
    _mm256_storeu_ps(dst + i, diff);
  }
  /* Scalar fallback for remaining elements. */
  for (; i < len; i++) {
    dst[i] = src[i] - src[i - 1];
  }
#else
  for (size_t i = 1; i < len; i++) {
    dst[i] = src[i] - src[i - 1];
  }
#endif
}

void barq_delta_decode_f32(const float *src, float *dst, size_t len) {
  if (len == 0)
    return;

  dst[0] = src[0];

#ifdef __AVX2__
  /*
   * Prefix-sum cannot be trivially vectorised the same way as delta-encode
   * because each output depends on the previous accumulated value.
   * We use a block-scan approach: compute partial sums within 8-wide chunks
   * then fix up the inter-chunk carries.
   */
  size_t i = 1;
  float carry = dst[0];
  for (; i + 8 <= len; i += 8) {
    float block[8];
    for (int j = 0; j < 8; j++) {
      carry += src[i + j];
      block[j] = carry;
    }
    _mm256_storeu_ps(dst + i, _mm256_loadu_ps(block));
  }
  for (; i < len; i++) {
    dst[i] = dst[i - 1] + src[i];
  }
#else
  for (size_t i = 1; i < len; i++) {
    dst[i] = dst[i - 1] + src[i];
  }
#endif
}
