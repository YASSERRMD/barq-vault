use uuid::Uuid;

/// All errors that can occur within the barq-vault system.
#[derive(Debug, thiserror::Error)]
pub enum BarqError {
    /// Compression or decompression failed.
    #[error("Compression error: {0}")]
    Compression(String),

    /// RocksDB or storage layer failure.
    #[error("Storage error: {0}")]
    Storage(String),

    /// Index operation failure.
    #[error("Index error: {0}")]
    Index(String),

    /// Ingestion pipeline failure.
    #[error("Ingest error: {0}")]
    Ingest(String),

    /// Requested record was not found.
    #[error("Record not found: {0}")]
    NotFound(Uuid),

    /// Bad or malformed input data.
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// External LLM, embedding, or media provider API failure.
    #[error("Provider error: {0}")]
    ProviderError(String),

    /// Underlying I/O error.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON serialization/deserialization error.
    #[error("Serde error: {0}")]
    SerdeError(#[from] serde_json::Error),
}

/// Convenience alias for `Result<T, BarqError>`.
pub type BarqResult<T> = Result<T, BarqError>;
