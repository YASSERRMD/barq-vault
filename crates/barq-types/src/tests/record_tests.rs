use crate::record::BarqRecord;
use crate::modality::{Modality, StorageMode};
use uuid::Uuid;

#[test]
fn test_record_json_round_trip() {
    let mut record = BarqRecord::default();
    record.id = Uuid::new_v4();
    record.summary = "test summary".to_string();
    record.embedding = vec![0.1, 0.2, 0.3];
    record.embedding_dim = 3;
    
    let json = serde_json::to_string(&record).unwrap();
    let decoded: BarqRecord = serde_json::from_str(&json).unwrap();
    
    assert_eq!(record.id, decoded.id);
    assert_eq!(record.summary, decoded.summary);
    assert_eq!(record.embedding, decoded.embedding);
}

#[test]
fn test_record_binary_round_trip() {
    let mut record = BarqRecord::default();
    record.id = Uuid::new_v4();
    record.summary = "test binary serialization".to_string();
    
    // We use serde_json binary encoding for RocksDB storage to support dynamic Value metadata
    let encoded = serde_json::to_vec(&record).unwrap();
    let decoded: BarqRecord = serde_json::from_slice(&encoded).unwrap();
    
    assert_eq!(record.id, decoded.id);
    assert_eq!(record.summary, decoded.summary);
}

#[test]
fn test_record_empty_embedding() {
    let mut record = BarqRecord::default();
    record.embedding = vec![];
    record.embedding_dim = 0;
    
    let json = serde_json::to_string(&record).unwrap();
    let decoded: BarqRecord = serde_json::from_str(&json).unwrap();
    
    assert!(decoded.embedding.is_empty());
}

#[test]
fn test_record_max_fields() {
    let mut record = BarqRecord::default();
    record.summary = "a".repeat(10000);
    record.embedding = vec![0.0; 1536];
    record.embedding_dim = 1536;
    
    let json = serde_json::to_string(&record).unwrap();
    let decoded: BarqRecord = serde_json::from_str(&json).unwrap();
    
    assert_eq!(decoded.summary.len(), 10000);
    assert_eq!(decoded.embedding.len(), 1536);
}

#[test]
fn test_record_default() {
    let record = BarqRecord::default();
    assert_eq!(record.modality, Modality::Text);
    assert_eq!(record.storage_mode, StorageMode::TextOnly);
}
