use async_trait::async_trait;
use base64::Engine;
use barq_types::{BarqError, BarqResult, Modality};
use tokio::process::Command;

use super::TextExtractor;

/// STT provider configuration.
#[derive(Debug, Clone)]
pub enum SttProvider {
    WhisperLocal,
    OpenAiWhisper,
}

/// Configuration for speech-to-text extraction.
#[derive(Debug, Clone)]
pub struct SttConfig {
    pub provider: SttProvider,
    pub api_key: Option<String>,
    pub model: String,
    pub whisper_bin_path: Option<String>,
}

impl Default for SttConfig {
    fn default() -> Self {
        Self {
            provider: SttProvider::OpenAiWhisper,
            api_key: None,
            model: "whisper-1".to_string(),
            whisper_bin_path: None,
        }
    }
}

/// Extracts transcriptions from audio files via Whisper (local or OpenAI API).
pub struct SttExtractor {
    pub config: SttConfig,
}

impl SttExtractor {
    pub fn new(config: SttConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl TextExtractor for SttExtractor {
    fn can_handle(&self, modality: &Modality) -> bool {
        matches!(modality, Modality::Audio)
    }

    async fn extract(&self, raw: &[u8], filename: &str) -> BarqResult<String> {
        match self.config.provider {
            SttProvider::WhisperLocal => transcribe_local(raw, filename, &self.config).await,
            SttProvider::OpenAiWhisper => transcribe_openai(raw, filename, &self.config).await,
        }
    }
}

async fn transcribe_local(raw: &[u8], filename: &str, config: &SttConfig) -> BarqResult<String> {
    use std::io::Write;

    let ext = std::path::Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{}", e))
        .unwrap_or_else(|| ".wav".to_string());

    let mut tmp = tempfile::Builder::new()
        .suffix(&ext)
        .tempfile()
        .map_err(|e| BarqError::Ingest(format!("STT tmpfile: {}", e)))?;
    tmp.write_all(raw)
        .map_err(|e| BarqError::Ingest(format!("STT write: {}", e)))?;

    let whisper_bin = config
        .whisper_bin_path
        .as_deref()
        .unwrap_or("whisper");

    let output = Command::new(whisper_bin)
        .arg(tmp.path())
        .arg("--output-txt")
        .arg("--output-file")
        .arg("/dev/stdout")
        .output()
        .await
        .map_err(|e| BarqError::Ingest(format!("whisper binary error: {}", e)))?;

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

async fn transcribe_openai(raw: &[u8], filename: &str, config: &SttConfig) -> BarqResult<String> {
    let api_key = config
        .api_key
        .as_deref()
        .ok_or_else(|| BarqError::Ingest("OpenAI API key required for STT".to_string()))?;

    let file_part = reqwest::multipart::Part::bytes(raw.to_vec())
        .file_name(filename.to_string())
        .mime_str("audio/mpeg")
        .map_err(|e| BarqError::Ingest(format!("multipart: {}", e)))?;

    let form = reqwest::multipart::Form::new()
        .part("file", file_part)
        .text("model", config.model.clone());

    let client = reqwest::Client::new();
    let resp = client
        .post("https://api.openai.com/v1/audio/transcriptions")
        .header("Authorization", format!("Bearer {}", api_key))
        .multipart(form)
        .timeout(std::time::Duration::from_secs(60))
        .send()
        .await
        .map_err(|e| BarqError::ProviderError(format!("OpenAI STT: {}", e)))?
        .json::<serde_json::Value>()
        .await
        .map_err(|e| BarqError::ProviderError(format!("OpenAI STT parse: {}", e)))?;

    resp["text"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| BarqError::ProviderError("No transcript in OpenAI response".to_string()))
}
