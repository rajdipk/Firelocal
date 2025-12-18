use crate::store::io::{FileHandle, Storage};
use crc32fast::Hasher;
use serde::{Deserialize, Serialize};
use std::io::{self, BufReader, Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WalOp {
    Put,
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalEntry {
    pub op: WalOp,
    pub key: String,
    pub value: Option<Vec<u8>>,
    pub batch_id: Option<String>,
}

impl WalEntry {
    pub fn put(key: String, value: Vec<u8>, batch_id: Option<&str>) -> Self {
        Self {
            op: WalOp::Put,
            key,
            value: Some(value),
            batch_id: batch_id.map(|s| s.to_string()),
        }
    }

    pub fn delete(key: String, batch_id: Option<&str>) -> Self {
        Self {
            op: WalOp::Delete,
            key,
            value: None,
            batch_id: batch_id.map(|s| s.to_string()),
        }
    }
}

pub struct WriteAheadLog<S: Storage> {
    file: S::File,
    path: PathBuf,
    storage: Arc<S>,
}

impl<S: Storage> WriteAheadLog<S> {
    pub fn open(storage: Arc<S>, path: impl AsRef<Path>) -> io::Result<Self> {
        let p = path.as_ref().to_path_buf();
        // WAL must be append-only.
        // In StdStorage we would use OpenOptions::append(true).
        // Our FileHandle trait doesn't strictly enforce open mode,
        // so we depend on the Storage implementation or seek to end.

        let mut file = if storage.exists(&p) {
            storage.open(&p)?
        } else {
            storage.create(&p)?
        };

        // Ensure we are at the end for appending
        file.seek(std::io::SeekFrom::End(0))?;

        Ok(Self {
            file,
            path: p,
            storage,
        })
    }

    pub fn append(&mut self, data: &[u8]) -> io::Result<()> {
        let len = data.len() as u32;
        let mut hasher = Hasher::new();
        hasher.update(data);
        let crc = hasher.finalize();

        self.file.write_all(&len.to_le_bytes())?;
        self.file.write_all(&crc.to_le_bytes())?;
        self.file.write_all(data)?;
        self.file.sync_all()?;
        Ok(())
    }

    pub fn iter(&self) -> io::Result<WalIterator<S::File>> {
        // We need a readable handle from the start.
        // Our FileHandle supports seek, so we could technically use the same handle if we locked it,
        // but for iteration we usually want a separate reader.
        // `Storage::open` returns a new handle.
        let file = self.storage.open(&self.path)?;
        Ok(WalIterator {
            reader: BufReader::new(file),
        })
    }
}

pub struct WalIterator<F: Read> {
    reader: BufReader<F>,
}

impl<F: Read> Iterator for WalIterator<F> {
    type Item = io::Result<Vec<u8>>;

    fn next(&mut self) -> Option<Self::Item> {
        // Read LEN (4 bytes)
        let mut len_buf = [0u8; 4];
        match self.reader.read_exact(&mut len_buf) {
            Ok(_) => {}
            Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => return None,
            Err(e) => return Some(Err(e)),
        }
        let len = u32::from_le_bytes(len_buf) as usize;

        // Read CRC (4 bytes)
        let mut crc_buf = [0u8; 4];
        if let Err(e) = self.reader.read_exact(&mut crc_buf) {
            return Some(Err(e));
        }
        let expected_crc = u32::from_le_bytes(crc_buf);

        // Read Data (len bytes)
        let mut data = vec![0u8; len];
        if let Err(e) = self.reader.read_exact(&mut data) {
            return Some(Err(e));
        }

        // Verify CRC
        let mut hasher = Hasher::new();
        hasher.update(&data);
        if hasher.finalize() != expected_crc {
            return Some(Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "CRC mismatch",
            )));
        }

        Some(Ok(data))
    }
}
