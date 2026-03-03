use crate::embedder::{embed, EmbedConfig, EmbedProvider};
use barq_test_utils::mocks::MockLlmServer;
use barq_test_utils::builders::random_embedding;

#[tokio::test]
async fn test_embed_local_provider_returns_zeros() {
    let config = EmbedConfig {
        provider: EmbedProvider::Local,
        api_key: None,
        model: "local".to_string(),
        base_url: None,
        expected_dim: 128,
    };

    let result = embed("test text", &config).await;
    assert!(result.is_ok());
    let emb = result.unwrap();
    assert_eq!(emb.len(), 128);
    assert!(emb.iter().all(|&v| v == 0.0));
}

#[tokio::test]
async fn test_embed_openai_compat_mock() {
    let mock = MockLlmServer::start().await;
    let fake_emb = random_embedding(384);
    mock.mock_openai_embeddings(fake_emb.clone()).await;

    let config = EmbedConfig {
        provider: EmbedProvider::OpenAi,
        api_key: Some("test-key".to_string()),
        model: "text-embedding-3-small".to_string(),
        base_url: Some(mock.base_url()),
        expected_dim: 384,
    };

    let result = embed("hello world", &config).await;
    assert!(result.is_ok(), "Expected Ok got: {:?}", result.err());
    let emb = result.unwrap();
    assert_eq!(emb.len(), 384);
}

#[tokio::test]
async fn test_embed_mistral_mock() {
    let mock = MockLlmServer::start().await;
    let fake_emb = random_embedding(1024);
    mock.mock_openai_embeddings(fake_emb.clone()).await;

    let config = EmbedConfig {
        provider: EmbedProvider::Mistral,
        api_key: Some("test-key".to_string()),
        model: "mistral-embed".to_string(),
        base_url: Some(mock.base_url()),
        expected_dim: 1024,
    };

    let result = embed("embed this text", &config).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 1024);
}

#[tokio::test]
async fn test_embed_dimension_mismatch_returns_error() {
    let mock = MockLlmServer::start().await;
    // Return 8-dim embedding but config expects 384
    let small_emb: Vec<f32> = vec![0.1; 8];
    mock.mock_openai_embeddings(small_emb).await;

    let config = EmbedConfig {
        provider: EmbedProvider::OpenAi,
        api_key: Some("test-key".to_string()),
        model: "text-embedding-3-small".to_string(),
        base_url: Some(mock.base_url()),
        expected_dim: 384,
    };

    let result = embed("hello", &config).await;
    assert!(result.is_err(), "Expected dimension mismatch error");
}
