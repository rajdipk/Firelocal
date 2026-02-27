use crate::store::io::Storage;
use crate::store::wal::{WalEntry, WriteAheadLog};
use anyhow::Result;
use std::collections::HashMap;
use std::io;
use uuid::Uuid;

/// WriteBatch allows batching multiple write operations into a single atomic commit
pub struct WriteBatch {
    operations: Vec<BatchOperation>,
    batch_id: String,
}

#[derive(Debug, Clone)]
pub enum BatchOperation {
    Set { path: String, data: Vec<u8> },
    Update { path: String, data: Vec<u8> },
    Delete { path: String },
}

impl WriteBatch {
    /// Create a new write batch
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
            batch_id: Uuid::new_v4().to_string(),
        }
    }

    /// Add a set operation to the batch
    pub fn set(&mut self, path: String, data: Vec<u8>) -> &mut Self {
        self.operations.push(BatchOperation::Set { path, data });
        self
    }

    /// Add an update operation to the batch
    pub fn update(&mut self, path: String, data: Vec<u8>) -> &mut Self {
        self.operations.push(BatchOperation::Update { path, data });
        self
    }

    /// Add a delete operation to the batch
    pub fn delete(&mut self, path: String) -> &mut Self {
        self.operations.push(BatchOperation::Delete { path });
        self
    }

    /// Get the batch ID
    pub fn batch_id(&self) -> &str {
        &self.batch_id
    }

    /// Get the operations in this batch
    pub fn operations(&self) -> &[BatchOperation] {
        &self.operations
    }

    /// Get the number of operations
    pub fn len(&self) -> usize {
        self.operations.len()
    }

    /// Check if batch is empty
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }
}

impl Default for WriteBatch {
    fn default() -> Self {
        Self::new()
    }
}

/// Transaction provides read-write transaction support with optimistic concurrency
pub struct Transaction {
    reads: HashMap<String, Option<(Vec<u8>, u64)>>, // path -> (data, version)
    writes: Vec<BatchOperation>,
    transaction_id: String,
    write_conflicts: HashMap<String, Vec<u8>>, // Track write conflicts
}

impl Transaction {
    /// Create a new transaction
    pub fn new() -> Self {
        Self {
            reads: HashMap::new(),
            writes: Vec::new(),
            transaction_id: Uuid::new_v4().to_string(),
            write_conflicts: HashMap::new(),
        }
    }

    /// Read a document in the transaction
    pub fn get(
        &mut self,
        path: &str,
        current_data: Option<Vec<u8>>,
        version: u64,
    ) -> Option<Vec<u8>> {
        // Record the read
        self.reads.insert(
            path.to_string(),
            current_data.clone().map(|d| (d.clone(), version)),
        );
        current_data
    }

    /// Set a document in the transaction
    pub fn set(&mut self, path: String, data: Vec<u8>) {
        self.writes.push(BatchOperation::Set { path, data });
    }

    /// Update a document in the transaction
    pub fn update(&mut self, path: String, data: Vec<u8>) {
        self.writes.push(BatchOperation::Update { path, data });
    }

    /// Delete a document in the transaction
    pub fn delete(&mut self, path: String) {
        self.writes.push(BatchOperation::Delete { path });
    }

    /// Record a write conflict for a document
    pub fn record_write_conflict(&mut self, path: String, conflicting_data: Vec<u8>) {
        self.write_conflicts.insert(path, conflicting_data);
    }

    /// Get the transaction ID
    pub fn transaction_id(&self) -> &str {
        &self.transaction_id
    }

    /// Get the write operations
    pub fn writes(&self) -> &[BatchOperation] {
        &self.writes
    }

    /// Get write conflicts
    pub fn write_conflicts(&self) -> &HashMap<String, Vec<u8>> {
        &self.write_conflicts
    }

    /// Validate that read versions haven't changed and check for write conflicts
    pub fn validate<F1, F2>(&self, get_current_version: F1, _get_current_data: F2) -> Result<()>
    where
        F1: Fn(&str) -> Option<u64>,
        F2: Fn(&str) -> Option<u64>,
    {
        // Check read conflicts (original validation)
        for (path, read_data) in &self.reads {
            let read_version = read_data.as_ref().map(|(_, v)| *v);
            let current_version = get_current_version(path);

            if read_version != current_version {
                return Err(io::Error::other(format!(
                    "Transaction conflict: document {} was modified (read version: {:?}, current version: {:?})",
                    path, read_version, current_version
                ))
                .into());
            }
        }

        // Check write conflicts
        for (path, _) in &self.write_conflicts {
            // If any write conflicts exist, the transaction should fail
            return Err(io::Error::other(format!(
                "Transaction conflict: document {} was modified by another transaction",
                path
            ))
            .into());
        }

        // Check for write-write conflicts (multiple transactions writing same document)
        for write_op in &self.writes {
            let path = match write_op {
                BatchOperation::Set { path, .. } => path,
                BatchOperation::Update { path, .. } => path,
                BatchOperation::Delete { path } => path,
            };

            // Check if this document was written by another transaction
            // In a real implementation, this would involve checking a global transaction log
            // For now, we'll just check if the current version changed since we started
            if let Some(current_version) = get_current_version(path) {
                if let Some(Some((_, read_version))) = self.reads.get(path) {
                    if current_version != *read_version {
                        return Err(io::Error::other(format!(
                            "Write-write conflict: document {} version changed during transaction",
                            path
                        ))
                        .into());
                    }
                }
            }
        }

        Ok(())
    }
}

impl Default for Transaction {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to execute a batch operation
pub fn execute_batch_operation<S: Storage>(
    op: &BatchOperation,
    wal: &mut WriteAheadLog<S>,
    memtable: &mut crate::store::memtable::Memtable,
    batch_id: Option<String>,
) -> Result<()> {
    match op {
        BatchOperation::Set { path, data } | BatchOperation::Update { path, data } => {
            let entry = WalEntry::put(path.clone(), data.clone(), batch_id.as_deref());
            let entry_bytes = serde_json::to_vec(&entry)?;
            wal.append(&entry_bytes)?;
            memtable.put(path.clone(), data.clone());
        }
        BatchOperation::Delete { path } => {
            let entry = WalEntry::delete(path.clone(), batch_id.as_deref());
            let entry_bytes = serde_json::to_vec(&entry)?;
            wal.append(&entry_bytes)?;
            memtable.delete(path.clone());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_batch() {
        let mut batch = WriteBatch::new();
        batch
            .set("users/alice".to_string(), b"alice_data".to_vec())
            .set("users/bob".to_string(), b"bob_data".to_vec())
            .delete("users/charlie".to_string());

        assert_eq!(batch.len(), 3);
        assert!(!batch.is_empty());
        assert!(!batch.batch_id().is_empty());
    }

    #[test]
    fn test_transaction() {
        let mut txn = Transaction::new();

        // Simulate reading a document
        let data = b"test_data".to_vec();
        let result = txn.get("users/alice", Some(data.clone()), 1);
        assert_eq!(result, Some(data));

        // Write in transaction
        txn.set("users/alice".to_string(), b"new_data".to_vec());

        assert_eq!(txn.writes().len(), 1);
        assert!(!txn.transaction_id().is_empty());
    }

    #[test]
    fn test_transaction_validation() {
        let mut txn = Transaction::new();

        // Read with version 1
        txn.get("users/alice", Some(b"data".to_vec()), 1);

        // Validation should pass if version is still 1
        let result = txn.validate(
            |path| if path == "users/alice" { Some(1) } else { None },
            |_| None,
        );
        assert!(result.is_ok());

        // Validation should fail if version changed to 2
        let result = txn.validate(
            |path| if path == "users/alice" { Some(2) } else { None },
            |_| None,
        );
        assert!(result.is_err());

        // Test write conflict detection
        let mut txn2 = Transaction::new();
        txn2.record_write_conflict("users/bob".to_string(), b"conflicting_data".to_vec());
        
        let result = txn2.validate(|_| None, |_| None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("write conflict"));
    }
}
