use crate::store::BarqStore;
use barq_test_utils::builders::BarqRecordBuilder;
use barq_test_utils::fixtures::temp_dir_path;
use serial_test::serial;
use uuid::Uuid;

#[test]
#[serial]
fn test_record_crud_lifecycle() {
    let path = temp_dir_path();
    let store = BarqStore::open(&path.to_str().unwrap()).unwrap();
    
    let record = BarqRecordBuilder::new()
        .summary("Testing CRUD".to_string())
        .build();
    let id = record.id;
    
    // Create
    store.put_record(&record).unwrap();
    
    // Read
    let retrieved = store.get_record(id).unwrap().expect("Record should exist");
    assert_eq!(retrieved.id, id);
    assert_eq!(retrieved.summary, "Testing CRUD");
    
    // Update
    let mut updated = retrieved;
    updated.summary = "Updated Summary".to_string();
    store.put_record(&updated).unwrap();
    
    let retrieved_updated = store.get_record(id).unwrap().expect("Record should exist");
    assert_eq!(retrieved_updated.summary, "Updated Summary");
    
    // Delete
    store.delete_record(id).unwrap();
    let after_delete = store.get_record(id).unwrap();
    assert!(after_delete.is_none());
}

#[test]
#[serial]
fn test_get_non_existent_record() {
    let path = temp_dir_path();
    let store = BarqStore::open(&path.to_str().unwrap()).unwrap();
    let res = store.get_record(Uuid::new_v4()).unwrap();
    assert!(res.is_none());
}

#[test]
#[serial]
fn test_list_records() {
    let path = temp_dir_path();
    let store = BarqStore::open(&path.to_str().unwrap()).unwrap();
    
    for i in 0..5 {
        let record = BarqRecordBuilder::new()
            .summary(format!("Record {}", i))
            .build();
        store.put_record(&record).unwrap();
    }
    
    let records: Vec<_> = store.iter_all_records().collect();
    assert_eq!(records.len(), 5);
}
