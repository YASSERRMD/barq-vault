#include "codec.h"
#include <lzma.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

int32_t barq_compress_lzma(const uint8_t *input, size_t in_len, uint8_t *output,
                           size_t *out_len, uint32_t level) {
  lzma_stream strm = LZMA_STREAM_INIT;
  lzma_ret ret = lzma_easy_encoder(&strm, level, LZMA_CHECK_CRC64);
  if (ret != LZMA_OK) {
    return -1;
  }

  strm.next_in = input;
  strm.avail_in = in_len;
  strm.next_out = output;
  strm.avail_out = *out_len;

  ret = lzma_code(&strm, LZMA_FINISH);
  if (ret != LZMA_STREAM_END) {
    lzma_end(&strm);
    return -1;
  }

  *out_len -= strm.avail_out;
  lzma_end(&strm);
  return 0;
}

int32_t barq_decompress_lzma(const uint8_t *input, size_t in_len,
                             uint8_t *output, size_t *out_len) {
  lzma_stream strm = LZMA_STREAM_INIT;
  lzma_ret ret = lzma_stream_decoder(&strm, UINT64_MAX, 0);
  if (ret != LZMA_OK) {
    return -1;
  }

  strm.next_in = input;
  strm.avail_in = in_len;
  strm.next_out = output;
  strm.avail_out = *out_len;

  ret = lzma_code(&strm, LZMA_FINISH);
  if (ret != LZMA_STREAM_END) {
    lzma_end(&strm);
    return -1;
  }

  *out_len -= strm.avail_out;
  lzma_end(&strm);
  return 0;
}

size_t barq_estimate_lzma_bound(size_t in_len) {
  /* Conservative upper bound: add 50% plus 128 bytes of overhead. */
  return in_len + in_len / 2 + 128;
}
