use barq_types::{IngestRequest, SearchRequest, Modality, StorageMode};
use crate::barqvault::v1::{ProtoIngestRequest, ProtoSearchRequest};
use barq_test_utils::builders::random_embedding;
use uuid::Uuid;
use serde_json::json;

// ---- IngestRequest round-trip ----

#[test]
fn test_ingest_request_round_trip() {
    let emb = random_embedding(32);
    let original = IngestRequest {
        summary: "Test summary".to_string(),
        embedding: emb.clone(),
        modality: Modality::Text,
        storage_mode: StorageMode::TextOnly,
        filename: Some("doc.txt".to_string()),
        raw_payload: Some(b"some bytes".to_vec()),
        metadata: json!({"author": "test"}),
        chunk_index: 0,
        total_chunks: 1,
        parent_id: None,
    };

    let proto: ProtoIngestRequest = original.clone().into();
    let recovered = IngestRequest::try_from(proto).unwrap();

    assert_eq!(original.summary, recovered.summary);
    assert_eq!(original.filename, recovered.filename);
    assert_eq!(original.modality, recovered.modality);
    assert_eq!(original.storage_mode, recovered.storage_mode);
    assert_eq!(original.chunk_index, recovered.chunk_index);
    for (a, b) in original.embedding.iter().zip(recovered.embedding.iter()) {
        assert!((a - b).abs() < 1e-6, "Embedding mismatch: {} vs {}", a, b);
    }
}

#[test]
fn test_ingest_request_with_parent_id() {
    let parent = Uuid::new_v4();
    let original = IngestRequest {
        summary: "Child chunk".to_string(),
        embedding: random_embedding(16),
        modality: Modality::Document,
        storage_mode: StorageMode::HybridFile,
        filename: None,
        raw_payload: None,
        metadata: json!({}),
        chunk_index: 1,
        total_chunks: 3,
        parent_id: Some(parent),
    };

    let proto: ProtoIngestRequest = original.clone().into();
    let recovered = IngestRequest::try_from(proto).unwrap();

    assert_eq!(recovered.parent_id, Some(parent));
    assert_eq!(recovered.chunk_index, 1);
    assert_eq!(recovered.total_chunks, 3);
}

#[test]
fn test_ingest_request_invalid_embedding_bytes() {
    let mut proto = ProtoIngestRequest::default();
    proto.embedding = vec![0u8; 5]; // not divisible by 4
    proto.modality = "text".to_string();
    proto.storage_mode = "text_only".to_string();

    let result = IngestRequest::try_from(proto);
    assert!(result.is_err());
}

// ---- SearchRequest round-trip ----

#[test]
fn test_search_request_round_trip() {
    let emb = random_embedding(32);
    let original = SearchRequest {
        query_embedding: emb.clone(),
        query_text: "find similar".to_string(),
        vector_weight: 0.7,
        top_k: 5,
        modality_filter: Some(Modality::Image),
        metadata_filters: json!({"author": "alice"}),
    };

    let proto: ProtoSearchRequest = original.clone().into();
    let recovered = SearchRequest::try_from(proto).unwrap();

    assert_eq!(original.query_text, recovered.query_text);
    assert_eq!(original.top_k, recovered.top_k);
    assert!((original.vector_weight - recovered.vector_weight).abs() < 1e-6);
    assert_eq!(recovered.modality_filter, Some(Modality::Image));
}

#[test]
fn test_search_request_no_filter() {
    let original = SearchRequest {
        query_embedding: vec![],
        query_text: "simple search".to_string(),
        vector_weight: 0.5,
        top_k: 10,
        modality_filter: None,
        metadata_filters: json!({}),
    };

    let proto: ProtoSearchRequest = original.clone().into();
    let recovered = SearchRequest::try_from(proto).unwrap();

    assert!(recovered.modality_filter.is_none());
    assert_eq!(recovered.query_text, "simple search");
}
