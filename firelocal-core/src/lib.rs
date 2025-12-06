pub mod api;
pub mod config;
pub mod ffi;
pub mod field_value;
pub mod index;
pub mod listener;
pub mod model;
pub mod rules;
pub mod store;
pub mod sync;
pub mod transaction;

use crate::config::FireLocalConfig;
use crate::field_value::process_field_values;
use crate::index::basic_index::BasicIndexProvider;
use crate::index::{IndexProvider, QueryAst};
use crate::listener::{ListenerManager, SnapshotCallback};
use crate::model::Document;
use crate::rules::RulesEngine;
use crate::store::compaction::{CompactionStats, Compactor};
use crate::store::memtable::Memtable;
use crate::store::sst::{SstBuilder, SstReader, SstSearchResult};
use crate::store::wal::WriteAheadLog;
use crate::sync::{MockRemoteStore, RemoteStore, SyncManager};
use crate::transaction::{Transaction, WriteBatch, execute_batch_operation};
use anyhow::Result;
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;

pub struct FireLocal {
    path: PathBuf,
    wal: WriteAheadLog,
    memtable: Memtable,
    ssts: Vec<Arc<std::sync::Mutex<SstReader>>>,
    index: Arc<dyn IndexProvider>,
    listeners: ListenerManager,
    rules: RulesEngine,
    sync: SyncManager,
    config: Option<FireLocalConfig>,
    document_versions: HashMap<String, u64>,
}

impl FireLocal {
    pub fn new(path: impl Into<PathBuf>) -> io::Result<Self> {
        let path = path.into();
        std::fs::create_dir_all(&path)?;

        let wal_path = path.join("wal.log");
        let wal = WriteAheadLog::open(wal_path)?;

        let index = Arc::new(BasicIndexProvider::new());

        let mut memtable = Memtable::new();

        // Replay WAL
        if let Ok(iter) = wal.iter() {
            for entry_res in iter {
                if let Ok(entry) = entry_res {
                    if entry.is_empty() {
                        continue;
                    }
                    let op = entry[0];
                    if entry.len() < 5 {
                        continue;
                    }
                    let k_len = u32::from_le_bytes(entry[1..5].try_into().unwrap()) as usize;
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
                        let v_len = u32::from_le_bytes(
                            entry[v_len_offset..v_len_offset + 4].try_into().unwrap(),
                        ) as usize;
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
        }

        // Load SSTs
        let mut ssts = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&path) {
            let mut sst_files = Vec::new();
            for entry in entries {
                if let Ok(entry) = entry {
                    let p = entry.path();
                    if let Some(ext) = p.extension() {
                        if ext == "sst" {
                            let mtime = if let Ok(meta) = entry.metadata() {
                                meta.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                            } else {
                                std::time::SystemTime::UNIX_EPOCH
                            };
                            sst_files.push((p, mtime));
                        }
                    }
                }
            }
            // Sort by mtime descending (newest first)
            sst_files.sort_by(|a, b| b.1.cmp(&a.1));

            for (p, _) in sst_files {
                if let Ok(reader) = SstReader::open(p) {
                    ssts.push(Arc::new(std::sync::Mutex::new(reader)));
                }
            }
        }

        Ok(Self {
            path,
            wal,
            memtable,
            ssts,
            index,
            listeners: ListenerManager::new(),
            rules: RulesEngine::new(),
            sync: SyncManager::new(Box::new(MockRemoteStore)),
            config: None,
            document_versions: HashMap::new(),
        })
    }

    /// Create a new FireLocal instance with configuration
    pub fn new_with_config(path: impl Into<PathBuf>) -> io::Result<Self> {
        let path_buf = path.into();
        let config = FireLocalConfig::load_or_create(Some(&path_buf))
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let mut instance = Self::new(&path_buf)?;
        instance.config = Some(config);
        Ok(instance)
    }

    // Allow swapping remote store
    pub fn set_remote_store(&mut self, remote: Box<dyn RemoteStore>) {
        self.sync = SyncManager::new(remote);
    }

    pub fn load_rules(&mut self, rules_str: &str) -> io::Result<()> {
        self.rules
            .load_rules(rules_str)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
    }

    fn check_rules(&self, path: &str, operation: &str) -> io::Result<()> {
        let full_path = format!("/databases/(default)/documents/{}", path);
        let context: HashMap<String, String> = HashMap::new();
        if self.rules.evaluate(&full_path, operation, &context) {
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Rules check failed",
            ))
        }
    }

    pub fn put(&mut self, key: String, value: Vec<u8>) -> io::Result<()> {
        self.check_rules(&key, "write")?;

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
            let mut sst = sst_mutex.lock().unwrap();
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
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

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
    pub fn sync_push(&self, key: &str) -> io::Result<()> {
        if let Some(bytes) = self.get(key) {
            if let Ok(s) = std::str::from_utf8(&bytes) {
                if let Ok(doc) = Document::from_json(s) {
                    self.sync
                        .push(&doc)
                        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                    return Ok(());
                }
            }
        }
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Doc not found or invalid",
        ))
    }

    pub fn sync_pull(&mut self, key: &str) -> io::Result<()> {
        if let Ok(Some(doc)) = self
            .sync
            .pull(key)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
        {
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
                "Remote doc not found",
            ))
        }
    }

    pub fn flush(&mut self) -> io::Result<()> {
        let uuid = uuid::Uuid::new_v4();
        let sst_path = self.path.join(format!("{}.sst", uuid));

        let builder = SstBuilder::new(sst_path)?;
        builder.build(&self.memtable)?;
        Ok(())
    }

    /// Create a new write batch
    pub fn batch(&self) -> WriteBatch {
        WriteBatch::new()
    }

    /// Commit a write batch atomically
    pub fn commit_batch(&mut self, batch: &WriteBatch) -> Result<()> {
        for op in batch.operations() {
            execute_batch_operation(
                op,
                &mut self.wal,
                &mut self.memtable,
                Some(batch.batch_id()),
            )?;
        }
        self.notify_listeners();
        Ok(())
    }

    /// Run a transaction with optimistic concurrency control
    pub fn run_transaction<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut Transaction, &FireLocal) -> Result<()>,
    {
        let mut txn = Transaction::new();

        // Execute transaction function
        f(&mut txn, self)?;

        // Validate versions haven't changed
        txn.validate(|path| self.document_versions.get(path).copied())?;

        // Apply writes
        for op in txn.writes() {
            execute_batch_operation(
                op,
                &mut self.wal,
                &mut self.memtable,
                Some(txn.transaction_id()),
            )?;
        }

        // Update versions
        for op in txn.writes() {
            if let Some(path) = self.get_operation_path(op) {
                let version = self.document_versions.get(&path).unwrap_or(&0) + 1;
                self.document_versions.insert(path, version);
            }
        }

        self.notify_listeners();
        Ok(())
    }

    /// Helper to extract path from batch operation
    fn get_operation_path(&self, _op: &crate::transaction::BatchOperation) -> Option<String> {
        // This is a workaround since BatchOperation is private
        // In production, we'd expose a method to get the path
        None // TODO: Implement properly
    }

    /// Run compaction to merge SST files and remove tombstones
    pub fn compact(&self) -> Result<CompactionStats> {
        let compactor = Compactor::new(self.path.clone());
        compactor.compact()
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
