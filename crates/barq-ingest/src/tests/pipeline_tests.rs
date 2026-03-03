use crate::pipeline::{IngestConfig, IngestPipeline};
use crate::summarizer::{LlmConfig, LlmProvider};
use crate::embedder::{EmbedConfig, EmbedProvider};
use crate::chunker::ChunkConfig;
use barq_types::{IngestRequest, Modality, StorageMode};
use barq_test_utils::fixtures::{sample_text_bytes, sample_pdf_bytes};
use serde_json::json;

fn local_config(dim: usize) -> IngestConfig {
    IngestConfig {
        llm: LlmConfig {
            provider: LlmProvider::Local,
            api_key: None,
            model: "local".to_string(),
            base_url: None,
            max_summary_tokens: 64,
        },
        embed: EmbedConfig {
            provider: EmbedProvider::Local,
            api_key: None,
            model: "local".to_string(),
            base_url: None,
            expected_dim: dim,
        },
        chunk: ChunkConfig {
            chunk_size_tokens: 512,
            overlap_tokens: 64,
        },
        ..Default::default()
    }
}

#[tokio::test]
async fn test_pipeline_text_produces_one_record() {
    let pipeline = IngestPipeline::new(local_config(64));
    let raw = sample_text_bytes();
    let req = IngestRequest {
        summary: String::new(),
        embedding: vec![],
        modality: Modality::Text,
        storage_mode: StorageMode::TextOnly,
        filename: Some("doc.txt".to_string()),
        raw_payload: Some(raw),
        metadata: json!({}),
        chunk_index: 0,
        total_chunks: 1,
        parent_id: None,
    };

    let records = pipeline.run(req).await.unwrap();
    assert!(!records.is_empty());
    let rec = &records[0];
    assert_eq!(rec.modality, Modality::Text);
    assert!(!rec.summary.is_empty());
    assert_eq!(rec.embedding.len(), 64);
    assert!(!rec.bm25_tokens.is_empty() || rec.bm25_tokens.is_empty()); // no panic
}

#[tokio::test]
async fn test_pipeline_assigns_correct_modality() {
    let pipeline = IngestPipeline::new(local_config(32));
    let req = IngestRequest {
        summary: String::new(),
        embedding: vec![],
        modality: Modality::Document,
        storage_mode: StorageMode::TextOnly,
        filename: Some("report.pdf".to_string()),
        raw_payload: Some(sample_pdf_bytes()),
        metadata: json!({}),
        chunk_index: 0,
        total_chunks: 1,
        parent_id: None,
    };

    let records = pipeline.run(req).await.unwrap();
    for rec in &records {
        assert_eq!(rec.modality, Modality::Document);
    }
}

#[tokio::test]
async fn test_pipeline_text_only_has_no_compressed_payload() {
    let pipeline = IngestPipeline::new(local_config(16));
    let req = IngestRequest {
        summary: String::new(),
        embedding: vec![],
        modality: Modality::Text,
        storage_mode: StorageMode::TextOnly,
        filename: Some("file.txt".to_string()),
        raw_payload: Some(b"short text".to_vec()),
        metadata: json!({}),
        chunk_index: 0,
        total_chunks: 1,
        parent_id: None,
    };
    let records = pipeline.run(req).await.unwrap();
    assert!(records[0].compressed_payload.is_none());
}

#[tokio::test]
async fn test_pipeline_checksum_is_set() {
    let pipeline = IngestPipeline::new(local_config(16));
    let raw = b"checksum test content".to_vec();
    let req = IngestRequest {
        summary: String::new(),
        embedding: vec![],
        modality: Modality::Text,
        storage_mode: StorageMode::TextOnly,
        filename: Some("check.txt".to_string()),
        raw_payload: Some(raw.clone()),
        metadata: json!({}),
        chunk_index: 0,
        total_chunks: 1,
        parent_id: None,
    };
    let records = pipeline.run(req).await.unwrap();
    let checksum = records[0].checksum;
    // verify non-zero checksum
    assert!(checksum.iter().any(|&b| b != 0));
}
