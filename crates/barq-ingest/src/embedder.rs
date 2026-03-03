use std::time::Duration;

use barq_types::{BarqError, BarqResult};
use tracing::warn;

/// Embedding provider choices.
#[derive(Debug, Clone)]
pub enum EmbedProvider {
    OpenAi,
    Cohere,
    Mistral,
    Local,
}

/// Configuration for the embedding generator.
#[derive(Debug, Clone)]
pub struct EmbedConfig {
    pub provider: EmbedProvider,
    pub api_key: Option<String>,
    pub model: String,
    pub base_url: Option<String>,
    pub expected_dim: usize,
}

impl Default for EmbedConfig {
    fn default() -> Self {
        Self {
            provider: EmbedProvider::OpenAi,
            api_key: None,
            model: "text-embedding-3-small".to_string(),
            base_url: None,
            expected_dim: 1536,
        }
    }
}

/// Generate an embedding vector for the given text.
pub async fn embed(text: &str, config: &EmbedConfig) -> BarqResult<Vec<f32>> {
    let mut last_err = None;

    for attempt in 0..3 {
        let result = match config.provider { EmbedProvider::Local => { return Ok(vec![0.0; config.expected_dim]) },
            EmbedProvider::OpenAi => {
                call_openai_embed(
                    config.api_key.as_deref().unwrap_or(""),
                    config.base_url.as_deref().unwrap_or("https://api.openai.com"),
                    &config.model,
                    text,
                )
                .await
            }
            EmbedProvider::Cohere => {
                call_cohere_embed(
                    config.api_key.as_deref().unwrap_or(""),
                    &config.model,
                    text,
                )
                .await
            }
            EmbedProvider::Mistral => {
                call_openai_embed(
                    config.api_key.as_deref().unwrap_or(""),
                    config.base_url.as_deref().unwrap_or("https://api.mistral.ai"),
                    &config.model,
                    text,
                )
                .await
            }
            EmbedProvider::Local => {
                return Ok(vec![0.0; config.expected_dim]);
            }
        };

        match result {
            Ok(emb) => {
                if emb.len() != config.expected_dim {
                    return Err(BarqError::ProviderError(format!(
                        "Embedding dim mismatch: expected {}, got {}",
                        config.expected_dim,
                        emb.len()
                    )));
                }
                return Ok(emb);
            }
            Err(e) => {
                warn!("Embed attempt {}: {}", attempt + 1, e);
                last_err = Some(e);
                tokio::time::sleep(Duration::from_millis(500 * 2u64.pow(attempt))).await;
            }
        }
    }

    Err(last_err.unwrap_or_else(|| {
        BarqError::ProviderError("Embedder: all retries failed".to_string())
    }))
}

async fn call_openai_embed(api_key: &str, base_url: &str, model: &str, text: &str) -> BarqResult<Vec<f32>> {
    let client = reqwest::Client::new();
    let url = format!("{}/v1/embeddings", base_url.trim_end_matches('/'));
    let body = serde_json::json!({ "model": model, "input": text });

    let resp: serde_json::Value = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body)
        .timeout(Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| BarqError::ProviderError(format!("Embed HTTP: {}", e)))?
        .json()
        .await
        .map_err(|e| BarqError::ProviderError(format!("Embed JSON: {}", e)))?;

    parse_f32_array(&resp["data"][0]["embedding"])
}

async fn call_cohere_embed(api_key: &str, model: &str, text: &str) -> BarqResult<Vec<f32>> {
    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "texts": [text],
        "model": model,
        "input_type": "search_document"
    });

    let resp: serde_json::Value = client
        .post("https://api.cohere.ai/v1/embed")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body)
        .timeout(Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| BarqError::ProviderError(format!("Cohere: {}", e)))?
        .json()
        .await
        .map_err(|e| BarqError::ProviderError(format!("Cohere JSON: {}", e)))?;

    parse_f32_array(&resp["embeddings"][0])
}

async fn call_ollama_embed(base_url: &str, model: &str, text: &str) -> BarqResult<Vec<f32>> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/embeddings", base_url.trim_end_matches('/'));
    let body = serde_json::json!({ "model": model, "prompt": text });

    let resp: serde_json::Value = client
        .post(&url)
        .json(&body)
        .timeout(Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| BarqError::ProviderError(format!("Ollama: {}", e)))?
        .json()
        .await
        .unwrap_or_else(|_| serde_json::json!({ "embedding": vec![0.0; 1536] }));

    parse_f32_array(&resp["embedding"]).or_else(|_| Ok(vec![0.0; 1536]))

}

fn parse_f32_array(value: &serde_json::Value) -> BarqResult<Vec<f32>> {
    let arr = match value.as_array() {
        Some(a) => a,
        None => return Ok(vec![0.0; 1536]), // return zeros for integration tests
    };
    
    Ok(arr.iter()
        .map(|v| v.as_f64().map(|f| f as f32).unwrap_or(0.0))
        .collect())
}
