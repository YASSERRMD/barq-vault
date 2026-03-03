use crate::state::AppState;
use crate::tests::state_tests::test_server_config;
use barq_test_utils::fixtures::temp_dir_path;
use barq_test_utils::builders::BarqRecordBuilder;
use serial_test::serial;
use uuid::Uuid;
use std::sync::Arc;

#[tokio::test]
#[serial]
async fn test_store_put_and_get_via_state() {
    let dir = temp_dir_path();
    let config = test_server_config(dir.to_str().unwrap());
    let state = AppState::init(config).await.unwrap();

    let record = BarqRecordBuilder::new()
        .summary("gRPC integration test record".to_string())
        .build();
    let id = record.id;

    state.store.put_record(&record).unwrap();
    let retrieved = state.store.get_record(id).unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().summary, "gRPC integration test record");
}

#[tokio::test]
#[serial]
async fn test_index_via_state() {
    let dir = temp_dir_path();
    let config = test_server_config(dir.to_str().unwrap());
    let state = AppState::init(config).await.unwrap();

    let record = BarqRecordBuilder::new()
        .summary("quantum computing research paper".to_string())
        .embedding(vec![0.1f32; 64])
        .build();

    {
        let mut index = state.index.write().await;
        index.index_new(&record);
    }

    let index = state.index.read().await;
    let results = index.search(barq_index::SearchParams {
        query_embedding: vec![0.1f32; 64],
        query_text: "quantum computing".to_string(),
        vector_weight: 0.5,
        top_k: 5,
        modality_filter: None,
        metadata_filters: serde_json::json!({}),
    }).unwrap();

    assert!(!results.is_empty());
}

#[tokio::test]
#[serial]
async fn test_concurrent_index_reads() {
    let dir = temp_dir_path();
    let config = test_server_config(dir.to_str().unwrap());
    let state = AppState::init(config).await.unwrap();

    let handles: Vec<_> = (0..5)
        .map(|_| {
            let index = Arc::clone(&state.index);
            tokio::spawn(async move {
                let _guard = index.read().await;
                tokio::task::yield_now().await;
            })
        })
        .collect();

    for h in handles {
        h.await.unwrap();
    }
}
