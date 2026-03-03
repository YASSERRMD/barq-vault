use crate::summarizer::{summarize, LlmConfig, LlmProvider};
use barq_types::Modality;
use barq_test_utils::mocks::MockLlmServer;

#[tokio::test]
async fn test_summarize_openai_compat_mock() {
    let mock = MockLlmServer::start().await;
    mock.mock_openai_chat("This document discusses quarterly results.").await;

    let config = LlmConfig {
        provider: LlmProvider::OpenAi,
        api_key: Some("test-key".to_string()),
        model: "gpt-4o-mini".to_string(),
        base_url: Some(mock.base_url()),
        max_summary_tokens: 128,
    };

    let result = summarize("Q3 earnings increased by 15%.", &Modality::Text, &config).await;
    assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
    let summary = result.unwrap();
    assert!(!summary.is_empty());
}

#[tokio::test]
async fn test_summarize_mistral_mock() {
    let mock = MockLlmServer::start().await;
    mock.mock_mistral_chat("A concise audio summary.").await;

    let config = LlmConfig {
        provider: LlmProvider::Mistral,
        api_key: Some("test-key".to_string()),
        model: "mistral-small".to_string(),
        base_url: Some(mock.base_url()),
        max_summary_tokens: 128,
    };

    let result = summarize("Audio content here.", &Modality::Audio, &config).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_summarize_local_provider_no_http() {
    // Local provider short-circuits — no HTTP call needed
    let config = LlmConfig {
        provider: LlmProvider::Local,
        api_key: None,
        model: "local".to_string(),
        base_url: None,
        max_summary_tokens: 128,
    };

    let result = summarize("Any text here", &Modality::Text, &config).await;
    assert!(result.is_ok());
    let s = result.unwrap();
    assert!(!s.is_empty());
}

#[tokio::test]
async fn test_summarize_returns_non_empty_for_all_modalities() {
    let modalities = [
        Modality::Text,
        Modality::Document,
        Modality::Image,
        Modality::Audio,
        Modality::Video,
    ];

    for modality in &modalities {
        let config = LlmConfig {
            provider: LlmProvider::Local,
            api_key: None,
            model: "local".to_string(),
            base_url: None,
            max_summary_tokens: 128,
        };
        let result = summarize("sample content", modality, &config).await;
        assert!(result.is_ok(), "Failed for {:?}", modality);
        assert!(!result.unwrap().is_empty());
    }
}
