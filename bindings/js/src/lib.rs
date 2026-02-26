use firelocal_core::FireLocal as CoreFireLocal;
use napi::{Error, Result, Status};
use napi_derive::napi;
use std::sync::{Arc, Mutex};

#[napi]
pub struct FireLocal {
    inner: Arc<Mutex<CoreFireLocal>>,
}

#[napi]
impl FireLocal {
    #[napi(constructor)]
    pub fn new(path: String) -> Result<Self> {
        let db = CoreFireLocal::new(path)
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
        Ok(FireLocal {
            inner: Arc::new(Mutex::new(db)),
        })
    }

    #[napi]
    pub fn load_rules(&self, rules: String) -> Result<()> {
        self.inner
            .lock()
            .map_err(|_| Error::new(Status::GenericFailure, "Lock error".to_string()))?
            .load_rules(&rules)
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
    }

    #[napi]
    pub fn put(&self, key: String, value_json: String) -> Result<()> {
        let mut db = self
            .inner
            .lock()
            .map_err(|_| Error::new(Status::GenericFailure, "Lock error".to_string()))?;

        let bytes = value_json.into_bytes();
        db.put(key, bytes)
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
    }

    #[napi]
    pub fn get(&self, key: String) -> Result<Option<String>> {
        let db = self
            .inner
            .lock()
            .map_err(|_| Error::new(Status::GenericFailure, "Lock error".to_string()))?;

        if let Some(bytes) = db.get(&key) {
            let s = String::from_utf8(bytes)
                .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
            Ok(Some(s))
        } else {
            Ok(None)
        }
    }

    #[napi]
    pub fn delete(&self, key: String) -> Result<()> {
        let mut db = self
            .inner
            .lock()
            .map_err(|_| Error::new(Status::GenericFailure, "Lock error".to_string()))?;

        db.delete(key)
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
    }

    #[napi]
    pub fn flush(&self) -> Result<()> {
        let mut db = self
            .inner
            .lock()
            .map_err(|_| Error::new(Status::GenericFailure, "Lock error".to_string()))?;

        db.flush()
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
    }

    /// Create a new write batch
    #[napi]
    pub fn batch(&self) -> Result<WriteBatch> {
        let db = self
            .inner
            .lock()
            .map_err(|_| Error::new(Status::GenericFailure, "Lock error".to_string()))?;

        let core_batch = db.batch();
        Ok(WriteBatch {
            inner: Arc::new(Mutex::new(core_batch)),
        })
    }

    /// Commit a write batch atomically
    #[napi]
    pub fn commit_batch(&self, batch: &WriteBatch) -> Result<()> {
        let mut db = self
            .inner
            .lock()
            .map_err(|_| Error::new(Status::GenericFailure, "Lock error".to_string()))?;

        let batch_inner = batch
            .inner
            .lock()
            .map_err(|_| Error::new(Status::GenericFailure, "Lock error".to_string()))?;

        db.commit_batch(&batch_inner)
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
    }

    /// Run compaction
    #[napi]
    pub fn compact(&self) -> Result<CompactionStats> {
        let db = self
            .inner
            .lock()
            .map_err(|_| Error::new(Status::GenericFailure, "Lock error".to_string()))?;

        let stats = db
            .compact()
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

        Ok(CompactionStats {
            files_before: stats.files_before as u32,
            files_after: stats.files_after as u32,
            entries_before: stats.entries_before as u32,
            entries_after: stats.entries_after as u32,
            tombstones_removed: stats.tombstones_removed as u32,
            size_before: stats.size_before as i64,
            size_after: stats.size_after as i64,
        })
    }
}

#[napi]
pub struct WriteBatch {
    inner: Arc<Mutex<firelocal_core::transaction::WriteBatch>>,
}

#[napi]
impl WriteBatch {
    #[napi]
    pub fn set(&self, path: String, data: String) -> Result<()> {
        let mut batch = self
            .inner
            .lock()
            .map_err(|_| Error::new(Status::GenericFailure, "Lock error".to_string()))?;

        batch.set(path, data.into_bytes());
        Ok(())
    }

    #[napi]
    pub fn update(&self, path: String, data: String) -> Result<()> {
        let mut batch = self
            .inner
            .lock()
            .map_err(|_| Error::new(Status::GenericFailure, "Lock error".to_string()))?;

        batch.update(path, data.into_bytes());
        Ok(())
    }

    #[napi]
    pub fn delete(&self, path: String) -> Result<()> {
        let mut batch = self
            .inner
            .lock()
            .map_err(|_| Error::new(Status::GenericFailure, "Lock error".to_string()))?;

        batch.delete(path);
        Ok(())
    }
}

#[napi(object)]
pub struct CompactionStats {
    pub files_before: u32,
    pub files_after: u32,
    pub entries_before: u32,
    pub entries_after: u32,
    pub tombstones_removed: u32,
    pub size_before: i64,
    pub size_after: i64,
}

/// FieldValue helpers
#[napi]
pub fn server_timestamp() -> String {
    serde_json::to_string(&firelocal_core::field_value::FieldValue::server_timestamp())
        .unwrap_or_default()
}

#[napi]
pub fn increment(n: i64) -> String {
    serde_json::to_string(&firelocal_core::field_value::FieldValue::increment(n))
        .unwrap_or_default()
}

#[napi]
pub fn array_union(elements: Vec<String>) -> String {
    let values: Vec<serde_json::Value> = elements
        .into_iter()
        .map(serde_json::Value::String)
        .collect();
    serde_json::to_string(&firelocal_core::field_value::FieldValue::array_union(
        values,
    ))
    .unwrap_or_default()
}

#[napi]
pub fn array_remove(elements: Vec<String>) -> String {
    let values: Vec<serde_json::Value> = elements
        .into_iter()
        .map(serde_json::Value::String)
        .collect();
    serde_json::to_string(&firelocal_core::field_value::FieldValue::array_remove(
        values,
    ))
    .unwrap_or_default()
}

#[napi]
pub fn delete_field() -> String {
    serde_json::to_string(&firelocal_core::field_value::FieldValue::delete()).unwrap_or_default()
}
