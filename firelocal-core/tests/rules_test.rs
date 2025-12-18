use firelocal_core::FireLocal;
use std::fs;

#[test]
fn test_rules_parser_and_enforcement() {
    let path = "tmp_test_db_rules";
    let _ = fs::remove_dir_all(path);

    let mut db = FireLocal::new(path).unwrap();

    // 1. Default: Allow all (dev mode) when no rules are loaded
    assert!(db.put("any/path".to_string(), b"data".to_vec()).is_ok());

    // 2. Load rules: Allow read, write to /users/{uid}
    let rules = r#"
        service cloud.firestore {
            match /databases/{database}/documents {
                match /users/{uid} {
                    allow read, write: if true;
                }
            }
        }
    "#;

    db.load_rules(rules).expect("Failed to parse rules");

    // 3. Test Allowed Write
    assert!(db
        .put("users/alice".to_string(), b"alice_data".to_vec())
        .is_ok());

    // 4. Test Allowed Read
    assert!(db.get("users/alice").is_some());

    // 5. Test Denied Write (path not matching)
    // "posts/123" not in users
    assert!(db.put("posts/123".to_string(), b"data".to_vec()).is_err());

    // 6. Test Denied Read
    assert!(db.get("posts/123").is_none()); // get returns None on error/missing

    let _ = fs::remove_dir_all(path);
}
