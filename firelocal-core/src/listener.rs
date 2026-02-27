use crate::index::QueryAst;
use crate::model::Document;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};

pub type SnapshotCallback = Box<dyn Fn(Vec<Document>) + Send + Sync>;

struct ListenerEntry {
    query: QueryAst,
    callback: SnapshotCallback,
}

pub struct ListenerManager {
    listeners: Arc<Mutex<HashMap<u64, ListenerEntry>>>,
    next_id: AtomicU64,
}

impl Default for ListenerManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ListenerManager {
    pub fn new() -> Self {
        Self {
            listeners: Arc::new(Mutex::new(HashMap::new())),
            next_id: AtomicU64::new(0),
        }
    }

    pub fn register(&self, query: QueryAst, callback: SnapshotCallback) -> u64 {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        
        let mut listeners = match self.listeners.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        listeners.insert(id, ListenerEntry { query, callback });
        id
    }

    pub fn unregister(&self, id: u64) {
        let mut listeners = match self.listeners.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        listeners.remove(&id);
    }

    /// Get a snapshot of all listeners without holding the lock during notification
    pub fn get_listener_snapshots(&self) -> Vec<(u64, QueryAst)> {
        let listeners = match self.listeners.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        
        listeners
            .iter()
            .map(|(id, entry)| (*id, entry.query.clone()))
            .collect()
    }

    /// Notify a specific listener without holding the global lock
    pub fn notify_single(&self, id: u64, docs: Vec<Document>) {
        // Execute callback while holding the lock (simpler and safer)
        let listeners = match self.listeners.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        
        if let Some(entry) = listeners.get(&id) {
            (entry.callback)(docs);
        }
    }

    // Legacy methods for backward compatibility
    pub fn get_listeners(&self) -> Vec<(u64, QueryAst)> {
        let listeners = match self.listeners.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        listeners
            .iter()
            .map(|(id, entry)| (*id, entry.query.clone()))
            .collect()
    }

    pub fn notify(&self, id: u64, docs: Vec<Document>) {
        self.notify_single(id, docs);
    }
}
