use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};
use serde_json::json;

pub struct MockLlmServer {
    server: MockServer,
}

impl MockLlmServer {
    /// Starts a new wiremock server on a random local port.
    pub async fn start() -> Self {
        let server = MockServer::start().await;
        Self { server }
    }

    /// Returns the base URL of the mock server.
    pub fn base_url(&self) -> String {
        self.server.uri()
    }

    /// Asserts that the exact number of requests were received by the server.
    pub async fn assert_received_calls(&self, count: usize) {
        let requests = self.server.received_requests().await.unwrap_or_default();
        assert_eq!(requests.len(), count, "Expected {} requests, got {}", count, requests.len());
    }

    /// Mocks OpenAI /v1/chat/completions
    pub async fn mock_openai_chat(&self, response_text: &str) {
        let body = json!({
            "choices": [{
                "message": { "content": response_text }
            }]
        });
        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .mount(&self.server)
            .await;
    }

    /// Mocks OpenAI /v1/embeddings
    pub async fn mock_openai_embeddings(&self, embedding: Vec<f32>) {
        let body = json!({
            "data": [{ "embedding": embedding }]
        });
        Mock::given(method("POST"))
            .and(path("/v1/embeddings"))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .mount(&self.server)
            .await;
    }

    /// Mocks Mistral /v1/chat/completions
    pub async fn mock_mistral_chat(&self, response_text: &str) {
        let body = json!({
            "choices": [{
                "message": { "content": response_text }
            }]
        });
        Mock::given(method("POST"))
            .and(path("/v1/chat/completions")) // Mistral API is virtually identical to OpenAI's
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .mount(&self.server)
            .await;
    }

    /// Mocks Gemini Generate Content endpoint (v1beta/models/...:generateContent)
    pub async fn mock_gemini_generate(&self, response_text: &str) {
        let body = json!({
            "candidates": [{
                "content": {
                    "parts": [{ "text": response_text }]
                }
            }]
        });
        // Match any path ending with generateContent
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .mount(&self.server)
            .await;
    }

    /// Mocks Ollama /api/generate
    pub async fn mock_ollama_generate(&self, response_text: &str) {
        let body = json!({
            "response": response_text
        });
        Mock::given(method("POST"))
            .and(path("/api/generate"))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .mount(&self.server)
            .await;
    }

    /// Mocks Ollama /api/embeddings
    pub async fn mock_ollama_embeddings(&self, embedding: Vec<f32>) {
        let body = json!({
            "embedding": embedding
        });
        Mock::given(method("POST"))
            .and(path("/api/embeddings"))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .mount(&self.server)
            .await;
    }
}
