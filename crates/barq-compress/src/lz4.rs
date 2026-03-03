use crate::lzma::sys;
use barq_types::{BarqError, BarqResult};

/// Compress `data` using LZ4 default compression.
pub fn compress_lz4(data: &[u8]) -> BarqResult<Vec<u8>> {
    let capacity = unsafe { sys::barq_estimate_lz4_bound(data.len()) };
    let mut out = vec![0u8; capacity];

    let written = unsafe {
        sys::barq_compress_lz4(data.as_ptr(), data.len(), out.as_mut_ptr(), capacity)
    };

    if written < 0 {
        return Err(BarqError::Compression(
            "LZ4 compression failed".to_string(),
        ));
    }

    out.truncate(written as usize);
    Ok(out)
}

/// Decompress LZ4-compressed `data`.
///
/// `original_size` is required by LZ4 to allocate the output buffer.
pub fn decompress_lz4(data: &[u8], original_size: usize) -> BarqResult<Vec<u8>> {
    let mut out = vec![0u8; original_size];

    let written = unsafe {
        sys::barq_decompress_lz4(data.as_ptr(), data.len(), out.as_mut_ptr(), original_size)
    };

    if written < 0 {
        return Err(BarqError::Compression(
            "LZ4 decompression failed".to_string(),
        ));
    }

    out.truncate(written as usize);
    Ok(out)
}
