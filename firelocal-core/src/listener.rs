use crate::index::QueryAst;
use crate::model::Document;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type SnapshotCallback = Box<dyn Fn(Vec<Document>) + Send + Sync>;

struct ListenerEntry {
    query: QueryAst,
    callback: SnapshotCallback,
}

pub struct ListenerManager {
    listeners: Arc<Mutex<HashMap<u64, ListenerEntry>>>,
    next_id: Arc<Mutex<u64>>,
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
            next_id: Arc::new(Mutex::new(0)),
        }
    }

    pub fn register(&self, query: QueryAst, callback: SnapshotCallback) -> u64 {
        let mut listeners = match self.listeners.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let mut id_guard = match self.next_id.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let id = *id_guard;
        *id_guard += 1;

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

    // In a real system, we'd efficiently match changed docs to queries.
    // For M3 MVP, we might re-run all active queries?
    // Or just notify everyone "something changed" and let them re-query?
    // TDD says: "Snapshot listener subscribed to users receives an added change with the doc."
    // Let's implement full re-run for now (easiest correctness).
    // We need 'FireLocal' to re-run queries.
    // This implies ListenerManager needs access to DB or DB calls ListenerManager with results?
    // Structure: DB calls ListenerManager::notify_change(&self, db_instance)

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
        let listeners = match self.listeners.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        if let Some(entry) = listeners.get(&id) {
            (entry.callback)(docs);
        }
    }
}
