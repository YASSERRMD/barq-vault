use crate::config::{
    AuthConfig, IndexConfig, IngestCfg, ServerConfig, ServerEndpointConfig, TlsConfig,
};
use crate::state::AppState;
use barq_test_utils::fixtures::temp_dir_path;
use serial_test::serial;

/// Build a minimal in-process ServerConfig that points to a temp dir.
pub(crate) fn test_server_config(store_path: &str) -> ServerConfig {
    ServerConfig {
        server: ServerEndpointConfig {
            grpc_addr: "127.0.0.1:0".to_string(),
            rest_addr: "127.0.0.1:0".to_string(),
            store_path: store_path.to_string(),
            max_payload_bytes: 10 * 1024 * 1024,
            tls: TlsConfig {
                enabled: false,
                cert_path: String::new(),
                key_path: String::new(),
            },
            auth: AuthConfig {
                enabled: false,
                token: String::new(),
                skip_ping: true,
            },
        },
        index: IndexConfig {
            vector_dim: 64,
            hnsw_ef_construction: 40,
            hnsw_m: 16,
            bm25_k1: 1.5,
            bm25_b: 0.75,
        },
        ingest: IngestCfg {
            chunk_size_tokens: 512,
            chunk_overlap_tokens: 64,
            default_storage_mode: "text_only".to_string(),
        },
    }
}

#[tokio::test]
#[serial]
async fn test_app_state_init_opens_store() {
    let dir = temp_dir_path();
    let config = test_server_config(dir.to_str().unwrap());
    let state = AppState::init(config).await;
    assert!(state.is_ok(), "AppState::init failed: {:?}", state.err());
}

#[tokio::test]
#[serial]
async fn test_app_state_bootstrap_empty_db_is_fast() {
    let dir = temp_dir_path();
    let config = test_server_config(dir.to_str().unwrap());
    let t0 = std::time::Instant::now();
    let state = AppState::init(config).await.unwrap();
    let elapsed = t0.elapsed();
    // Bootstrap of an empty DB should complete in well under 1 second
    assert!(
        elapsed.as_millis() < 1000,
        "Bootstrap took too long: {:?}",
        elapsed
    );
    drop(state);
}

#[tokio::test]
#[serial]
async fn test_app_state_started_at_is_recent() {
    let dir = temp_dir_path();
    let config = test_server_config(dir.to_str().unwrap());
    let state = AppState::init(config).await.unwrap();
    // started_at should be very recent
    assert!(state.started_at.elapsed().as_secs() < 5);
}

#[tokio::test]
#[serial]
async fn test_app_state_index_is_accessible() {
    let dir = temp_dir_path();
    let config = test_server_config(dir.to_str().unwrap());
    let state = AppState::init(config).await.unwrap();
    let _index = state.index.read().await;
    // If we can acquire the read lock, the index is properly initialized
}
