use async_trait::async_trait;
use barq_types::{BarqError, BarqResult, Modality};
use tokio::process::Command;

use super::TextExtractor;

/// Extracts text from documents: PDF (via pdftotext), DOCX (XML unzip), others.
pub struct DocumentExtractor;

#[async_trait]
impl TextExtractor for DocumentExtractor {
    fn can_handle(&self, modality: &Modality) -> bool {
        matches!(modality, Modality::Document)
    }

    async fn extract(&self, raw: &[u8], filename: &str) -> BarqResult<String> {
        let ext = std::path::Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());

        match ext.as_deref() {
            Some("pdf") => extract_pdf(raw).await,
            Some("docx") => extract_docx(raw),
            _ => Ok(String::from_utf8_lossy(raw).into_owned()),
        }
    }
}

async fn extract_pdf(raw: &[u8]) -> BarqResult<String> {
    use std::io::Write;

    // Write to a temp file and call pdftotext
    let tmp = tempfile::NamedTempFile::new()
        .map_err(|e| BarqError::Ingest(format!("tmpfile: {}", e)))?;
    let tmp_path = tmp.path().to_path_buf();

    {
        let mut f = tmp.as_file();
        f.write_all(raw)
            .map_err(|e| BarqError::Ingest(format!("write pdf tmp: {}", e)))?;
    }

    let output = Command::new("pdftotext")
        .arg(&tmp_path)
        .arg("-")
        .output()
        .await;

    match output {
        Ok(out) if out.status.success() => {
            Ok(String::from_utf8_lossy(&out.stdout).into_owned())
        }
        Ok(_) | Err(_) => {
            // Graceful fallback: return raw UTF-8 attempt
            Ok(String::from_utf8_lossy(raw).into_owned())
        }
    }
}

fn extract_docx(raw: &[u8]) -> BarqResult<String> {
    use std::io::Read;
    let cursor = std::io::Cursor::new(raw);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| BarqError::Ingest(format!("DOCX zip error: {}", e)))?;

    let mut xml_content = String::new();
    if let Ok(mut file) = archive.by_name("word/document.xml") {
        file.read_to_string(&mut xml_content)
            .map_err(|e| BarqError::Ingest(format!("DOCX read error: {}", e)))?;
    } else {
        return Ok(String::from_utf8_lossy(raw).into_owned());
    }

    // Strip XML tags, collapse whitespace
    Ok(strip_xml_tags(&xml_content))
}

fn strip_xml_tags(xml: &str) -> String {
    let mut result = String::with_capacity(xml.len());
    let mut in_tag = false;

    for ch in xml.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            c if !in_tag => result.push(c),
            _ => {}
        }
    }

    // Collapse excessive whitespace
    result
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
