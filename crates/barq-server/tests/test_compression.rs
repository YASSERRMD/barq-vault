use barq_server::state::AppState;
use barq_server::config::ServerConfig;
use barq_types::{IngestRequest, Modality, StorageMode};
use tempfile::TempDir;
use blake3::Hasher;

#[tokio::test]
async fn test_lzma_compression_roundtrip() {
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

    // Generate some repetitive binary data that compresses well
    let mut payload = Vec::new();
    for _ in 0..10_000 {
        payload.extend_from_slice(b"barq-vault ");
    }
    
    let mut hasher = Hasher::new();
    hasher.update(&payload);
    let original_hash = hasher.finalize().to_string();

    let req = IngestRequest {
        summary: String::new(),
        embedding: Vec::new(),
        modality: Modality::Document,
        storage_mode: StorageMode::FullRaw,
        filename: Some("large_repetitive.bin".to_string()),
        raw_payload: Some(payload.clone()),
        metadata: serde_json::Value::Object(Default::default()),
        chunk_index: 0,
        total_chunks: 1,
        parent_id: None,
    };

    let records = state.ingest_pipeline.run(req).await.expect("Ingest failed");
    let parent_id = records[0].id;

    // The pipeline generates the record but doesn't persist it. Persist it now:
    let record = &records[0];
    state.store.put_record(&record).expect("Failed to store record");
    if let Some(payload_bytes) = &record.compressed_payload {
        state.store.put_payload(record.id, payload_bytes).expect("Failed to store payload");
    }

    // Fetch back the raw bytes directly from store
    let fetched = state.store.get_payload(parent_id).expect("Failed reading payload").expect("Payload missing");

    let decompressed = barq_compress::decompress(&fetched, barq_compress::Codec::Lzma(3), payload.len()).expect("Failed decomp");

    let mut fetched_hasher = Hasher::new();
    fetched_hasher.update(&decompressed);
    let fetched_hash = fetched_hasher.finalize().to_string();

    assert_eq!(original_hash, fetched_hash, "Checksum mismatch after roundtrip");
    assert_eq!(decompressed.len(), payload.len(), "Length mismatch after compression roundtrip");
}
