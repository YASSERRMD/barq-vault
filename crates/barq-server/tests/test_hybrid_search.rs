use barq_server::state::AppState;
use barq_server::config::ServerConfig;
use barq_types::{IngestRequest, Modality, StorageMode};
use barq_index::SearchParams;
use tempfile::TempDir;

#[tokio::test]
async fn test_hybrid_search_rrf() {
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

    // Ingest dummy files
    for i in 0..5 {
        let req = IngestRequest {
            summary: format!("unique{} document number", i),
            embedding: vec![0.1 * ((i + 1) as f32); 1536], // fake embeddings > 0.0
            modality: Modality::Text,
            storage_mode: StorageMode::HybridFile,
            filename: Some(format!("doc_{}.txt", i)),
            raw_payload: Some(format!("unique{} document number {}", i, i).into_bytes()),
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

    // Search 1: Very high vector weight
    let req_vector = SearchParams {
        query_embedding: vec![0.4; 1536],
        query_text: "document".to_string(),
        vector_weight: 0.9,
        top_k: 5,
        modality_filter: None,
        metadata_filters: serde_json::Value::Object(Default::default()),
    };

    let res_vector = state.index.read().await.search(req_vector).expect("Search failed");

    // Search 2: Pure BM25
    let req_bm25 = SearchParams {
        query_embedding: vec![0.4; 1536],
        query_text: "unique4".to_string(),
        vector_weight: 0.0,
        top_k: 5,
        modality_filter: None,
        metadata_filters: serde_json::Value::Object(Default::default()),
    };

    let res_bm25 = state.index.read().await.search(req_bm25).expect("Search failed");

    // Since RRF is active and indices differ, ordering should differ based on the weights
    assert!(!res_vector.is_empty());
    assert!(!res_bm25.is_empty());

    // The doc we queried for "number 4" is doc_4.txt, which is the 5th loop iteration
    // We expect its BM25 RRF score to be non-zero because w_b > 0.0 and it matches the token.
    println!("Vector results: {:#?}", res_vector);
    println!("BM25 results: {:#?}", res_bm25);
    
    // Find the record that actually matched "number 4" and check its score.
    // The top result should have a score > 0.0 if BM25 found it.
    assert!(res_bm25[0].1 > 0.0, "Top BM25 result should have a positive RRF score, got: {}", res_bm25[0].1);
}
