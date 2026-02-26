use firelocal_core::FireLocal;
use std::fs;

#[test]
fn test_invalid_rules_format() {
    let test_dir = "test_db_invalid_rules";
    let _ = fs::remove_dir_all(test_dir);
    fs::create_dir_all(test_dir).unwrap();

    let mut db = FireLocal::new(test_dir).expect("Failed to create database");

    // Try to load invalid rules
    let result = db.load_rules("this is not valid rules");
    assert!(result.is_err(), "Invalid rules should fail");

    // Cleanup
    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_permission_denied_on_write() {
    let test_dir = "test_db_permission_denied";
    let _ = fs::remove_dir_all(test_dir);
    fs::create_dir_all(test_dir).unwrap();

    let mut db = FireLocal::new(test_dir).expect("Failed to create database");

    // Load restrictive rules
    let rules = r#"
        service cloud.firestore {
            match /databases/{database}/documents {
                match /{document=**} {
                    allow read, write: if false;
                }
            }
        }
    "#;
    db.load_rules(rules).expect("Failed to load rules");

    // Try to write - should fail
    let result = db.put("users/alice".to_string(), br#"{"name":"Alice"}"#.to_vec());
    assert!(result.is_err(), "Write should be denied by rules");

    // Cleanup
    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_permission_denied_on_read() {
    let test_dir = "test_db_read_denied";
    let _ = fs::remove_dir_all(test_dir);
    fs::create_dir_all(test_dir).unwrap();

    let mut db = FireLocal::new(test_dir).expect("Failed to create database");

    // First, allow writes
    let write_rules = r#"
        service cloud.firestore {
            match /databases/{database}/documents {
                match /{document=**} {
                    allow write: if true;
                }
            }
        }
    "#;
    db.load_rules(write_rules).expect("Failed to load rules");

    // Put a document
    db.put("users/alice".to_string(), br#"{"name":"Alice"}"#.to_vec())
        .expect("Failed to put document");

    // Now change rules to deny reads
    let read_rules = r#"
        service cloud.firestore {
            match /databases/{database}/documents {
                match /{document=**} {
                    allow read: if false;
                }
            }
        }
    "#;
    db.load_rules(read_rules).expect("Failed to load rules");

    // Try to read - should return None (permission denied)
    let result = db.get("users/alice");
    assert!(result.is_none(), "Read should be denied by rules");

    // Cleanup
    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_invalid_utf8_data() {
    let test_dir = "test_db_invalid_utf8";
    let _ = fs::remove_dir_all(test_dir);
    fs::create_dir_all(test_dir).unwrap();

    let mut db = FireLocal::new(test_dir).expect("Failed to create database");

    // Put invalid UTF-8 data - should fail
    let invalid_utf8 = vec![0xFF, 0xFE, 0xFD];
    let result = db.put("data/invalid".to_string(), invalid_utf8);
    assert!(result.is_err(), "Should fail to put invalid UTF-8 data");

    // Cleanup
    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_get_nonexistent_document() {
    let test_dir = "test_db_nonexistent";
    let _ = fs::remove_dir_all(test_dir);
    fs::create_dir_all(test_dir).unwrap();

    let db = FireLocal::new(test_dir).expect("Failed to create database");

    // Get nonexistent document
    let result = db.get("nonexistent/path");
    assert!(result.is_none(), "Nonexistent document should return None");

    // Cleanup
    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_delete_nonexistent_document() {
    let test_dir = "test_db_delete_nonexistent";
    let _ = fs::remove_dir_all(test_dir);
    fs::create_dir_all(test_dir).unwrap();

    let mut db = FireLocal::new(test_dir).expect("Failed to create database");

    // Delete nonexistent document - should succeed
    let result = db.delete("nonexistent/path".to_string());
    assert!(
        result.is_ok(),
        "Deleting nonexistent document should succeed"
    );

    // Cleanup
    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_batch_with_permission_denied() {
    let test_dir = "test_db_batch_permission";
    let _ = fs::remove_dir_all(test_dir);
    fs::create_dir_all(test_dir).unwrap();

    let mut db = FireLocal::new(test_dir).expect("Failed to create database");

    // Load restrictive rules
    let rules = r#"
        service cloud.firestore {
            match /databases/{database}/documents {
                match /{document=**} {
                    allow read, write: if false;
                }
            }
        }
    "#;
    db.load_rules(rules).expect("Failed to load rules");

    // Create batch with operations
    let mut batch = db.batch();
    batch.set("users/alice".to_string(), b"data".to_vec());

    // Commit should fail due to permissions
    let result = db.commit_batch(&batch);
    assert!(
        result.is_err(),
        "Batch commit should fail due to permissions"
    );

    // Cleanup
    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_empty_path() {
    let test_dir = "test_db_empty_path";
    let _ = fs::remove_dir_all(test_dir);
    fs::create_dir_all(test_dir).unwrap();

    let mut db = FireLocal::new(test_dir).expect("Failed to create database");

    // Put with empty path - may succeed or fail depending on implementation
    let result = db.put("".to_string(), br#"{"data":"test"}"#.to_vec());
    // Just verify it doesn't panic
    let _ = result;

    // Cleanup
    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_very_long_path() {
    let test_dir = "test_db_long_path";
    let _ = fs::remove_dir_all(test_dir);
    fs::create_dir_all(test_dir).unwrap();

    let mut db = FireLocal::new(test_dir).expect("Failed to create database");

    // Create very long path
    let long_path = "a/".repeat(100) + "document";

    // Put with long path - should fail due to depth limit
    let result = db.put(long_path.clone(), br#"{"data":"test"}"#.to_vec());
    assert!(result.is_err(), "Should fail for path too deep");

    // Cleanup
    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_concurrent_operations_same_document() {
    let test_dir = "test_db_concurrent_same";
    let _ = fs::remove_dir_all(test_dir);
    fs::create_dir_all(test_dir).unwrap();

    let mut db = FireLocal::new(test_dir).expect("Failed to create database");

    // Put initial document
    db.put("counter".to_string(), br#"{"value":0}"#.to_vec())
        .expect("Failed to put initial document");

    // Simulate multiple updates
    for i in 1..10 {
        let data = format!(r#"{{"value":{}}}"#, i);
        db.put("counter".to_string(), data.into_bytes())
            .expect("Failed to update document");
    }

    // Verify final value
    let final_doc = db.get("counter").expect("Failed to get document");
    assert!(
        !final_doc.is_empty(),
        "Document should exist after concurrent operations"
    );

    // Cleanup
    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_recovery_from_corrupted_wal() {
    let test_dir = "test_db_corrupted_wal";
    let _ = fs::remove_dir_all(test_dir);
    fs::create_dir_all(test_dir).unwrap();

    // Create database and put data
    {
        let mut db = FireLocal::new(test_dir).expect("Failed to create database");
        db.put("users/alice".to_string(), br#"{"name":"Alice"}"#.to_vec())
            .expect("Failed to put document");
    }

    // Corrupt WAL file
    let wal_path = format!("{}/wal.log", test_dir);
    if std::path::Path::new(&wal_path).exists() {
        fs::write(&wal_path, vec![0xFF; 100]).expect("Failed to corrupt WAL");
    }

    // Try to open database - should recover or fail gracefully
    let result = FireLocal::new(test_dir);
    // Just verify it doesn't panic
    let _ = result;

    // Cleanup
    let _ = fs::remove_dir_all(test_dir);
}
