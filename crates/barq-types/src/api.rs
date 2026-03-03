use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{Modality, StorageMode};

/// Request to ingest a new file or chunk into the vault.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestRequest {
    /// LLM-generated summary of this chunk.
    pub summary: String,
    /// Embedding vector generated from the summary.
    pub embedding: Vec<f32>,
    /// Content modality.
    pub modality: Modality,
    /// Storage mode determining whether raw bytes are stored.
    pub storage_mode: StorageMode,
    /// Original filename, if any.
    pub filename: Option<String>,
    /// Raw bytes for storage (None when storage_mode = TextOnly).
    pub raw_payload: Option<Vec<u8>>,
    /// Arbitrary metadata tags.
    pub metadata: serde_json::Value,
    /// Zero-based chunk index.
    pub chunk_index: u32,
    /// Total number of chunks for this document.
    pub total_chunks: u32,
    /// Parent document ID (when chunking a larger file).
    pub parent_id: Option<Uuid>,
}

/// Response after a successful ingest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestResponse {
    /// ID of the newly created record.
    pub id: Uuid,
    /// Whether the ingest succeeded.
    pub success: bool,
    /// Size of compressed payload in bytes.
    pub compressed_size: u64,
    /// compression_ratio = original_size / compressed_size.
    pub compression_ratio: f32,
    /// Error message if `success` is false.
    pub error: Option<String>,
}

/// Request to perform a hybrid search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    /// Query embedding vector.
    pub query_embedding: Vec<f32>,
    /// Raw query text (used for BM25).
    pub query_text: String,
    /// Weight of vector score vs BM25 (0.0 = pure BM25, 1.0 = pure vector).
    pub vector_weight: f32,
    /// Maximum number of results to return.
    pub top_k: usize,
    /// Optional modality filter.
    pub modality_filter: Option<Modality>,
    /// Optional metadata key-value filter as JSON object.
    pub metadata_filters: serde_json::Value,
}

/// A single search result entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Record ID.
    pub id: Uuid,
    /// LLM-generated summary.
    pub summary: String,
    /// Original filename.
    pub filename: Option<String>,
    /// Content modality.
    pub modality: Modality,
    /// Hybrid fusion score.
    pub score: f32,
    /// Whether a raw payload is available for fetch.
    pub has_payload: bool,
    /// Record metadata.
    pub metadata: serde_json::Value,
}

/// Aggregated search response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total_found: usize,
    /// Query latency in milliseconds.
    pub took_ms: u64,
}

/// Request to fetch the raw payload of a record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchRequest {
    /// ID of the record to fetch.
    pub id: Uuid,
    /// Whether to decompress the payload before returning.
    pub decompress: bool,
}

/// Response containing the raw bytes for a record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchResponse {
    pub id: Uuid,
    pub modality: Modality,
    pub filename: Option<String>,
    /// Raw (or decompressed) bytes.
    pub data: Vec<u8>,
    pub original_size: u64,
    pub compressed_size: u64,
}
