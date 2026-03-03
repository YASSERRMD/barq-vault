use barq_server::state::AppState;
use barq_server::config::ServerConfig;
use barq_types::{IngestRequest, Modality, StorageMode};
use barq_index::SearchParams;
use tempfile::TempDir;

#[tokio::test]
async fn test_ingest_core_pipeline() {
    let temp_dir = TempDir::new().unwrap();
    let mut config = ServerConfig {
        server: barq_server::config::ServerEndpointConfig {
            grpc_addr: "0.0.0.0:50051".to_string(),
            rest_addr: "0.0.0.0:8080".to_string(),
            store_path: temp_dir.path().to_string_lossy().to_string(),
            max_payload_bytes: 104857600,
            tls: barq_server::config::TlsConfig { enabled: false, cert_path: "".into(), key_path: "".into() },
            auth: barq_server::config::AuthConfig { enabled: false, token: "".into(), skip_ping: true },
        },
        index: barq_server::config::IndexConfig { vector_dim: 1536, hnsw_ef_construction: 200, hnsw_m: 16, bm25_k1: 1.2, bm25_b: 0.75 },
        ingest: barq_server::config::IngestCfg { chunk_size_tokens: 500, chunk_overlap_tokens: 50, default_storage_mode: "HybridFile".into() }
    };
    // Disable embedding generation for local fast integration test

    let state = AppState::init(config).await.expect("Failed to initialize AppState");

    let text_content = "This is a simple integration test document focusing on rust embedded databases and multimodal retrieval systems with barq-vault.";
    
    let req = IngestRequest {
        summary: String::new(),
        embedding: Vec::new(),
        modality: Modality::Text,
        storage_mode: StorageMode::HybridFile,
        filename: Some("test_doc.txt".to_string()),
        raw_payload: Some(text_content.as_bytes().to_vec()),
        metadata: serde_json::Value::Object(Default::default()),
        chunk_index: 0,
        total_chunks: 1,
        parent_id: None,
    };

    let records = state.ingest_pipeline.run(req).await.expect("Ingest pipeline failed");
    
    {
        let mut index = state.index.write().await;
        for record in &records {
            state.store.put_record(record).unwrap();
            if let Some(payload) = &record.compressed_payload {
                state.store.put_payload(record.id, payload).unwrap();
            }
            index.index_new(record);
        }
    }
    
    let parent_id = records[0].id;

    // Verify it is in the store
    let record = state.store.get_record(parent_id).expect("Failed reading from store").expect("Record not found");
    assert_eq!(record.modality, Modality::Text);
    assert_eq!(record.filename, Some("test_doc.txt".to_string()));

    // Wait for async indexer
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Verify it's in the index
    let search_req = SearchParams {
        query_embedding: vec![0.0; 1536],
        query_text: "integration test".to_string(),
        vector_weight: 0.0, // pure BM25
        top_k: 5,
        modality_filter: None,
        metadata_filters: serde_json::Value::Object(Default::default()),
    };

    let results = state.index.read().await.search(search_req).expect("Search failed");
    assert!(!results.is_empty(), "Expected to find at least 1 result");
    assert_eq!(results[0].0, parent_id);
}
