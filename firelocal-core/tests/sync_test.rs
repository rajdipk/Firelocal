use firelocal_core::model::Document;
use firelocal_core::sync::RemoteStore;
use firelocal_core::FireLocal;
use std::fs;
use std::sync::{Arc, Mutex};

// A mock store that remembers what we pushed
struct MemoryRemoteStore {
    storage: Arc<Mutex<Vec<Document>>>,
}

impl RemoteStore for MemoryRemoteStore {
    fn push(&self, doc: &Document) -> Result<(), String> {
        self.storage.lock().unwrap().push(doc.clone());
        Ok(())
    }

    fn pull(&self, path: &str) -> Result<Option<Document>, String> {
        let storage = self.storage.lock().unwrap();
        Ok(storage.iter().find(|d| d.path == path).cloned())
    }
}

#[test]
fn test_sync_flow() {
    let path = "tmp_test_db_sync";
    let _ = fs::remove_dir_all(path);

    let mut db = FireLocal::new(path).unwrap();
    // Load rules
    db.load_rules("service cloud.firestore { match /databases/{database}/documents { match /{document=**} { allow read, write: if true; } } }").unwrap();

    let shared_storage = Arc::new(Mutex::new(Vec::new()));
    let remote = Box::new(MemoryRemoteStore {
        storage: shared_storage.clone(),
    });

    db.set_remote_store(remote);

    // 1. Put local
    db.put(
        "users/sync_user".to_string(),
        br#"{"path":"users/sync_user","fields":{"name":"Sync"}}"#.to_vec(),
    )
    .unwrap();

    // 2. Sync Push
    db.sync_push("users/sync_user").unwrap();

    // Verify it's in "remote"
    {
        let storage = shared_storage.lock().unwrap();
        assert_eq!(storage.len(), 1);
        assert_eq!(storage[0].path, "users/sync_user");
    }

    // 3. Clear local
    // (Simulate by deleting)
    db.delete("users/sync_user".to_string()).unwrap();
    assert!(db.get("users/sync_user").is_none());

    // 4. Sync Pull
    db.sync_pull("users/sync_user").unwrap();

    // Verify it's back
    assert!(db.get("users/sync_user").is_some());

    let _ = fs::remove_dir_all(path);
}
