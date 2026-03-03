use async_trait::async_trait;
use base64::Engine as _;
use barq_types::{BarqError, BarqResult, Modality};
use tokio::process::Command;

use super::TextExtractor;
use crate::extraction::stt::{SttConfig, SttExtractor};

/// VLM provider choices.
#[derive(Debug, Clone)]
pub enum VlmProvider {
    Gemini,
    OpenAiVision,
    Local,
}

/// Configuration for the vision-language model extractor.
#[derive(Debug, Clone)]
pub struct VlmConfig {
    pub provider: VlmProvider,
    pub api_key: Option<String>,
    pub model: String,
    pub base_url: Option<String>,
}

impl Default for VlmConfig {
    fn default() -> Self {
        Self {
            provider: VlmProvider::OpenAiVision,
            api_key: None,
            model: "gpt-4o".to_string(),
            base_url: None,
        }
    }
}

pub struct VlmExtractor {
    pub config: VlmConfig,
    pub stt_config: SttConfig,
}

impl VlmExtractor {
    pub fn new(config: VlmConfig, stt_config: SttConfig) -> Self {
        Self { config, stt_config }
    }
}

#[async_trait]
impl TextExtractor for VlmExtractor {
    fn can_handle(&self, modality: &Modality) -> bool {
        matches!(modality, Modality::Image | Modality::Video)
    }

    async fn extract(&self, raw: &[u8], filename: &str) -> BarqResult<String> {
        let ext = std::path::Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());

        if matches!(
            ext.as_deref(),
            Some("mp4") | Some("mkv") | Some("avi") | Some("mov") | Some("webm")
        ) {
            self.extract_video(raw, filename).await
        } else {
            self.extract_image(raw).await
        }
    }
}

impl VlmExtractor {
    async fn extract_image(&self, raw: &[u8]) -> BarqResult<String> {
        let b64 = base64::engine::general_purpose::STANDARD.encode(raw);
        let prompt = "Describe this image in detail. Extract all visible text, objects, people, scenes, and relevant information.";
        call_vlm(&self.config, &b64, prompt).await
    }

    async fn extract_video(&self, raw: &[u8], filename: &str) -> BarqResult<String> {
        use std::io::Write;

        let ext = std::path::Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{}", e))
            .unwrap_or_else(|| ".mp4".to_string());

        let mut tmp_video = tempfile::Builder::new()
            .suffix(&ext)
            .tempfile()
            .map_err(|e| BarqError::Ingest(format!("video tmpfile: {}", e)))?;
        tmp_video
            .write_all(raw)
            .map_err(|e| BarqError::Ingest(format!("video write: {}", e)))?;
        let video_path = tmp_video.path().to_path_buf();

        // Extract 3 keyframes (first, middle, last)
        let frame_images = extract_keyframes(&video_path).await?;

        // Extract audio track and transcribe
        let audio_text = extract_audio_transcript(&video_path, &self.stt_config).await.unwrap_or_default();

        // Describe each frame
        let prompt = "Describe what happens in this video based on these keyframes. Include any visible text, actions, scenes.";
        let mut descriptions = Vec::new();
        for frame_bytes in &frame_images {
            let b64 = base64::engine::general_purpose::STANDARD.encode(frame_bytes);
            if let Ok(desc) = call_vlm(&self.config, &b64, prompt).await {
                descriptions.push(desc);
            }
        }

        let visual = descriptions.join(" | ");
        if audio_text.is_empty() {
            Ok(visual)
        } else {
            Ok(format!("[Visual]: {} [Audio]: {}", visual, audio_text))
        }
    }
}

async fn extract_keyframes(video_path: &std::path::Path) -> BarqResult<Vec<Vec<u8>>> {
    let tmp_dir = tempfile::tempdir()
        .map_err(|e| BarqError::Ingest(format!("tmpdir: {}", e)))?;

    // Extract 3 frames at 10%, 50%, 90% of duration
    let _out = Command::new("ffmpeg")
        .args([
            "-i", video_path.to_str().unwrap_or(""),
            "-vf", "select='eq(n\\,0)+eq(n\\,50)+eq(n\\,100)',setpts=N/FRAME_RATE/TB",
            "-vsync", "0",
            "-frames:v", "3",
            &format!("{}/frame_%03d.jpg", tmp_dir.path().display()),
        ])
        .output()
        .await
        .ok();

    let mut frames = Vec::new();
    for i in 1..=3 {
        let frame_path = tmp_dir.path().join(format!("frame_{:03}.jpg", i));
        if let Ok(bytes) = tokio::fs::read(&frame_path).await {
            frames.push(bytes);
        }
    }
    Ok(frames)
}

async fn extract_audio_transcript(
    video_path: &std::path::Path,
    stt_config: &SttConfig,
) -> BarqResult<String> {
    use std::io::Write;

    let tmp_audio = tempfile::Builder::new()
        .suffix(".wav")
        .tempfile()
        .map_err(|e| BarqError::Ingest(format!("audio tmpfile: {}", e)))?;

    let _ = Command::new("ffmpeg")
        .args([
            "-i", video_path.to_str().unwrap_or(""),
            "-vn", "-ar", "16000", "-ac", "1", "-f", "wav",
            tmp_audio.path().to_str().unwrap_or(""),
            "-y",
        ])
        .output()
        .await;

    let audio_bytes = tokio::fs::read(tmp_audio.path())
        .await
        .unwrap_or_default();
    if audio_bytes.is_empty() {
        return Ok(String::new());
    }

    let extractor = SttExtractor::new(stt_config.clone());
    extractor.extract(&audio_bytes, "audio.wav").await
}

async fn call_vlm(config: &VlmConfig, b64_image: &str, prompt: &str) -> BarqResult<String> {
    let client = reqwest::Client::new();

    match config.provider {
        VlmProvider::Gemini => {
            let api_key = config.api_key.as_deref().ok_or_else(|| {
                BarqError::ProviderError("Gemini API key required".to_string())
            })?;
            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
                config.model, api_key
            );
            let body = serde_json::json!({
                "contents": [{
                    "parts": [
                        { "text": prompt },
                        { "inline_data": { "mime_type": "image/jpeg", "data": b64_image } }
                    ]
                }]
            });
            let resp: serde_json::Value = client
                .post(&url)
                .json(&body)
                .timeout(std::time::Duration::from_secs(30))
                .send()
                .await
                .map_err(|e| BarqError::ProviderError(format!("Gemini: {}", e)))?
                .json()
                .await
                .map_err(|e| BarqError::ProviderError(format!("Gemini parse: {}", e)))?;
            Ok(resp["candidates"][0]["content"]["parts"][0]["text"]
                .as_str()
                .unwrap_or("")
                .to_string())
        }
        VlmProvider::OpenAiVision | VlmProvider::Local => {
            let api_key = config.api_key.as_deref().unwrap_or("none");
            let base_url = config
                .base_url
                .as_deref()
                .unwrap_or("https://api.openai.com");
            let url = format!("{}/v1/chat/completions", base_url);
            let body = serde_json::json!({
                "model": config.model,
                "messages": [{
                    "role": "user",
                    "content": [
                        { "type": "text", "text": prompt },
                        { "type": "image_url", "image_url": { "url": format!("data:image/jpeg;base64,{}", b64_image) } }
                    ]
                }],
                "max_tokens": 1024
            });
            let resp: serde_json::Value = client
                .post(&url)
                .header("Authorization", format!("Bearer {}", api_key))
                .json(&body)
                .timeout(std::time::Duration::from_secs(30))
                .send()
                .await
                .map_err(|e| BarqError::ProviderError(format!("VLM: {}", e)))?
                .json()
                .await
                .map_err(|e| BarqError::ProviderError(format!("VLM parse: {}", e)))?;
            Ok(resp["choices"][0]["message"]["content"]
                .as_str()
                .unwrap_or("")
                .to_string())
        }
    }
}
