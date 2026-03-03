use async_trait::async_trait;
use barq_types::{BarqResult, Modality};

use super::TextExtractor;

/// Extracts text from plain text files via UTF-8 decoding.
pub struct PlainTextExtractor;

#[async_trait]
impl TextExtractor for PlainTextExtractor {
    fn can_handle(&self, modality: &Modality) -> bool {
        matches!(modality, Modality::Text)
    }

    async fn extract(&self, raw: &[u8], _filename: &str) -> BarqResult<String> {
        Ok(String::from_utf8_lossy(raw).into_owned())
    }
}
