use async_trait::async_trait;
use barq_types::{BarqResult, Modality};

pub mod document;
pub mod ocr;
pub mod stt;
pub mod text;
pub mod vlm;

/// Trait for text extraction from raw file bytes.
#[async_trait]
pub trait TextExtractor: Send + Sync {
    /// Returns `true` if this extractor can handle the given modality.
    fn can_handle(&self, modality: &Modality) -> bool;

    /// Extract a text representation from raw bytes.
    async fn extract(&self, raw: &[u8], filename: &str) -> BarqResult<String>;
}
