use async_trait::async_trait;
use barq_types::{BarqError, BarqResult, Modality};
use tokio::process::Command;

use super::TextExtractor;

/// Extracts text from images using the tesseract OCR binary.
pub struct OcrExtractor;

#[async_trait]
impl TextExtractor for OcrExtractor {
    fn can_handle(&self, modality: &Modality) -> bool {
        matches!(modality, Modality::Image)
    }

    async fn extract(&self, raw: &[u8], _filename: &str) -> BarqResult<String> {
        use std::io::Write;

        // Write image bytes to a temp file
        let mut tmp = tempfile::NamedTempFile::new()
            .map_err(|e| BarqError::Ingest(format!("OCR tmpfile: {}", e)))?;
        tmp.write_all(raw)
            .map_err(|e| BarqError::Ingest(format!("OCR write: {}", e)))?;
        let tmp_path = tmp.path().to_path_buf();

        let output = Command::new("tesseract")
            .arg(&tmp_path)
            .arg("stdout")
            .output()
            .await;

        match output {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Ok("[Image: OCR not available — install tesseract]".to_string())
            }
            Err(e) => Err(BarqError::Ingest(format!("tesseract error: {}", e))),
            Ok(out) => {
                let text = String::from_utf8_lossy(&out.stdout);
                let cleaned: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
                if cleaned.len() < 20 {
                    Ok("[Image: no readable text detected]".to_string())
                } else {
                    Ok(cleaned)
                }
            }
        }
    }
}
