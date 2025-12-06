pub mod enhanced;
pub mod firebase;

use crate::model::Document;

pub trait RemoteStore: Send + Sync {
    fn push(&self, doc: &Document) -> Result<(), String>;
    fn pull(&self, path: &str) -> Result<Option<Document>, String>;
}

pub struct SyncManager {
    remote: Box<dyn RemoteStore>,
}

impl SyncManager {
    pub fn new(remote: Box<dyn RemoteStore>) -> Self {
        Self { remote }
    }

    pub fn push(&self, doc: &Document) -> Result<(), String> {
        self.remote.push(doc)
    }

    pub fn pull(&self, path: &str) -> Result<Option<Document>, String> {
        self.remote.pull(path)
    }
}

pub struct MockRemoteStore;

impl RemoteStore for MockRemoteStore {
    fn push(&self, _doc: &Document) -> Result<(), String> {
        Ok(())
    }

    fn pull(&self, _path: &str) -> Result<Option<Document>, String> {
        Ok(None)
    }
}
