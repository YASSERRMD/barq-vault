#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Compress `in_len` bytes at `input` using LZMA at the given `level` (1-9).
 * Output is written into `output` buffer of size `*out_len`.
 * On success, `*out_len` is updated to the actual compressed size.
 * Returns 0 on success, -1 on failure.
 */
int32_t barq_compress_lzma(const uint8_t *input, size_t in_len,
                           uint8_t *output, size_t *out_len,
                           uint32_t level);

/**
 * Decompress LZMA data from `input` (size `in_len`) into `output` (size `*out_len`).
 * On success, `*out_len` is updated to the actual decompressed size.
 * Returns 0 on success, -1 on failure.
 */
int32_t barq_decompress_lzma(const uint8_t *input, size_t in_len,
                             uint8_t *output, size_t *out_len);

/**
 * Compress `in_len` bytes at `input` using LZ4 default compression.
 * Output buffer must be at least `barq_estimate_lz4_bound(in_len)` bytes.
 * Returns the number of bytes written on success, -1 on failure.
 */
int32_t barq_compress_lz4(const uint8_t *input, size_t in_len,
                          uint8_t *output, size_t out_capacity);

/**
 * Decompress LZ4 data from `input` (size `in_len`) into `output` (size `out_capacity`).
 * Returns the number of bytes written on success, -1 on failure.
 */
int32_t barq_decompress_lz4(const uint8_t *input, size_t in_len,
                            uint8_t *output, size_t out_capacity);

/**
 * Delta-encode an array of `len` float32 values from `src` into `dst`.
 * `dst[0] = src[0]`, `dst[i] = src[i] - src[i-1]` for i > 0.
 * Uses AVX2 where available for batch processing.
 */
void barq_delta_encode_f32(const float *src, float *dst, size_t len);

/**
 * Reverse of `barq_delta_encode_f32`: prefix-sum to reconstruct original values.
 * Uses AVX2 where available.
 */
void barq_delta_decode_f32(const float *src, float *dst, size_t len);

/**
 * Returns a safe upper-bound estimate for the LZMA compressed output buffer size.
 */
size_t barq_estimate_lzma_bound(size_t in_len);

/**
 * Returns a safe upper-bound estimate for the LZ4 compressed output buffer size.
 */
size_t barq_estimate_lz4_bound(size_t in_len);

#ifdef __cplusplus
} /* extern "C" */
#endif
