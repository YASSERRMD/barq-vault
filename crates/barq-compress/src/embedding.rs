use crate::lzma::sys;
use barq_types::{BarqError, BarqResult};
use bytemuck::cast_slice;

/// Compress an embedding vector using delta-f32 encoding followed by Zstd.
///
/// The delta step decorrelates adjacent float values, significantly improving
/// Zstd's compression ratio on dense embedding vectors.
pub fn compress_embedding(embedding: &[f32]) -> BarqResult<Vec<u8>> {
    let len = embedding.len();
    let mut delta_buf = vec![0f32; len];

    unsafe {
        sys::barq_delta_encode_f32(
            embedding.as_ptr(),
            delta_buf.as_mut_ptr(),
            len,
        );
    }

    // Cast &[f32] → &[u8] for Zstd (bytemuck ensures valid alignment)
    let raw_bytes: &[u8] = cast_slice(&delta_buf);

    let compressed = zstd::bulk::compress(raw_bytes, 3)
        .map_err(|e| BarqError::Compression(format!("Zstd compress error: {}", e)))?;

    Ok(compressed)
}

/// Decompress a delta-Zstd encoded embedding back to a Vec<f32>.
///
/// `dim` must match the original embedding dimension.
pub fn decompress_embedding(data: &[u8], dim: usize) -> BarqResult<Vec<f32>> {
    // Decompress Zstd → raw bytes
    let raw = zstd::bulk::decompress(data, dim * std::mem::size_of::<f32>())
        .map_err(|e| BarqError::Compression(format!("Zstd decompress error: {}", e)))?;

    // Cast &[u8] → &[f32]
    let delta_slice: &[f32] = cast_slice(&raw);

    // Reverse delta encoding
    let mut out = vec![0f32; dim];
    unsafe {
        sys::barq_delta_decode_f32(delta_slice.as_ptr(), out.as_mut_ptr(), dim);
    }

    Ok(out)
}
