use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{CodecType, Modality, StorageMode};

/// Master record struct stored per chunk in barq-vault.
///
/// `embedding` and `compressed_payload` are in-memory only for
/// transit; the store strips them out before persisting to RocksDB.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarqRecord {
    /// Unique record ID (per chunk).
    pub id: Uuid,
    /// Parent document ID — set when a file is split into chunks.
    pub parent_id: Option<Uuid>,
    /// Zero-based index of this chunk within the parent document.
    pub chunk_index: u32,
    /// Total number of chunks the parent was split into.
    pub total_chunks: u32,

    /// Content modality.
    pub modality: Modality,
    /// How raw bytes are stored.
    pub storage_mode: StorageMode,
    /// Compression codec used for the payload.
    pub codec: CodecType,

    /// Original filename (if available).
    pub filename: Option<String>,
    /// Detected MIME type.
    pub mime_type: Option<String>,

    /// LLM-generated plaintext summary.
    pub summary: String,

    /// In-memory embedding vector (not persisted in raw form).
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub embedding: Vec<f32>,
    /// Delta+Zstd compressed embedding for storage.
    pub compressed_embed: Vec<u8>,
    /// Dimension of the embedding vector.
    pub embedding_dim: u32,

    /// BM25 token list, derived from `summary`.
    pub bm25_tokens: Vec<String>,

    /// Arbitrary key-value metadata tags.
    pub metadata: serde_json::Value,

    /// Compressed raw bytes of the original file (None for TextOnly mode).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub compressed_payload: Option<Vec<u8>>,

    /// Original (uncompressed) size in bytes.
    pub original_size: u64,
    /// Compressed size in bytes.
    pub compressed_size: u64,
    /// compression_ratio = original_size / compressed_size (>1.0 means compressed smaller).
    pub compression_ratio: f32,

    /// Unix timestamp (seconds) at record creation.
    pub created_at: i64,
    /// Unix timestamp (seconds) at last update.
    pub updated_at: i64,

    /// Blake3 hash of the original raw bytes.
    pub checksum: [u8; 32],
}

impl Default for BarqRecord {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            parent_id: None,
            chunk_index: 0,
            total_chunks: 1,
            modality: Modality::Text,
            storage_mode: StorageMode::TextOnly,
            codec: CodecType::Lzma(6),
            filename: None,
            mime_type: None,
            summary: String::new(),
            embedding: Vec::new(),
            compressed_embed: Vec::new(),
            embedding_dim: 0,
            bm25_tokens: Vec::new(),
            metadata: serde_json::Value::Object(serde_json::Map::new()),
            compressed_payload: None,
            original_size: 0,
            compressed_size: 0,
            compression_ratio: 1.0,
            created_at: 0,
            updated_at: 0,
            checksum: [0u8; 32],
        }
    }
}
