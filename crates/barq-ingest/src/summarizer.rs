use std::time::Duration;

use barq_types::{BarqError, BarqResult, Modality};
use tracing::warn;

/// LLM provider choices.
#[derive(Debug, Clone)]
pub enum LlmProvider {
    OpenAi,
    Mistral,
    Gemini,
    Anthropic,
    Local,
}

/// LLM configuration for summarization.
#[derive(Debug, Clone)]
pub struct LlmConfig {
    pub provider: LlmProvider,
    pub api_key: Option<String>,
    pub model: String,
    pub base_url: Option<String>,
    pub max_summary_tokens: usize,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: LlmProvider::OpenAi,
            api_key: None,
            model: "gpt-4o-mini".to_string(),
            base_url: None,
            max_summary_tokens: 256,
        }
    }
}

fn summary_prompt(modality: &Modality) -> &'static str {
    match modality {
        Modality::Text | Modality::Document => {
            "Summarize this document concisely in 2-3 sentences. Capture main topics, key entities, and important facts."
        }
        Modality::Image => "Summarize what is shown in this image in 2-3 sentences.",
        Modality::Audio => {
            "Summarize the key content of this audio transcript in 2-3 sentences."
        }
        Modality::Video => "Summarize what happens in this video in 2-3 sentences.",
    }
}

/// Summarize `text` using the configured LLM provider.
pub async fn summarize(text: &str, modality: &Modality, config: &LlmConfig) -> BarqResult<String> {
    let prompt = summary_prompt(modality);
    let user_msg = format!("{}\n\n---\n\n{}", prompt, text);

    let mut last_err = None;
    for attempt in 0..3 {
        let result = match config.provider {
            LlmProvider::OpenAi => {
                call_openai_compat(
                    config.api_key.as_deref().unwrap_or(""),
                    config.base_url.as_deref().unwrap_or("https://api.openai.com"),
                    &config.model,
                    &user_msg,
                    config.max_summary_tokens,
                )
                .await
            }
            LlmProvider::Mistral => {
                call_openai_compat(
                    config.api_key.as_deref().unwrap_or(""),
                    config.base_url.as_deref().unwrap_or("https://api.mistral.ai"),
                    &config.model,
                    &user_msg,
                    config.max_summary_tokens,
                )
                .await
            }
            LlmProvider::Gemini => {
                call_gemini(
                    config.api_key.as_deref().unwrap_or(""),
                    &config.model,
                    &user_msg,
                )
                .await
            }
            LlmProvider::Anthropic => {
                call_anthropic(
                    config.api_key.as_deref().unwrap_or(""),
                    &config.model,
                    &user_msg,
                    config.max_summary_tokens,
                )
                .await
            }
            LlmProvider::Local => {
                call_openai_compat(
                    "none",
                    config.base_url.as_deref().unwrap_or("http://localhost:11434"),
                    &config.model,
                    &user_msg,
                    config.max_summary_tokens,
                )
                .await
            }
        };

        match result {
            Ok(summary) => return Ok(summary),
            Err(e) => {
                warn!("LLM attempt {}: {}", attempt + 1, e);
                last_err = Some(e);
                tokio::time::sleep(Duration::from_millis(500 * 2u64.pow(attempt))).await;
            }
        }
    }

    Err(last_err.unwrap_or_else(|| {
        BarqError::ProviderError("Summarizer: all retries failed".to_string())
    }))
}

async fn call_openai_compat(
    api_key: &str,
    base_url: &str,
    model: &str,
    prompt: &str,
    max_tokens: usize,
) -> BarqResult<String> {
    let client = reqwest::Client::new();
    let url = format!("{}/v1/chat/completions", base_url.trim_end_matches('/'));
    let body = serde_json::json!({
        "model": model,
        "messages": [{ "role": "user", "content": prompt }],
        "max_tokens": max_tokens
    });

    let resp: serde_json::Value = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body)
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| BarqError::ProviderError(format!("LLM HTTP: {}", e)))?
        .json()
        .await
        .map_err(|e| BarqError::ProviderError(format!("LLM JSON: {}", e)))?;

    resp["choices"][0]["message"]["content"]
        .as_str()
        .map(|s| s.trim().to_string())
        .ok_or_else(|| BarqError::ProviderError("No content in LLM response".to_string()))
}

async fn call_gemini(api_key: &str, model: &str, prompt: &str) -> BarqResult<String> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model, api_key
    );
    let body = serde_json::json!({
        "contents": [{ "parts": [{ "text": prompt }] }]
    });

    let resp: serde_json::Value = client
        .post(&url)
        .json(&body)
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| BarqError::ProviderError(format!("Gemini: {}", e)))?
        .json()
        .await
        .map_err(|e| BarqError::ProviderError(format!("Gemini JSON: {}", e)))?;

    Ok(resp["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .unwrap_or("")
        .trim()
        .to_string())
}

async fn call_anthropic(
    api_key: &str,
    model: &str,
    prompt: &str,
    max_tokens: usize,
) -> BarqResult<String> {
    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "model": model,
        "max_tokens": max_tokens,
        "messages": [{ "role": "user", "content": prompt }]
    });

    let resp: serde_json::Value = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&body)
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| BarqError::ProviderError(format!("Anthropic: {}", e)))?
        .json()
        .await
        .map_err(|e| BarqError::ProviderError(format!("Anthropic JSON: {}", e)))?;

    Ok(resp["content"][0]["text"]
        .as_str()
        .unwrap_or("")
        .trim()
        .to_string())
}
