use firelocal_core::index::{QueryAst, QueryOperator};
use firelocal_core::FireLocal;
use serde_json::json;
use std::fs;

#[test]
fn test_query_indexing() {
    let path = "tmp_test_db_query";
    let _ = fs::remove_dir_all(path);

    let mut db = FireLocal::new(path).unwrap();
    db.load_rules("service cloud.firestore { match /databases/{database}/documents { match /{document=**} { allow read, write: if true; } } }").unwrap();

    // 1. Put docs
    let doc1 = json!({
        "path": "users/alice",
        "fields": {
            "name": "Alice",
            "age": 30,
            "active": true
        }
    });
    let doc2 = json!({
        "path": "users/bob",
        "fields": {
            "name": "Bob",
            "age": 25,
            "active": false
        }
    });
    let doc3 = json!({
        "path": "users/charlie",
        "fields": {
            "name": "Charlie",
            "age": 35,
            "active": true
        }
    });

    db.put(
        "users/alice".to_string(),
        serde_json::to_vec(&doc1).unwrap(),
    )
    .unwrap();
    db.put("users/bob".to_string(), serde_json::to_vec(&doc2).unwrap())
        .unwrap();
    db.put(
        "users/charlie".to_string(),
        serde_json::to_vec(&doc3).unwrap(),
    )
    .unwrap();

    // 2. Query: active == true
    let q = QueryAst {
        collection: Some("users".to_string()),
        field: "active".to_string(),
        operator: QueryOperator::Equal(json!(true)),
    };

    let results = db.query(&q).unwrap();
    assert_eq!(results.len(), 2);
    let names: Vec<String> = results
        .iter()
        .map(|d| d.fields.get("name").unwrap().as_str().unwrap().to_string())
        .collect();
    assert!(names.contains(&"Alice".to_string()));
    assert!(names.contains(&"Charlie".to_string()));

    // 3. Query: age == 25
    let q2 = QueryAst {
        collection: Some("users".to_string()),
        field: "age".to_string(),
        operator: QueryOperator::Equal(json!(25)),
    };
    let results2 = db.query(&q2).unwrap();
    assert_eq!(results2.len(), 1);
    assert_eq!(results2[0].fields.get("name").unwrap(), "Bob");

    let _ = fs::remove_dir_all(path);
}
