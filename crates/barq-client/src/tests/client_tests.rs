use barq_types::{SearchResult, Modality};
use uuid::Uuid;
use serde_json::json;

/// Test that BarqClient::connect rejects invalid URLs (scheme not http/https)
#[tokio::test]
async fn test_connect_invalid_url_returns_error() {
    let result = crate::client::BarqClient::connect("not_a_valid_url").await;
    assert!(result.is_err(), "Expected error for invalid URL");
}

/// Test that BarqClient::connect with a valid scheme but unreachable host gives a clear error
/// (does not panic or hang — connection failure returns Err quickly with lazy_connect not available)
#[tokio::test]
async fn test_connect_unreachable_host_scheme_is_valid() {
    // tonic channel construction is lazy: from_shared succeeds but connect may fail
    // This verifies the code path doesn't panic
    let result = crate::client::BarqClient::connect("http://127.0.0.1:1").await;
    // Connection to port 1 should fail
    assert!(result.is_err(), "Expected connection error for unreachable host");
}

/// Test SearchResult field mapping (unit test of the data struct itself)
#[test]
fn test_search_result_fields() {
    let id = Uuid::new_v4();
    let result = SearchResult {
        id,
        summary: "Test summary".to_string(),
        filename: Some("doc.pdf".to_string()),
        modality: Modality::Document,
        score: 0.95,
        has_payload: true,
        metadata: json!({"key": "value"}),
    };

    assert_eq!(result.id, id);
    assert_eq!(result.modality, Modality::Document);
    assert!((result.score - 0.95).abs() < 1e-6);
    assert_eq!(result.filename, Some("doc.pdf".to_string()));
    assert!(result.has_payload);
}

/// Test that SearchResult can be serialized to JSON
#[test]
fn test_search_result_serializable() {
    let result = SearchResult {
        id: Uuid::new_v4(),
        summary: "JSON test".to_string(),
        filename: None,
        modality: Modality::Text,
        score: 0.5,
        has_payload: false,
        metadata: json!({}),
    };

    let json = serde_json::to_string(&result);
    assert!(json.is_ok(), "SearchResult should serialize to JSON");
    let str = json.unwrap();
    assert!(str.contains("JSON test"));
}

/// Verify that modality string parsing works consistently with what the client would receive
#[test]
fn test_modality_parse_from_proto_string() {
    let modalities = ["text", "document", "image", "audio", "video"];
    for m in &modalities {
        let parsed: Result<Modality, _> = m.parse();
        assert!(parsed.is_ok(), "Failed to parse modality: {}", m);
    }
}
