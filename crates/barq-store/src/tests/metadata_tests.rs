use crate::store::BarqStore;
use barq_test_utils::fixtures::temp_dir_path;
use serial_test::serial;
use uuid::Uuid;
use serde_json::json;

#[test]
#[serial]
fn test_metadata_crud() {
    let path = temp_dir_path();
    let store = BarqStore::open(&path.to_str().unwrap()).unwrap();
    let id = Uuid::new_v4();
    let meta = json!({
        "author": "antigravity",
        "version": 1.0,
        "tags": ["test", "barq"]
    });
    
    // Put
    store.put_metadata(id, &meta).unwrap();
    
    // Get
    let retrieved = store.get_metadata(id).unwrap().expect("Metadata should exist");
    assert_eq!(retrieved, meta);
    assert_eq!(retrieved["author"], "antigravity");
}

#[test]
#[serial]
fn test_metadata_search() {
    let path = temp_dir_path();
    let store = BarqStore::open(&path.to_str().unwrap()).unwrap();
    
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    
    store.put_metadata(id1, &json!({"domain": "legal", "priority": "high"})).unwrap();
    store.put_metadata(id2, &json!({"domain": "finance", "priority": "high"})).unwrap();
    
    // Search by key-value
    let high_priority = store.search_by_metadata_key("priority", "high").unwrap();
    assert_eq!(high_priority.len(), 2);
    assert!(high_priority.contains(&id1));
    assert!(high_priority.contains(&id2));
    
    let legal_only = store.search_by_metadata_key("domain", "legal").unwrap();
    assert_eq!(legal_only.len(), 1);
    assert_eq!(legal_only[0], id1);
}

#[test]
#[serial]
fn test_metadata_update() {
    let path = temp_dir_path();
    let store = BarqStore::open(&path.to_str().unwrap()).unwrap();
    let id = Uuid::new_v4();
    
    store.put_metadata(id, &json!({"status": "draft"})).unwrap();
    store.put_metadata(id, &json!({"status": "final"})).unwrap();
    
    let retrieved = store.get_metadata(id).unwrap().unwrap();
    assert_eq!(retrieved["status"], "final");
}
