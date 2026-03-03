use crate::store::BarqStore;
use barq_test_utils::fixtures::temp_dir_path;
use serial_test::serial;
use uuid::Uuid;

#[test]
#[serial]
fn test_payload_crud() {
    let path = temp_dir_path();
    let store = BarqStore::open(&path.to_str().unwrap()).unwrap();
    let id = Uuid::new_v4();
    let data = b"some payload data".to_vec();
    
    // Put
    store.put_payload(id, &data).unwrap();
    
    // Get
    let retrieved = store.get_payload(id).unwrap().expect("Payload should exist");
    assert_eq!(retrieved, data);
    
    // Size
    let size = store.get_payload_size(id).unwrap().expect("Size should exist");
    assert_eq!(size, data.len() as u64);
    
    // Delete
    store.delete_payload(id).unwrap();
    assert!(store.get_payload(id).unwrap().is_none());
}

#[test]
#[serial]
fn test_large_payload() {
    let path = temp_dir_path();
    let store = BarqStore::open(&path.to_str().unwrap()).unwrap();
    let id = Uuid::new_v4();
    let data = vec![0u8; 5 * 1024 * 1024]; // 5MB
    
    store.put_payload(id, &data).unwrap();
    let retrieved = store.get_payload(id).unwrap().expect("Payload should exist");
    assert_eq!(retrieved.len(), data.len());
}
