use firelocal_core::FireLocal;
use firelocal_core::api::CollectionReference;
use serde_json::json;
use std::fs;
use std::sync::{Arc, Mutex};

#[test]
fn test_api_listeners() {
    let path = "tmp_test_db_api";
    let _ = fs::remove_dir_all(path);

    let api_db = Arc::new(Mutex::new(FireLocal::new(path).unwrap()));
    {
        let mut db = api_db.lock().unwrap();
        db.load_rules("service cloud.firestore { match /databases/{database}/documents { match /users/{uid} { allow read, write: if true; } } }").unwrap();
    }

    // 1. Create Collection Reference
    let users_col = CollectionReference::new(api_db.clone(), "users".to_string());

    // 2. Set up listener for active users
    let query = users_col.where_eq("active", json!(true));

    let received_docs = Arc::new(Mutex::new(Vec::new()));
    let received_docs_clone = received_docs.clone();

    let _listener_id = query.on_snapshot(move |docs| {
        let mut guard = received_docs_clone.lock().unwrap();
        *guard = docs;
    });

    // 3. Add a document
    let doc_ref = users_col.doc("alice");
    doc_ref
        .set(json!({
            "name": "Alice",
            "active": true
        }))
        .unwrap();

    // Wait for listener (it's synchronous in our impl but let's be safe)
    // Actually our impl calls notify_listeners synchronously in put().

    {
        let docs = received_docs.lock().unwrap();
        assert_eq!(docs.len(), 1);
        assert_eq!(
            docs[0].fields.get("name").unwrap().as_str().unwrap(),
            "Alice"
        );
    }

    // 4. Add another doc that matches
    users_col
        .doc("bob")
        .set(json!({
            "name": "Bob",
            "active": true
        }))
        .unwrap();

    {
        let docs = received_docs.lock().unwrap();
        assert_eq!(docs.len(), 2);
    }

    // 5. Add doc that doesn't match
    users_col
        .doc("charlie")
        .set(json!({
            "name": "Charlie",
            "active": false
        }))
        .unwrap();

    {
        let docs = received_docs.lock().unwrap();
        assert_eq!(docs.len(), 2); // Should still be 2
    }

    // 6. Delete matching doc
    users_col.doc("alice").delete().unwrap();

    {
        let docs = received_docs.lock().unwrap();
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].fields.get("name").unwrap().as_str().unwrap(), "Bob");
    }

    let _ = fs::remove_dir_all(path);
}
