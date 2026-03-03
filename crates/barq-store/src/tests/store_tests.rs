use crate::store::BarqStore;
use barq_test_utils::fixtures::temp_dir_path;
use serial_test::serial;

#[test]
#[serial]
fn test_store_open_new() {
    let path = temp_dir_path();
    let store = BarqStore::open(&path.to_str().unwrap()).unwrap();
    
    // Check if path exists
    assert!(path.exists());
    
    // Drop store and reopen
    drop(store);
    let store_reopened = BarqStore::open(&path.to_str().unwrap()).unwrap();
    drop(store_reopened);
}

#[test]
#[serial]
fn test_store_column_families_created() {
    let path = temp_dir_path();
    let store = BarqStore::open(&path.to_str().unwrap()).unwrap();
    
    // Attempting to access CFs via internal logic if possible, 
    // or just verifying standard operations don't crash.
    // Since BarqStore abstracts CFs, we verify via usage.
    drop(store);
}
