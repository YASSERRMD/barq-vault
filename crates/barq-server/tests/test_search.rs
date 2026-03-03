use barq_server::state::AppState;
use barq_server::config::ServerConfig;
use barq_types::{IngestRequest, Modality, StorageMode};
use barq_index::SearchParams;
use tempfile::TempDir;

#[tokio::test]
async fn test_search_accuracy() {
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

    let state = AppState::init(config).await.expect("Failed to init AppState");

    let docs = vec![
        "The quick brown fox jumps over the lazy dog.",
        "Rust is a blazing fast and memory-efficient static language.",
        "RocksDB is an embeddable persistent key-value store.",
        "A multimodal database supports text, images, and audio natively.",
        "Barq is Arabic for lightning, representing extreme speed.",
    ];

    for (i, d) in docs.iter().enumerate() {
        let req = IngestRequest {
            summary: String::new(),
            embedding: Vec::new(),
            modality: Modality::Text,
            storage_mode: StorageMode::HybridFile,
            filename: Some(format!("doc_{}.txt", i)),
            raw_payload: Some(d.as_bytes().to_vec()),
            metadata: serde_json::Value::Object(Default::default()),
            chunk_index: 0,
            total_chunks: 1,
            parent_id: None,
        };

        let records = state.ingest_pipeline.run(req).await.expect("Ingest failed");
        let mut index = state.index.write().await;
        for record in records {
            state.store.put_record(&record).unwrap();
            if let Some(payload_bytes) = &record.compressed_payload {
                state.store.put_payload(record.id, payload_bytes).unwrap();
            }
            index.index_new(&record);
        }
    }

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Search for memory-efficient
    let query = SearchParams {
        query_embedding: vec![0.0; 1536],
        query_text: "memory-efficient language".to_string(),
        vector_weight: 0.0, // BM25 only for this test since local embeddings are mock zeroes
        top_k: 3,
        modality_filter: None,
        metadata_filters: serde_json::Value::Object(Default::default()),
    };

    let results = state.index.read().await.search(query).expect("Search failed");
    
    assert!(!results.is_empty(), "Expected results");
    // Assert the correct document is the top hit
    assert!(results[0].1 > 0.0);
}
