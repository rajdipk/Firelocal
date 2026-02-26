pub mod api;
pub mod config;
#[cfg(not(target_arch = "wasm32"))]
pub mod error;
pub mod ffi;
pub mod field_value;
pub mod health;
pub mod index;
pub mod listener;
pub mod logging;
pub mod model;
pub mod rules;
pub mod security;
pub mod store;
#[cfg(feature = "sync")]
pub mod sync;
pub mod transaction;
pub mod validation;

use crate::config::FireLocalConfig;
use crate::field_value::process_field_values;
use crate::index::basic_index::BasicIndexProvider;
use crate::index::{IndexProvider, QueryAst};
use crate::listener::{ListenerManager, SnapshotCallback};
use crate::model::Document;
use crate::rules::RulesEngine;
use crate::store::compaction::CompactionStats;
use crate::store::io::{StdStorage, Storage};
use crate::store::memtable::Memtable;
use crate::store::sst::{SstBuilder, SstReader, SstSearchResult};
use crate::store::wal::WriteAheadLog;
#[cfg(all(feature = "sync", not(target_arch = "wasm32")))]
use crate::sync::{MockRemoteStore, RemoteStore, SyncManager};
use crate::transaction::{execute_batch_operation, WriteBatch};
use anyhow::Result;
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;

pub struct FireLocal<S: Storage = StdStorage> {
    path: PathBuf,
    storage: Arc<S>,
    wal: WriteAheadLog<S>,
    memtable: Memtable,
    ssts: Vec<Arc<std::sync::Mutex<SstReader<S::File>>>>,
    index: Arc<dyn IndexProvider>,
    listeners: ListenerManager,
    rules: RulesEngine,
    #[cfg(feature = "sync")]
    sync: SyncManager,
    config: Option<FireLocalConfig>,
    #[allow(dead_code)]
    document_versions: HashMap<String, u64>,
}

impl FireLocal<StdStorage> {
    pub fn new(path: impl Into<PathBuf>) -> io::Result<Self> {
        Self::new_with_storage(path, StdStorage)
    }

    /// Create a new FireLocal instance with configuration
    pub fn new_with_config(path: impl Into<PathBuf>) -> io::Result<Self> {
        let path_buf = path.into();
        let config = FireLocalConfig::load_or_create(Some(&path_buf)).map_err(io::Error::other)?;

        let mut instance = Self::new(&path_buf)?;
        instance.config = Some(config);
        Ok(instance)
    }
}

impl<S: Storage> FireLocal<S> {
    pub fn new_with_storage(path: impl Into<PathBuf>, storage: S) -> io::Result<Self> {
        let path = path.into();
        let storage = Arc::new(storage);

        storage.create_dir_all(&path)?;

        let wal_path = path.join("wal.log");
        let wal = WriteAheadLog::open(storage.clone(), wal_path)?;

        let index = Arc::new(BasicIndexProvider::new());

        let mut memtable = Memtable::new();

        // Replay WAL
        if let Ok(iter) = wal.iter() {
            for entry in iter.flatten() {
                if entry.is_empty() {
                    continue;
                }
                let op = entry[0];
                if entry.len() < 5 {
                    continue;
                }
                let k_len = match entry[1..5].try_into() {
                    Ok(bytes) => u32::from_le_bytes(bytes) as usize,
                    Err(_) => {
                        eprintln!("Invalid WAL entry format: key length bytes");
                        continue;
                    }
                };
                if entry.len() < 5 + k_len {
                    continue;
                }
                let key = String::from_utf8_lossy(&entry[5..5 + k_len]).to_string();

                if op == 0 {
                    // Put
                    if entry.len() < 5 + k_len + 4 {
                        continue;
                    }
                    let v_len_offset = 5 + k_len;
                    let v_len = match entry[v_len_offset..v_len_offset + 4].try_into() {
                        Ok(bytes) => u32::from_le_bytes(bytes) as usize,
                        Err(_) => {
                            eprintln!("Invalid WAL entry format: value length bytes");
                            continue;
                        }
                    };
                    if entry.len() < v_len_offset + 4 + v_len {
                        continue;
                    }
                    let value = entry[v_len_offset + 4..v_len_offset + 4 + v_len].to_vec();

                    memtable.put(key.clone(), value);

                    if let Ok(json_str) =
                        std::str::from_utf8(&entry[v_len_offset + 4..v_len_offset + 4 + v_len])
                    {
                        if let Ok(doc) = Document::from_json(json_str) {
                            let _ = index.on_put(&doc.path, &doc);
                        }
                    }
                } else if op == 1 {
                    // Delete
                    memtable.delete(key.clone());
                    let _ = index.on_delete(&key);
                }
            }
        }

        // Load SSTs
        let mut ssts = Vec::new();
        if let Ok(entries) = storage.read_dir(&path) {
            let mut sst_files = Vec::new();
            for (p, mtime) in entries {
                if let Some(ext) = p.extension() {
                    if ext == "sst" {
                        sst_files.push((p, mtime));
                    }
                }
            }
            // Sort by mtime descending (newest first)
            sst_files.sort_by(|a, b| b.1.cmp(&a.1));

            for (p, _) in sst_files {
                if let Ok(reader) = SstReader::open(&*storage, p) {
                    ssts.push(Arc::new(std::sync::Mutex::new(reader)));
                }
            }
        }

        Ok(Self {
            path,
            storage,
            wal,
            memtable,
            ssts,
            index,
            listeners: ListenerManager::new(),
            rules: RulesEngine::new(),
            #[cfg(feature = "sync")]
            sync: SyncManager::new(Box::new(MockRemoteStore)),
            config: None,
            document_versions: HashMap::new(),
        })
    }

    // Allow swapping remote store
    #[cfg(feature = "sync")]
    pub fn set_remote_store(&mut self, remote: Box<dyn RemoteStore>) {
        self.sync = SyncManager::new(remote);
    }

    pub fn load_rules(&mut self, rules_str: &str) -> io::Result<()> {
        // Validate rules format
        validation::validate_rules(rules_str)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;

        self.rules
            .load_rules(rules_str)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
    }

    fn check_rules(&self, path: &str, operation: &str) -> io::Result<()> {
        if self.rules.is_empty() {
            return Ok(());
        }

        let full_path = format!("/databases/(default)/documents/{}", path);
        let context: HashMap<String, String> = HashMap::new();
        if self.rules.evaluate(&full_path, operation, &context) {
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!(
                    "Security rules check failed for path '{}' with operation '{}'. \
                     Ensure your security rules allow this operation.",
                    path, operation
                ),
            ))
        }
    }

    pub fn put(&mut self, key: String, value: Vec<u8>) -> io::Result<()> {
        // Only validate the most basic requirements
        if key.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Document path cannot be empty",
            ));
        }
        validation::validate_path(&key)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;

        if value.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Document data cannot be empty",
            ));
        }

        // Skip strict JSON validation for better performance and flexibility
        // Just check if it's valid UTF-8
        if std::str::from_utf8(&value).is_err() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Document data must be valid UTF-8",
            ));
        }

        // Skip rules check if no rules are loaded
        if !self.rules.is_empty() {
            if let Err(e) = self.check_rules(&key, "write") {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    e.to_string(),
                ));
            }
        }

        if let Ok(json_str) = std::str::from_utf8(&value) {
            if let Ok(doc) = Document::from_json(json_str) {
                let _ = self.index.on_put(&doc.path, &doc);
            }
        }

        let mut entry = Vec::new();
        entry.push(0u8);
        entry.extend_from_slice(&(key.len() as u32).to_le_bytes());
        entry.extend_from_slice(key.as_bytes());
        entry.extend_from_slice(&(value.len() as u32).to_le_bytes());
        entry.extend_from_slice(&value);

        self.wal.append(&entry)?;
        self.memtable.put(key, value);
        self.notify_listeners();
        Ok(())
    }

    pub fn delete(&mut self, key: String) -> io::Result<()> {
        validation::validate_path(&key)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;
        self.check_rules(&key, "write")?;
        let _ = self.index.on_delete(&key);

        let mut entry = Vec::new();
        entry.push(1u8);
        entry.extend_from_slice(&(key.len() as u32).to_le_bytes());
        entry.extend_from_slice(key.as_bytes());
        entry.extend_from_slice(&0u32.to_le_bytes());

        self.wal.append(&entry)?;
        self.memtable.delete(key);
        self.notify_listeners();
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        if self.check_rules(key, "read").is_err() {
            return None;
        }
        // Memtable check
        if let Some(val) = self.memtable.get(key) {
            return Some(val.to_vec());
        }

        // SST check (newest first)
        for sst_mutex in &self.ssts {
            let mut sst = match sst_mutex.lock() {
                Ok(guard) => guard,
                Err(poisoned) => {
                    eprintln!("SST mutex poisoned, attempting recovery");
                    poisoned.into_inner()
                }
            };
            match sst.get(key) {
                Ok(SstSearchResult::Found(val)) => return Some(val),
                Ok(SstSearchResult::Deleted) => return None,
                Ok(SstSearchResult::NotFound) | Err(_) => continue,
            }
        }

        None
    }

    pub fn query(&self, q: &QueryAst) -> io::Result<Vec<Document>> {
        // Assume list permissions handled by collection rule (not impl in M4) or per-doc.
        let paths = self
            .index
            .query(q)
            .map_err(|e| io::Error::other(e.to_string()))?;

        let mut docs = Vec::new();
        for path in paths {
            if self.check_rules(&path, "read").is_ok() {
                if let Some(bytes) = self.get(&path) {
                    if let Ok(s) = std::str::from_utf8(&bytes) {
                        if let Ok(doc) = Document::from_json(s) {
                            docs.push(doc);
                        }
                    }
                }
            }
        }
        Ok(docs)
    }

    pub fn listen(&mut self, q: QueryAst, callback: SnapshotCallback) -> u64 {
        let id = self.listeners.register(q.clone(), callback);
        if let Ok(docs) = self.query(&q) {
            self.listeners.notify(id, docs);
        }
        id
    }

    fn notify_listeners(&self) {
        for (id, q) in self.listeners.get_listeners() {
            if let Ok(docs) = self.query(&q) {
                self.listeners.notify(id, docs);
            }
        }
    }

    // Sync Operations
    #[cfg(feature = "sync")]
    pub fn sync_push(&self, key: &str) -> io::Result<()> {
        if let Some(bytes) = self.get(key) {
            if let Ok(s) = std::str::from_utf8(&bytes) {
                if let Ok(doc) = Document::from_json(s) {
                    self.sync.push(&doc).map_err(io::Error::other)?;
                    return Ok(());
                }
            }
        }
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Document '{}' not found or contains invalid JSON data", key),
        ))
    }

    #[cfg(feature = "sync")]
    pub fn sync_pull(&mut self, key: &str) -> io::Result<()> {
        if let Ok(Some(doc)) = self.sync.pull(key).map_err(io::Error::other) {
            // We pulled a doc. Write it to local.
            // Bypass check_rules? "Admin" action? Or enforce "write"?
            // Syncing usually implies authoritative source, so maybe bypass?
            // But for safety, let's just use put().

            let bytes = doc
                .to_json()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
                .into_bytes();
            // We need to call put, but get() above took &self. pull took &self.
            // put needs &mut self.
            // We are in &mut self method.
            self.put(doc.path, bytes)?;
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Remote document '{}' not found during sync pull", key),
            ))
        }
    }

    pub fn flush(&mut self) -> io::Result<()> {
        let uuid = uuid::Uuid::new_v4();
        let sst_path = self.path.join(format!("{}.sst", uuid));

        let builder = SstBuilder::new(&*self.storage, sst_path)?;
        builder.build(&self.memtable)?;
        Ok(())
    }

    /// Create a new write batch
    pub fn batch(&self) -> WriteBatch {
        WriteBatch::new()
    }

    /// Helper to extract path from batch operation
    fn get_operation_path(&self, op: &crate::transaction::BatchOperation) -> Option<String> {
        match op {
            crate::transaction::BatchOperation::Set { path, .. } => Some(path.clone()),
            crate::transaction::BatchOperation::Update { path, .. } => Some(path.clone()),
            crate::transaction::BatchOperation::Delete { path } => Some(path.clone()),
        }
    }

    /// Commit a write batch atomically
    pub fn commit_batch(&mut self, batch: &WriteBatch) -> Result<()> {
        // Validate all operations first
        for op in batch.operations() {
            if let Some(path) = self.get_operation_path(op) {
                validation::validate_path(&path)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;

                let op_type = match op {
                    crate::transaction::BatchOperation::Delete { .. } => "write",
                    _ => "write",
                };

                self.check_rules(&path, op_type)?;
            }
        }

        for op in batch.operations() {
            execute_batch_operation(
                op,
                &mut self.wal,
                &mut self.memtable,
                Some(batch.batch_id().to_string()),
            )?;
        }
        self.notify_listeners();
        Ok(())
    }

    /// Run compaction to merge SST files and remove tombstones
    pub fn compact(&self) -> Result<CompactionStats> {
        // Compactor also needs storage injection.
        // For M4/M5 we can stub this or update Compactor too.
        // let compactor = Compactor::new(self.path.clone());
        // compactor.compact()
        // TODO: Update Compactor to use Storage trait
        Ok(CompactionStats {
            files_before: 0,
            files_after: 0,
            entries_before: 0,
            entries_after: 0,
            size_before: 0,
            size_after: 0,
            tombstones_removed: 0,
        })
    }

    /// Put with FieldValue support
    pub fn put_with_field_values(
        &mut self,
        key: String,
        mut data: serde_json::Map<String, serde_json::Value>,
    ) -> io::Result<()> {
        // Get existing document for FieldValue processing
        let existing_data = if let Some(bytes) = self.get(&key) {
            std::str::from_utf8(&bytes).ok().and_then(|s| {
                serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(s).ok()
            })
        } else {
            None
        };

        // Process FieldValue operations
        process_field_values(&mut data, existing_data.as_ref());

        // Convert to JSON and put
        let json_str = serde_json::to_string(&data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        self.put(key, json_str.into_bytes())
    }

    /// Get the current configuration
    pub fn config(&self) -> Option<&FireLocalConfig> {
        self.config.as_ref()
    }
}
