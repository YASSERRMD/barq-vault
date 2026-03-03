// barq-vault: barq-compress

pub mod embedding;
pub mod lz4;
pub mod lzma;

// The sys module is shared between lzma and embedding — re-export for lz4
pub(crate) use lzma::sys;

#[cfg(test)]
mod tests;

use barq_types::{BarqError, BarqResult, Modality};

pub use embedding::{compress_embedding, decompress_embedding};

/// Codec dispatch enum — mirrors barq_types::CodecType but lives here
/// so that barq-compress carries no cyclic dependency.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Codec {
    Lzma(u32),
    Lz4,
    Zstd(i32),
}

/// Compress `data` using the specified codec.
pub fn compress(data: &[u8], codec: Codec) -> BarqResult<Vec<u8>> {
    match codec {
        Codec::Lzma(level) => lzma::compress_lzma(data, level),
        Codec::Lz4 => lz4::compress_lz4(data),
        Codec::Zstd(level) => zstd::bulk::compress(data, level as i32)
            .map_err(|e| BarqError::Compression(format!("Zstd compress: {}", e))),
    }
}

/// Decompress `data` using the specified codec.
///
/// `hint_size` provides an initial output buffer size for Zstd and LZMA.
pub fn decompress(data: &[u8], codec: Codec, hint_size: usize) -> BarqResult<Vec<u8>> {
    match codec {
        Codec::Lzma(_) => lzma::decompress_lzma(data, hint_size),
        Codec::Lz4 => lz4::decompress_lz4(data, hint_size),
        Codec::Zstd(_) => zstd::bulk::decompress(data, hint_size)
            .map_err(|e| BarqError::Compression(format!("Zstd decompress: {}", e))),
    }
}

/// Choose the optimal codec for a given modality.
///
/// - Text / Document → LZMA level 6 (maximum ratio for text)
/// - Audio           → LZMA level 4 (good ratio, faster encode)
/// - Image           → Zstd level 9 (low overhead for pre-compressed data)
/// - Video           → LZ4 (speed priority; video is near-incompressible)
pub fn select_codec_for_modality(modality: &Modality) -> Codec {
    match modality {
        Modality::Text | Modality::Document => Codec::Lzma(6),
        Modality::Audio => Codec::Lzma(4),
        Modality::Image => Codec::Zstd(9),
        Modality::Video => Codec::Lz4,
    }
}
