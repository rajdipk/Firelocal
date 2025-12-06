use firelocal_core::FireLocal;
use std::fs;

#[test]
fn test_basic_put_get() {
    let path = "tmp_test_db";
    let _ = fs::remove_dir_all(path);

    let mut db = FireLocal::new(path).unwrap();
    db.load_rules("service cloud.firestore { match /databases/{database}/documents { match /{document=**} { allow read, write: if true; } } }").unwrap();

    db.put("key1".to_string(), b"val1".to_vec()).unwrap();

    assert_eq!(db.get("key1"), Some(b"val1".to_vec()));
    assert_eq!(db.get("key2"), None);

    let _ = fs::remove_dir_all(path);
}

#[test]
fn test_delete() {
    let path = "tmp_test_db_del";
    let _ = fs::remove_dir_all(path);

    let mut db = FireLocal::new(path).unwrap();
    db.load_rules("service cloud.firestore { match /databases/{database}/documents { match /{document=**} { allow read, write: if true; } } }").unwrap();

    db.put("k".to_string(), b"v".to_vec()).unwrap();
    assert_eq!(db.get("k").unwrap(), b"v".to_vec());

    db.delete("k".to_string()).unwrap();
    // Memtable.delete sets tombstone.
    // Memtable.get needs to handle tombstone to return None?
    // Let's check Memtable implementation...
    // My memtable.get returns Some(val) for Put, and None for Delete.
    // So this should pass.
    assert!(db.get("k").is_none());

    let _ = fs::remove_dir_all(path);
}

#[test]
fn test_flush() {
    let path = "tmp_test_db_flush";
    let _ = fs::remove_dir_all(path);

    let mut db = FireLocal::new(path).unwrap();
    db.load_rules("service cloud.firestore { match /databases/{database}/documents { match /{document=**} { allow read, write: if true; } } }").unwrap();

    db.put("a".to_string(), b"1".to_vec()).unwrap();
    db.put("b".to_string(), b"2".to_vec()).unwrap();

    db.flush().unwrap();

    // verify sst file exists
    let entries = fs::read_dir(path).unwrap();
    let sst_count = entries
        .filter(|e| e.as_ref().unwrap().path().extension().unwrap() == "sst")
        .count();
    assert_eq!(sst_count, 1);

    let _ = fs::remove_dir_all(path);
}
